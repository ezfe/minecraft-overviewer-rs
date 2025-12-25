use image::{
    Rgba, RgbaImage,
    imageops::{self, overlay},
};
use std::collections::HashMap;

const BLOCK_RENDER_SIZE: u32 = 24;
const BLOCK_SIDE_HEIGHT: u32 = 18;

/// Isometric renderer for Minecraft chunks/sections
pub struct IsometricRenderer {
    texture_cache: HashMap<String, RgbaImage>,
    block_cache: HashMap<String, RgbaImage>,
    assets_path: String,
}

impl IsometricRenderer {
    pub fn new(assets_path: &str) -> Self {
        Self {
            texture_cache: HashMap::new(),
            block_cache: HashMap::new(),
            assets_path: assets_path.to_string(),
        }
    }

    /// Load a texture from the assets folder
    pub fn load_texture(&mut self, texture_name: &str) -> Option<RgbaImage> {
        if let Some(cached) = self.texture_cache.get(texture_name) {
            return Some(cached.clone());
        }

        let path = format!(
            "{}/minecraft/textures/block/{}.png",
            self.assets_path, texture_name
        );

        if let Ok(img) = image::open(&path) {
            let rgba = img.to_rgba8();
            self.texture_cache
                .insert(texture_name.to_string(), rgba.clone());
            Some(rgba)
        } else {
            None
        }
    }

    /// Load an animated texture (extracts just the first 16x16 frame)
    pub fn load_animated_texture(&mut self, texture_name: &str) -> Option<RgbaImage> {
        let cache_key = format!("{}_frame0", texture_name);
        if let Some(cached) = self.texture_cache.get(&cache_key) {
            return Some(cached.clone());
        }

        let path = format!(
            "{}/minecraft/textures/block/{}.png",
            self.assets_path, texture_name
        );

        if let Ok(img) = image::open(&path) {
            let rgba = img.to_rgba8();
            // Animated textures have multiple frames stacked vertically
            // Extract just the first 16x16 frame
            let frame = imageops::crop_imm(&rgba, 0, 0, 16, 16).to_image();
            self.texture_cache.insert(cache_key, frame.clone());
            Some(frame)
        } else {
            None
        }
    }

