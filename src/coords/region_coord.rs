#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct RegionCoord {
    pub rx: isize,
    pub rz: isize,
}

impl RegionCoord {
    pub fn file_name(&self) -> String {
        format!("r.{}.{}.mca", self.rx, self.rz)
    }
}
