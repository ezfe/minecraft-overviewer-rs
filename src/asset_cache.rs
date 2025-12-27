use std::collections::HashMap;

use image::{RgbaImage, imageops};

pub struct AssetCache {
    pub texture_cache: HashMap<String, RgbaImage>,
    pub block_cache: HashMap<String, RgbaImage>,
    pub assets_path: String,
}

impl AssetCache {
    pub fn new(assets_path: &str) -> Self {
        Self {
            texture_cache: HashMap::new(),
            block_cache: HashMap::new(),
            assets_path: assets_path.to_string(),
        }
    }

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
}
