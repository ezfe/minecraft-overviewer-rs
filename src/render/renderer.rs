use crate::light_data::LightData;
use crate::render::render_cube::{CubeSpritePlan, render_block_3d};
use crate::{
    asset_cache::AssetCache,
    blocks::is_air_block,
    chunk_store::ChunkStore,
    coords::{
        constants::MC_CHUNK_SIZE, world_block_coord::WorldBlockCoord,
        world_chunk_coord::WorldChunkCoord,
    },
};
use image::{RgbaImage, imageops::overlay};
use rayon::prelude::*;

pub const SPRITE_SIZE: u32 = 24;

/// Get or create a rendered block sprite for a block name
/// Block name should be like "minecraft:stone" or "minecraft:grass_block"
pub fn get_block_sprite(cache: &AssetCache, block_name: &str, light_data: LightData) -> RgbaImage {
    let name = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    let sprite = create_block_sprite(cache, name, light_data);
    sprite
}

/// Translate a palette block name into 3 texture names
fn plan_block_sprite(block_name: &str) -> Option<CubeSpritePlan> {
    let name = block_name.strip_prefix("minecraft:").unwrap_or(block_name);

    // TODO: Cache block sprite plans?

    if name == "air" || name == "cave_air" || name == "void_air" {
        return None;
    }

    Some(CubeSpritePlan {
        face_east: name.to_string(),
        face_south: name.to_string(),
        face_top: name.to_string(),
    })
}

/// Create a block sprite from a block name
fn create_block_sprite(cache: &AssetCache, name: &str, light_data: LightData) -> RgbaImage {
    let cube_plan = plan_block_sprite(name);

    match cube_plan {
        None => RgbaImage::new(SPRITE_SIZE, SPRITE_SIZE),
        Some(cube_plan) => render_block_3d(cache, cube_plan, light_data),
    }
}

/// Render multiple chunks in a grid
/// chunk_range: (min_cx, min_cz, max_cx, max_cz) inclusive
/// get_block takes world coordinates (world_x, world_y, world_z)
pub fn render_world(
    cache: &AssetCache,
    store: &ChunkStore,
    chunk_min: &WorldChunkCoord,
    chunk_max: &WorldChunkCoord,
    min_y: isize,
    max_y: isize,
) -> RgbaImage {
    let chunk_width_x = chunk_max.cx - chunk_min.cx + 1;
    let chunk_width_z = chunk_max.cz - chunk_min.cz + 1;

    let total_height = max_y - min_y + 1;

    // Calculate output image size
    let xz_area_factor = MC_CHUNK_SIZE * (chunk_width_x + chunk_width_z) * 12;
    let y_area_factor = total_height * 12;
    let width = xz_area_factor as u32;
    let height = (xz_area_factor + y_area_factor + 24) as u32;

    println!(
        "Rendering world region: chunks ({}) to ({})",
        chunk_min, chunk_max
    );
    println!(
        "World coords: ({}) to ({})",
        chunk_min.world_block_coord_min(min_y),
        chunk_max.world_block_coord_max(max_y)
    );
    println!("Output image size: {}x{}", width, height);

    let mut img = RgbaImage::new(width, height);

    // Render from back to front, bottom to top (painter's algorithm)
    // For multiple chunks, we need to iterate in the correct order:
    // - Y from low to high
    // - Diagonal slices from back (high x+z) to front (low x+z)

    let chunk_coords: Vec<WorldChunkCoord> = chunk_min.painters_range_to(chunk_max).collect();
    let chunk_renders: Vec<ChunkRenderResult> = chunk_coords
        .par_iter()
        .map(|chunk_coord| {
            render_chunk(
                cache,
                |coords| store.get_block_at(coords),
                |coords| store.get_block_light_at(coords),
                *chunk_coord,
                min_y,
                max_y,
            )
        })
        .collect();

    for chunk_render in chunk_renders {
        let chunk_pos = img_coords(
            chunk_render.img.width(),
            chunk_render.coord.world_block_coord_min(min_y),
            chunk_render.coord.world_block_coord_max(max_y),
            chunk_render.coord.world_block_coord_min(min_y),
        );
        let screen_pos = img_coords(
            img.width(),
            chunk_min.world_block_coord_min(min_y),
            chunk_max.world_block_coord_max(max_y),
            chunk_render.coord.world_block_coord_min(min_y),
        );

        let screen_x = screen_pos.0 - chunk_pos.0;
        let screen_y = screen_pos.1 - chunk_pos.1;

        overlay(
            &mut img,
            &chunk_render.img,
            screen_x as i64,
            screen_y as i64,
        );
    }

    img
}

struct ChunkRenderResult {
    coord: WorldChunkCoord,
    img: RgbaImage,
}

fn render_chunk<F, FL>(
    cache: &AssetCache,
    mut get_block: F,
    mut get_block_light: FL,
    chunk_coord: WorldChunkCoord,
    min_y: isize,
    max_y: isize,
) -> ChunkRenderResult
where
    F: FnMut(&WorldBlockCoord) -> Option<String>,
    FL: FnMut(&WorldBlockCoord) -> Option<u8>,
{
    // Calculate world coordinate ranges
    let world_min = chunk_coord.world_block_coord_min(min_y);
    let world_max = chunk_coord.world_block_coord_max(max_y);

    let total_height = max_y - min_y + 1;

    // Calculate output image size
    let width = (MC_CHUNK_SIZE * 24) as u32;
    let height = (MC_CHUNK_SIZE * 12 + total_height * 12 + 24) as u32;

    let mut img = RgbaImage::new(width, height);

    for block_coord in world_min.painters_range_to(&world_max) {
        if let Some(block_name) = get_block(&block_coord) {
            if !is_air_block(&block_name) {
                let light_info = LightData {
                    light_top: get_block_light(&block_coord.top_pos_y()).unwrap_or(0),
                    light_east: get_block_light(&block_coord.east_pos_x()).unwrap_or(0),
                    light_south: get_block_light(&block_coord.south_pos_z()).unwrap_or(0),
                };

                let sprite = get_block_sprite(cache, &block_name, light_info);

                let screen_pos = img_coords(width, world_min, world_max, block_coord);
                overlay(&mut img, &sprite, screen_pos.0 as i64, screen_pos.1 as i64);
            }
        }
    }

    ChunkRenderResult {
        coord: chunk_coord,
        img,
    }
}

fn img_coords(
    img_width: u32,
    world_min: WorldBlockCoord,
    world_max: WorldBlockCoord,
    anchor_coord: WorldBlockCoord,
) -> (u32, u32) {
    let total_height = world_max.y - world_min.y + 1;

    // Calculate screen position
    // Normalize coordinates relative to the world minimum
    let rel_x = anchor_coord.x - world_min.x;
    let rel_z = anchor_coord.z - world_min.z;

    let screen_x = ((rel_x - rel_z) * 12 + (img_width as isize / 2) - 12) as u32;
    let screen_y =
        ((rel_x + rel_z) * 6 - (anchor_coord.y - world_min.y) * 12 + (total_height * 12)) as u32;

    (screen_x, screen_y)
}
