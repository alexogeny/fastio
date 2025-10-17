use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Static(String),
    Param { name: Arc<str>, kind: ParamKind },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParamKind {
    String,
    Int,
    Float,
    Path,
}

#[derive(Debug, Clone)]
struct CompiledRoute {
    method: Arc<str>,
    host: Option<Arc<str>>,
    segments: Vec<Segment>,
    id: i64,
}

#[derive(Error, Debug)]
enum RouterError {
    #[error("duplicate parameter name '{0}'")]
    DuplicateParam(String),
    #[error("invalid parameter pattern '{0}'")]
    InvalidParam(String),
}

fn parse_segment(segment: &str) -> Result<Segment, RouterError> {
    if !segment.starts_with(':') {
        return Ok(Segment::Static(segment.to_owned()));
    }
    let parts: Vec<&str> = segment[1..].splitn(2, ':').collect();
    let name = parts[0];
    if name.is_empty() {
        return Err(RouterError::InvalidParam(segment.to_owned()));
    }
    let kind = if parts.len() == 2 {
        match parts[1] {
            "int" => ParamKind::Int,
            "float" => ParamKind::Float,
            "path" => ParamKind::Path,
            other => return Err(RouterError::InvalidParam(other.to_owned())),
        }
    } else {
        ParamKind::String
    };
    Ok(Segment::Param {
        name: Arc::from(name.to_owned()),
        kind,
    })
}

fn compile_path(path: &str) -> Result<Vec<Segment>, RouterError> {
    if !path.starts_with('/') {
        return Err(RouterError::InvalidParam(path.to_owned()));
    }
    let mut segments = Vec::new();
    let mut seen = HashMap::new();
    for part in path.split('/') {
        if part.is_empty() {
            segments.push(Segment::Static(String::new()));
            continue;
        }
        let segment = parse_segment(part)?;
        if let Segment::Param { name, .. } = &segment {
            if seen.insert(name.clone(), ()).is_some() {
                return Err(RouterError::DuplicateParam(name.to_string()));
            }
        }
        segments.push(segment);
    }
    Ok(segments)
}

struct RouteInput {
    method: String,
    host: Option<String>,
    path: String,
    id: i64,
}

impl<'source> FromPyObject<'source> for RouteInput {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        let tuple = obj.downcast::<PyTuple>()?;
        match tuple.len() {
            3 => Ok(RouteInput {
                method: tuple.get_item(0)?.extract()?,
                host: None,
                path: tuple.get_item(1)?.extract()?,
                id: tuple.get_item(2)?.extract()?,
            }),
            4 => Ok(RouteInput {
                method: tuple.get_item(0)?.extract()?,
                host: tuple.get_item(1)?.extract()?,
                path: tuple.get_item(2)?.extract()?,
                id: tuple.get_item(3)?.extract()?,
            }),
            _ => Err(PyValueError::new_err(
                "route tuples must have length 3 or 4",
            )),
        }
    }
}

#[pyclass(module = "fastmatch")]
struct Router {
    case_sensitive: bool,
    routes: Vec<CompiledRoute>,
}

#[pymethods]
impl Router {
    #[new]
    #[pyo3(signature = (routes, *, case_sensitive=true))]
    fn new(routes: Vec<RouteInput>, case_sensitive: bool) -> PyResult<Self> {
        let compiled = routes
            .into_iter()
            .map(|route| -> PyResult<CompiledRoute> {
                let method_key = if case_sensitive {
                    Arc::from(route.method)
                } else {
                    Arc::from(route.method.to_uppercase())
                };
                let host = route.host.map(Arc::from);
                let segments = compile_path(&route.path)
                    .map_err(|err| PyValueError::new_err(err.to_string()))?;
                Ok(CompiledRoute {
                    method: method_key,
                    host,
                    segments,
                    id: route.id,
                })
            })
            .collect::<PyResult<Vec<_>>>()?;
        Ok(Self {
            case_sensitive,
            routes: compiled,
        })
    }

    #[pyo3(signature = (method, path, host=None))]
    fn lookup(
        &self,
        _py: Python<'_>,
        method: &str,
        path: &str,
        host: Option<String>,
    ) -> PyResult<(i64, Vec<(String, String)>)> {
        let method_key = if self.case_sensitive {
            method.to_owned()
        } else {
            method.to_uppercase()
        };
        for route in &self.routes {
            if route.method.as_ref() != method_key {
                continue;
            }
            if route.host.as_ref().map(|h| h.as_ref()) != host.as_deref() {
                continue;
            }
            if let Some(params) = match_segments(&route.segments, path)? {
                return Ok((route.id, params));
            }
        }
        Err(PyKeyError::new_err("no route matches"))
    }
}

fn match_segments(segments: &[Segment], path: &str) -> PyResult<Option<Vec<(String, String)>>> {
    let parts: Vec<&str> = path.split('/').collect();
    if segments.len() != parts.len() {
        // Handle trailing slash by allowing empty segment
        if !(segments
            .last()
            .map(|seg| {
                matches!(
                    seg,
                    Segment::Param {
                        kind: ParamKind::Path,
                        ..
                    }
                )
            })
            .unwrap_or(false))
        {
            return Ok(None);
        }
    }
    let mut params = Vec::new();
    let mut path_iter = parts.into_iter();
    for segment in segments {
        let piece = match path_iter.next() {
            Some(v) => v,
            None => "",
        };
        match segment {
            Segment::Static(expected) => {
                if expected != piece {
                    return Ok(None);
                }
            }
            Segment::Param { name, kind } => {
                let value = match kind {
                    ParamKind::String => piece.to_owned(),
                    ParamKind::Int => match piece.parse::<i64>() {
                        Ok(v) => v.to_string(),
                        Err(_) => return Ok(None),
                    },
                    ParamKind::Float => match piece.parse::<f64>() {
                        Ok(v) => v.to_string(),
                        Err(_) => return Ok(None),
                    },
                    ParamKind::Path => {
                        let remainder: Vec<&str> =
                            std::iter::once(piece).chain(path_iter.by_ref()).collect();
                        let joined = remainder.join("/");
                        params.push((name.to_string(), joined));
                        return Ok(Some(params));
                    }
                };
                params.push((name.to_string(), value));
            }
        }
    }
    if path_iter.next().is_some() {
        return Ok(None);
    }
    Ok(Some(params))
}

#[pymodule]
fn _fastmatch(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Router>()?;
    m.add("__all__", vec!["Router"].into_py(py))?;
    Ok(())
}
