use core::default::Default;
use surfio_rs::{Irap, IrapHeader, irap};

fn create_dummy_irap() -> Irap {
    let header = IrapHeader {
        ncol: 3,
        nrow: 2,
        xori: 100.0,
        yori: 200.0,
        xmax: 100.0 + (3.0 - 1.0) * 10.0,
        ymax: 200.0 + (2.0 - 1.0) * 10.0,
        xinc: 10.0,
        yinc: 10.0,
        rot: Default::default(),
        xrot: Default::default(),
        yrot: Default::default(),
    };

    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

    Irap { header, values }
}

#[test]
fn test_round_trip_ascii_string() {
    let irap = create_dummy_irap();
    let ascii = irap::ascii::to_string(&irap).unwrap();
    let irap_read = irap::ascii::from_string(&ascii).unwrap();

    assert_eq!(irap.header, irap_read.header);
    assert_eq!(irap.values, irap_read.values);
}

#[test]
fn test_round_trip_binary_buffer() {
    let irap = create_dummy_irap();
    let path = "test_output.grd";
    irap::binary::to_file(path.to_string(), &irap).unwrap();
    let irap_read = irap::binary::from_file(path.to_string()).unwrap();

    assert_eq!(irap.header, irap_read.header);
    assert_eq!(irap.values, irap_read.values);

    let _ = std::fs::remove_file(path);
}

#[test]
fn test_round_trip_ascii_file() {
    let irap = create_dummy_irap();
    let path = "test_output.irap";
    irap::ascii::to_file(path.to_string(), &irap).unwrap();
    let irap_read = irap::ascii::from_file(path.to_string()).unwrap();

    assert_eq!(irap.header, irap_read.header);
    assert_eq!(irap.values, irap_read.values);

    let _ = std::fs::remove_file(path);
}
