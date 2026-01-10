use std::collections::HashMap;
use std::sync::RwLock;

use crate::coords::block_face::BlockFace;
use crate::light_data::LightData;
use image::RgbaImage;

#[derive(Hash, Eq, PartialEq)]
pub struct BlockPartKey {
    pub face: BlockFace,
    pub name: String,
}

#[derive(Hash, Eq, PartialEq)]
pub struct BlockSpriteKey {
    pub light: LightData,
    // pub light: u8,
    pub name: String,
}

pub struct AssetCache {
    pub texture_cache: RwLock<HashMap<String, RgbaImage>>,
    pub block_part_cache: RwLock<HashMap<BlockPartKey, RgbaImage>>,
    pub block_sprite_cache: RwLock<HashMap<BlockSpriteKey, RgbaImage>>,
    pub assets_path: String,
}

impl AssetCache {
    pub fn new(assets_path: &str) -> Self {
        Self {
            texture_cache: RwLock::new(HashMap::new()),
            block_part_cache: RwLock::new(HashMap::new()),
            block_sprite_cache: RwLock::new(HashMap::new()),
            assets_path: assets_path.to_string(),
        }
    }

    pub fn load_texture(&self, texture_name: &str) -> Option<RgbaImage> {
        {
            let cache = self.texture_cache.read().unwrap();
            if let Some(cached) = cache.get(texture_name) {
                return Some(cached.clone());
            }
        }

        let path = format!(
            "{}/minecraft/textures/block/{}.png",
            self.assets_path, texture_name
        );

        if let Ok(img) = image::open(&path) {
            let rgba = img.to_rgba8();
            let mut cache = self.texture_cache.write().unwrap();
            cache.insert(texture_name.to_string(), rgba.clone());
            Some(rgba)
        } else {
            println!("texture {} not found", texture_name);
            None
        }
    }
}
