use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};

use flate2::bufread::ZlibDecoder;

use crate::{chunk::Chunk, coords::world_chunk_coord::WorldChunkCoord};

pub fn read_chunk(path: PathBuf, chunk_coord: &WorldChunkCoord) -> Option<Chunk> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);

    // Read location table (first 4096 bytes, uncompressed)
    let mut location_table = [0u8; 4096];
    reader.read_exact(&mut location_table).ok()?;

    // Read timestamp table (next 4096 bytes, uncompressed)
    let mut timestamp_table = [0u8; 4096];
    reader.read_exact(&mut timestamp_table).ok()?;

    // Handle negative chunk coordinates properly
    let local_x = chunk_coord.cx.rem_euclid(32);
    let local_z = chunk_coord.cz.rem_euclid(32);
    let chunk_index = (local_x + local_z * 32) * 4;
    let chunk_index = chunk_index as usize;

    // Parse location as big-endian u32
    let location = u32::from_be_bytes([
        location_table[chunk_index],
        location_table[chunk_index + 1],
        location_table[chunk_index + 2],
        location_table[chunk_index + 3],
    ]);

    let offset = ((location >> 8) * 4096) as u64;
    let _sectors = (location & 0xFF) as u8;

    if offset == 0 {
        return None; // Chunk doesn't exist
    }

    // Seek to chunk data
    reader.seek(SeekFrom::Start(offset)).ok()?;

    // Read chunk header (5 bytes)
    let mut header = [0u8; 5];
    reader.read_exact(&mut header).ok()?;

    let data_length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
    let compression_type = header[4];

    if compression_type != 2 {
        eprintln!(
            "Warning: unexpected compression type {} (expected 2 for zlib)",
            compression_type
        );
        return None;
    }

    // Read compressed chunk data (data_length - 1 because we already read compression byte)
    let mut compressed_data = vec![0u8; (data_length - 1) as usize];
    reader.read_exact(&mut compressed_data).ok()?;

    // Decompress with zlib
    let mut decoder = ZlibDecoder::new(&compressed_data[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).ok()?;

    // Parse as NBT (owned deserialization)
    fastnbt::from_bytes(&decompressed).ok()
}
