use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self},
    io::Result,
    path::PathBuf,
};

use crate::chunk::Chunk;
use crate::region::read_chunk;
use crate::renderer::IsometricRenderer;

mod chunk;
mod region;
mod renderer;
mod section;

/// A collection of loaded chunks indexed by (chunk_x, chunk_z)
struct ChunkStore {
    chunks: HashMap<(isize, isize), Chunk>,
}

impl ChunkStore {
    fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    fn insert(&mut self, cx: isize, cz: isize, chunk: Chunk) {
        self.chunks.insert((cx, cz), chunk);
    }

    fn get(&self, cx: isize, cz: isize) -> Option<&Chunk> {
        self.chunks.get(&(cx, cz))
    }

    /// Get block at world coordinates
    fn get_block_at(&self, world_x: isize, world_y: isize, world_z: isize) -> Option<String> {
        // Calculate chunk coordinates
        let cx = world_x.div_euclid(16);
        let cz = world_z.div_euclid(16);

        let chunk = self.get(cx, cz)?;

        // Calculate local coordinates within chunk
        let local_x = world_x.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;

        // Find the section for this Y level
        let section_y = world_y.div_euclid(16) as i8;
        let section = chunk.sections.iter().find(|s| s.y == section_y)?;

        // Get block within section
        let local_y = world_y.rem_euclid(16) as usize;
        section
            .block_at(local_x, local_y, local_z)
            .map(|p| p.name.clone())
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
            // Calculate which region file this chunk is in
            let region_x = cx.div_euclid(32);
            let region_z = cz.div_euclid(32);
            let region_name = format!("r.{}.{}.mca", region_x, region_z);

            // Find the region file
            let region_path = region_files
                .iter()
                .find(|p| p.file_name() == Some(OsStr::new(&region_name)));

            if let Some(path) = region_path {
                if let Some(chunk) = read_chunk(path.clone(), cx, cz) {
                    println!("Loaded chunk ({}, {})", cx, cz);
                    store.insert(cx, cz, chunk);
                } else {
                    println!("Chunk ({}, {}) not found in region", cx, cz);
                }
            } else {
                println!(
                    "Region file {} not found for chunk ({}, {})",
                    region_name, cx, cz
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
    let mut renderer = IsometricRenderer::new(ASSETS);

    println!("Rendering 3x3 chunk region...");

    // Render all chunks
    let img = renderer.render_world(
        |x, y, z| store.get_block_at(x, y, z),
        chunk_min_x,
        chunk_min_z,
        chunk_max_x,
        chunk_max_z,
        min_y,
        max_y,
    );

    // Save the rendered image
    let output_path = "output_world.png";
    img.save(output_path).expect("Failed to save image");
    println!(
        "Rendered world saved to {} ({}x{} pixels)",
        output_path,
        img.width(),
        img.height()
    );

    Ok(())
}
