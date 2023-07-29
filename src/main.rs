use std::error::Error;
use std::path::PathBuf;
use seven_zip::lzma_decompress;
use bson2json::*;

mod bson2json;

fn main() -> Result<(), Box<dyn Error>>{
    let path = PathBuf::from(std::env::args().nth(1).ok_or(String::from("no input filename"))?);
    let mut input = std::fs::File::open(&path)?;
    let mut bytes = vec![];
    lzma_decompress(&mut input, &mut bytes)?;
    let document = bson::RawDocument::from_bytes(bytes.as_slice())?;
    let mut output = std::io::stdout().lock();
    let mut writer = JsonWriter { formatter: serde_json::ser::PrettyFormatter::default(), writer: &mut output };
    traverse_document(document, &mut writer)?;
    Ok(())
}
