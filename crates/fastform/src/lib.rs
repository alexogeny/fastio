use memchr::memmem::Finder;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyByteArray, PyBytes, PyDict};
use std::collections::HashMap;

fn read_bytes<'py>(obj: &'py PyAny) -> PyResult<&'py [u8]> {
    if let Ok(bytes) = obj.downcast::<PyBytes>() {
        return Ok(bytes.as_bytes());
    }
    if let Ok(ba) = obj.downcast::<PyByteArray>() {
        unsafe {
            return Ok(ba.as_bytes());
        }
    }
    Err(PyValueError::new_err("expected bytes-like object"))
}

fn percent_decode(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut idx = 0;
    while idx < input.len() {
        match input[idx] {
            b'+' => {
                out.push(b' ');
                idx += 1;
            }
            b'%' if idx + 2 < input.len() => {
                let hex = &input[idx + 1..idx + 3];
                if let Ok(value) = u8::from_str_radix(std::str::from_utf8(hex).unwrap_or("00"), 16)
                {
                    out.push(value);
                    idx += 3;
                } else {
                    out.push(b'%');
                    idx += 1;
                }
            }
            byte => {
                out.push(byte);
                idx += 1;
            }
        }
    }
    out
}

#[pyfunction]
fn parse_query(_py: Python<'_>, data: &PyAny) -> PyResult<Vec<(Vec<u8>, Vec<u8>)>> {
    let data = read_bytes(data)?;
    let mut pairs = Vec::new();
    for slice in data.split(|b| *b == b'&') {
        if slice.is_empty() {
            continue;
        }
        let mut split = slice.splitn(2, |b| *b == b'=');
        let key = split.next().unwrap_or(&[]);
        let value = split.next().unwrap_or(&[]);
        pairs.push((percent_decode(key), percent_decode(value)));
    }
    Ok(pairs)
}

#[pyclass(module = "fastform")]
struct Part {
    #[pyo3(get)]
    headers: Py<PyDict>,
    #[pyo3(get)]
    filename: Option<String>,
    #[pyo3(get)]
    name: Option<String>,
    #[pyo3(get)]
    data: Py<PyBytes>,
}

fn read_stream(_py: Python<'_>, stream: &PyAny) -> PyResult<Vec<u8>> {
    if let Ok(bytes) = read_bytes(stream) {
        return Ok(bytes.to_vec());
    }
    if stream.hasattr("read")? {
        let mut buffer = Vec::new();
        loop {
            let chunk = stream.call_method1("read", (8192,))?;
            if chunk.is_none() {
                break;
            }
            if let Ok(bytes) = chunk.downcast::<PyBytes>() {
                let data = bytes.as_bytes();
                if data.is_empty() {
                    break;
                }
                buffer.extend_from_slice(data);
            } else if let Ok(bytearray) = chunk.downcast::<PyByteArray>() {
                let data = unsafe { bytearray.as_bytes() };
                if data.is_empty() {
                    break;
                }
                buffer.extend_from_slice(data);
            } else {
                return Err(PyValueError::new_err("read() must return bytes"));
            }
        }
        return Ok(buffer);
    }
    Err(PyValueError::new_err("object does not support read"))
}

fn parse_headers(section: &str) -> HashMap<String, Vec<u8>> {
    let mut headers = HashMap::new();
    for line in section.lines() {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(
                name.trim().to_ascii_lowercase(),
                value.trim().as_bytes().to_vec(),
            );
        }
    }
    headers
}

fn parse_content_disposition(value: &[u8]) -> (Option<String>, Option<String>) {
    let string = String::from_utf8_lossy(value);
    let mut name = None;
    let mut filename = None;
    for part in string.split(';') {
        let trimmed = part.trim();
        if let Some(rest) = trimmed.strip_prefix("name=") {
            name = Some(rest.trim_matches('"').to_string());
        } else if let Some(rest) = trimmed.strip_prefix("filename=") {
            filename = Some(rest.trim_matches('"').to_string());
        }
    }
    (name, filename)
}

fn parse_multipart_impl<'py>(py: Python<'py>, body: &[u8], boundary: &[u8]) -> PyResult<Vec<Part>> {
    let delimiter = {
        let mut v = Vec::with_capacity(boundary.len() + 2);
        v.extend_from_slice(b"--");
        v.extend_from_slice(boundary);
        v
    };
    if !body.starts_with(&delimiter) {
        return Err(PyValueError::new_err("invalid multipart body"));
    }
    let mut parts = Vec::new();
    let finder = Finder::new(&delimiter);
    let mut index = 0;
    let mut offset = 0;
    while let Some(pos) = finder.find(&body[offset..]) {
        let absolute = offset + pos;
        if absolute == 0 {
            offset = absolute + delimiter.len();
            if body.get(offset..offset + 2) == Some(b"\r\n") {
                offset += 2;
            }
            index = offset;
            continue;
        }
        let end = absolute - 2; // strip CRLF
        let section = &body[index..end];
        if section.starts_with(b"--") {
            break;
        }
        let headers_end = section
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|pos| pos + 4)
            .ok_or_else(|| PyValueError::new_err("missing header terminator"))?;
        let header_bytes = &section[..headers_end - 4];
        let body_bytes = &section[headers_end..];
        let header_string = String::from_utf8_lossy(header_bytes);
        let headers_map = parse_headers(&header_string);
        let headers_dict = PyDict::new(py);
        for (name, value) in &headers_map {
            headers_dict.set_item(name, PyBytes::new(py, value))?;
        }
        let (name, filename) = headers_map
            .get("content-disposition")
            .map(|value| parse_content_disposition(value))
            .unwrap_or((None, None));
        let data = PyBytes::new(py, body_bytes).into();
        parts.push(Part {
            headers: headers_dict.into(),
            filename,
            name,
            data,
        });
        offset = absolute + delimiter.len();
        if body.get(offset..offset + 2) == Some(b"--") {
            break;
        }
        if body.get(offset..offset + 2) == Some(b"\r\n") {
            offset += 2;
        }
        index = offset;
    }
    Ok(parts)
}

#[pyfunction]
fn parse_multipart(py: Python<'_>, stream: &PyAny, boundary: &PyAny) -> PyResult<Vec<Part>> {
    let boundary = read_bytes(boundary)?;
    let body = read_stream(py, stream)?;
    parse_multipart_impl(py, &body, boundary)
}

#[pymodule]
fn _fastform(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_query, m)?)?;
    m.add_function(wrap_pyfunction!(parse_multipart, m)?)?;
    m.add_class::<Part>()?;
    Ok(())
}
