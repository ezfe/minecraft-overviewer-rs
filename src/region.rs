use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};

use flate2::bufread::ZlibDecoder;

use crate::chunk::Chunk;

pub fn read_chunk(path: PathBuf, chunk_x: i32, chunk_z: i32) -> Option<Chunk> {
    println!("Reading region file: {:?}", path);

    let file = File::open(path).expect("Failed to open region file");
    let mut reader = BufReader::new(file);

    // Read location table (first 4096 bytes, uncompressed)
    let mut location_table = [0u8; 4096];
    reader
        .read_exact(&mut location_table)
        .expect("Failed to read location table");

    // Read timestamp table (next 4096 bytes, uncompressed)
    let mut timestamp_table = [0u8; 4096];
    reader
        .read_exact(&mut timestamp_table)
        .expect("Failed to read timestamp table");

    let chunk_index = ((chunk_x % 32) + (chunk_z % 32) * 32) * 4; // *4 because each entry is 4 bytes
    let chunk_index: usize = chunk_index.try_into().unwrap(); // usize for rust indexing

    // Parse location as big-endian u32
    let location = u32::from_be_bytes([
        location_table[chunk_index],
        location_table[chunk_index + 1],
        location_table[chunk_index + 2],
        location_table[chunk_index + 3],
    ]);

    let offset = ((location >> 8) * 4096) as u64;
    let sectors = (location & 0xFF) as u8;

    if offset == 0 {
        println!("Chunk {},{} doesn't exist in this region", chunk_x, chunk_z);
        return None;
    }

    println!(
        "Chunk {},{} found at offset {} ({} sectors)",
        chunk_x, chunk_z, offset, sectors
    );

    // Seek to chunk data
    reader
        .seek(SeekFrom::Start(offset))
        .expect("Failed to seek at chunk index");

    // Read chunk header (5 bytes)
    let mut header = [0u8; 5];
    reader
        .read_exact(&mut header)
        .expect("Failed to read chunk header");

    let data_length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
    let compression_type = header[4];

    println!(
        "Chunk data length: {}, compression type: {}",
        data_length, compression_type
    );

    if compression_type != 2 {
        panic!(
            "Warning: unexpected compression type {} (expected 2 for zlib)",
            compression_type
        );
    }

    // Read compressed chunk data (data_length - 1 because we already read compression byte)
    let mut compressed_data = vec![0u8; (data_length - 1) as usize];
    reader
        .read_exact(&mut compressed_data)
        .expect("Failed to read compressed chunk data");

    // Decompress with zlib
    let mut decoder = ZlibDecoder::new(&compressed_data[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .expect("Failed to decompress chunk data");

    // Parse as NBT (owned deserialization)
    let chunk: Chunk = fastnbt::from_bytes(&decompressed).expect("Failed to parse chunk NBT data");
    return Some(chunk);
}
