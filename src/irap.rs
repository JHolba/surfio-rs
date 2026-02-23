use pyo3::prelude::*;

pub const IRAP_HEADER_ID: i32 = -996;
pub const UNDEF_MAP_IRAP_ASCII: f32 = 9999900.0;
pub const UNDEF_MAP_IRAP_BINARY: f32 = 1e30;

#[pymethods]
impl IrapHeader {
    #[new]
    #[pyo3(signature = (
        ncol, nrow, xori = 0.0, yori = 0.0, xmax = 0.0, ymax = 0.0,
        xinc = 1.0, yinc = 1.0, rot = 0.0, xrot = 0.0, yrot = 0.0
    ))]
    fn py_new(
        ncol: u32,
        nrow: u32,
        xori: f64,
        yori: f64,
        xmax: f64,
        ymax: f64,
        xinc: f64,
        yinc: f64,
        rot: f64,
        xrot: f64,
        yrot: f64,
    ) -> Self {
        IrapHeader {
            ncol,
            nrow,
            xori,
            yori,
            xmax,
            ymax,
            xinc,
            yinc,
            rot,
            xrot,
            yrot,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "<IrapHeader(ncol={}, nrow={}, xori={}, yori={}, xmax={}, ymax={}, xinc={}, yinc={}, rot={}, xrot={}, yrot={})>",
            self.ncol,
            self.nrow,
            self.xori,
            self.yori,
            self.xmax,
            self.ymax,
            self.xinc,
            self.yinc,
            self.rot,
            self.xrot,
            self.yrot
        )
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }

    fn __ne__(&self, other: &Self) -> bool {
        self != other
    }

    #[classattr]
    fn id() -> i32 {
        IRAP_HEADER_ID
    }
}

impl IrapHeader {
    pub const ID: i32 = IRAP_HEADER_ID;
}

#[pyclass(from_py_object)]
#[derive(Default, Clone, PartialEq, Debug)]
pub struct IrapHeader {
    #[pyo3(get, set)]
    pub ncol: u32,
    #[pyo3(get, set)]
    pub nrow: u32,
    #[pyo3(get, set)]
    pub xori: f64,
    #[pyo3(get, set)]
    pub yori: f64,
    #[pyo3(get, set)]
    pub xmax: f64,
    #[pyo3(get, set)]
    pub ymax: f64,
    #[pyo3(get, set)]
    pub xinc: f64,
    #[pyo3(get, set)]
    pub yinc: f64,
    #[pyo3(get, set)]
    pub rot: f64,
    #[pyo3(get, set)]
    pub xrot: f64,
    #[pyo3(get, set)]
    pub yrot: f64,
}

use pyo3::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Irap {
    pub header: IrapHeader,
    pub values: Vec<f32>,
}
