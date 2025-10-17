use pyo3::prelude::*;

#[pyfunction]
fn parse_form(data: &str) -> PyResult<Vec<(String, String)>> {
    Ok(data
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect())
}

#[pymodule]
fn _fastform(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_form, m)?)?;
    Ok(())
}
