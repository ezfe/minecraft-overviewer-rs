use std::io::Read;
use std::{
    ffi::OsStr,
    fs::{self},
    io::Result,
};

use fastnbt::LongArray;
use flate2::read::ZlibDecoder;
use serde::Deserialize;

use crate::region::read_chunk;

mod region;

fn main() -> Result<()> {
    const SOURCE: &str = "sample_map/region";

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
        let mut reader = read_chunk(entry.path(), 0, 0);

        // Read chunk header (5 bytes)
        let mut header = [0u8; 5];
        reader.read_exact(&mut header)?;

        let data_length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
        let compression_type = header[4];

        println!(
            "Chunk data length: {}, compression type: {}",
            data_length, compression_type
        );

        if compression_type != 2 {
            println!(
                "Warning: unexpected compression type {} (expected 2 for zlib)",
                compression_type
            );
        }

        // Read compressed chunk data (data_length - 1 because we already read compression byte)
        let mut compressed_data = vec![0u8; (data_length - 1) as usize];
        reader.read_exact(&mut compressed_data)?;

        // Decompress with zlib
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        println!("Decompressed {} bytes", decompressed.len());

        // Parse as NBT
        let chunk: Chunk = fastnbt::from_bytes(&decompressed).unwrap();

        println!("Chunk successfully parsed!");
        println!("DataVersion: {}", chunk.data_version);

        // Print section info if available
        if let Some(sections) = &chunk.sections {
            println!("Number of sections: {}", sections.len());
            for section in sections {
                println!("  Section Y={}", section.y);
                if let Some(ref states) = section.block_states {
                    println!("    Block states: {} longs", states.len());
                }
            }
        }

        // Only process first region file for now
        break;
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Chunk {
    #[serde(rename = "DataVersion")]
    pub data_version: i32,
    #[serde(rename = "sections")]
    pub sections: Option<Vec<Section>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Section {
    pub block_states: Option<LongArray>,
    #[serde(rename = "Y")]
    pub y: i8,
}
