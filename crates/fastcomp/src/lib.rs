use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyBytes};
use std::io::{Read, Write};

fn to_bytes<'py>(obj: &'py PyAny) -> PyResult<&'py [u8]> {
    if let Ok(bytes) = obj.downcast::<PyBytes>() {
        return Ok(bytes.as_bytes());
    }
    if let Ok(bytearray) = obj.downcast::<PyByteArray>() {
        unsafe {
            return Ok(bytearray.as_bytes());
        }
    }
    Err(PyValueError::new_err("expected bytes-like object"))
}

fn gzip_compress_impl(data: &[u8], level: u32) -> PyResult<Vec<u8>> {
    let compression = Compression::new(level as u32);
    let mut encoder = GzEncoder::new(Vec::new(), compression);
    encoder
        .write_all(data)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    encoder
        .finish()
        .map_err(|err| PyValueError::new_err(err.to_string()))
}

fn gzip_decompress_impl(data: &[u8]) -> PyResult<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    Ok(out)
}

fn brotli_compress_impl(data: &[u8], level: u32) -> PyResult<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut writer = brotli::CompressorWriter::new(&mut out, 4096, level, 22);
        writer
            .write_all(data)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
    }
    Ok(out)
}

fn brotli_decompress_impl(data: &[u8]) -> PyResult<Vec<u8>> {
    let mut out = Vec::new();
    brotli::Decompressor::new(data, 4096)
        .read_to_end(&mut out)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    Ok(out)
}

fn zstd_compress_impl(data: &[u8], level: i32) -> PyResult<Vec<u8>> {
    zstd::stream::encode_all(data, level).map_err(|err| PyValueError::new_err(err.to_string()))
}

fn zstd_decompress_impl(data: &[u8]) -> PyResult<Vec<u8>> {
    zstd::stream::decode_all(data).map_err(|err| PyValueError::new_err(err.to_string()))
}

#[pyfunction]
#[pyo3(text_signature = "(data, level=5)")]
fn gzip_compress(py: Python<'_>, data: &PyAny, level: Option<u32>) -> PyResult<Py<PyBytes>> {
    let data = to_bytes(data)?;
    let level = level.unwrap_or(5);
    let output = py.allow_threads(|| gzip_compress_impl(data, level))?;
    Ok(PyBytes::new(py, &output).into())
}

#[pyfunction]
fn gzip_decompress(py: Python<'_>, data: &PyAny) -> PyResult<Py<PyBytes>> {
    let data = to_bytes(data)?;
    let output = py.allow_threads(|| gzip_decompress_impl(data))?;
    Ok(PyBytes::new(py, &output).into())
}

#[pyfunction]
fn brotli_compress(py: Python<'_>, data: &PyAny, level: Option<u32>) -> PyResult<Py<PyBytes>> {
    let data = to_bytes(data)?;
    let output = py.allow_threads(|| brotli_compress_impl(data, level.unwrap_or(5)))?;
    Ok(PyBytes::new(py, &output).into())
}

#[pyfunction]
fn brotli_decompress(py: Python<'_>, data: &PyAny) -> PyResult<Py<PyBytes>> {
    let data = to_bytes(data)?;
    let output = py.allow_threads(|| brotli_decompress_impl(data))?;
    Ok(PyBytes::new(py, &output).into())
}

#[pyfunction]
fn zstd_compress(py: Python<'_>, data: &PyAny, level: Option<i32>) -> PyResult<Py<PyBytes>> {
    let data = to_bytes(data)?;
    let output = py.allow_threads(|| zstd_compress_impl(data, level.unwrap_or(3)))?;
    Ok(PyBytes::new(py, &output).into())
}

#[pyfunction]
fn zstd_decompress(py: Python<'_>, data: &PyAny) -> PyResult<Py<PyBytes>> {
    let data = to_bytes(data)?;
    let output = py.allow_threads(|| zstd_decompress_impl(data))?;
    Ok(PyBytes::new(py, &output).into())
}

#[pyclass(module = "fastcomp")]
struct GzipCompressor {
    level: u32,
}

#[pymethods]
impl GzipCompressor {
    #[new]
    fn new(level: Option<u32>) -> PyResult<Self> {
        Ok(Self {
            level: level.unwrap_or(5),
        })
    }

    fn update<'py>(&mut self, py: Python<'py>, data: &PyAny) -> PyResult<&'py PyBytes> {
        let data = to_bytes(data)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.level));
        encoder
            .write_all(data)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let chunk = encoder
            .finish()
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        Ok(PyBytes::new(py, &chunk))
    }

    fn finish<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyBytes> {
        Ok(PyBytes::new(py, &[]))
    }
}

#[pyclass(module = "fastcomp")]
struct BrotliCompressor {
    level: u32,
}

#[pymethods]
impl BrotliCompressor {
    #[new]
    fn new(level: Option<u32>) -> PyResult<Self> {
        Ok(Self {
            level: level.unwrap_or(5),
        })
    }

    fn update<'py>(&mut self, py: Python<'py>, data: &PyAny) -> PyResult<&'py PyBytes> {
        let data = to_bytes(data)?;
        let output = brotli_compress_impl(data, self.level)?;
        Ok(PyBytes::new(py, &output))
    }

    fn finish<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyBytes> {
        Ok(PyBytes::new(py, &[]))
    }
}

#[pyclass(module = "fastcomp")]
struct ZstdCompressor {
    level: i32,
}

#[pymethods]
impl ZstdCompressor {
    #[new]
    fn new(level: Option<i32>) -> PyResult<Self> {
        Ok(Self {
            level: level.unwrap_or(3),
        })
    }

    fn update<'py>(&mut self, py: Python<'py>, data: &PyAny) -> PyResult<&'py PyBytes> {
        let data = to_bytes(data)?;
        let output = zstd_compress_impl(data, self.level)?;
        Ok(PyBytes::new(py, &output))
    }

    fn finish<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyBytes> {
        Ok(PyBytes::new(py, &[]))
    }
}

#[pymodule]
fn _fastcomp(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gzip_compress, m)?)?;
    m.add_function(wrap_pyfunction!(gzip_decompress, m)?)?;
    m.add_function(wrap_pyfunction!(brotli_compress, m)?)?;
    m.add_function(wrap_pyfunction!(brotli_decompress, m)?)?;
    m.add_function(wrap_pyfunction!(zstd_compress, m)?)?;
    m.add_function(wrap_pyfunction!(zstd_decompress, m)?)?;
    m.add_class::<GzipCompressor>()?;
    m.add_class::<BrotliCompressor>()?;
    m.add_class::<ZstdCompressor>()?;
    Ok(())
}
