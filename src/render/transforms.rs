use crate::coords::block_face::{BlockFace, VerticalBlockFace};
use crate::render::missing_block_tex::create_missing_block_texture;
use crate::utils::darken_image;
use image::{RgbaImage, imageops};

pub fn transform(texture: &RgbaImage, face: &BlockFace) -> RgbaImage {
    match face {
        BlockFace::Top => transform_top(texture),
        BlockFace::Bottom => create_missing_block_texture(), // TODO
        BlockFace::North => transform_side(texture, VerticalBlockFace::North),
        BlockFace::South => transform_side(texture, VerticalBlockFace::South),
        BlockFace::East => transform_side(texture, VerticalBlockFace::East),
        BlockFace::West => transform_side(texture, VerticalBlockFace::West),
    }
}

/// Transform a texture for the top face of an isometric block
/// Rotates 45 degrees and scales Y by 0.5
/// Output: 24x12 pixels
fn transform_top(texture: &RgbaImage) -> RgbaImage {
    // Resize to 17x17 for better sampling
    let resized = imageops::resize(texture, 17, 17, imageops::FilterType::Triangle);

    // Create output image (24x12 for top face)
    let mut output = RgbaImage::new(crate::render::renderer::SPRITE_SIZE, 12);

    // The transformation matrix for isometric top view:
    // 1. Rotate 45 degrees
    // 2. Scale Y by 0.5 (compress vertically)
    //
    // For each pixel in the output, we calculate the corresponding source pixel
    // using inverse transformation
    let cos45 = std::f64::consts::FRAC_1_SQRT_2;
    let sin45 = std::f64::consts::FRAC_1_SQRT_2;

    for out_y in 0..12 {
        for out_x in 0..crate::render::renderer::SPRITE_SIZE {
            // Transform output coordinates back to source coordinates
            // First, center the output coordinates
            let cx = out_x as f64 - 12.0;
            let cy = (out_y as f64 - 6.0) * 2.0; // Scale Y back up

            // Inverse rotate by -45 degrees
            let src_x = cx * cos45 + cy * sin45;
            let src_y = -cx * sin45 + cy * cos45;

            // Translate to source image coordinates
            let src_x = src_x + 8.5;
            let src_y = src_y + 8.5;

            // Sample from source image if in bounds
            if src_x >= 0.0 && src_x < 17.0 && src_y >= 0.0 && src_y < 17.0 {
                let sx = src_x as u32;
                let sy = src_y as u32;
                if sx < 17 && sy < 17 {
                    let pixel = resized.get_pixel(sx, sy);
                    output.put_pixel(out_x, out_y, *pixel);
                }
            }
        }
    }

    output
}

fn transform_side(texture: &RgbaImage, side: VerticalBlockFace) -> RgbaImage {
    const RESIZED_DIM: u32 = 12;
    const SHEARED_HEIGHT: u32 = 18;

    // Resize to 12x12
    let resized = imageops::resize(
        texture,
        RESIZED_DIM,
        RESIZED_DIM,
        imageops::FilterType::Triangle,
    );

    // Create output image (12x18 for side face after shear)
    let mut output = RgbaImage::new(RESIZED_DIM, SHEARED_HEIGHT);

    let shear_factor = 0.5;

    // Shear transformation: y_new = y + 0.5 * x
    // Inverse: y_src = y_out - 0.5 * x_out
    for out_y in 0..SHEARED_HEIGHT {
        for out_x in 0..RESIZED_DIM {
            // Apply inverse shear to find source coordinates
            let src_x = out_x as f64;
            let src_y = out_y as f64 - shear_factor * out_x as f64;

            if src_x >= 0.0 && src_y >= 0.0 {
                let sx = src_x as u32;
                let sy = src_y as u32;
                if sy < resized.height() && sx < resized.width() {
                    let pixel = resized.get_pixel(sx, sy);
                    output.put_pixel(out_x, out_y, *pixel);
                }
            }
        }
    }

    // Darken the sides (left 0.9, right 0.8)
    let side_darken_factor = match side {
        VerticalBlockFace::South => 0.9,
        VerticalBlockFace::East => 0.8,
        _ => 1.0, // TODO
    };
    let darkened_img = darken_image(&output, side_darken_factor);

    match side {
        VerticalBlockFace::North => {
            imageops::flip_vertical(&imageops::flip_horizontal(&darkened_img))
        }
        VerticalBlockFace::South => darkened_img,
        VerticalBlockFace::East => imageops::flip_horizontal(&darkened_img),
        VerticalBlockFace::West => imageops::flip_vertical(&darkened_img),
    }
}
