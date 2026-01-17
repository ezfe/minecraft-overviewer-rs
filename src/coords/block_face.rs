#[derive(Hash, Eq, PartialEq, Clone)]
pub enum VisibleBlockFace {
    Top,
    South,
    East,
}

impl VisibleBlockFace {
    pub fn to_block_face(&self) -> BlockFace {
        match self {
            VisibleBlockFace::Top => BlockFace::Top,
            VisibleBlockFace::South => BlockFace::South,
            VisibleBlockFace::East => BlockFace::East,
        }
    }
}

pub enum VerticalBlockFace {
    North,
    South,
    East,
    West,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum BlockFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}
