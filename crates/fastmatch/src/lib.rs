use pyo3::prelude::*;

#[pyclass(module = "fastmatch")]
pub struct Router;

#[pymethods]
impl Router {
    #[new]
    fn new() -> Self {
        Self
    }

    fn lookup(&self, method: &str, path: &str) -> PyResult<(String, String)> {
        Ok((method.to_owned(), path.to_owned()))
    }
}

#[pymodule]
fn _fastmatch(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Router>()?;
    Ok(())
}
