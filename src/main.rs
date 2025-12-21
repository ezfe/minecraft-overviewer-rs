use std::io::Read;
use std::{
    ffi::OsStr,
    fs::{self},
    io::{BufReader, Result},
};

use fastnbt::LongArray;
use flate2::bufread::ZlibDecoder;
use serde::Deserialize;

fn main() -> Result<()> {
    const SOURCE: &str = "sample_map/region";

    let region_files = fs::read_dir(SOURCE)?.filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.is_file() && path.extension() == Some(OsStr::new("mca")) {
            return Some(entry);
        } else {
            return None;
        }
    });

    for entry in region_files {
        let file = fs::File::open(entry.path())?;
        let reader = BufReader::new(file);

        let mut decoder = ZlibDecoder::new(reader);
        let mut data = vec![];
        decoder.read_to_end(&mut data).unwrap();

        let section: Section = fastnbt::from_bytes(&data).unwrap();

        let states = section.block_states.unwrap();

        for long in states.iter() {
            println!("{}", long);
        }
    }

    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Section {
    pub block_states: Option<LongArray>,
    pub y: i8,
}
