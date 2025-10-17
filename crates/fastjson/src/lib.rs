use pyo3::prelude::*;
use pyo3::types::PyAny;

#[pyfunction]
fn loads(data: &str) -> PyResult<String> {
    Ok(data.to_owned())
}

#[pyfunction]
fn dumps(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    Ok(obj.str()?.to_str()?.to_owned())
}

#[pymodule]
fn _fastjson(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(loads, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    Ok(())
}
