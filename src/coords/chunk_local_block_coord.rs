#[derive(Debug, Clone, Copy)]
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
