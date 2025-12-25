use image::RgbaImage;

/// Tint an image with a color (for grass, leaves, etc.)
pub fn tint_image(img: &RgbaImage, tint: [f64; 3]) -> RgbaImage {
    let mut result = img.clone();
    for pixel in result.pixels_mut() {
        // Multiply the RGB channels by the tint color
        pixel[0] = (pixel[0] as f64 * tint[0]) as u8;
        pixel[1] = (pixel[1] as f64 * tint[1]) as u8;
        pixel[2] = (pixel[2] as f64 * tint[2]) as u8;
    }
    result
}

/// Darken an image by a factor (0.0 = black, 1.0 = original)
pub fn darken_image(img: &RgbaImage, factor: f64) -> RgbaImage {
    return tint_image(img, [factor, factor, factor]);
}
