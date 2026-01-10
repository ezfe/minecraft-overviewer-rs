use crate::coords::block_face::BlockFace;

pub struct LightData {
    pub light_east: u8,
    pub light_south: u8,
    pub light_top: u8,
}

impl LightData {
    fn calc_factor(light: u8) -> f64 {
        let mut factor = light as f64 / 15.0;
        factor *= 0.7;
        factor += 0.3;
        factor
    }

    pub fn factor(&self, face: BlockFace) -> f64 {
        match face {
            BlockFace::East => Self::calc_factor(self.light_east),
            BlockFace::South => Self::calc_factor(self.light_south),
            BlockFace::Top => Self::calc_factor(self.light_top),
        }
    }
}
