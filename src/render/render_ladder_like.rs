use crate::asset_cache::{AssetCache, BlockPartKey, BlockSpriteKey, LadderSpriteKey};
use crate::coords::block_face::BlockFace;
use crate::light_data::LightData;
use crate::render::missing_block_tex::create_missing_block_texture;
use crate::render::renderer::SPRITE_SIZE;
use crate::render::transforms::transform;
use crate::utils::darken_image;
use image::RgbaImage;
use image::imageops::{crop_imm, overlay};

pub struct LadderLikePlan {
    pub face: BlockFace,
    pub name: String,
}

fn load_face(cache: &AssetCache, face: &BlockFace, mut name: String) -> RgbaImage {
    if name.starts_with("waxed_") {
        name = name.replace("waxed_", "");
    }

    let search_name = match name.as_str() {
        _ => name,
    };

    let key = BlockPartKey {
        face: face.clone(),        // TODO
        name: search_name.clone(), // TODO
    };
    {
        let block_part_cache = cache.block_part_cache.read().unwrap();
        if let Some(cached) = block_part_cache.get(&key) {
            return cached.clone();
        }
    }

    let mut block_part_cache = cache.block_part_cache.write().unwrap();

    let mut texture_img = cache
        .load_texture(search_name.as_str())
        .unwrap_or(create_missing_block_texture());

    if texture_img.width() > 16 || texture_img.height() > 16 {
        texture_img = crop_imm(&texture_img, 0, 0, 16, 16).to_image();
    }

    let img = transform(&texture_img, face);

    block_part_cache.insert(key, img.clone());

    img
}

pub fn render_ladder_like(
    cache: &AssetCache,
    name: &str,
    plan: LadderLikePlan,
    light_data: LightData,
) -> RgbaImage {
    // cache read
    let block_sprite_key = LadderSpriteKey {
        face: plan.face.clone(),
        name: name.to_string(),
    };
    {
        let cache = cache.ladder_like_cache.read().unwrap();
        if let Some(img) = cache.get(&block_sprite_key) {
            return img.clone();
        }
    }
    // end cache read

    // lock writeable cache
    let mut ladder_cache = cache.ladder_like_cache.write().unwrap();

    let mut img = RgbaImage::new(SPRITE_SIZE, SPRITE_SIZE);

    // Read block part
    let face_img = load_face(cache, &plan.face, plan.name);

    // Composite images
    match plan.face {
        BlockFace::Top => overlay(&mut img, &face_img, 0, 0),
        BlockFace::Bottom => {} // TODO
        BlockFace::North => overlay(&mut img, &face_img, 12, 0),
        BlockFace::South => overlay(&mut img, &face_img, 0, 6),
        BlockFace::East => overlay(&mut img, &face_img, 12, 6),
        BlockFace::West => overlay(&mut img, &face_img, 0, 0),
    }

    // darken block
    let img = darken_image(&img, light_data.factor_internal());

    // write to cache
    ladder_cache.insert(block_sprite_key, img.clone());

    img
}
