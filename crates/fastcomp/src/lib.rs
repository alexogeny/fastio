use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyfunction]
fn compress(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyBytes>> {
    Ok(PyBytes::new_bound(py, data).into())
}

#[pyfunction]
fn decompress(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyBytes>> {
    Ok(PyBytes::new_bound(py, data).into())
}

#[pymodule]
fn _fastcomp(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compress, m)?)?;
    m.add_function(wrap_pyfunction!(decompress, m)?)?;
    Ok(())
}
