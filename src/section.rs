use fastnbt::{ByteArray, LongArray};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Section {
    pub y: i8,
    pub block_states: Option<BlockStates>,
    pub block_light: Option<ByteArray>,
    pub sky_light: Option<ByteArray>,
}

#[derive(Deserialize, Debug)]
pub struct BlockStates {
    pub palette: Vec<PaletteEntry>,
    pub data: LongArray,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PaletteEntry {
    pub name: String,
}
