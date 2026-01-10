use crate::asset_cache::{AssetCache, BlockPartKey};
use crate::coords::block_face::BlockFace;
use crate::light_data::LightData;
use crate::render::renderer::SPRITE_SIZE;
use crate::render::transforms::{BlockSpriteSide, transform_side, transform_top};
use crate::utils::darken_image;
use image::imageops::overlay;
use image::{Rgba, RgbaImage};

fn load_face(cache: &AssetCache, face: BlockFace, name: String) -> RgbaImage {
    let key = BlockPartKey {
        face: face.clone(), // TODO
        name: name.clone(), // TODO
    };
    {
        let block_part_cache = cache.block_part_cache.read().unwrap();
        if let Some(cached) = block_part_cache.get(&key) {
            return cached.clone();
        }
    }

    let texture_img = cache
        .load_texture(name.as_str())
        .unwrap_or(create_missing_block_texture());

    let img = match face {
        BlockFace::East => transform_side(&texture_img, BlockSpriteSide::SideRight),
        BlockFace::South => transform_side(&texture_img, BlockSpriteSide::SideLeft),
        BlockFace::Top => transform_top(&texture_img),
    };

    let mut block_part_cache = cache.block_part_cache.write().unwrap();
    block_part_cache.insert(key, img.clone());

    img
}

/// Build a full isometric block from top and side textures
/// Returns a 24x24 image
pub fn render_block_3d(
    cache: &AssetCache,
    plan: CubeSpritePlan,
    light_data: LightData,
) -> RgbaImage {
    let mut img = RgbaImage::new(SPRITE_SIZE, SPRITE_SIZE);

    // Read full-brightness block parts
    let top_transformed = load_face(cache, BlockFace::Top, plan.face_top);
    let side_left = load_face(cache, BlockFace::South, plan.face_south);
    let side_right = load_face(cache, BlockFace::East, plan.face_east);

    let top_transformed = darken_image(&top_transformed, light_data.factor(BlockFace::Top));
    let side_left = darken_image(&side_left, light_data.factor(BlockFace::South));
    let side_right = darken_image(&side_right, light_data.factor(BlockFace::East));

    // Composite: first the top at (0, 0), then left side at (0, 6), then right at (12, 6)
    overlay(&mut img, &top_transformed, 0, 0);
    overlay(&mut img, &side_left, 0, 6);
    overlay(&mut img, &side_right, 12, 6);

    img
}

pub struct CubeSpritePlan {
    pub face_east: String,
    pub face_south: String,
    pub face_top: String,
}

/// Create a "missing texture" block (pink/black checkerboard)
fn create_missing_block_texture() -> RgbaImage {
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
