use crate::asset_cache::{AssetCache, BlockPartKey, BlockSpriteKey};
use crate::coords::block_face::BlockFace;
use crate::light_data::LightData;
use crate::render::renderer::SPRITE_SIZE;
use crate::render::transforms::{BlockSpriteSide, transform_side, transform_top};
use crate::utils::darken_image;
use image::imageops::{crop_imm, overlay};
use image::{Rgba, RgbaImage};

fn name_top_side(name: String, face: &BlockFace) -> String {
    match face {
        BlockFace::Top => format!("{}_top", name),
        BlockFace::South | BlockFace::East => format!("{}_side", name),
    }
}

fn load_face(cache: &AssetCache, face: BlockFace, mut name: String) -> RgbaImage {
    if name.starts_with("waxed_") {
        name = name.replace("waxed_", "");
    }

    let search_name = match name.as_str() {
        "lava" => "lava_still".to_string(),
        "water" => "water_still".to_string(),
        "grass_block" => match face {
            BlockFace::Top => "grass_block_top".to_string(),
            BlockFace::South | BlockFace::East => "grass_block_side".to_string(),
        },
        "dirt_path" => match face {
            BlockFace::Top => "dirt_path_top".to_string(),
            BlockFace::South | BlockFace::East => "dirt_path_side".to_string(),
        },
        "snow_block" => "snow".to_string(),
        "stripped_oak_wood" => match face {
            BlockFace::Top => "stripped_oak_log_top".to_string(),
            BlockFace::South | BlockFace::East => "stripped_oak_log".to_string(),
        },
        "vault" => match face {
            BlockFace::Top => "vault_top".to_string(),
            BlockFace::South | BlockFace::East => "vault_front_off".to_string(),
        },
        "lilac" => "lilac_top".to_string(), // TODO block states
        "rose_bush" => "rose_bush_bottom".to_string(), // TODO flowers
        "peony" => "peony_top".to_string(),
        "tall_seagrass" => "tall_seagrass_bottom".to_string(),
        "glass_pane" => "glass".to_string(), // TODO
        "hopper" => match face {
            BlockFace::Top => "hopper_top".to_string(),
            BlockFace::South | BlockFace::East => "hopper_outside".to_string(),
        },
        "bell" | "cauldron" | "stonecutter" | "composter" | "loom" | "hay_block" | "pumpkin"
        | "bee_nest" | "sculk_catalyst" | "sculk_sensor" | "sculk_shrieker" | "barrel"
        | "bone_block" => name_top_side(name, &face),
        "oak_door" => "oak_door_top".to_string(), // TODO
        "cobblestone_stairs" | "cobblestone_wall" => "cobblestone".to_string(), // TODO stairs
        "oak_stairs" | "oak_slab" | "oak_fence_gate" | "oak_pressure_plate" | "oak_button"
        | "oak_fence" => "oak_planks".to_string(), // TODO stairs
        "stone_brick_stairs" | "stone_brick_slab" => "stone_bricks".to_string(), // TODO stairs
        "mossy_stone_brick_stairs" | "mossy_stone_brick_slab" => "mossy_stone_bricks".to_string(), // TODO stairs
        "oxidized_cut_copper_slab" | "oxidized_cut_copper_stairs" => {
            "oxidized_cut_copper".to_string()
        }
        "oxidized_copper_door" => "oxidized_copper_door_top".to_string(),
        "wheat" => "wheat_stage7".to_string(), // TODO block states
        "carrots" => "carrots_stage3".to_string(), // TODO block states
        "beetroots" => "beetroots_stage3".to_string(), // TODO block states
        "potatoes" => "potatoes_stage3".to_string(),
        "dispenser" => "dispenser_front".to_string(),
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

    let img = match face {
        BlockFace::East => transform_side(&texture_img, BlockSpriteSide::SideRight),
        BlockFace::South => transform_side(&texture_img, BlockSpriteSide::SideLeft),
        BlockFace::Top => transform_top(&texture_img),
    };

    block_part_cache.insert(key, img.clone());

    img
}

/// Build a full isometric block from top and side textures
/// Returns a 24x24 image
pub fn render_block_3d(
    cache: &AssetCache,
    name: &str,
    plan: CubeSpritePlan,
    light_data: LightData,
) -> RgbaImage {
    // cache read
    let block_sprite_key = BlockSpriteKey {
        light: light_data.clone(),
        name: name.to_string(),
    };
    {
        let cache = cache.block_sprite_cache.read().unwrap();
        if let Some(img) = cache.get(&block_sprite_key) {
            return img.clone();
        }
    }
    // end cache read

    // lock writeable cache
    let mut block_sprite_cache = cache.block_sprite_cache.write().unwrap();

    let mut img = RgbaImage::new(SPRITE_SIZE, SPRITE_SIZE);

    // Read full-brightness block parts
    let top_transformed = load_face(cache, BlockFace::Top, plan.face_top);
    let side_left = load_face(cache, BlockFace::South, plan.face_south);
    let side_right = load_face(cache, BlockFace::East, plan.face_east);

    // darken block faces
    let top_transformed = darken_image(&top_transformed, light_data.factor(BlockFace::Top));
    let side_left = darken_image(&side_left, light_data.factor(BlockFace::South));
    let side_right = darken_image(&side_right, light_data.factor(BlockFace::East));

    // Composite: first the top at (0, 0), then left side at (0, 6), then right at (12, 6)
    overlay(&mut img, &top_transformed, 0, 0);
    overlay(&mut img, &side_left, 0, 6);
    overlay(&mut img, &side_right, 12, 6);

    // write to cache
    block_sprite_cache.insert(block_sprite_key, img.clone());

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
