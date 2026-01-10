use crate::coords::world_block_coord::WorldBlockCoord;
use crate::region::read_chunk;
use crate::{
    asset_cache::AssetCache, chunk_store::ChunkStore, coords::world_chunk_coord::WorldChunkCoord,
};
use render::renderer::render_world;
use std::{
    ffi::OsStr,
    fs::{self},
    io::Result,
    path::PathBuf,
};

mod asset_cache;
mod blocks;
mod chunk;
mod chunk_store;
mod coords;
mod light_data;
mod region;
mod render;
mod section;
mod utils;

fn main() -> Result<()> {
    const SOURCE: &str = "sample_map/region";
    const ASSETS: &str = "assets";

    // Define the 3x3 chunk grid centered at (0, 0)
    let r = 20;
    let chunk_min = WorldChunkCoord {
        cx: 0 - r,
        cz: 0 - r,
    };
    let chunk_max = WorldChunkCoord {
        cx: 0 + r,
        cz: 0 + r,
    };

    println!("Loading chunks from ({}) to ({})", chunk_min, chunk_max);

    // Collect all region files
    let region_files: Vec<PathBuf> = fs::read_dir(SOURCE)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("mca")) {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    println!("Found {} region files", region_files.len());

    // Load all chunks in the range
    let mut store = ChunkStore::new();

    for chunk_coord in chunk_min.range_to(&chunk_max) {
        // Calculate which region file this chunk is in
        let region_coord = chunk_coord.region_coord();
        let region_name = region_coord.file_name();

        // Find the region file
        let region_path = region_files
            .iter()
            .find(|p| p.file_name() == Some(OsStr::new(&region_name)));

        if let Some(path) = region_path {
            if let Some(chunk) = read_chunk(path.clone(), &chunk_coord) {
                store.insert(chunk_coord, chunk);
            } else {
                println!("Chunk ({}) not found in region", chunk_coord);
            }
        } else {
            println!(
                "Region file {} not found for chunk ({})",
                region_name, chunk_coord
            );
        }
    }

    println!("Loaded {} chunks total", store.chunks.len());

    if store.chunks.is_empty() {
        println!("No chunks loaded, exiting");
        return Ok(());
    }

    // Get Y range from loaded chunks
    let (min_y, max_y) = store.get_y_range();
    println!("Y range across all chunks: {} to {}", min_y, max_y);

    // Create the isometric renderer
    let mut asset_cache = AssetCache::new(ASSETS);

    println!("Rendering chunk region...");

    // Render all chunks
    let img = render_world(
        &mut asset_cache,
        &mut store,
        &chunk_min,
        &chunk_max,
        min_y,
        max_y,
    );

    // Save the rendered image
    let output_path = "out/world.png";
    img.save(output_path).expect("Failed to save image");
    println!(
        "Rendered world saved to {} ({}x{} pixels)",
        output_path,
        img.width(),
        img.height()
    );

    Ok(())
}
