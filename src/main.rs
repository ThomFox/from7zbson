use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use seven_zip::lzma_decompress;
use serde::ser::Serialize;

fn main() -> Result<(), Box<dyn Error>>{
    let path = PathBuf::from(std::env::args().nth(1).ok_or(String::from("no input filename"))?);
    let mut input = std::fs::File::open(&path)?;
    let mut bytes = vec![];
    lzma_decompress(&mut input, &mut bytes)?;
    let document = bson::Document::from_reader(&mut bytes.as_slice())?;
    let mut serializer = serde_json::Serializer::pretty(vec![]);
    document.serialize(&mut serializer)?;
    let mut output = std::io::stdout().lock();
    let json = serializer.into_inner();
    output.write_all(&json)?;
    Ok(())
}
