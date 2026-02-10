use crate::irap::{self, Irap, IrapHeader};
use crate::utils;
use byteorder::{BigEndian, ReadBytesExt};
use memmap::Mmap;
use std::fs::File;
use std::io::Cursor;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn read_header(cursor: &mut Cursor<&[u8]>) -> Result<IrapHeader> {
    // Header is 100 bytes long.
    // Chunk guards are 4 bytes each.
    // Structure:
    // Guard (4)
    // ID, nrow, xori, xmax, yori, ymax, xinc, yinc (8 values * 4 = 32 bytes)
    // Guard (4)

    // Guard (4)
    // ncol, rot, xrot, yrot (4 values * 4 = 16 bytes)
    // Guard (4)

    // Guard (4)
    // 7 dummy values (7 * 4 = 28 bytes)
    // Guard (4)

    // Total: 4+32+4 + 4+16+4 + 4+28+4 = 100 bytes.

    let mut header = IrapHeader::default();

    // Chunk 1
    let chunk1_size = cursor.read_i32::<BigEndian>()?;
    if chunk1_size != 32 {
        return Err(format!("Incorrect chunk 1 size: {}", chunk1_size).into());
    }

    let id = cursor.read_i32::<BigEndian>()?;
    if id != IrapHeader::ID {
        return Err(format!(
            "Incorrect magic number: {}. Expected {}",
            id,
            IrapHeader::ID
        )
        .into());
    }

    header.nrow = cursor.read_i32::<BigEndian>()? as u32;
    header.xori = cursor.read_f32::<BigEndian>()? as f64;
    header.xmax = cursor.read_f32::<BigEndian>()? as f64;
    header.yori = cursor.read_f32::<BigEndian>()? as f64;
    header.ymax = cursor.read_f32::<BigEndian>()? as f64;
    header.xinc = cursor.read_f32::<BigEndian>()? as f64;
    header.yinc = cursor.read_f32::<BigEndian>()? as f64;

    let chunk1_end_guard = cursor.read_i32::<BigEndian>()?;
    if chunk1_end_guard != chunk1_size {
        return Err(format!("Chunk 1 end guard mismatch: {}", chunk1_end_guard).into());
    }

    // Chunk 2
    let chunk2_size = cursor.read_i32::<BigEndian>()?;
    if chunk2_size != 16 {
        return Err(format!("Incorrect chunk 2 size: {}", chunk2_size).into());
    }

    header.ncol = cursor.read_i32::<BigEndian>()? as u32;
    header.rot = cursor.read_f32::<BigEndian>()? as f64;
    header.xrot = cursor.read_f32::<BigEndian>()? as f64;
    header.yrot = cursor.read_f32::<BigEndian>()? as f64;

    let chunk2_end_guard = cursor.read_i32::<BigEndian>()?;
    if chunk2_end_guard != chunk2_size {
        return Err(format!("Chunk 2 end guard mismatch: {}", chunk2_end_guard).into());
    }

    // Chunk 3 (Dummies)
    let chunk3_size = cursor.read_i32::<BigEndian>()?;
    if chunk3_size != 28 {
        return Err(format!("Incorrect chunk 3 size: {}", chunk3_size).into());
    }

    // Skip 7 dummies (4 bytes each)
    cursor.set_position(cursor.position() + 28);

    let chunk3_end_guard = cursor.read_i32::<BigEndian>()?;
    if chunk3_end_guard != chunk3_size {
        return Err(format!("Chunk 3 end guard mismatch: {}", chunk3_end_guard).into());
    }

    Ok(header)
}

fn read_values(cursor: &mut Cursor<&[u8]>, ncol: usize, nrow: usize) -> Result<Vec<f32>> {
    let nvalues = ncol * nrow;
    let mut values = vec![0.0; nvalues];

    let mut i = 0;
    while i < nvalues {
        let chunk_size_bytes = cursor.read_i32::<BigEndian>()?;
        let items_in_chunk = (chunk_size_bytes / 4) as usize;

        // if items_in_chunk > (nvalues - i) return error?

        for _ in 0..items_in_chunk {
            if i >= nvalues {
                break;
            }

            let val = cursor.read_f32::<BigEndian>()?;
            let ic = utils::column_major_to_row_major_index(i, ncol, nrow);

            if val >= irap::UNDEF_MAP_IRAP_BINARY {
                values[ic] = f32::NAN;
            } else {
                values[ic] = val;
            }
            i += 1;
        }

        let end_guard = cursor.read_i32::<BigEndian>()?;
        if end_guard != chunk_size_bytes {
            return Err(format!(
                "Block size mismatch. Start: {}, End: {}",
                chunk_size_bytes, end_guard
            )
            .into());
        }
    }

    Ok(values)
}

pub fn read_file(path: String) -> Result<Irap> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let mut cursor = Cursor::new(&mmap[..]);

    let header = read_header(&mut cursor)?;
    let values = read_values(&mut cursor, header.ncol as usize, header.nrow as usize)?;

    Ok(Irap { header, values })
}

pub fn read_buffer(buffer: &[u8]) -> Result<Irap> {
    let mut cursor = Cursor::new(buffer);
    let header = read_header(&mut cursor)?;
    let values = read_values(&mut cursor, header.ncol as usize, header.nrow as usize)?;

    Ok(Irap { header, values })
}
