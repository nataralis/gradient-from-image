#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use colors_transform::{Color, Hsl};
use eframe::egui;
use image::{open, DynamicImage, GenericImageView, ImageBuffer, Pixel};

fn rgb_to_colors_transform(image_rgb: &image::Rgb<u8>) -> colors_transform::Rgb {
    let rgb_channels = image_rgb.channels();

    let red = rgb_channels[0].into();
    let green = rgb_channels[1].into();
    let blue = rgb_channels[2].into();

    colors_transform::Rgb::from(red, green, blue)
}

fn rgb_to_image(colors_transform_rgb: &colors_transform::Rgb) -> image::Rgb<u8> {
    let red = colors_transform_rgb.get_red() as u8;
    let green = colors_transform_rgb.get_green() as u8;
    let blue = colors_transform_rgb.get_blue() as u8;

    image::Rgb([red, green, blue])
}

fn create_pixel_array_hsl(
    img_width: u32,
    img_height: u32,
    img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>,
) -> Vec<Hsl> {
    let pixel_count = (img_width * img_height).try_into().unwrap();

    let mut pixel_array_hsl = Vec::with_capacity(pixel_count);
    for y in 0..img_height {
        for x in 0..img_width {
            let img_pixel = img.get_pixel(x, y).to_rgb();
            pixel_array_hsl.push(rgb_to_colors_transform(&img_pixel).to_hsl());
        }
    }

    pixel_array_hsl
}

fn create_result_array_rgb(pixel_array_hsl: &Vec<Hsl>) -> Vec<image::Rgb<u8>> {
    let mut result_array_rgb = Vec::with_capacity(pixel_array_hsl.len());
    for &item in pixel_array_hsl.iter() {
        result_array_rgb.push(rgb_to_image(&item.to_rgb()));
    }

    result_array_rgb
}

fn sort_pixels_by_lightness(
    img_width: u32,
    img_height: u32,
    img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>,
) -> Vec<image::Rgb<u8>> {
    let mut pixel_array_hsl = create_pixel_array_hsl(img_width, img_height, &img);

    pixel_array_hsl.sort_by(|a, b| a.get_lightness().partial_cmp(&b.get_lightness()).unwrap());

    let result_array_rgb = create_result_array_rgb(&pixel_array_hsl);

    result_array_rgb
}

fn generate_raw_gradient(
    result_width: u32,
    result_height: u32,
    img: &DynamicImage,
) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let (img_width, img_height) = img.dimensions();
    let imgbuf = ImageBuffer::from_fn(img_width, img_height, |x, y| img.get_pixel(x, y).to_rgb());

    println!("imgbuf done!");

    let sorted_array = sort_pixels_by_lightness(img_width, img_height, &imgbuf);

    println!("sorted!");

    let result_imgbuf = ImageBuffer::from_fn(result_width, result_height, |x, y| {
        if (x % result_width) == 0 {
            println!("{} lines completed!", y + 1);
        }

        sorted_array[(img_width * img_height / result_height * y) as usize]
    });

    println!("gradient done!");

    result_imgbuf
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Gradient Generator",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<GradientGenerator>::default()
        }),
    )
}

struct GradientGenerator {
    img_path: Option<String>,
    result_path: Option<String>,
    img: DynamicImage,
    imgbuf: ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    width: u32,
    height: u32,
}

impl Default for GradientGenerator {
    fn default() -> Self {
        Self {
            img_path: None,
            result_path: None,
            img: DynamicImage::new(0, 0, image::ColorType::Rgb8),
            imgbuf: ImageBuffer::new(0, 0),
            width: 0,
            height: 0,
        }
    }
}

impl eframe::App for GradientGenerator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gradient Generator");

            if ui.button("Choose image path...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.img_path = Some(path.display().to_string());
                }
            }

            if let Some(path) = &self.img_path {
                ui.horizontal(|ui| {
                    ui.label("Image path:");
                    ui.monospace(path);
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("Image path:");
                    ui.monospace("No path chosen yet");
                });
            }

            if ui.button("Upload file").clicked() {
                if let Some(path) = &self.img_path {
                    self.img = open(&path).unwrap();
                    let (img_width, img_height) = self.img.dimensions();
                    self.width = img_width;
                    self.height = img_height;
                }
            }

            ui.horizontal(|ui| {
                let width_label = ui.label("Width (px): ");
                ui.add(egui::DragValue::new(&mut self.width).speed(1.0))
                    .labelled_by(width_label.id);
            });

            ui.horizontal(|ui| {
                let height_label = ui.label("Height (px): ");
                ui.add(egui::DragValue::new(&mut self.height).speed(1.0))
                    .labelled_by(height_label.id);
            });

            if ui.button("Generate gradient").clicked() {
                self.imgbuf = generate_raw_gradient(self.width, self.height, &self.img);
            }

            if ui.button("Choose save path...").clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    self.result_path = Some(path.display().to_string());
                }
            }

            if let Some(path) = &self.result_path {
                ui.horizontal(|ui| {
                    ui.label("Save path:");
                    ui.monospace(path);
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("Save path:");
                    ui.monospace("No path chosen yet");
                });
            }

            if ui.button("Save file").clicked() {
                if let Some(path) = &self.result_path {
                    self.imgbuf.save(path).unwrap();
                    println!("saving done!");
                }
            }
        });
    }
}