    /// Transform a texture for the top face of an isometric block
    /// Rotates 45 degrees and scales Y by 0.5
    /// Output: 24x12 pixels
    pub fn transform_top(texture: &RgbaImage) -> RgbaImage {
        // Resize to 17x17 for better sampling
        let resized = imageops::resize(texture, 17, 17, imageops::FilterType::Triangle);

        // Create output image (24x12 for top face)
        let mut output = RgbaImage::new(24, 12);

        // The transformation matrix for isometric top view:
        // 1. Rotate 45 degrees
        // 2. Scale Y by 0.5 (compress vertically)
        //
        // For each pixel in the output, we calculate the corresponding source pixel
        // using inverse transformation
        let cos45 = std::f64::consts::FRAC_1_SQRT_2;
        let sin45 = std::f64::consts::FRAC_1_SQRT_2;

        for out_y in 0..12 {
            for out_x in 0..24 {
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
    pub fn transform_side_left(texture: &RgbaImage) -> RgbaImage {
        // Resize to 12x12
        let resized = imageops::resize(
            texture,
            BLOCK_RENDER_SIZE / 2,
            BLOCK_RENDER_SIZE / 2,
            imageops::FilterType::Triangle,
        );

        // Create output image (12x18 for side face after shear)
        let mut output = RgbaImage::new(BLOCK_RENDER_SIZE / 2, BLOCK_SIDE_HEIGHT);

        // Shear transformation: y_new = y + 0.5 * x
        // Inverse: y_src = y_out - 0.5 * x_out
        for out_y in 0..18 {
            for out_x in 0..12 {
                // Apply inverse shear to find source coordinates
                let src_x = out_x as f64;
                let src_y = out_y as f64 - 0.5 * out_x as f64;

                if src_y >= 0.0 && src_y < 12.0 {
                    let sx = src_x as u32;
                    let sy = src_y as u32;
                    if sx < 12 && sy < 12 {
                        let pixel = resized.get_pixel(sx, sy);
                        output.put_pixel(out_x, out_y, *pixel);
                    }
                }
            }
        }

        output
    }

    /// Transform a texture for the right side of an isometric block
    /// Mirror of left side
    pub fn transform_side_right(texture: &RgbaImage) -> RgbaImage {
        let left = Self::transform_side_left(texture);
        imageops::flip_horizontal(&left)
    }

    /// Darken an image by a factor (0.0 = black, 1.0 = original)
    pub fn darken(img: &RgbaImage, factor: f32) -> RgbaImage {
        let mut result = img.clone();
        for pixel in result.pixels_mut() {
            pixel[0] = (pixel[0] as f32 * factor) as u8;
            pixel[1] = (pixel[1] as f32 * factor) as u8;
            pixel[2] = (pixel[2] as f32 * factor) as u8;
            // Keep alpha unchanged
        }
        result
    }

    /// Build a full isometric block from top and side textures
    /// Returns a 24x24 image
    pub fn build_block(&self, top: &RgbaImage, side: &RgbaImage) -> RgbaImage {
        let mut img = RgbaImage::new(BLOCK_RENDER_SIZE, BLOCK_RENDER_SIZE);

        // Transform the top
        let top_transformed = Self::transform_top(top);

        // Transform the sides
        let side_left = Self::transform_side_left(side);
        let side_right = Self::transform_side_right(side);

        // Darken the sides (left 0.9, right 0.8)
        let side_left = Self::darken(&side_left, 0.9);
        let side_right = Self::darken(&side_right, 0.8);

        // Composite: first the top at (0, 0), then left side at (0, 6), then right at (12, 6)
        overlay(&mut img, &top_transformed, 0, 0);
        overlay(&mut img, &side_left, 0, 6);
        overlay(&mut img, &side_right, 12, 6);

        img
    }

    /// Get or create a rendered block sprite for a block name
    /// Block name should be like "minecraft:stone" or "minecraft:grass_block"
    pub fn get_block_sprite(&mut self, block_name: &str) -> RgbaImage {
        // Check cache first
        if let Some(cached) = self.block_cache.get(block_name) {
            return cached.clone();
        }

        // Strip minecraft: prefix if present
        let name = block_name.strip_prefix("minecraft:").unwrap_or(block_name);

        // Try to load textures based on common naming patterns
        let sprite = self.create_block_sprite(name);

        self.block_cache
            .insert(block_name.to_string(), sprite.clone());
        sprite
    }

    /// Create a block sprite from a block name
    fn create_block_sprite(&mut self, name: &str) -> RgbaImage {
        // Handle special cases first
        if name == "air" || name == "cave_air" || name == "void_air" {
            return RgbaImage::new(24, 24); // Transparent
        }

        // Handle transparent/partial blocks that we skip for now
        if name.contains("litter")
            || name.contains("sapling")
            || name.contains("flower")
            || name.contains("grass") && !name.contains("block")
            || name.contains("fern")
            || name.contains("dead_bush")
            || name.contains("seagrass")
            || name.contains("kelp")
            || name.contains("vine")
            || name.contains("lily_pad")
            || name.contains("torch")
            || name.contains("fire")
            || name.contains("redstone_wire")
            || name.contains("rail")
            || name.contains("ladder")
            || name.contains("lever")
            || name.contains("button")
            || name.contains("pressure_plate")
            || name.contains("tripwire")
            || name.contains("string")
            || name.contains("carpet") && !name.contains("moss")
            || name.contains("fence") && !name.contains("gate")
            || name.contains("wall") && !name.contains("sign")
            || name.contains("bars")
            || name.contains("chain")
            || name.contains("lantern")
            || name.contains("candle")
            || name.contains("rod")
            || name.contains("banner")
            || name.contains("sign")
            || name.contains("head")
            || name.contains("skull")
            || name.contains("dripstone")
            || name.contains("pointed")
            || name.contains("amethyst_cluster")
            || name.contains("amethyst_bud")
        {
            return RgbaImage::new(24, 24); // Transparent for now - complex geometry
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

            if let (Some(top), Some(side)) =
                (self.load_texture(&top_name), self.load_texture(&side_name))
            {
                return self.build_block(&top, &side);
            }
            if let (Some(top), Some(side)) = (
                self.load_texture(&top_name2),
                self.load_texture(&side_name2),
            ) {
                return self.build_block(&top, &side);
            }
            // Just the log texture
            if let Some(tex) = self.load_texture(&side_name) {
                return self.build_block(&tex, &tex);
            }
        }

        // Pattern 2: Water and lava (animated textures - use first frame)
        if name == "water" {
            if let Some(tex) = self.load_animated_texture("water_still") {
                let tinted = Self::tint_image(&tex, [63, 118, 228]); // Water blue tint
                return self.build_block(&tinted, &tinted);
            }
        }
        if name == "lava" {
            if let Some(tex) = self.load_animated_texture("lava_still") {
                return self.build_block(&tex, &tex);
            }
        }

        // Pattern 3: block_name (e.g., "stone.png")
        if let Some(tex) = self.load_texture(name) {
            return self.build_block(&tex, &tex);
        }

        // Pattern 3: block_name_top and block_name_side (e.g., "grass_block_top.png", "grass_block_side.png")
        let top_name = format!("{}_top", name);
        let side_name = format!("{}_side", name);
        if let (Some(top), Some(side)) =
            (self.load_texture(&top_name), self.load_texture(&side_name))
        {
            // Tint grass blocks green
            if name == "grass_block" {
                let tinted_top = Self::tint_image(&top, [124, 189, 107]);
                return self.build_block(&tinted_top, &side);
            }
            return self.build_block(&top, &side);
        }

        // Pattern 4: Just _top exists, use it for all faces
        if let Some(top) = self.load_texture(&top_name) {
            return self.build_block(&top, &top);
        }

        // Pattern 5: _planks suffix (e.g., oak -> oak_planks)
        let planks_name = format!("{}_planks", name);
        if let Some(tex) = self.load_texture(&planks_name) {
            return self.build_block(&tex, &tex);
        }

        // Pattern 6: _block suffix (e.g., diamond -> diamond_block)
        let block_name = format!("{}_block", name);
        if let Some(tex) = self.load_texture(&block_name) {
            return self.build_block(&tex, &tex);
        }

        // Pattern 7: Leaves
        if name.ends_with("_leaves") {
            if let Some(tex) = self.load_texture(name) {
                // Tint leaves green
                let tinted = Self::tint_image(&tex, [100, 180, 80]);
                return self.build_block(&tinted, &tinted);
            }
        }

        // Fallback: create a pink "missing texture" block
        eprintln!("Missing texture for block: {}", name);
        self.create_missing_block()
    }

    /// Tint an image with a color (for grass, leaves, etc.)
    fn tint_image(img: &RgbaImage, tint: [u8; 3]) -> RgbaImage {
        let mut result = img.clone();
        for pixel in result.pixels_mut() {
            // Multiply the RGB channels by the tint color
            pixel[0] = ((pixel[0] as u32 * tint[0] as u32) / 255) as u8;
            pixel[1] = ((pixel[1] as u32 * tint[1] as u32) / 255) as u8;
            pixel[2] = ((pixel[2] as u32 * tint[2] as u32) / 255) as u8;
        }
        result
    }

    /// Create a "missing texture" block (pink/black checkerboard)
    fn create_missing_block(&self) -> RgbaImage {
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
        self.build_block(&tex, &tex)
    }

    /// Render a full 16x16x16 section
    /// Returns the rendered image with proper isometric layering
    pub fn render_section<F>(&mut self, get_block: F) -> RgbaImage
    where
        F: Fn(usize, usize, usize) -> Option<String>,
    {
        // Calculate output image size
        // In isometric view:
        // - X increases: moves right and down
        // - Z increases: moves left and down
        // - Y increases: moves up
        //
        // For a 16x16x16 section:
        // Width: 16 blocks * 24 pixels wide = 384, but blocks overlap, so roughly 16*12 + 16*12 = 384
        // Height: 16 blocks high * 12 pixels each + base = needs calculation
        //
        // More precisely:
        // - Each block is 24x24 but the top is 24x12
        // - Horizontal: spans from col = -15 to col = +15 (for x=0..15, z=0..15)
        //   where col = x - z, so total width ~ 32 * 12 = 384
        // - Vertical: spans from y=0 to y=15, plus the block heights

        let width = 16 * 12 + 16 * 12; // = 384
        let height = 16 * 12 + 16 * 6 + 24; // 16 blocks of Y height + base + extra

        let mut img = RgbaImage::new(width, height);

        // Render from back to front, bottom to top (painter's algorithm)
        // Back to front: high Z to low Z for back, low X to high X for front
        // Bottom to top: low Y to high Y

        for y in 0..16 {
            // Render in diagonal slices from back-left to front-right
            for sum in 0..32 {
                // sum = x + z, from 0 to 30
                for x in 0..=sum {
                    let z = sum - x;
                    if x < 16 && z < 16 {
                        if let Some(block_name) = get_block(x, y, z) {
                            if block_name != "minecraft:air"
                                && block_name != "minecraft:cave_air"
                                && block_name != "minecraft:void_air"
                            {
                                let sprite = self.get_block_sprite(&block_name);

                                // Calculate screen position
                                // Isometric projection: col = x - z, row = x + z
                                // Then adjust for Y height
                                let screen_x = (x - z) as u32 * 12 + (width / 2) - 12;
                                let screen_y =
                                    (x + z) as u32 * 6 - (y as u32) * 12 + (height - 16 * 6 - 24);

                                overlay(&mut img, &sprite, screen_x as i64, screen_y as i64);
                            }
                        }
                    }
                }
            }
        }

        img
    }

    /// Render an entire chunk (all Y levels from min_y to max_y)
    /// Returns the rendered image with proper isometric layering
    pub fn render_chunk<F>(&mut self, get_block: F, min_y: isize, max_y: isize) -> RgbaImage
    where
        F: Fn(isize, isize, isize) -> Option<String>,
    {
        let total_height = max_y - min_y;

        // Calculate output image size
        // Width: same as section (16 blocks in X and Z)
        // Height: needs to accommodate all Y levels
        let width = 16 * 12 + 16 * 12; // = 384

        // Height calculation:
        // - Base plane at lowest Y: 16*6 pixels for the X+Z diagonal
        // - Each Y level adds 12 pixels of height
        // - Plus block height (24 pixels for the topmost blocks)
        let height = ((total_height * 12) + (16 * 6) + 24) as u32;

        let mut img = RgbaImage::new(width, height);

        // Render from back to front, bottom to top (painter's algorithm)
        for y in min_y..max_y {
            // Render in diagonal slices from back-left to front-right
            for sum in 0..32isize {
                for x in 0..=sum {
                    let z = sum - x;
                    if x < 16 && z < 16 {
                        if let Some(block_name) = get_block(x, y, z) {
                            if block_name != "minecraft:air"
                                && block_name != "minecraft:cave_air"
                                && block_name != "minecraft:void_air"
                            {
                                let sprite = self.get_block_sprite(&block_name);

                                // Calculate screen position
                                let screen_x = (x - z) as u32 * 12 + (width / 2) - 12;
                                let screen_y = (x + z) as u32 * 6 - (y - min_y) as u32 * 12
                                    + (height - 16 * 6 - 24);

                                overlay(&mut img, &sprite, screen_x as i64, screen_y as i64);
                            }
                        }
                    }
                }
            }
        }

        img
    }

    /// Render multiple chunks in a grid
    /// chunk_range: (min_cx, min_cz, max_cx, max_cz) inclusive
    /// get_block takes world coordinates (world_x, world_y, world_z)
    pub fn render_world<F>(
        &mut self,
        get_block: F,
        chunk_min_x: isize,
        chunk_min_z: isize,
        chunk_max_x: isize,
        chunk_max_z: isize,
        min_y: isize,
        max_y: isize,
    ) -> RgbaImage
    where
        F: Fn(isize, isize, isize) -> Option<String>,
    {
        // Calculate world coordinate ranges
        let world_min_x = chunk_min_x * 16;
        let world_max_x = chunk_max_x * 16 + 15;
        let world_min_z = chunk_min_z * 16;
        let world_max_z = chunk_max_z * 16 + 15;

        let world_width_x = (world_max_x - world_min_x + 1) as u32;
        let world_width_z = (world_max_z - world_min_z + 1) as u32;
        let total_height = (max_y - min_y) as u32;

        // Calculate output image size
        // Screen X is based on (world_x - world_z), range from min to max
        // Screen Y is based on (world_x + world_z) - y * 2
        let width = (world_width_x + world_width_z) * 12;
        let height = (world_width_x + world_width_z) * 6 + total_height * 12 + 24;

        println!(
            "Rendering world region: chunks ({},{}) to ({},{})",
            chunk_min_x, chunk_min_z, chunk_max_x, chunk_max_z
        );
        println!(
            "World coords: ({},{}) to ({},{}), Y: {} to {}",
            world_min_x, world_min_z, world_max_x, world_max_z, min_y, max_y
        );
        println!("Output image size: {}x{}", width, height);

        let mut img = RgbaImage::new(width, height);

        // Render from back to front, bottom to top (painter's algorithm)
        // For multiple chunks, we need to iterate in the correct order:
        // - Y from low to high
        // - Diagonal slices from back (high x+z) to front (low x+z)

        for y in min_y..max_y {
            // Render in diagonal slices
            // sum = world_x + world_z, from max to min (back to front)
            let min_sum = world_min_x + world_min_z;
            let max_sum = world_max_x + world_max_z;

            for sum in min_sum..=max_sum {
                // For each diagonal, iterate through valid (x, z) pairs
                for world_x in world_min_x..=world_max_x {
                    let world_z = sum - world_x;
                    if world_z >= world_min_z && world_z <= world_max_z {
                        if let Some(block_name) = get_block(world_x, y, world_z) {
                            if block_name != "minecraft:air"
                                && block_name != "minecraft:cave_air"
                                && block_name != "minecraft:void_air"
                            {
                                let sprite = self.get_block_sprite(&block_name);

                                // Calculate screen position
                                // Normalize coordinates relative to the world minimum
                                let rel_x = world_x - world_min_x;
                                let rel_z = world_z - world_min_z;

                                let screen_x =
                                    ((rel_x - rel_z) * 12 + (width as isize / 2) - 12) as u32;
                                let screen_y = ((rel_x + rel_z) * 6 - (y - min_y) * 12
                                    + (height as isize
                                        - (world_width_x + world_width_z) as isize * 6 / 2
                                        - 24))
                                    as u32;

                                overlay(&mut img, &sprite, screen_x as i64, screen_y as i64);
                            }
                        }
                    }
                }
            }
        }

        img
    }
}
