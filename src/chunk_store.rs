use std::collections::HashMap;

use crate::section::Section;
use crate::{
    chunk::Chunk,
    coords::{world_block_coord::WorldBlockCoord, world_chunk_coord::WorldChunkCoord},
};

pub struct ChunkStore {
    pub chunks: HashMap<WorldChunkCoord, Chunk>,
}

impl ChunkStore {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn insert(&mut self, coord: WorldChunkCoord, mut chunk: Chunk) {
        chunk.ensure_unpacked();
        self.chunks.insert(coord, chunk);
    }

    fn get(&self, coord: WorldChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    fn get_section(&self, block_coords: &WorldBlockCoord) -> Option<&Section> {
        let chunk = self.get(block_coords.chunk_coord())?;
        Some(
            chunk
                .sections
                .iter()
                .find(|s| s.y == block_coords.chunk_y_section())?,
        )
    }

    /// Get block at world coordinates
    pub fn get_block_at(&self, block_coords: &WorldBlockCoord) -> Option<String> {
        let local_coords = block_coords.section_local_coord();
        let section = self.get_section(block_coords)?;
        section.block_at(local_coords).map(|p| p.name.clone())
    }

    pub fn get_block_light_at(&self, block_coords: &WorldBlockCoord) -> Option<u8> {
        let local_coords = block_coords.section_local_coord();
        let section = self.get_section(block_coords)?;
        section.block_light_at(local_coords)
    }

    /// Get the Y range across all loaded chunks
    pub fn get_y_range(&self) -> (isize, isize) {
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
