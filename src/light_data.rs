use crate::coords::block_face::VisibleBlockFace;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct LightData {
    pub light_internal: u8,
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

    pub fn factor_internal(&self) -> f64 {
        Self::calc_factor(self.light_internal)
    }

    pub fn factor(&self, face: VisibleBlockFace) -> f64 {
        match face {
            VisibleBlockFace::East => Self::calc_factor(self.light_east),
            VisibleBlockFace::South => Self::calc_factor(self.light_south),
            VisibleBlockFace::Top => Self::calc_factor(self.light_top),
        }
    }
}
