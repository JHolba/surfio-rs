use crate::irap::{self, Irap, IrapHeader};
use crate::utils;
use memmap::Mmap;
use std::fs::File;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn is_ascii_whitespace(byte: u8) -> bool {
    matches!(byte, 0x20 | 0x09 | 0x0A | 0x0D | 0x0C | 0x0B)
}

fn skip_whitespace(buffer: &[u8]) -> usize {
    buffer
        .iter()
        .position(|&c| !is_ascii_whitespace(c))
        .unwrap_or(buffer.len())
}

fn read_value_float(buffer: &[u8]) -> Result<(f64, usize)> {
    let pos = skip_whitespace(buffer);
    if pos == buffer.len() {
        return Err("Unexpected end of file".into());
    }
    let (num, len) = fast_float::parse_partial::<f64, _>(&buffer[pos..])?;
    Ok((num, len + pos))
}

fn read_value_int(buffer: &[u8]) -> Result<(i32, usize)> {
    let pos = skip_whitespace(buffer);
    if pos == buffer.len() {
        return Err("Unexpected end of file".into());
    }

    let sub = &buffer[pos..];
    if let Some(end) = sub.iter().position(|&c| is_ascii_whitespace(c)) {
        let s = std::str::from_utf8(&sub[..end])?;
        return Ok((s.parse()?, end + pos));
    }

    // Handle case where file ends with the number
    let s = std::str::from_utf8(sub)?;
    Ok((s.parse()?, sub.len() + pos))
}

fn read_header(buffer: &[u8]) -> Result<(irap::IrapHeader, usize)> {
    let mut header = IrapHeader::default();
    let mut index = 0;

    let (id, len) = read_value_int(buffer)?;
    index += len;

    if id != IrapHeader::ID {
        return Err(format!(
            "Incorrect magic number. Expected {}, got {}",
            IrapHeader::ID,
            id
        )
        .into());
    }

    let (value, len) = read_value_int(&buffer[index..])?;
    header.nrow = value as u32;
    index += len;

    let (value, len) = read_value_float(&buffer[index..])?;
    header.xinc = value;
    index += len;

    let (value, len) = read_value_float(&buffer[index..])?;
    header.yinc = value;
    index += len;

    let (value, len) = read_value_float(&buffer[index..])?;
    header.xori = value;
    index += len;

    let (value, len) = read_value_float(&buffer[index..])?;
    header.xmax = value;
    index += len;

    let (value, len) = read_value_float(&buffer[index..])?;
    header.yori = value;
    index += len;

    let (value, len) = read_value_float(&buffer[index..])?;
    header.ymax = value;
    index += len;

    let (value, len) = read_value_int(&buffer[index..])?;
    header.ncol = value as u32;
    index += len;

    let (value, len) = read_value_float(&buffer[index..])?;
    header.rot = value;
    index += len;

    let (value, len) = read_value_float(&buffer[index..])?;
    header.xrot = value;
    index += len;

    let (value, len) = read_value_float(&buffer[index..])?;
    header.yrot = value;
    index += len;

    if header.rot < 0.0 {
        header.rot += 360.0;
    }

    // dummy values at end of header
    for _ in 0..7 {
        let (_, len) = read_value_float(&buffer[index..])?;
        index += len;
    }

    Ok((header, index))
}

fn validate_header(header: &IrapHeader) -> Result<()> {
    if header.ncol <= 0 || header.nrow <= 0 {
        return Err(format!(
            "Invalid dimensions: ncol={}, nrow={}",
            header.ncol, header.nrow
        )
        .into());
    }
    Ok(())
}

fn read_values(buffer: &[u8], ncol: usize, nrow: usize) -> Result<Vec<f32>> {
    let nvalues = ncol * nrow;
    let mut values = vec![0.0; nvalues];
    let mut index = 0;

    for i in 0..nvalues {
        let (val, len) = read_value_float(&buffer[index..])?;
        index += len;

        let ic = utils::column_major_to_row_major_index(i, ncol, nrow);

        if val >= irap::UNDEF_MAP_IRAP_ASCII as f64 {
            values[ic] = f32::NAN;
        } else {
            values[ic] = val as f32;
        }
    }

    Ok(values)
}

pub fn read_file(path: String) -> Result<Irap> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let (header, index) = read_header(&mmap[..])?;
    validate_header(&header)?;

    let values = read_values(&mmap[index..], header.ncol as usize, header.nrow as usize)?;

    Ok(Irap { header, values })
}

pub fn read_string(data: &str) -> Result<Irap> {
    let buffer = data.as_bytes();

    let (header, index) = read_header(buffer)?;
    validate_header(&header)?;

    let values = read_values(&buffer[index..], header.ncol as usize, header.nrow as usize)?;

    Ok(Irap { header, values })
}
