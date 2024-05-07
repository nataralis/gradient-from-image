use colors_transform::{Color, Hsl};
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

fn create_pixel_array_hsl(img_width: u32, img_height: u32, img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Vec<Hsl> {
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
    };

    result_array_rgb
}

fn sort_pixels_by_lightness(img_width: u32, img_height: u32, img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Vec<image::Rgb<u8>> {
    let mut pixel_array_hsl = create_pixel_array_hsl(img_width, img_height, &img);

    pixel_array_hsl.sort_by(|a, b| {
        a.get_lightness().partial_cmp(&b.get_lightness()).unwrap()
    });

    let result_array_rgb = create_result_array_rgb(&pixel_array_hsl);

    result_array_rgb
}

fn generate_raw_gradient(result_width: u32, result_height: u32, img: &DynamicImage) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let (img_width, img_height) = img.dimensions();
    let imgbuf = ImageBuffer::from_fn(img_width, img_height, |x, y| {
        img.get_pixel(x, y).to_rgb()
    });

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

fn main() {
    let img = open("images/low_res.jpg").unwrap();

    let (img_width, img_height) = img.dimensions();

    let input = String::new();

    let result_width = 5000;
    let result_height = 5000;

    let imgbuf = generate_raw_gradient(result_width, result_height, &img);

    imgbuf.save("images/modified_low_res.png").unwrap();

    println!("saving done!");
}