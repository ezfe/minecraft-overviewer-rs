use core::fmt;

use crate::coords::region_coord::RegionCoord;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct WorldChunkCoord {
    pub cx: isize,
    pub cz: isize,
}

impl WorldChunkCoord {
    pub fn region_coord(&self) -> RegionCoord {
        RegionCoord {
            rx: self.cx.div_euclid(32),
            rz: self.cz.div_euclid(32),
        }
    }

    pub fn range_to(self, other: &WorldChunkCoord) -> WorldChunkCoordIterator {
        let min_coord = WorldChunkCoord {
            cx: self.cx.min(other.cx),
            cz: self.cz.min(other.cz),
        };
        let max_coord = WorldChunkCoord {
            cx: self.cx.max(other.cx),
            cz: self.cz.max(other.cz),
        };
        WorldChunkCoordIterator {
            current: min_coord.clone(),
            min: min_coord,
            max: max_coord,
        }
    }
}

impl fmt::Display for WorldChunkCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.cx, self.cz)
    }
}

pub struct WorldChunkCoordIterator {
    current: WorldChunkCoord,
    min: WorldChunkCoord,
    max: WorldChunkCoord,
}

impl Iterator for WorldChunkCoordIterator {
    type Item = WorldChunkCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.cx > self.max.cx {
            return None;
        }
        let res = self.current;
        self.current.cz += 1;
        if self.current.cz > self.max.cz {
            self.current.cz = self.min.cz;
            self.current.cx += 1;
        }
        Some(res)
    }
}
