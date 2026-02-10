#[pymethods]
impl IrapHeader {
    #[new]
    #[pyo3(signature = (
        ncol = None, nrow = None, xori = None, yori = None, xmax = None, ymax = None,
        xinc = None, yinc = None, rot = None, xrot = None, yrot = None
    ))]
    fn py_new(
        ncol: Option<u32>,
        nrow: Option<u32>,
        xori: Option<f64>,
        yori: Option<f64>,
        xmax: Option<f64>,
        ymax: Option<f64>,
        xinc: Option<f64>,
        yinc: Option<f64>,
        rot: Option<f64>,
        xrot: Option<f64>,
        yrot: Option<f64>,
    ) -> Self {
        IrapHeader {
            ncol: ncol.unwrap_or(0),
            nrow: nrow.unwrap_or(0),
            xori: xori.unwrap_or(0.0),
            yori: yori.unwrap_or(0.0),
            xmax: xmax.unwrap_or(0.0),
            ymax: ymax.unwrap_or(0.0),
            xinc: xinc.unwrap_or(1.0),
            yinc: yinc.unwrap_or(1.0),
            rot: rot.unwrap_or(0.0),
            xrot: xrot.unwrap_or(0.0),
            yrot: yrot.unwrap_or(0.0),
        }
    }
}
impl IrapHeader {
    pub const ID: i32 = -996;
}
pub const UNDEF_MAP_IRAP_ASCII: f32 = 9999900.0;
pub const UNDEF_MAP_IRAP_BINARY: f32 = 1e30;

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
