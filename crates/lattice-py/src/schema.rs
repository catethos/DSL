//! Type schema and function signature conversion for PyO3 bindings

use pyo3::prelude::*;
use pyo3::types::PyDict;

use lattice::runtime::{FieldSchema, FunctionSignature, ParameterSchema, TypeSchema};

/// Convert TypeSchema to Python dict
pub fn type_schema_to_py(py: Python<'_>, schema: &TypeSchema) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);

    match schema {
        TypeSchema::Null => {
            dict.set_item("type", "null")?;
        }
        TypeSchema::Int => {
            dict.set_item("type", "int")?;
        }
        TypeSchema::Float => {
            dict.set_item("type", "float")?;
        }
        TypeSchema::String => {
            dict.set_item("type", "string")?;
        }
        TypeSchema::Bool => {
            dict.set_item("type", "bool")?;
        }
        TypeSchema::Path => {
            dict.set_item("type", "path")?;
        }
        TypeSchema::Any => {
            dict.set_item("type", "any")?;
        }
        TypeSchema::List(inner) => {
            dict.set_item("type", "list")?;
            dict.set_item("inner", type_schema_to_py(py, inner)?)?;
        }
        TypeSchema::Map { key, value } => {
            dict.set_item("type", "map")?;
            dict.set_item("key", type_schema_to_py(py, key)?)?;
            dict.set_item("value", type_schema_to_py(py, value)?)?;
        }
        TypeSchema::Optional(inner) => {
            dict.set_item("type", "optional")?;
            dict.set_item("inner", type_schema_to_py(py, inner)?)?;
        }
        TypeSchema::Struct(s) => {
            dict.set_item("type", "struct")?;
            dict.set_item("name", &s.name)?;

            let fields_list: Vec<Py<PyAny>> = s
                .fields
                .iter()
                .map(|f| field_schema_to_py(py, f))
                .collect::<PyResult<Vec<_>>>()?;
            dict.set_item("fields", fields_list)?;

            if let Some(desc) = &s.description {
                dict.set_item("description", desc)?;
            }
        }
        TypeSchema::Enum(e) => {
            dict.set_item("type", "enum")?;
            dict.set_item("name", &e.name)?;
            dict.set_item("variants", &e.variants)?;
        }
        TypeSchema::Named(name) => {
            dict.set_item("type", "named")?;
            dict.set_item("name", name)?;
        }
    }

    Ok(dict.unbind().into_any())
}

fn field_schema_to_py(py: Python<'_>, field: &FieldSchema) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("name", &field.name)?;
    dict.set_item("type_schema", type_schema_to_py(py, &field.type_schema)?)?;
    dict.set_item("optional", field.optional)?;
    if let Some(desc) = &field.description {
        dict.set_item("description", desc)?;
    }
    Ok(dict.unbind().into_any())
}

/// Convert FunctionSignature to Python dict
pub fn function_signature_to_py(py: Python<'_>, sig: &FunctionSignature) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);

    dict.set_item("name", &sig.name)?;
    dict.set_item("is_llm", sig.is_llm)?;
    dict.set_item("is_async", sig.is_async)?;
    dict.set_item("return_type", type_schema_to_py(py, &sig.return_type)?)?;

    let params: Vec<Py<PyAny>> = sig
        .params
        .iter()
        .map(|p| param_schema_to_py(py, p))
        .collect::<PyResult<Vec<_>>>()?;
    dict.set_item("params", params)?;

    Ok(dict.unbind().into_any())
}

fn param_schema_to_py(py: Python<'_>, param: &ParameterSchema) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("name", &param.name)?;
    dict.set_item("type_schema", type_schema_to_py(py, &param.type_schema)?)?;
    Ok(dict.unbind().into_any())
}
