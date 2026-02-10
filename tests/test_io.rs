use std::sync::Once;
use surfio_rs::{
    Irap, IrapHeader, read_irap_ascii_file, read_irap_ascii_string, read_irap_binary_file,
    write_irap_ascii_file, write_irap_ascii_string, write_irap_binary_file,
};

static INIT: Once = Once::new();

fn create_dummy_irap() -> Irap {
    INIT.call_once(|| {
        pyo3::prepare_freethreaded_python();
    });

    let mut header = IrapHeader::default();
    header.ncol = 3;
    header.nrow = 2;
    header.xori = 100.0;
    header.yori = 200.0;
    header.xmax = 100.0 + (3.0 - 1.0) * 10.0;
    header.ymax = 200.0 + (2.0 - 1.0) * 10.0;
    header.xinc = 10.0;
    header.yinc = 10.0;

    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

    // In code we stored row-major for convenience but export handles column-major.
    // Wait, the Irap struct holds values in internal order which is row-major.
    // 3 cols, 2 rows.
    // Row 0: 1.0 2.0 3.0
    // Row 1: 4.0 5.0 6.0
    // Flattened: 1, 2, 3, 4, 5, 6.

    Irap { header, values }
}

#[test]
fn test_round_trip_ascii_string() {
    let irap = create_dummy_irap();
    let ascii = write_irap_ascii_string(&irap).unwrap();
    let irap_read = read_irap_ascii_string(ascii).unwrap();

    assert_eq!(irap.header, irap_read.header);
    assert_eq!(irap.values, irap_read.values);
}

#[test]
fn test_round_trip_binary_buffer() {
    // Need Python context for `write_irap_binary_buffer` because it returns PyBytes?
    // Wait, the Rust function signature I created in lib.rs takes `Python`.
    // That makes it hard to test without initializing Python interpreter.
    // However, I can test `import_irap_binary` and `export_irap_binary` directly if I expose them via lib or make them pub modules.
    // I made the *wrapper* functions public in lib.rs, but `write_irap_binary_buffer` requires Python.
    // `export_irap_binary::to_binary_buffer` returns `Result<Vec<u8>>` and does not need Python.
    // But `export_irap_binary` module is private in `lib.rs` (I didn't make `mod export_irap_binary` public, I only made the functions public).
    // Ah, I did `mod export_irap_ascii;` only, not `pub mod`.
    // But I can't access `export_irap_binary` from outside unless I make it `pub mod`.

    // Alternative: Use `write_irap_binary_file` which does not need Python context.

    let irap = create_dummy_irap();
    let path = "test_output.grd";
    write_irap_binary_file(path.to_string(), &irap).unwrap();
    let irap_read = read_irap_binary_file(path.to_string()).unwrap();

    assert_eq!(irap.header, irap_read.header);
    assert_eq!(irap.values, irap_read.values);

    let _ = std::fs::remove_file(path);
}

#[test]
fn test_round_trip_ascii_file() {
    let irap = create_dummy_irap();
    let path = "test_output.irap";
    write_irap_ascii_file(path.to_string(), &irap).unwrap();
    let irap_read = read_irap_ascii_file(path.to_string()).unwrap();

    assert_eq!(irap.header, irap_read.header);
    assert_eq!(irap.values, irap_read.values);

    let _ = std::fs::remove_file(path);
}
