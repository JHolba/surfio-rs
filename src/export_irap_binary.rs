use crate::irap::{Irap, IrapHeader, UNDEF_MAP_IRAP_BINARY};
use byteorder::{BigEndian, WriteBytesExt};
use std::fs::File;
use std::io::{BufWriter, Write};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const PER_LINE_BINARY: usize = 8;

fn write_header<W: Write>(header: &IrapHeader, out: &mut W) -> std::io::Result<()> {
    // Chunk 1: 8 values (32 bytes)
    out.write_i32::<BigEndian>(32)?;
    out.write_i32::<BigEndian>(IrapHeader::ID)?;
    out.write_u32::<BigEndian>(header.nrow)?;
    out.write_f32::<BigEndian>(header.xori as f32)?;
    out.write_f32::<BigEndian>(header.xmax as f32)?;
    out.write_f32::<BigEndian>(header.yori as f32)?;
    out.write_f32::<BigEndian>(header.ymax as f32)?;
    out.write_f32::<BigEndian>(header.xinc as f32)?;
    out.write_f32::<BigEndian>(header.yinc as f32)?;
    out.write_i32::<BigEndian>(32)?;

    // Chunk 2: 4 values (16 bytes)
    out.write_i32::<BigEndian>(16)?;
    out.write_u32::<BigEndian>(header.ncol)?;
    out.write_f32::<BigEndian>(header.rot as f32)?;
    out.write_f32::<BigEndian>(header.xrot as f32)?;
    out.write_f32::<BigEndian>(header.yrot as f32)?;
    out.write_i32::<BigEndian>(16)?;

    // Chunk 3: 7 dummies (28 bytes)
    out.write_i32::<BigEndian>(28)?;
    for _ in 0..7 {
        out.write_i32::<BigEndian>(0)?;
    }
    out.write_i32::<BigEndian>(28)?;

    Ok(())
}

fn write_values<W: Write>(header: &IrapHeader, values: &[f32], out: &mut W) -> std::io::Result<()> {
    let mut chunk_buffer = Vec::with_capacity(PER_LINE_BINARY);

    for row in 0..header.nrow {
        for col in 0..header.ncol {
            let idx = (col * header.nrow + row) as usize;
            let val = values[idx];
            let val_to_write = if val.is_nan() {
                UNDEF_MAP_IRAP_BINARY
            } else {
                val
            };

            chunk_buffer.push(val_to_write);

            if chunk_buffer.len() == PER_LINE_BINARY {
                let size_bytes = (PER_LINE_BINARY * 4) as i32;
                out.write_i32::<BigEndian>(size_bytes)?;
                for v in &chunk_buffer {
                    out.write_f32::<BigEndian>(*v)?;
                }
                out.write_i32::<BigEndian>(size_bytes)?;
                chunk_buffer.clear();
            }
        }
    }

    // Write remaining
    if !chunk_buffer.is_empty() {
        let size_bytes = (chunk_buffer.len() * 4) as i32;
        out.write_i32::<BigEndian>(size_bytes)?;
        for v in &chunk_buffer {
            out.write_f32::<BigEndian>(*v)?;
        }
        out.write_i32::<BigEndian>(size_bytes)?;
    }

    Ok(())
}

fn write_values_fortran<W: Write>(values: &[f32], out: &mut W) -> std::io::Result<()> {
    let mut chunk_buffer = Vec::with_capacity(PER_LINE_BINARY);

    for val in values {
        let val_to_write = if val.is_nan() {
            UNDEF_MAP_IRAP_BINARY
        } else {
            *val
        };

        chunk_buffer.push(val_to_write);

        if chunk_buffer.len() == PER_LINE_BINARY {
            // Write chunk
            let size_bytes = (PER_LINE_BINARY * 4) as i32;
            out.write_i32::<BigEndian>(size_bytes)?;
            for v in &chunk_buffer {
                out.write_f32::<BigEndian>(*v)?;
            }
            out.write_i32::<BigEndian>(size_bytes)?;
            chunk_buffer.clear();
        }
    }

    // Write remaining
    if !chunk_buffer.is_empty() {
        let size_bytes = (chunk_buffer.len() * 4) as i32;
        out.write_i32::<BigEndian>(size_bytes)?;
        for v in &chunk_buffer {
            out.write_f32::<BigEndian>(*v)?;
        }
        out.write_i32::<BigEndian>(size_bytes)?;
    }

    Ok(())
}

pub fn to_binary_file(path: String, data: &Irap) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    write_header(&data.header, &mut writer)?;
    write_values(&data.header, &data.values, &mut writer)?;

    Ok(())
}

pub fn to_binary_buffer(data: &Irap) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    write_header(&data.header, &mut buffer)?;
    write_values(&data.header, &data.values, &mut buffer)?;
    Ok(buffer)
}

pub fn to_binary_file_fortran(path: String, header: &IrapHeader, values: &[f32]) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    write_header(header, &mut writer)?;
    write_values_fortran(values, &mut writer)?;

    Ok(())
}

pub fn to_binary_buffer_fortran(header: &IrapHeader, values: &[f32]) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    write_header(header, &mut buffer)?;
    write_values_fortran(values, &mut buffer)?;
    Ok(buffer)
}
