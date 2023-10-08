pub mod hallways;

use image::{ImageBuffer, Rgb, RgbImage};
use ndarray::Array2;

pub fn render_to_img(a: &Array2<bool>) -> RgbImage {
    let (w, h) = a.dim();
    let mut img = ImageBuffer::new(w as u32, h as u32);
    for ((x, y), v) in a.indexed_iter() {
        img.put_pixel(
            x as u32,
            y as u32,
            if *v {
                Rgb([0u8, 0, 0])
            } else {
                Rgb([255, 255, 255])
            },
        );
    }

    img
}
