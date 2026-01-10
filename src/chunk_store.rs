use std::collections::HashMap;

use crate::coords::chunk_local_block_coord::ChunkLocalBlockCoord;
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
        chunk
            .sections
            .iter()
            .find(|s| s.y == block_coords.chunk_y_section())
    }

    fn get_section_above(&self, section: &Section, in_chunk: WorldChunkCoord) -> Option<&Section> {
        let chunk = self.get(in_chunk)?;
        chunk.sections.iter().find(|s| s.y == section.y + 1)
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

    pub fn get_sky_light_at(&self, coords: &WorldBlockCoord) -> u8 {
        let local_coords = coords.section_local_coord();
        let Some(mut section) = self.get_section(coords) else {
            return 0xF;
        };

        // If there is sky-light in this section at the current coordinates,
        // use that value
        if let Some(sky_light) = section.sky_light_at(local_coords) {
            return sky_light;
        }
        let section_bottom_coords = ChunkLocalBlockCoord {
            lx: local_coords.lx,
            ly: 0,
            lz: local_coords.lz,
        };
        loop {
            // If there wasn't sky light there, then we need to look at the section above
            // If there is no section above, then we return full brightness
            if let Some(section_above) = self.get_section_above(&section, coords.chunk_coord()) {
                section = section_above;
                // If we found a section above, then we need to examine it for sky light at the
                // lowest y-value in the section
                if let Some(sky_light) = section.sky_light_at(section_bottom_coords) {
                    return sky_light;
                }
            } else {
                return 0xF;
            }
            // If we didn't find light, and there was a section above then the loop continues
        }
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
