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
    pub sections: Vec<Section>,
}

impl Chunk {
    pub fn section_for(&self, y: usize) -> Option<&Section> {
        let section_index = (y / 16) as i8;
        return self
            .sections
            .iter()
            .find(|section| section.y == section_index);
    }

    pub fn ensure_unpacked(&mut self) {
        for section in &mut self.sections {
            section.ensure_unpacked();
        }
    }
}
