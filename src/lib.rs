use pyo3::prelude::*;
use pyo3::types::PyBytes;

mod export_irap_ascii;
mod export_irap_binary;
mod import_irap_ascii;
mod import_irap_binary;
mod irap;
mod utils;

use crate::irap::Irap;
pub use irap::IrapHeader;
use numpy::ndarray::Array2;
use numpy::{IntoPyArray, PyArray2, PyArrayMethods, PyUntypedArrayMethods};

#[pyclass(from_py_object, name = "Irap")]
#[derive(Debug)]
pub struct PythonIrap {
    #[pyo3(get, set)]
    pub header: Py<IrapHeader>,
    #[pyo3(get, set)]
    pub values: Py<PyArray2<f32>>,
}

impl Default for PythonIrap {
    fn default() -> Self {
        pyo3::Python::attach(|py| {
            let header = Py::new(py, IrapHeader::default()).unwrap();
            let values = PyArray2::<f32>::zeros(py, (0, 0), false).into();
            PythonIrap { header, values }
        })
    }
}

impl Clone for PythonIrap {
    fn clone(&self) -> Self {
        pyo3::Python::attach(|py| PythonIrap {
            header: self.header.clone_ref(py),
            values: self.values.clone_ref(py),
        })
    }
}

impl PartialEq for PythonIrap {
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
impl PythonIrap {
    #[new]
    #[pyo3(signature = (header = None, values = None))]
    fn py_new(
        py: Python,
        header: Option<Py<IrapHeader>>,
        values: Option<Py<PyArray2<f32>>>,
    ) -> Self {
        let header = header.unwrap_or_else(|| Py::new(py, IrapHeader::default()).unwrap());
        let values = values.unwrap_or_else(|| PyArray2::<f32>::zeros(py, (0, 0), false).into());
        PythonIrap { header, values }
    }

