use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};

pub fn read_chunk(path: PathBuf, chunk_x: usize, chunk_z: usize) -> BufReader<File> {
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
        panic!("Chunk {},{} doesn't exist in this region", chunk_x, chunk_z);
    }

    println!(
        "Chunk {},{} found at offset {} ({} sectors)",
        chunk_x, chunk_z, offset, sectors
    );

    // Seek to chunk data
    reader
        .seek(SeekFrom::Start(offset))
        .expect("Failed to seek at chunk index");

    return reader;
}
