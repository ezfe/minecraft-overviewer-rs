use serde::Deserialize;

use crate::section::Section;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Chunk {
    pub data_version: i32,

    #[serde(rename = "xPos")]
    pub x_pos: i32,
    #[serde(rename = "zPos")]
    pub z_pos: i32,
    /// Lowest Y section position in the chunk.
    /// `-4` in modern versions meaning Y=-64
    #[serde(rename = "yPos")]
    pub y_pos: i32,

    /// Status of the chunk generation process.
    /// `minecraft:full` means fully generated.
    pub status: String,

    #[serde(rename = "sections")]
    pub sections: Vec<Section>,  // Owned, not borrowed
}
