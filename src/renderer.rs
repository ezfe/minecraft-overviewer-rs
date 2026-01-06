use image::{
    Rgba, RgbaImage,
    imageops::{self, overlay},
};

use crate::{
    asset_cache::AssetCache,
    blocks::{is_air_block, is_complex_geometry},
    coords::{world_block_coord::WorldBlockCoord, world_chunk_coord::WorldChunkCoord},
    utils::{darken_image, tint_image},
};

const SPRITE_SIZE: u32 = 24;
const MC_CHUNK_SIZE: isize = 16;

/// Transform a texture for the top face of an isometric block
/// Rotates 45 degrees and scales Y by 0.5
/// Output: 24x12 pixels
fn transform_top(texture: &RgbaImage) -> RgbaImage {
    // Resize to 17x17 for better sampling
    let resized = imageops::resize(texture, 17, 17, imageops::FilterType::Triangle);

    // Create output image (24x12 for top face)
    let mut output = RgbaImage::new(SPRITE_SIZE, 12);

    // The transformation matrix for isometric top view:
    // 1. Rotate 45 degrees
    // 2. Scale Y by 0.5 (compress vertically)
    //
    // For each pixel in the output, we calculate the corresponding source pixel
    // using inverse transformation
    let cos45 = std::f64::consts::FRAC_1_SQRT_2;
    let sin45 = std::f64::consts::FRAC_1_SQRT_2;

    for out_y in 0..12 {
        for out_x in 0..SPRITE_SIZE {
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

/// Transform a texture for the left side of an isometric block
/// Applies a shear transformation
/// Output: 12x18 pixels
fn transform_side(texture: &RgbaImage, side: BlockSpriteSide) -> RgbaImage {
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

    match side {
        BlockSpriteSide::SideLeft => output,
        BlockSpriteSide::SideRight => imageops::flip_horizontal(&output),
    }
}

/// Build a full isometric block from top and side textures
/// Returns a 24x24 image
fn render_block_3d(top: &RgbaImage, side: &RgbaImage) -> RgbaImage {
    let mut img = RgbaImage::new(SPRITE_SIZE, SPRITE_SIZE);

    // Transform the top
    let top_transformed = transform_top(top);

    // Transform the sides
    let side_left = transform_side(side, BlockSpriteSide::SideLeft);
    let side_right = transform_side(side, BlockSpriteSide::SideRight);

    // Darken the sides (left 0.9, right 0.8)
    let side_left = darken_image(&side_left, 0.9);
    let side_right = darken_image(&side_right, 0.8);

    // Composite: first the top at (0, 0), then left side at (0, 6), then right at (12, 6)
    overlay(&mut img, &top_transformed, 0, 0);
    overlay(&mut img, &side_left, 0, 6);
    overlay(&mut img, &side_right, 12, 6);

    img
}

/// Get or create a rendered block sprite for a block name
/// Block name should be like "minecraft:stone" or "minecraft:grass_block"
pub fn get_block_sprite(cache: &mut AssetCache, block_name: &str) -> RgbaImage {
    // Check cache first
    if let Some(cached) = cache.block_cache.get(block_name) {
        return cached.clone();
    }

    // Strip minecraft: prefix if present
    let name = block_name.strip_prefix("minecraft:").unwrap_or(block_name);

    // Try to load textures based on common naming patterns
    let sprite = create_block_sprite(cache, name);

    cache
        .block_cache
        .insert(block_name.to_string(), sprite.clone());
    sprite
}

/// Create a block sprite from a block name
fn create_block_sprite(cache: &mut AssetCache, name: &str) -> RgbaImage {
    // Handle special cases first
    if name == "air" || name == "cave_air" || name == "void_air" {
        return RgbaImage::new(SPRITE_SIZE, SPRITE_SIZE); // Transparent
    }

    // Handle transparent/partial blocks that we skip for now
    if is_complex_geometry(name) {
        return RgbaImage::new(SPRITE_SIZE, SPRITE_SIZE); // Transparent for now - complex geometry
    }

    // Try different texture naming patterns

    // Pattern 1: Logs - have _top for end grain and the log itself for bark
    if name.ends_with("_log") || name.ends_with("_wood") || name.ends_with("_stem") {
        let base = name
            .strip_suffix("_log")
            .or_else(|| name.strip_suffix("_wood"))
            .or_else(|| name.strip_suffix("_stem"))
            .unwrap_or(name);
        let top_name = format!("{}_log_top", base);
        let side_name = format!("{}_log", base);

        // Try stem versions for nether trees
        let top_name2 = format!("{}_stem_top", base);
        let side_name2 = format!("{}_stem", base);

        if let (Some(top), Some(side)) = (
            cache.load_texture(&top_name),
            cache.load_texture(&side_name),
        ) {
            return render_block_3d(&top, &side);
        }
        if let (Some(top), Some(side)) = (
            cache.load_texture(&top_name2),
            cache.load_texture(&side_name2),
        ) {
            return render_block_3d(&top, &side);
        }
        // Just the log texture
        if let Some(tex) = cache.load_texture(&side_name) {
            return render_block_3d(&tex, &tex);
        }
    }

    // Pattern 2: Water and lava (animated textures - use first frame)
    if name == "water" {
        if let Some(tex) = cache.load_animated_texture("water_still") {
            let tinted = tint_image(&tex, [63. / 255., 118. / 255., 228. / 255.]); // Water blue tint
            return render_block_3d(&tinted, &tinted);
        }
    }
    if name == "lava" {
        if let Some(tex) = cache.load_animated_texture("lava_still") {
            return render_block_3d(&tex, &tex);
        }
    }

    // Pattern 3: block_name (e.g., "stone.png")
    if let Some(tex) = cache.load_texture(name) {
        return render_block_3d(&tex, &tex);
    }

    // Pattern 3: block_name_top and block_name_side (e.g., "grass_block_top.png", "grass_block_side.png")
    let top_name = format!("{}_top", name);
    let side_name = format!("{}_side", name);
    if let (Some(top), Some(side)) = (
        cache.load_texture(&top_name),
        cache.load_texture(&side_name),
    ) {
        // Tint grass blocks green
        if name == "grass_block" {
            let tinted_top = tint_image(&top, [124. / 255., 189. / 255., 107. / 255.]);
            return render_block_3d(&tinted_top, &side);
        }
        return render_block_3d(&top, &side);
    }

    // Pattern 4: Just _top exists, use it for all faces
    if let Some(top) = cache.load_texture(&top_name) {
        return render_block_3d(&top, &top);
    }

    // Pattern 5: _planks suffix (e.g., oak -> oak_planks)
    let planks_name = format!("{}_planks", name);
    if let Some(tex) = cache.load_texture(&planks_name) {
        return render_block_3d(&tex, &tex);
    }

    // Pattern 6: _block suffix (e.g., diamond -> diamond_block)
    let block_name = format!("{}_block", name);
    if let Some(tex) = cache.load_texture(&block_name) {
        return render_block_3d(&tex, &tex);
    }

    // Pattern 7: Leaves
    if name.ends_with("_leaves") {
        if let Some(tex) = cache.load_texture(name) {
            // Tint leaves green
            let tinted = tint_image(&tex, [100. / 255., 180. / 255., 80. / 255.]);
            return render_block_3d(&tinted, &tinted);
        }
    }

    // Fallback: create a pink "missing texture" block
    eprintln!("Missing texture for block: {}", name);
    create_missing_block()
}

/// Create a "missing texture" block (pink/black checkerboard)
fn create_missing_block() -> RgbaImage {
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
    render_block_3d(&tex, &tex)
}

/// Render multiple chunks in a grid
/// chunk_range: (min_cx, min_cz, max_cx, max_cz) inclusive
/// get_block takes world coordinates (world_x, world_y, world_z)
pub fn render_world<F>(
    cache: &mut AssetCache,
    mut get_block: F,
    chunk_min: &WorldChunkCoord,
    chunk_max: &WorldChunkCoord,
    min_y: isize,
    max_y: isize,
) -> RgbaImage
where
    F: FnMut(&WorldBlockCoord) -> Option<String>,
{
    // Calculate world coordinate ranges
    let world_min = WorldBlockCoord {
        x: chunk_min.cx * MC_CHUNK_SIZE,
        y: min_y,
        z: chunk_min.cz * MC_CHUNK_SIZE,
    };
    let world_max = WorldBlockCoord {
        x: chunk_max.cx * MC_CHUNK_SIZE + MC_CHUNK_SIZE - 1,
        y: max_y - 1,
        z: chunk_max.cz * MC_CHUNK_SIZE + MC_CHUNK_SIZE - 1,
    };

    let world_width_x = world_max.x - world_min.x + 1;
    let world_width_z = world_max.z - world_min.z + 1;
    let total_height = world_max.y - world_min.y + 1;

    // Calculate output image size
    // Screen X is based on (world_x - world_z), range from min to max
    // Screen Y is based on (world_x + world_z) - y * 2
    let width = ((world_width_x + world_width_z) * 12) as u32;
    let height = ((world_width_x + world_width_z) * 6 + total_height * 12 + 24) as u32;

    println!(
        "Rendering world region: chunks ({}) to ({})",
        chunk_min, chunk_max
    );
    println!("World coords: ({}) to ({})", world_min, world_max);
    println!("Output image size: {}x{}", width, height);

    let mut img = RgbaImage::new(width, height);

    // Render from back to front, bottom to top (painter's algorithm)
    // For multiple chunks, we need to iterate in the correct order:
    // - Y from low to high
    // - Diagonal slices from back (high x+z) to front (low x+z)

    for block_coords in world_min.painters_range_to(&world_max) {
        if let Some(block_name) = get_block(&block_coords) {
            if !is_air_block(&block_name) {
                let sprite = get_block_sprite(cache, &block_name);

                // Calculate screen position
                // Normalize coordinates relative to the world minimum
                let rel_x = block_coords.x - world_min.x;
                let rel_z = block_coords.z - world_min.z;

                let screen_x =
                    ((rel_x - rel_z) * 12 + (width as isize / 2) - 12) as u32;
                let screen_y = ((rel_x + rel_z) * 6 - (block_coords.y - min_y) * 12
                    + (total_height * 12)) as u32;

                overlay(&mut img, &sprite, screen_x as i64, screen_y as i64);
            }
        }
    }

    img
}

enum BlockSpriteSide {
    SideLeft,
    SideRight,
}
