use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{
    PyBool, PyByteArray, PyBytes, PyDict, PyFloat, PyInt, PyList, PyString, PyTuple,
};
use pyo3::wrap_pyfunction;
use serde::Serialize;
use serde_json::ser::{CompactFormatter, PrettyFormatter, Serializer};
use serde_json::{Map, Value};

fn read_bytes_like<'py>(_py: Python<'py>, obj: &'py PyAny) -> PyResult<&'py [u8]> {
    if let Ok(bytes) = obj.downcast::<PyBytes>() {
        return Ok(bytes.as_bytes());
    }
    if let Ok(bytearray) = obj.downcast::<PyByteArray>() {
        return Ok(unsafe { bytearray.as_bytes() });
    }
    Err(PyValueError::new_err("expected bytes-like object"))
}

fn serialize_value(value: &Value, indent: Option<usize>) -> PyResult<Vec<u8>> {
    let mut buffer = Vec::with_capacity(128);
    if let Some(width) = indent {
        let indent_str = vec![b' '; width];
        let formatter = PrettyFormatter::with_indent(&indent_str);
        let mut serializer = Serializer::with_formatter(&mut buffer, formatter);
        value
            .serialize(&mut serializer)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
    } else {
        let mut serializer = Serializer::with_formatter(&mut buffer, CompactFormatter);
        value
            .serialize(&mut serializer)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
    }
    Ok(buffer)
}

fn py_to_value(obj: &PyAny) -> PyResult<Value> {
    if obj.is_none() {
        return Ok(Value::Null);
    }
    if obj.is_instance_of::<PyBool>() {
        return Ok(Value::Bool(obj.extract::<bool>()?));
    }
    if obj.is_instance_of::<PyInt>() {
        if let Ok(v) = obj.extract::<i64>() {
            return Ok(Value::Number(v.into()));
        }
        if let Ok(v) = obj.extract::<u64>() {
            return Ok(Value::Number(v.into()));
        }
        let string_repr = obj.str()?.to_str()?.to_owned();
        let parsed: Value = serde_json::from_str(&string_repr)
            .map_err(|_| PyValueError::new_err("integer out of range"))?;
        if let Value::Number(number) = parsed {
            return Ok(Value::Number(number));
        }
        return Err(PyValueError::new_err("integer out of range"));
    }
    if obj.is_instance_of::<PyFloat>() {
        return Ok(Value::Number(
            serde_json::Number::from_f64(obj.extract::<f64>()?)
                .ok_or_else(|| PyValueError::new_err("NaN is not supported"))?,
        ));
    }
    if let Ok(string) = obj.downcast::<PyString>() {
        return Ok(Value::String(string.to_str()?.to_owned()));
    }
    if let Ok(list) = obj.downcast::<PyList>() {
        let mut out = Vec::with_capacity(list.len());
        for item in list.iter() {
            out.push(py_to_value(item)?);
        }
        return Ok(Value::Array(out));
    }
    if let Ok(tuple) = obj.downcast::<PyTuple>() {
        let mut out = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            out.push(py_to_value(item)?);
        }
        return Ok(Value::Array(out));
    }
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = Map::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            map.insert(key_str, py_to_value(value)?);
        }
        return Ok(Value::Object(map));
    }
    Err(PyValueError::new_err(
        "unsupported type for JSON serialization",
    ))
}

fn value_to_py(py: Python<'_>, value: &Value) -> PyResult<PyObject> {
    Ok(match value {
        Value::Null => py.None(),
        Value::Bool(v) => (*v).into_py(py),
        Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                i.into_py(py)
            } else if let Some(u) = num.as_u64() {
                u.into_py(py)
            } else if let Some(f) = num.as_f64() {
                f.into_py(py)
            } else {
                let builtins = py.import("builtins")?;
                let int_ctor = builtins.getattr("int")?;
                let number_string = num.to_string();
                let py_int = int_ctor.call1((PyString::new(py, &number_string),))?;
                py_int.into_py(py)
            }
        }
        Value::String(s) => PyString::new(py, s).into_py(py),
        Value::Array(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(value_to_py(py, item)?)?;
            }
            list.into_py(py)
        }
        Value::Object(map) => {
            let dict = PyDict::new(py);
            for (key, val) in map {
                dict.set_item(key, value_to_py(py, val)?)?;
            }
            dict.into_py(py)
        }
    })
}

#[pyfunction]
#[pyo3(text_signature = "(data, /, *, allow_nan=False)")]
fn loads(py: Python<'_>, data: &PyAny, allow_nan: bool) -> PyResult<PyObject> {
    let bytes = read_bytes_like(py, data)?;
    if allow_nan {
        let json = py.import("json")?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("allow_nan", true)?;
        let text =
            std::str::from_utf8(bytes).map_err(|err| PyValueError::new_err(err.to_string()))?;
        let result = json.call_method("loads", (PyString::new(py, text),), Some(kwargs))?;
        return Ok(result.into());
    }
    let value: Value =
        serde_json::from_slice(bytes).map_err(|err| PyValueError::new_err(err.to_string()))?;
    value_to_py(py, &value)
}

#[pyfunction]
#[pyo3(signature = (obj, *, indent=None, ensure_ascii=false), text_signature = "(obj, /, *, indent=None, ensure_ascii=False)")]
fn dumps(
    py: Python<'_>,
    obj: &PyAny,
    indent: Option<usize>,
    ensure_ascii: bool,
) -> PyResult<Py<PyBytes>> {
    if ensure_ascii {
        let json = py.import("json")?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("ensure_ascii", true)?;
        if let Some(level) = indent {
            kwargs.set_item("indent", level)?;
        }
        let result = json.call_method("dumps", (obj,), Some(kwargs))?;
        let text = result.downcast::<PyString>()?.to_str()?;
        return Ok(PyBytes::new(py, text.as_bytes()).into());
    }
    let value = py_to_value(obj)?;
    let data = serialize_value(&value, indent)?;
    Ok(PyBytes::new(py, &data).into())
}

#[pymodule]
fn _fastjson(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(loads, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    m.add("__all__", vec!["loads", "dumps"].into_py(py))?;
    Ok(())
}
