use std::env::var;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use crate::commons::error::DortCapResult;

fn in_range(input: u8, min: u8, max: u8) -> bool {
    input > min && input < max
}

pub fn process_dynamic_image(
    dynamic_img: DynamicImage,
    variant: &str,
) -> DortCapResult<DynamicImage> {
    let mut new_img = DynamicImage::new_rgba8(dynamic_img.width(), dynamic_img.height());
    let mut nx = 0;
    let mut ny = 0;
    for (x, y, pixel) in dynamic_img.pixels() {
        let red = pixel.0[1];
        let green = pixel.0[1];
        let blue = pixel.0[1];
        match variant {
            "numericalmatch" | "orbit_match_game" => {
                if red > 220 && green > 220 && blue > 220 {
                    new_img.put_pixel(x, y, pixel);
                }
            }
            _ => {
                new_img.put_pixel(x, y, pixel);
            }
        }
    }
    Ok(new_img)
}