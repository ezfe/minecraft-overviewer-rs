use core::fmt;

use crate::coords::{
    chunk_local_block_coord::ChunkLocalBlockCoord, world_chunk_coord::WorldChunkCoord,
};

#[derive(Debug, Clone, Copy)]
pub struct WorldBlockCoord {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl WorldBlockCoord {
    pub fn chunk_coord(&self) -> WorldChunkCoord {
        WorldChunkCoord {
            cx: self.x.div_euclid(16),
            cz: self.z.div_euclid(16),
        }
    }

    pub fn chunk_local_coord(&self) -> ChunkLocalBlockCoord {
        ChunkLocalBlockCoord {
            lx: self.x.rem_euclid(16) as usize,
            ly: self.y.rem_euclid(16) as usize,
            lz: self.z.rem_euclid(16) as usize,
        }
    }

    pub fn chunk_y_section(&self) -> i8 {
        self.y.div_euclid(16) as i8
    }
}

impl fmt::Display for WorldBlockCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}
