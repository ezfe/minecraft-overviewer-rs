use core::fmt;

use crate::coords::{
    chunk_local_block_coord::ChunkLocalBlockCoord, painters_range::PaintersRange,
    world_chunk_coord::WorldChunkCoord,
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

    pub fn section_local_coord(&self) -> ChunkLocalBlockCoord {
        ChunkLocalBlockCoord {
            lx: self.x.rem_euclid(16) as usize,
            ly: self.y.rem_euclid(16) as usize,
            lz: self.z.rem_euclid(16) as usize,
        }
    }

    pub fn above_pos_y(&self) -> WorldBlockCoord {
        WorldBlockCoord {
            x: self.x,
            y: self.y + 1,
            z: self.z,
        }
    }

    pub fn south_pos_z(&self) -> WorldBlockCoord {
        WorldBlockCoord {
            x: self.x,
            y: self.y,
            z: self.z + 1,
        }
    }

    pub fn east_pos_x(&self) -> WorldBlockCoord {
        WorldBlockCoord {
            x: self.x + 1,
            y: self.y,
            z: self.z,
        }
    }

    pub fn chunk_y_section(&self) -> i8 {
        self.y.div_euclid(16) as i8
    }

    pub fn painters_range_to(self, other: &WorldBlockCoord) -> WorldBlockCoordIterator {
        let min_coord = WorldBlockCoord {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        };
        let max_coord = WorldBlockCoord {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        };

        WorldBlockCoordIterator {
            curr_x: min_coord.x,
            curr_y: min_coord.y,
            curr_sum: min_coord.x + min_coord.z,

            min: min_coord,
            max: max_coord,
        }
    }
}

impl fmt::Display for WorldBlockCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}

impl PaintersRange for WorldBlockCoord {
    type Iter = WorldBlockCoordIterator;

    fn painters_range_to(&self, other: &Self) -> Self::Iter {
        Self::painters_range_to(*self, other)
    }
}

pub struct WorldBlockCoordIterator {
    curr_y: isize,
    curr_sum: isize,
    curr_x: isize,

    min: WorldBlockCoord,
    max: WorldBlockCoord,
}

impl Iterator for WorldBlockCoordIterator {
    type Item = WorldBlockCoord;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.curr_y > self.max.y {
                return None;
            }

            let z = self.curr_sum - self.curr_x;
            let mut result = None;

            if z >= self.min.z && z <= self.max.z {
                result = Some(WorldBlockCoord {
                    x: self.curr_x,
                    y: self.curr_y,
                    z,
                });
            }

            // Advance state
            self.curr_x += 1;
            if self.curr_x > self.max.x {
                self.curr_sum += 1;
                self.curr_x = self.min.x;

                if self.curr_sum > (self.max.x + self.max.z) {
                    self.curr_y += 1;
                    self.curr_sum = self.min.x + self.min.z;
                }
            }

            if result.is_some() {
                return result;
            }
        }
    }
}
