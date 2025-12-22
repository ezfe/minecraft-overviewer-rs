use std::{
    ffi::OsStr,
    fs::{self},
    io::Result,
};

use crate::region::read_chunk;

mod chunk;
mod region;
mod section;

fn main() -> Result<()> {
    const SOURCE: &str = "sample_map/region";

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

        let b_x = 9;
        let b_y = 44;
        let b_z = 11;

        let section = chunk
            .section_for(b_y)
            .expect(format!("Failed to find section for Y={}", b_y).as_str());

        let palette = section
            .block_at(b_x, b_y, b_z)
            .expect(format!("Failed to find block for {}, {}, {}", b_x, b_y, b_z).as_str());

        println!("Block at {}, {}, {}: {:?}", b_x, b_y, b_z, palette);
        break;
    }

    Ok(())
}
