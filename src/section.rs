use std::collections::HashMap;

use fastnbt::LongArray;
use serde::Deserialize;

use crate::coords::chunk_local_block_coord::ChunkLocalBlockCoord;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Section {
    pub y: i8,
    #[serde(rename = "block_states")]
    pub block_states: Option<BlockStates>,
    // pub block_light: Option<ByteArray>,
    // pub sky_light: Option<ByteArray>,
}

#[derive(Deserialize, Debug)]
pub struct BlockStates {
    pub palette: Vec<PaletteEntry>,
    pub data: Option<LongArray>,
    #[serde(default)]
    pub unpacked_data: Option<Vec<u16>>,
}

impl BlockStates {
    fn unpack_blockstates(data: &LongArray) -> Vec<u16> {
        const BLOCK_COUNT: usize = 4096; // 16 * 16 * 16

        // Calculate bits per value (minimum 4 bits)
        let bits_per_value = std::cmp::max(4, (data.len() * 64) / BLOCK_COUNT);

        // How many values fit in one 64-bit long
        let values_per_long = 64 / bits_per_value;

        // Create bitmask for extracting values
        let mask = (1u64 << bits_per_value) - 1;

        let mut result = Vec::with_capacity(BLOCK_COUNT);

        // Extract values from packed longs
        'outer: for &long_value in data.iter() {
            for j in 0..values_per_long {
                if result.len() >= BLOCK_COUNT {
                    break 'outer;
                }

                // Extract the value at position j from this long
                let value = (((long_value as u64) >> (j * bits_per_value)) & mask) as u16;
                result.push(value);
            }
        }

        result
    }

    fn block_at(&mut self, index: usize) -> Option<&PaletteEntry> {
        match &self.data {
            None => return self.palette.first(),
            Some(data) => {
                if self.unpacked_data.is_none() {
                    self.unpacked_data = Some(Self::unpack_blockstates(data));
                }

                let palette_index = self.unpacked_data.as_ref().unwrap()[index];
                self.palette.get(palette_index as usize)
            }
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PaletteEntry {
    pub name: String,
    pub properties: Option<HashMap<String, String>>,
}

impl Section {
    pub fn block_at(&mut self, coords: ChunkLocalBlockCoord) -> Option<&PaletteEntry> {
        match &mut self.block_states {
            None => return None,
            Some(states) => states.block_at(coords.index()),
        }
    }
}
