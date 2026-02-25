use crate::irap::{Irap, IrapHeader};

use std::fs::File;
use std::io::{BufWriter, Write};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const UNDEF_MAP_IRAP_STRING: &str = "9999900.000000";

fn write_header<W: Write>(header: &IrapHeader, out: &mut W) -> std::io::Result<()> {
    let h = &header;
    writeln!(out, "{} {} {} {}", IrapHeader::ID, h.nrow, h.xinc, h.yinc)?;
    writeln!(out, "{} {} {} {}", h.xori, h.xmax, h.yori, h.ymax)?;
    writeln!(out, "{} {} {} {}", h.ncol, h.rot, h.xrot, h.yrot)?;
    writeln!(out, "0 0 0 0 0 0 0")?;
    Ok(())
}

fn write_values<W: Write>(header: &IrapHeader, values: &[f32], out: &mut W) -> std::io::Result<()> {
    let mut values_on_current_line = 0;

    // File format is Column-Major, but internal storage is Row-Major.
    for row in 0..header.nrow {
        for col in 0..header.ncol {
            let idx = (col * header.nrow + row) as usize;
            let val = values[idx];

            if values_on_current_line > 0 {
                write!(out, " ")?;
            }

            if val.is_nan() {
                write!(out, "{}", UNDEF_MAP_IRAP_STRING)?;
            } else {
                write!(out, "{:.4}", val)?;
            }

            values_on_current_line = (values_on_current_line + 1) % 8;
            if values_on_current_line == 0 {
                writeln!(out)?;
            }
        }
    }

    if values_on_current_line > 0 {
        writeln!(out)?;
    }

    Ok(())
}

fn write_values_fortran<W: Write>(values: &[f32], out: &mut W) -> std::io::Result<()> {
    let mut line = String::with_capacity(128);
    let mut values_on_current_line = 0;
    let mut ryu_buf = ryu::Buffer::new();

    for val in values {
        if values_on_current_line > 0 {
            line.push(' ');
        }
        if val.is_nan() {
            line.push_str(UNDEF_MAP_IRAP_STRING);
        } else {
            line.push_str(ryu_buf.format_finite(*val));
        }
        values_on_current_line += 1;
        if values_on_current_line == 8 {
            line.push('\n');
            out.write_all(line.as_bytes())?;
            line.clear();
            values_on_current_line = 0;
        }
    }
    if values_on_current_line > 0 {
        line.push('\n');
        out.write_all(line.as_bytes())?;
    }
    Ok(())
}

pub fn to_file(path: String, data: &Irap) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    write_header(&data.header, &mut writer)?;
    write_values(&data.header, &data.values, &mut writer)?;

    Ok(())
}

pub fn to_string(data: &Irap) -> Result<String> {
    let mut buffer = Vec::new();
    write_header(&data.header, &mut buffer)?;
    write_values(&data.header, &data.values, &mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}

pub fn to_file_fortran(path: String, header: &IrapHeader, values: &[f32]) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    write_header(header, &mut writer)?;
    write_values_fortran(values, &mut writer)?;

    Ok(())
}

pub fn to_string_fortran(header: &IrapHeader, values: &[f32]) -> Result<String> {
    let mut buffer = Vec::new();
    write_header(header, &mut buffer)?;
    write_values_fortran(values, &mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}
