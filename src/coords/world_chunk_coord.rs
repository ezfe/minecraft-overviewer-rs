use core::fmt;

use crate::coords::{
    constants::MC_CHUNK_SIZE, painters_range::PaintersRange, region_coord::RegionCoord,
    world_block_coord::WorldBlockCoord,
};

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
            current: min_coord,
            min: min_coord,
            max: max_coord,
        }
    }

    pub fn painters_range_to(self, other: &WorldChunkCoord) -> WorldChunkCoordPaintersIterator {
        let min_coord = WorldChunkCoord {
            cx: self.cx.min(other.cx),
            cz: self.cz.min(other.cz),
        };
        let max_coord = WorldChunkCoord {
            cx: self.cx.max(other.cx) + 1,
            cz: self.cz.max(other.cz) + 1,
        };

        WorldChunkCoordPaintersIterator {
            exhausted: false,
            curr: None,
            min: min_coord,
            max: max_coord,
        }
    }

    pub fn world_block_coord_min(&self, min_y: isize) -> WorldBlockCoord {
        WorldBlockCoord {
            x: self.cx * MC_CHUNK_SIZE,
            y: min_y,
            z: self.cz * MC_CHUNK_SIZE,
        }
    }

    pub fn world_block_coord_max(&self, max_y: isize) -> WorldBlockCoord {
        WorldBlockCoord {
            x: self.cx * MC_CHUNK_SIZE + MC_CHUNK_SIZE - 1,
            y: max_y - 1,
            z: self.cz * MC_CHUNK_SIZE + MC_CHUNK_SIZE - 1,
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

pub struct WorldChunkCoordPaintersIterator {
    /// The iterator is complete
    exhausted: bool,
    /// Last returned value
    curr: Option<WorldChunkCoord>,
    /// Minimum coordinate (included)
    min: WorldChunkCoord,
    /// Maximum coordinate (excluded)
    max: WorldChunkCoord,
}

impl Iterator for WorldChunkCoordPaintersIterator {
    type Item = WorldChunkCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        if let Some(curr) = self.curr {
            self.curr = self.get_next_curr(curr);
        } else {
            self.curr = Some(self.min);
        };

        self.curr
    }
}

impl WorldChunkCoordPaintersIterator {
    fn get_next_curr(&self, curr: WorldChunkCoord) -> Option<WorldChunkCoord> {
        let next_on_row = WorldChunkCoord {
            cx: curr.cx + 1,
            cz: curr.cz - 1,
        };

        if next_on_row.cz >= self.min.cz && next_on_row.cx < self.max.cx {
            // in bounds on current row
            Some(next_on_row)
        } else {
            let curr_row = curr.cx - self.min.cx + curr.cz - self.min.cz;
            let last_row = self.max.cx - 1 - self.min.cx + self.max.cz - 1 - self.min.cz;
            let last_ascending_row = self.max.cz - self.min.cz - 1; // along x origin so ignore

            if curr_row >= last_row {
                // if we're past the maximum value on the final row, we're done
                return None;
            }

            if curr_row < last_ascending_row {
                let first_next_row = WorldChunkCoord {
                    cz: self.min.cz + curr_row + 1, // starts at row 0, so +1 -> row 1
                    cx: self.min.cx,
                };
                Some(first_next_row)
            } else {
                let first_next_row = WorldChunkCoord {
                    cz: self.max.cz - 1, // -1 because max is exclusive
                    cx: self.min.cx + (curr_row - last_ascending_row + 1),
                };
                Some(first_next_row)
            }
        }
    }
}

impl PaintersRange for WorldChunkCoord {
    type Iter = WorldChunkCoordPaintersIterator;

    fn painters_range_to(&self, other: &Self) -> Self::Iter {
        Self::painters_range_to(*self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_coord_painters_iterator_0_wide() {
        let min = WorldChunkCoord { cx: 0, cz: 0 };
        let max = WorldChunkCoord { cx: 0, cz: 0 };
        let iter = min.painters_range_to(&max);
        let x: Vec<WorldChunkCoord> = iter.collect();

        assert_eq!(x, vec![WorldChunkCoord { cx: 0, cz: 0 }])
    }

    #[test]
    fn chunk_coord_painters_iterator_0_wide_negative() {
        let min = WorldChunkCoord { cx: -5, cz: -5 };
        let max = WorldChunkCoord { cx: -5, cz: -5 };
        let iter = min.painters_range_to(&max);
        let x: Vec<WorldChunkCoord> = iter.collect();

        assert_eq!(x, vec![WorldChunkCoord { cx: -5, cz: -5 }])
    }

    #[test]
    fn chunk_coord_painters_iterator_non_rect() {
        let min = WorldChunkCoord { cx: -2, cz: -1 };
        let max = WorldChunkCoord { cx: 1, cz: 1 };
        let iter = min.painters_range_to(&max);
        let x: Vec<WorldChunkCoord> = iter.take(20).collect();

        assert_eq!(
            x,
            vec![
                WorldChunkCoord { cx: -2, cz: -1 },
                WorldChunkCoord { cx: -2, cz: 0 },
                WorldChunkCoord { cx: -1, cz: -1 },
                WorldChunkCoord { cx: -2, cz: 1 },
                WorldChunkCoord { cx: -1, cz: 0 },
                WorldChunkCoord { cx: 0, cz: -1 },
                WorldChunkCoord { cx: -1, cz: 1 },
                WorldChunkCoord { cx: 0, cz: 0 },
                WorldChunkCoord { cx: 1, cz: -1 },
                WorldChunkCoord { cx: 0, cz: 1 },
                WorldChunkCoord { cx: 1, cz: 0 },
                WorldChunkCoord { cx: 1, cz: 1 },
            ]
        )
    }
}
