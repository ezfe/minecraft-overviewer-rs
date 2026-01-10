use std::collections::HashMap;

use fastnbt::{ByteArray, LongArray};
use serde::Deserialize;

use crate::coords::chunk_local_block_coord::ChunkLocalBlockCoord;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Section {
    pub y: i8,
    #[serde(rename = "block_states")]
    pub block_states: Option<BlockStates>,
    pub block_light: Option<ByteArray>,
    pub sky_light: Option<ByteArray>,
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

    fn block_at(&self, index: usize) -> Option<&PaletteEntry> {
        match &self.data {
            None => self.palette.first(),
            Some(_) => {
                if let Some(unpacked) = &self.unpacked_data {
                    let palette_index = unpacked[index];
                    self.palette.get(palette_index as usize)
                } else {
                    // Should be ensured unpacked before calling
                    None
                }
            }
        }
    }

    pub fn ensure_unpacked(&mut self) {
        if let Some(data) = &self.data {
            if self.unpacked_data.is_none() {
                self.unpacked_data = Some(Self::unpack_blockstates(data));
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
    pub fn ensure_unpacked(&mut self) {
        if let Some(states) = &mut self.block_states {
            states.ensure_unpacked();
        }
    }

    pub fn block_at(&self, coords: ChunkLocalBlockCoord) -> Option<&PaletteEntry> {
        match &self.block_states {
            None => None,
            Some(states) => states.block_at(coords.index()),
        }
    }

    fn light_index(coords: ChunkLocalBlockCoord, light_data: &ByteArray) -> u8 {
        let index = coords.index();
        let byte = light_data[index / 2];
        let byte = byte.clone() as u8;
        let shift: i8 = if index % 2 == 0 { 0 } else { 4 };
        (byte >> shift) & 0x0F
    }

    pub fn block_light_at(&self, coords: ChunkLocalBlockCoord) -> Option<u8> {
        self.block_light
            .as_ref()
            .map(|light| Self::light_index(coords, light))
    }

    pub fn sky_light_at(&self, coords: ChunkLocalBlockCoord) -> Option<u8> {
        self.sky_light
            .as_ref()
            .map(|light| Self::light_index(coords, light))
    }
}
