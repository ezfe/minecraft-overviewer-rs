use core::fmt;

pub struct RegionCoord {
    pub rx: isize,
    pub rz: isize,
}

impl RegionCoord {
    pub fn file_name(&self) -> String {
        format!("r.{}.{}.mca", self.rx, self.rz)
    }
}

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

pub struct ChunkLocalBlockCoord {
    pub lx: usize,
    pub ly: usize,
    pub lz: usize,
}

impl ChunkLocalBlockCoord {
    pub fn index(&self) -> usize {
        self.ly * 256 + self.lz * 16 + self.lx
    }
}

#[derive(Hash, Eq, PartialEq)]
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
}

impl fmt::Display for WorldChunkCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.cx, self.cz)
    }
}
