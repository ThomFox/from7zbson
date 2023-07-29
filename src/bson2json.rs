use std::io::Write;
use bson::{DateTime, RawArray, RawBsonRef, RawDocument};
use bson::spec::ElementType;
use serde_json::ser::CharEscape;
use thiserror::Error;

#[derive(Error,Debug)]
pub enum TraverseError {
    #[error("unexpected bson type {0:?}")]
    UnexpectedBsonType(ElementType),
    #[error("bson raw error")]
    BsonRawError(#[from] bson::raw::Error),
    #[error("bson datetime error")]
    BsonDatetimeError(#[from] bson::datetime::Error),
    #[error("io error")]
    IoError(#[from] std::io::Error)
}

pub struct JsonWriter<F: serde_json::ser::Formatter, W: Write> {
    pub formatter: F,
    pub writer: W
}

fn write_date_time<F: serde_json::ser::Formatter,W: Write>(w: &mut JsonWriter<F,W>, dt: DateTime) -> Result<(), TraverseError> {
    let date_string = dt.try_to_rfc3339_string()?;
    write_string(w, &date_string)?;
    Ok(())
}

fn write_string<F: serde_json::ser::Formatter,W: Write>(w: &mut JsonWriter<F,W>, s: &str) -> Result<(), TraverseError> {
    w.formatter.begin_string(&mut w.writer)?;
    let mut start = 0;
    let bytes = s.as_bytes();
    for (i,&byte) in bytes.iter().enumerate() {
        if byte >= b' ' && byte != b'"' && byte != b'\\' { continue; }

        if start < i {
            w.formatter.write_string_fragment(&mut w.writer, &s[start..i])?;
        }

        let escape = match byte {
            b'"' => CharEscape::Quote,
            b'\\' => CharEscape::ReverseSolidus,
            8 => CharEscape::Backspace,
            12 => CharEscape::FormFeed,
            10 => CharEscape::LineFeed,
            13 => CharEscape::CarriageReturn,
            9 => CharEscape::Tab,
            x => CharEscape::AsciiControl(x)
        };

        w.formatter.write_char_escape(&mut w.writer, escape)?;

        start = i + 1;
    }
    if start < bytes.len() {
        w.formatter.write_string_fragment(&mut w.writer, &s[start..])?;
    }
    w.formatter.end_string(&mut w.writer)?;
    Ok(())
}

fn traverse_value<F: serde_json::ser::Formatter,W: Write>(value: &RawBsonRef, w: &mut JsonWriter<F,W>) -> Result<(), TraverseError> {
    match *value {
        RawBsonRef::Double(f) => w.formatter.write_f64(&mut w.writer, f)?,
        RawBsonRef::String(s) => write_string(w, s)?,
        RawBsonRef::Array(array) => traverse_array(array, w)?,
        RawBsonRef::Document(inner) => traverse_document(inner, w)?,
        RawBsonRef::Boolean(b) => w.formatter.write_bool(&mut w.writer, b)?,
        RawBsonRef::Null => w.formatter.write_null(&mut w.writer)?,
        RawBsonRef::Int32(n) => w.formatter.write_i32(&mut w.writer, n)?,
        RawBsonRef::Int64(n) => w.formatter.write_i64(&mut w.writer, n)?,
        RawBsonRef::DateTime(dt) => write_date_time(w, dt)?,
        _ => return Err(TraverseError::UnexpectedBsonType(value.element_type()))
    }

    Ok(())
}

fn traverse_array<F: serde_json::ser::Formatter,W: Write>(array: &RawArray, w: &mut JsonWriter<F,W>) -> Result<(), TraverseError> {
    w.formatter.begin_array(&mut w.writer)?;
    let mut first = true;
    for value in array.into_iter() {
        let value = value?;
        w.formatter.begin_array_value(&mut w.writer, first)?; first = false;
        traverse_value(&value, w)?;
        w.formatter.end_array_value(&mut w.writer)?;
    }
    w.formatter.end_array(&mut w.writer)?;
    Ok(())
}

pub fn traverse_document<F: serde_json::ser::Formatter,W: Write>(document: impl AsRef<RawDocument>, w: &mut JsonWriter<F,W>) -> Result<(), TraverseError> {
    let document = document.as_ref();
    w.formatter.begin_object(&mut w.writer)?;
    let mut first = true;
    for pair in document.into_iter() {
        let (key, value) = pair?;
        w.formatter.begin_object_key(&mut w.writer, first)?; first = false;
        write_string(w, key)?;
        w.formatter.end_object_key(&mut w.writer)?;
        w.formatter.begin_object_value(&mut w.writer)?;
        traverse_value(&value, w)?;
        w.formatter.end_object_value(&mut w.writer)?;
    }
    w.formatter.end_object(&mut w.writer)?;
    Ok(())
}
