//! Value conversion between LatticeValue and Python objects

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use lattice::runtime::LatticeValue;

/// Convert LatticeValue to Python object
pub fn lattice_value_to_py(py: Python<'_>, value: &LatticeValue) -> PyResult<Py<PyAny>> {
    match value {
        LatticeValue::Null => Ok(py.None()),

        LatticeValue::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().into_any().unbind()),

        LatticeValue::Int(i) => Ok(i.into_pyobject(py)?.to_owned().into_any().unbind()),

        LatticeValue::Float(f) => Ok(f.into_pyobject(py)?.to_owned().into_any().unbind()),

        LatticeValue::String(s) => Ok(s.into_pyobject(py)?.to_owned().into_any().unbind()),

        LatticeValue::Path(p) => {
            // Convert to pathlib.Path for Pythonic API
            let pathlib = py.import("pathlib")?;
            let path_class = pathlib.getattr("Path")?;
            let path_obj = path_class.call1((p,))?;
            Ok(path_obj.unbind())
        }

        LatticeValue::List(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(lattice_value_to_py(py, item)?)?;
            }
            Ok(list.unbind().into_any())
        }

        LatticeValue::Map(pairs) => {
            let dict = PyDict::new(py);
            for (key, value) in pairs {
                dict.set_item(key, lattice_value_to_py(py, value)?)?;
            }
            Ok(dict.unbind().into_any())
        }
    }
}

/// Convert Python object to LatticeValue
pub fn py_to_lattice_value(obj: &Bound<'_, PyAny>) -> PyResult<LatticeValue> {
    // Check for None
    if obj.is_none() {
        return Ok(LatticeValue::Null);
    }

    // Check for bool (must come before int, since bool is subclass of int in Python)
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(LatticeValue::Bool(b));
    }

    // Check for int
    if let Ok(i) = obj.extract::<i64>() {
        return Ok(LatticeValue::Int(i));
    }

    // Check for float
    if let Ok(f) = obj.extract::<f64>() {
        return Ok(LatticeValue::Float(f));
    }

    // Check for str
    if let Ok(s) = obj.extract::<String>() {
        return Ok(LatticeValue::String(s));
    }

    // Check for pathlib.Path
    let py = obj.py();
    let pathlib = py.import("pathlib")?;
    let path_class = pathlib.getattr("Path")?;
    if obj.is_instance(&path_class)? {
        let path_str: String = obj.call_method0("__str__")?.extract()?;
        return Ok(LatticeValue::Path(path_str));
    }

    // Check for list
    if let Ok(seq) = obj.clone().cast_exact::<PyList>() {
        let items: PyResult<Vec<LatticeValue>> = seq
            .iter()
            .map(|item| py_to_lattice_value(&item))
            .collect();
        return Ok(LatticeValue::List(items?));
    }

    // Also accept tuples as lists
    if obj.get_type().name()? == "tuple" {
        let len = obj.len()?;
        let mut items = Vec::with_capacity(len);
        for i in 0..len {
            let item = obj.get_item(i)?;
            items.push(py_to_lattice_value(&item)?);
        }
        return Ok(LatticeValue::List(items));
    }

    // Check for dict
    if let Ok(dict) = obj.clone().cast_exact::<PyDict>() {
        let mut pairs = Vec::new();
        for (key, value) in dict.iter() {
            let key_str: String = key
                .extract()
                .map_err(|_| PyTypeError::new_err("Dict keys must be strings"))?;
            pairs.push((key_str, py_to_lattice_value(&value)?));
        }
        return Ok(LatticeValue::Map(pairs));
    }

    Err(PyTypeError::new_err(format!(
        "Cannot convert {} to LatticeValue",
        obj.get_type().name()?
    )))
}
