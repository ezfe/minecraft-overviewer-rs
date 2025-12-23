use std::{
    ffi::OsStr,
    fs::{self},
    io::Result,
};

use crate::chunk::Chunk;
use crate::region::read_chunk;
use crate::renderer::IsometricRenderer;

mod chunk;
mod region;
mod renderer;
mod section;

fn main() -> Result<()> {
    const SOURCE: &str = "sample_map/region";
    const ASSETS: &str = "assets";

    let c_x: i32 = 0;
    let c_z: i32 = 0;

    // Calculate region coordinates from chunk coordinates
    let region_x = c_x / 32;
    let region_z = c_z / 32;
    let expected_region = format!("r.{}.{}.mca", region_x, region_z);

    println!(
        "Looking for chunk ({}, {}) in region file: {}",
        c_x, c_z, expected_region
    );

    let region_files = fs::read_dir(SOURCE)?.filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.is_file() && path.extension() == Some(OsStr::new("mca")) {
            return Some(entry);
        } else {
            return None;
        }
    });

    for entry in region_files {
        let path = entry.path();
        if path.file_name() != Some(OsStr::new(&expected_region)) {
            println!("Skipping region file: {}", path.display());
            continue;
        }

        let chunk = match read_chunk(entry.path(), c_x, c_z) {
            None => {
                println!(
                    "Did not find chunk {}, {} in {}",
                    c_x,
                    c_z,
                    entry.path().display()
                );
                continue;
            }
            Some(c) => c,
        };
        println!("Chunk successfully parsed!");
        println!("DataVersion: {}", chunk.data_version);

        // Create the isometric renderer
        let mut renderer = IsometricRenderer::new(ASSETS);

        // Find the Y range of the chunk
        let min_section_y = chunk.sections.iter().map(|s| s.y).min().unwrap_or(0);
        let max_section_y = chunk.sections.iter().map(|s| s.y).max().unwrap_or(0);

        let min_y = min_section_y as i32 * 16;
        let max_y = (max_section_y as i32 + 1) * 16;

        println!(
            "Chunk Y range: {} to {} (sections {} to {})",
            min_y, max_y, min_section_y, max_section_y
        );

        // Print section info
        println!("Chunk has {} sections:", chunk.sections.len());
        for section in &chunk.sections {
            let block_count = section
                .block_states
                .as_ref()
                .map(|bs| bs.palette.len())
                .unwrap_or(0);
            println!("  Section Y={}: {} palette entries", section.y, block_count);
        }

        println!("Rendering full chunk...");

        // Render the entire chunk
        let img = renderer.render_chunk(|x, y, z| get_block_at(&chunk, x, y, z), min_y, max_y);

        // Save the rendered image
        let output_path = "output_chunk.png";
        img.save(output_path).expect("Failed to save image");
        println!(
            "Rendered chunk saved to {} ({}x{} pixels)",
            output_path,
            img.width(),
            img.height()
        );

        break;
    }

    Ok(())
}

/// Get the block at world coordinates (x, y, z) within a chunk
fn get_block_at(chunk: &Chunk, x: usize, y: i32, z: usize) -> Option<String> {
    // Find the section for this Y level
    let section_y = (y.div_euclid(16)) as i8;

    let section = chunk.sections.iter().find(|s| s.y == section_y)?;

    // Get the block within the section
    let local_y = y.rem_euclid(16) as usize;

    section.block_at(x, local_y, z).map(|p| p.name.clone())
}
