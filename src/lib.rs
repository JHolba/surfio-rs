use pyo3::prelude::*;
use pyo3::types::PyBytes;

pub mod irap;
mod utils;

pub use irap::{Irap, IrapHeader};
use numpy::ndarray::Array2;
use numpy::{IntoPyArray, PyArray2, PyArrayMethods, PyUntypedArrayMethods};

#[pyclass(from_py_object, name = "IrapSurface")]
#[derive(Debug)]
pub struct IrapSurface {
    #[pyo3(get, set)]
    pub header: Py<IrapHeader>,
    #[pyo3(get, set)]
    pub values: Py<PyArray2<f32>>,
}

impl Default for IrapSurface {
    fn default() -> Self {
        pyo3::Python::attach(|py| {
            let header = Py::new(py, IrapHeader::default()).unwrap();
            let values = PyArray2::<f32>::zeros(py, (0, 0), false).into();
            IrapSurface { header, values }
        })
    }
}

impl Clone for IrapSurface {
    fn clone(&self) -> Self {
        pyo3::Python::attach(|py| IrapSurface {
            header: self.header.clone_ref(py),
            values: self.values.clone_ref(py),
        })
    }
}

impl PartialEq for IrapSurface {
    fn eq(&self, other: &Self) -> bool {
        pyo3::Python::attach(|py| {
            // Compare header by extracting and using PartialEq
            let h1 = self.header.bind(py).extract::<IrapHeader>().ok();
            let h2 = other.header.bind(py).extract::<IrapHeader>().ok();
            if h1 != h2 {
                return false;
            }
            // Compare values by extracting as slices
            let arr1 = self.values.bind(py).readonly();
            let arr2 = other.values.bind(py).readonly();
            arr1.as_slice().ok() == arr2.as_slice().ok()
        })
    }
}

#[pymethods]
impl IrapSurface {
    #[new]
    fn py_new(_py: Python, header: Py<IrapHeader>, values: Py<PyArray2<f32>>) -> Self {
        IrapSurface { header, values }
    }

    fn __repr__(&self, py: Python) -> String {
        let header: IrapHeader = self.header.extract(py).unwrap_or_default();
        format!(
            "<IrapSurface(header=IrapHeader(ncol={}, nrow={}, xori={}, yori={}, xmax={}, ymax={}, xinc={}, yinc={}, rot={}, xrot={}, yrot={}), values=...)>",
            header.ncol,
            header.nrow,
            header.xori,
            header.yori,
            header.xmax,
            header.ymax,
            header.xinc,
            header.yinc,
            header.rot,
            header.xrot,
            header.yrot
        )
    }

    fn __str__(&self, py: Python) -> String {
        self.__repr__(py)
    }

    #[staticmethod]
    fn from_ascii_file(py: Python, path: String) -> PyResult<IrapSurface> {
        let irap = irap::ascii::from_file(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        irap_to_surface(py, &irap)
    }

    #[staticmethod]
    fn from_ascii_string(py: Python, data: String) -> PyResult<IrapSurface> {
        let irap = irap::ascii::from_string(&data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        irap_to_surface(py, &irap)
    }

    #[staticmethod]
    fn from_binary_file(py: Python, path: String) -> PyResult<IrapSurface> {
        let irap = irap::binary::from_file(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        irap_to_surface(py, &irap)
    }

    #[staticmethod]
    fn from_binary_buffer(py: Python, data: &[u8]) -> PyResult<IrapSurface> {
        let irap = irap::binary::from_buffer(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        irap_to_surface(py, &irap)
    }

    fn to_ascii_string(&self, py: Python) -> PyResult<String> {
        let arr = self.values.as_ref();
        let arr = arr.cast_bound::<PyArray2<f32>>(py).unwrap();
        let f_ordered = arr.is_fortran_contiguous();

        if f_ordered {
            let slice: &[f32] = unsafe { std::slice::from_raw_parts(arr.data(), arr.len()) };
            let mut header: IrapHeader = self
                .header
                .extract(py)
                .expect("Unable to extract Irap header");
            utils::fill_header(&mut header);
            irap::ascii::to_string_fortran(&header, slice)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
        } else {
            let data = surface_to_irap(py, self);
            irap::ascii::to_string(&data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
        }
    }

    fn to_ascii_file(&self, py: Python, path: String) -> PyResult<()> {
        let arr = self.values.as_ref();
        let arr = arr.cast_bound::<PyArray2<f32>>(py).unwrap();
        let f_ordered = arr.is_fortran_contiguous();

        if f_ordered {
            let slice: &[f32] = unsafe { std::slice::from_raw_parts(arr.data(), arr.len()) };
            let mut header: IrapHeader = self
                .header
                .extract(py)
                .expect("Unable to extract Irap header");
            utils::fill_header(&mut header);
            irap::ascii::to_file_fortran(path, &header, slice)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
        } else {
            let data = surface_to_irap(py, self);
            irap::ascii::to_file(path, &data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
        }
    }

    fn to_binary_buffer<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let arr = self.values.as_ref();
        let arr = arr.cast_bound::<PyArray2<f32>>(py).unwrap();
        let f_ordered = arr.is_fortran_contiguous();

        let bytes = if f_ordered {
            let slice: &[f32] = unsafe { std::slice::from_raw_parts(arr.data(), arr.len()) };
            let mut header: IrapHeader = self
                .header
                .extract(py)
                .expect("Unable to extract Irap header");
            utils::fill_header(&mut header);
            irap::binary::to_buffer_fortran(&header, slice)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
        } else {
            let data = surface_to_irap(py, self);
            irap::binary::to_buffer(&data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
        };

        Ok(PyBytes::new(py, &bytes))
    }

    fn to_binary_file(&self, py: Python, path: String) -> PyResult<()> {
        let arr = self.values.as_ref();
        let arr = arr.cast_bound::<PyArray2<f32>>(py).unwrap();
        let f_ordered = arr.is_fortran_contiguous();

        if f_ordered {
            let slice: &[f32] = unsafe { std::slice::from_raw_parts(arr.data(), arr.len()) };
            let mut header: IrapHeader = self
                .header
                .extract(py)
                .expect("Unable to extract Irap header");
            utils::fill_header(&mut header);
            irap::binary::to_file_fortran(path, &header, slice)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
        } else {
            let data = surface_to_irap(py, self);
            irap::binary::to_file(path, &data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
        }
    }
}

pub fn irap_to_surface<'py>(py: Python<'py>, irap: &Irap) -> PyResult<IrapSurface> {
    let h = irap.header.clone();
    let shape = (h.ncol as usize, h.nrow as usize);
    let np_arr = Array2::from_shape_vec(shape, irap.values.clone()).expect("Error reshaping array");
    let values = np_arr.into_pyarray(py);
    Ok(IrapSurface {
        header: Py::new(py, h).expect("Failed to create new IrapHeader"),
        values: values.into(),
    })
}

fn surface_to_irap(py: Python, surface: &IrapSurface) -> Irap {
    let mut header: IrapHeader = surface
        .header
        .extract(py)
        .expect("Unable to extract Irap header");
    utils::fill_header(&mut header);
    let arr = surface.values.as_ref();
    let arr = arr.cast_bound::<PyArray2<f32>>(py).unwrap();

    let values = arr.readonly().as_slice().unwrap().to_vec();
    Irap { header, values }
}

#[pymodule]
fn surfio_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<IrapSurface>()?;
    m.add_class::<IrapHeader>()?;
    Ok(())
}
