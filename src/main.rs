use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self},
    io::Result,
    path::PathBuf,
};

use crate::{
    asset_cache::AssetCache,
    chunk::Chunk,
    coords::{WorldBlockCoord, WorldChunkCoord},
};
use crate::{region::read_chunk, renderer::render_world};

mod asset_cache;
mod blocks;
mod chunk;
mod coords;
mod region;
mod renderer;
mod section;
mod utils;

/// A collection of loaded chunks indexed by (chunk_x, chunk_z)
struct ChunkStore {
    chunks: HashMap<WorldChunkCoord, Chunk>,
}

impl ChunkStore {
    fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    fn insert(&mut self, coord: WorldChunkCoord, chunk: Chunk) {
        self.chunks.insert(coord, chunk);
    }

    fn get(&self, coord: WorldChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    /// Get block at world coordinates
    fn get_block_at(&self, block_coords: &WorldBlockCoord) -> Option<String> {
        let chunk = self.get(block_coords.chunk_coord())?;

        let local_coords = block_coords.chunk_local_coord();

        let section = chunk
            .sections
            .iter()
            .find(|s| s.y == block_coords.chunk_y_section())?;

        section.block_at(local_coords).map(|p| p.name.clone())
    }

    /// Get the Y range across all loaded chunks
    fn get_y_range(&self) -> (isize, isize) {
        let mut found_any = false;
        let mut min_y = isize::MAX;
        let mut max_y = isize::MIN;

        for chunk in self.chunks.values() {
            for section in &chunk.sections {
                found_any = true;
                let section_min = section.y as isize * 16;
                let section_max = section_min + 16;
                min_y = min_y.min(section_min);
                max_y = max_y.max(section_max);
            }
        }

        if !found_any {
            (0, 256) // Default range
        } else {
            (min_y, max_y)
        }
    }
}

fn main() -> Result<()> {
    const SOURCE: &str = "sample_map/region";
    const ASSETS: &str = "assets";

    // Define the 3x3 chunk grid centered at (0, 0)
    let chunk_min_x: isize = -1;
    let chunk_max_x: isize = 1;
    let chunk_min_z: isize = -1;
    let chunk_max_z: isize = 1;

    println!(
        "Loading chunks from ({},{}) to ({},{})",
        chunk_min_x, chunk_min_z, chunk_max_x, chunk_max_z
    );

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

    for cx in chunk_min_x..=chunk_max_x {
        for cz in chunk_min_z..=chunk_max_z {
            let chunk_coord = WorldChunkCoord { cx, cz };

            // Calculate which region file this chunk is in
            let region_coord = chunk_coord.region_coord();
            let region_name = region_coord.file_name();

            // Find the region file
            let region_path = region_files
                .iter()
                .find(|p| p.file_name() == Some(OsStr::new(&region_name)));

            if let Some(path) = region_path {
                if let Some(chunk) = read_chunk(path.clone(), &chunk_coord) {
                    println!("Loaded chunk ({})", chunk_coord);
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

    println!("Rendering 3x3 chunk region...");

    // Render all chunks
    let img = render_world(
        &mut asset_cache,
        |coords| store.get_block_at(coords),
        chunk_min_x,
        chunk_min_z,
        chunk_max_x,
        chunk_max_z,
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
