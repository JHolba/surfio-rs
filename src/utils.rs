pub fn column_major_to_row_major_index(idx: usize, ncol: usize, nrow: usize) -> usize {
    idx / ncol + (idx % ncol) * nrow
}

pub fn fill_header(header: &mut crate::irap::IrapHeader) {
    header.xmax = header.xori + (header.ncol - 1) as f64 * header.xinc;
    header.ymax = header.yori + (header.nrow - 1) as f64 * header.yinc;
}
