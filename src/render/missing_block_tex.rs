use image::{Rgba, RgbaImage};

pub fn create_missing_block_texture() -> RgbaImage {
    let mut tex = RgbaImage::new(16, 16);
    for y in 0..16 {
        for x in 0..16 {
            let color = if (x / 8 + y / 8) % 2 == 0 {
                Rgba([255, 0, 255, 255]) // Magenta
            } else {
                Rgba([0, 0, 0, 255]) // Black
            };
            tex.put_pixel(x, y, color);
        }
    }
    tex
}