    fn __repr__(&self) -> String {
        format!("<IrapSurface(header={:?}, values=...)>", self.header)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

pub fn irap_to_mutable<'py>(py: Python<'py>, irap: &Irap) -> PyResult<PythonIrap> {
    let h = irap.header.clone();
    let shape = (h.ncol as usize, h.nrow as usize);
    let np_arr = Array2::from_shape_vec(shape, irap.values.clone()).expect("Error reshaping array");
    let values = np_arr.into_pyarray(py);
    Ok(PythonIrap {
        header: Py::new(py, h).expect("Failed to create new IrapHeader"),
        values: values.into(),
    })
}

fn mutable_to_irap(py: Python, m: &PythonIrap) -> Irap {
    let mut header: IrapHeader = m.header.extract(py).expect("Unable to extract Irap header");
    utils::fill_header(&mut header);
    let arr = m.values.as_ref();
    let arr = arr.cast_bound::<PyArray2<f32>>(py).unwrap();

    let values = arr.readonly().as_slice().unwrap().to_vec();
    Irap { header, values }
}

#[pyfunction]
pub fn read_irap_ascii_file(py: Python, path: String) -> PyResult<PythonIrap> {
    let irap = import_irap_ascii::read_file(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    irap_to_mutable(py, &irap)
}

#[pyfunction]
pub fn read_irap_ascii_string(py: Python, data: String) -> PyResult<PythonIrap> {
    let irap = import_irap_ascii::read_string(&data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    irap_to_mutable(py, &irap)
}

#[pyfunction]
pub fn read_irap_binary_file(py: Python, path: String) -> PyResult<PythonIrap> {
    let irap = import_irap_binary::read_file(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    irap_to_mutable(py, &irap)
}

#[pyfunction]
pub fn read_irap_binary_buffer(py: Python, data: &[u8]) -> PyResult<PythonIrap> {
    let irap = import_irap_binary::read_buffer(data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    irap_to_mutable(py, &irap)
}

#[pyfunction]
pub fn write_irap_ascii_file(py: Python, path: String, data: &PythonIrap) -> PyResult<()> {
    let arr = data.values.as_ref();
    let arr = arr.cast_bound::<PyArray2<f32>>(py).unwrap();
    let f_ordered = arr.is_fortran_contiguous();

    if f_ordered {
        let slice: &[f32] = unsafe { std::slice::from_raw_parts(arr.data(), arr.len()) };
        let mut header: IrapHeader = data.header.extract(py).expect("Unable to extract Irap header");
        utils::fill_header(&mut header);
        export_irap_ascii::to_ascii_file_fortran(path, &header, slice)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    } else {
        let data = mutable_to_irap(py, data);
        export_irap_ascii::to_ascii_file(path, &data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
    }

}

#[pyfunction]
pub fn write_irap_ascii_string(py: Python, data: &PythonIrap) -> PyResult<String> {
    let arr = data.values.as_ref();
    let arr = arr.cast_bound::<PyArray2<f32>>(py).unwrap();
    let f_ordered = arr.is_fortran_contiguous();

    if f_ordered {
        let slice: &[f32] = unsafe { std::slice::from_raw_parts(arr.data(), arr.len()) };
        let mut header: IrapHeader = data.header.extract(py).expect("Unable to extract Irap header");
        utils::fill_header(&mut header);
        export_irap_ascii::to_ascii_string_fortran(&header, slice)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
    } else {
        let data = mutable_to_irap(py, data);
        export_irap_ascii::to_ascii_string(&data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
}

#[pyfunction]
pub fn write_irap_binary_file(py: Python, path: String, data: &PythonIrap) -> PyResult<()> {
    let arr = data.values.as_ref();
    let arr = arr.cast_bound::<PyArray2<f32>>(py).unwrap();
    let f_ordered = arr.is_fortran_contiguous();

    if f_ordered {
        let slice: &[f32] = unsafe { std::slice::from_raw_parts(arr.data(), arr.len()) };
        let mut header: IrapHeader = data.header.extract(py).expect("Unable to extract Irap header");
        utils::fill_header(&mut header);
        export_irap_binary::to_binary_file_fortran(path, &header, slice)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
    } else {
        let data = mutable_to_irap(py, data);
        export_irap_binary::to_binary_file(path, &data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
    }
}

#[pyfunction]
pub fn write_irap_binary_buffer<'py>(
    py: Python<'py>,
    data: &PythonIrap,
) -> PyResult<Bound<'py, PyBytes>> {
    let arr = data.values.as_ref();
    let arr = arr.cast_bound::<PyArray2<f32>>(py).unwrap();
    let f_ordered = arr.is_fortran_contiguous();


    let bytes = if f_ordered {
        let slice: &[f32] = unsafe { std::slice::from_raw_parts(arr.data(), arr.len()) };
        let mut header: IrapHeader = data.header.extract(py).expect("Unable to extract Irap header");
        utils::fill_header(&mut header);
        export_irap_binary::to_binary_buffer_fortran(&header, slice)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?
    } else {
        let data = mutable_to_irap(py, data);
        export_irap_binary::to_binary_buffer(&data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
    };
    Ok(PyBytes::new(py, &bytes))
}

/// A Python module implemented in Rust.
#[pymodule]
fn _surfio_rs_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PythonIrap>()?;
    m.add_class::<IrapHeader>()?;
    m.add_function(wrap_pyfunction!(read_irap_ascii_file, m)?)?;
    m.add_function(wrap_pyfunction!(read_irap_ascii_string, m)?)?;
    m.add_function(wrap_pyfunction!(read_irap_binary_file, m)?)?;
    m.add_function(wrap_pyfunction!(read_irap_binary_buffer, m)?)?;
    m.add_function(wrap_pyfunction!(write_irap_ascii_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_irap_ascii_string, m)?)?;
    m.add_function(wrap_pyfunction!(write_irap_binary_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_irap_binary_buffer, m)?)?;
    Ok(())
}
