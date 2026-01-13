# Python Binding Implementation Guide for Lattice

This document provides a comprehensive guide for implementing Python bindings to the `lattice` Rust crate, following the patterns established in the Elixir NIF binding (`lattice-nif`).

## Table of Contents

1. [Overview](#overview)
2. [Project Structure](#project-structure)
3. [Phase 1: Project Setup](#phase-1-project-setup)
4. [Phase 2: Core Runtime Wrapper](#phase-2-core-runtime-wrapper)
5. [Phase 3: Value Conversion](#phase-3-value-conversion)
6. [Phase 4: Runtime Methods](#phase-4-runtime-methods)
7. [Phase 5: Type Schema Export](#phase-5-type-schema-export)
8. [Phase 6: Function Signatures](#phase-6-function-signatures)
9. [Phase 7: Optional Features](#phase-7-optional-features)
10. [Phase 8: Python Enhancements](#phase-8-python-enhancements)
11. [Phase 9: Testing](#phase-9-testing)
12. [Phase 10: Distribution](#phase-10-distribution)
13. [Reference: Elixir NIF Patterns](#reference-elixir-nif-patterns)

---

## Overview

### Goal

Create a Python package `lattice` that exposes the Lattice runtime to Python, enabling:

```python
from lattice import Runtime

rt = Runtime()
result = rt.eval("1 + 2 * 3")
print(result)  # 7

rt.eval("def add(a: Int, b: Int) -> Int { a + b }")
result = rt.call("add", 3, 4)
print(result)  # 7
```

### Technology Stack

| Component | Elixir NIF | Python Binding |
|-----------|------------|----------------|
| FFI Library | Rustler 0.34 | PyO3 0.22 |
| Build Tool | Mix + Rustler | Maturin |
| Package Format | Hex package | PyPI wheel |

### Key Types from `lattice` crate

From `crates/lattice/src/runtime/mod.rs`:

```rust
// Main runtime - wraps VM and handles evaluation
pub struct LatticeRuntime { ... }

// FFI-safe value type
pub enum LatticeValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Path(String),
    List(Vec<LatticeValue>),
    Map(Vec<(String, LatticeValue)>),
}

// Type schema for introspection
pub enum TypeSchema {
    Null, Int, Float, String, Bool, Path, Any,
    List(Box<TypeSchema>),
    Map { key: Box<TypeSchema>, value: Box<TypeSchema> },
    Optional(Box<TypeSchema>),
    Struct(StructSchema),
    Enum(EnumSchema),
    Named(String),
}

// Function signature for introspection
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<ParameterSchema>,
    pub return_type: TypeSchema,
    pub is_llm: bool,
    pub is_async: bool,
}
```

---

## Project Structure

```
crates/lattice-py/
├── Cargo.toml              # Rust crate configuration
├── pyproject.toml          # Python package configuration (maturin)
├── src/
│   ├── lib.rs              # Main module, PyO3 initialization
│   ├── runtime.rs          # Runtime wrapper class
│   ├── convert.rs          # LatticeValue <-> PyObject conversion
│   ├── schema.rs           # TypeSchema -> Python dict conversion
│   └── error.rs            # Error types and conversion
├── lattice/                 # Python stub package (optional)
│   ├── __init__.py
│   └── __init__.pyi        # Type stubs for IDE support
└── tests/
    └── test_runtime.py     # Python tests
```

---

## Phase 1: Project Setup

### 1.1 Create Cargo.toml

**File:** `crates/lattice-py/Cargo.toml`

```toml
[package]
name = "lattice-py"
version.workspace = true
edition.workspace = true
description = "PyO3 bindings for embedding Lattice in Python"

[lib]
name = "lattice_py"
crate-type = ["cdylib"]

[dependencies]
lattice = { path = "../lattice", default-features = false }
pyo3 = { version = "0.22", features = ["extension-module"] }

[features]
default = []
sql = ["lattice/sql"]
```

### 1.2 Create pyproject.toml

**File:** `crates/lattice-py/pyproject.toml`

```toml
[build-system]
requires = ["maturin>=1.4,<2.0"]
build-backend = "maturin"

[project]
name = "lattice"
description = "Python bindings for the Lattice language runtime"
requires-python = ">=3.8"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: Implementation :: PyPy",
]
dynamic = ["version"]

[tool.maturin]
features = ["pyo3/extension-module"]
module-name = "lattice._native"
```

### 1.3 Create initial lib.rs

**File:** `crates/lattice-py/src/lib.rs`

```rust
//! PyO3 bindings for embedding Lattice in Python
//!
//! This crate provides Python bindings that allow Python applications
//! to use the Lattice runtime.
//!
//! # Usage in Python
//!
//! ```python
//! from lattice import Runtime
//!
//! rt = Runtime()
//! result = rt.eval("1 + 2 + 3")
//! print(result)  # 6
//! ```

use pyo3::prelude::*;

mod convert;
mod error;
mod runtime;
mod schema;

use runtime::Runtime;

/// The native Lattice Python module
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Runtime>()?;
    Ok(())
}
```

### 1.4 Add to workspace

**Edit:** `Cargo.toml` (workspace root)

Add to `[workspace]` members:
```toml
members = [
    # ... existing members
    "crates/lattice-py",
]
```

### 1.5 Create Python package wrapper (optional)

**File:** `crates/lattice-py/lattice/__init__.py`

```python
"""Lattice - A statically-typed language for structured LLM interactions"""

from lattice._native import Runtime

__all__ = ["Runtime"]
__version__ = "0.1.0"
```

---

## Phase 2: Core Runtime Wrapper

### 2.1 Create RuntimeResource

The NIF uses `ResourceArc<RuntimeResource>` with a `Mutex` to make the runtime thread-safe. PyO3 handles this differently.

**Reference from NIF (`lattice-nif/src/lib.rs:75-86`):**
```rust
/// Resource wrapper for LatticeRuntime
///
/// We wrap in a Mutex because Rustler resources must be Send + Sync,
/// and LatticeRuntime's eval/call methods require &mut self.
pub struct RuntimeResource(Mutex<LatticeRuntime>);
```

**File:** `crates/lattice-py/src/runtime.rs`

```rust
use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use std::sync::Mutex;

use lattice::runtime::{LatticeRuntime, RuntimeBuilder, LatticeValue};

use crate::convert::{lattice_value_to_py, py_to_lattice_value};
use crate::error::to_py_err;

/// Python wrapper for LatticeRuntime
///
/// Thread-safe via internal Mutex, matching the NIF pattern.
#[pyclass]
pub struct Runtime {
    inner: Mutex<LatticeRuntime>,
}

#[pymethods]
impl Runtime {
    /// Create a new Lattice runtime.
    ///
    /// Args:
    ///     llm: Enable LLM support (default: False)
    ///     sql: Enable SQL/DuckDB support (default: False)
    ///
    /// Returns:
    ///     A new Runtime instance
    ///
    /// Raises:
    ///     RuntimeError: If runtime creation fails
    #[new]
    #[pyo3(signature = (*, llm = false, sql = false))]
    fn new(llm: bool, sql: bool) -> PyResult<Self> {
        let mut builder = RuntimeBuilder::new();

        if llm {
            builder = builder
                .with_default_llm_provider()
                .map_err(to_py_err)?;
        } else {
            builder = builder.without_llm();
        }

        #[cfg(feature = "sql")]
        if sql {
            builder = builder
                .with_default_sql_provider()
                .map_err(to_py_err)?;
        } else {
            builder = builder.without_sql();
        }

        #[cfg(not(feature = "sql"))]
        if sql {
            return Err(PyValueError::new_err(
                "SQL support not compiled in. Rebuild with 'sql' feature."
            ));
        } else {
            builder = builder.without_sql();
        }

        let built = builder.build().map_err(to_py_err)?;
        let runtime = LatticeRuntime::from_built(built);

        Ok(Runtime {
            inner: Mutex::new(runtime),
        })
    }
}
```

### 2.2 Lock helper pattern

The NIF acquires locks inline. For cleaner Python code, consider a helper:

```rust
impl Runtime {
    /// Helper to acquire lock and handle poisoned mutex
    fn with_runtime<F, R>(&self, f: F) -> PyResult<R>
    where
        F: FnOnce(&mut LatticeRuntime) -> PyResult<R>,
    {
        let mut guard = self.inner.lock().map_err(|_| {
            PyRuntimeError::new_err("Runtime lock poisoned")
        })?;
        f(&mut guard)
    }
}
```

---

## Phase 3: Value Conversion

### 3.1 LatticeValue -> Python

**Reference from NIF (`lattice-nif/src/lib.rs:394-422`):**
```rust
fn lattice_value_to_term<'a>(env: Env<'a>, value: &LatticeValue) -> Term<'a> {
    match value {
        LatticeValue::Null => atoms::null().encode(env),
        LatticeValue::Bool(b) => b.encode(env),
        LatticeValue::Int(i) => i.encode(env),
        LatticeValue::Float(f) => f.encode(env),
        LatticeValue::String(s) => s.encode(env),
        LatticeValue::Path(p) => {
            (atoms::lattice_path(), p.as_str()).encode(env)
        }
        LatticeValue::List(items) => { /* ... */ }
        LatticeValue::Map(pairs) => { /* ... */ }
    }
}
```

**File:** `crates/lattice-py/src/convert.rs`

```rust
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};
use pyo3::exceptions::PyTypeError;
use std::path::PathBuf;

use lattice::runtime::LatticeValue;

/// Convert LatticeValue to Python object
pub fn lattice_value_to_py(py: Python<'_>, value: &LatticeValue) -> PyResult<PyObject> {
    match value {
        LatticeValue::Null => Ok(py.None()),

        LatticeValue::Bool(b) => Ok(b.into_py(py)),

        LatticeValue::Int(i) => Ok(i.into_py(py)),

        LatticeValue::Float(f) => Ok(f.into_py(py)),

        LatticeValue::String(s) => Ok(s.into_py(py)),

        LatticeValue::Path(p) => {
            // Convert to pathlib.Path for Pythonic API
            let pathlib = py.import_bound("pathlib")?;
            let path_class = pathlib.getattr("Path")?;
            let path_obj = path_class.call1((p,))?;
            Ok(path_obj.into())
        }

        LatticeValue::List(items) => {
            let list = PyList::empty_bound(py);
            for item in items {
                list.append(lattice_value_to_py(py, item)?)?;
            }
            Ok(list.into())
        }

        LatticeValue::Map(pairs) => {
            let dict = PyDict::new_bound(py);
            for (key, value) in pairs {
                dict.set_item(key, lattice_value_to_py(py, value)?)?;
            }
            Ok(dict.into())
        }
    }
}
```

### 3.2 Python -> LatticeValue

**Reference from NIF (`lattice-nif/src/lib.rs:428-493`):**
```rust
fn term_to_lattice_value(term: Term) -> NifResult<LatticeValue> {
    match term.get_type() {
        TermType::Atom => { /* handle null, true, false */ }
        TermType::Integer => { /* ... */ }
        TermType::Float => { /* ... */ }
        TermType::Binary => { /* string */ }
        TermType::List => { /* ... */ }
        TermType::Map => { /* ... */ }
        TermType::Tuple => { /* path tuple */ }
        _ => Err(...)
    }
}
```

**Add to `crates/lattice-py/src/convert.rs`:**

```rust
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
    let pathlib = py.import_bound("pathlib")?;
    let path_class = pathlib.getattr("Path")?;
    if obj.is_instance(&path_class)? {
        let path_str: String = obj.call_method0("__str__")?.extract()?;
        return Ok(LatticeValue::Path(path_str));
    }

    // Check for list/tuple
    if let Ok(seq) = obj.downcast::<PyList>() {
        let items: PyResult<Vec<LatticeValue>> = seq
            .iter()
            .map(|item| py_to_lattice_value(&item))
            .collect();
        return Ok(LatticeValue::List(items?));
    }

    // Also accept tuples as lists
    if let Ok(tuple) = obj.extract::<Vec<Bound<'_, PyAny>>>() {
        let items: PyResult<Vec<LatticeValue>> = tuple
            .iter()
            .map(|item| py_to_lattice_value(item))
            .collect();
        return Ok(LatticeValue::List(items?));
    }

    // Check for dict
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut pairs = Vec::new();
        for (key, value) in dict.iter() {
            let key_str: String = key.extract().map_err(|_| {
                PyTypeError::new_err("Dict keys must be strings")
            })?;
            pairs.push((key_str, py_to_lattice_value(&value)?));
        }
        return Ok(LatticeValue::Map(pairs));
    }

    Err(PyTypeError::new_err(format!(
        "Cannot convert {} to LatticeValue",
        obj.get_type().name()?
    )))
}
```

### 3.3 Value conversion table

| LatticeValue | Python Type | Notes |
|--------------|-------------|-------|
| `Null` | `None` | |
| `Bool(b)` | `bool` | Check before int! |
| `Int(i)` | `int` | i64 range |
| `Float(f)` | `float` | f64 |
| `String(s)` | `str` | |
| `Path(p)` | `pathlib.Path` | More Pythonic than tuple |
| `List(items)` | `list` | Also accept `tuple` |
| `Map(pairs)` | `dict` | Keys must be str |

---

## Phase 4: Runtime Methods

### 4.1 eval()

**Reference from NIF (`lattice-nif/src/lib.rs:169-184`):**
```rust
#[rustler::nif(schedule = "DirtyCpu")]
fn eval<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
    source: String,
) -> NifResult<Term<'a>> {
    let mut rt = runtime.0.lock().map_err(...)?;
    match rt.eval(&source) {
        Ok(value) => Ok((atoms::ok(), lattice_value_to_term(env, &value)).encode(env)),
        Err(e) => Ok((atoms::error(), format!("{}", e)).encode(env)),
    }
}
```

**Add to `crates/lattice-py/src/runtime.rs`:**

```rust
#[pymethods]
impl Runtime {
    /// Evaluate Lattice source code.
    ///
    /// Args:
    ///     source: Lattice source code to evaluate
    ///     bindings: Optional dict of variable bindings
    ///
    /// Returns:
    ///     The result of evaluation
    ///
    /// Raises:
    ///     RuntimeError: If evaluation fails
    ///
    /// Example:
    ///     >>> rt = Runtime()
    ///     >>> rt.eval("1 + 2 * 3")
    ///     7
    ///     >>> rt.eval("x + y", bindings={"x": 10, "y": 20})
    ///     30
    #[pyo3(signature = (source, *, bindings = None))]
    fn eval(
        &self,
        py: Python<'_>,
        source: &str,
        bindings: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyObject> {
        self.with_runtime(|rt| {
            let result = if let Some(bindings_dict) = bindings {
                // Convert Python dict to Vec<(String, LatticeValue)>
                let lattice_bindings: Vec<(String, LatticeValue)> = bindings_dict
                    .iter()
                    .map(|(k, v)| {
                        let key: String = k.extract()?;
                        let value = py_to_lattice_value(&v)?;
                        Ok((key, value))
                    })
                    .collect::<PyResult<Vec<_>>>()?;

                rt.eval_with_bindings(source, lattice_bindings)
            } else {
                rt.eval(source)
            };

            match result {
                Ok(value) => lattice_value_to_py(py, &value),
                Err(e) => Err(to_py_err(e)),
            }
        })
    }
}
```

### 4.2 eval_file()

**Reference from NIF (`lattice-nif/src/lib.rs:191-205`):**
```rust
#[rustler::nif(schedule = "DirtyCpu")]
fn eval_file<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
    path: String,
) -> NifResult<Term<'a>> {
    let mut rt = runtime.0.lock().map_err(...)?;
    match rt.eval_file(std::path::Path::new(&path)) {
        Ok(value) => Ok((atoms::ok(), lattice_value_to_term(env, &value)).encode(env)),
        Err(e) => Ok((atoms::error(), format!("{}", e)).encode(env)),
    }
}
```

**Add to `crates/lattice-py/src/runtime.rs`:**

```rust
#[pymethods]
impl Runtime {
    /// Evaluate a Lattice file.
    ///
    /// Supports both .lat files and .md files (markdown LLM functions).
    /// Resolves imports relative to the file's directory.
    ///
    /// Args:
    ///     path: Path to the file to evaluate
    ///
    /// Returns:
    ///     The result of evaluation
    ///
    /// Raises:
    ///     RuntimeError: If file cannot be read or evaluation fails
    fn eval_file(&self, py: Python<'_>, path: &str) -> PyResult<PyObject> {
        self.with_runtime(|rt| {
            match rt.eval_file(std::path::Path::new(path)) {
                Ok(value) => lattice_value_to_py(py, &value),
                Err(e) => Err(to_py_err(e)),
            }
        })
    }
}
```

### 4.3 call()

**Reference from NIF (`lattice-nif/src/lib.rs:263-285`):**
```rust
#[rustler::nif(schedule = "DirtyCpu")]
fn call_function<'a>(
    env: Env<'a>,
    runtime: ResourceArc<RuntimeResource>,
    name: String,
    args: Vec<Term<'a>>,
) -> NifResult<Term<'a>> {
    let lattice_args: Vec<LatticeValue> = args
        .into_iter()
        .map(term_to_lattice_value)
        .collect::<NifResult<Vec<_>>>()?;

    let mut rt = runtime.0.lock().map_err(...)?;
    match rt.call(&name, lattice_args) {
        Ok(value) => Ok((atoms::ok(), lattice_value_to_term(env, &value)).encode(env)),
        Err(e) => Ok((atoms::error(), format!("{}", e)).encode(env)),
    }
}
```

**Add to `crates/lattice-py/src/runtime.rs`:**

```rust
#[pymethods]
impl Runtime {
    /// Call a Lattice function by name.
    ///
    /// Args:
    ///     name: Name of the function to call
    ///     *args: Positional arguments to pass
    ///
    /// Returns:
    ///     The function's return value
    ///
    /// Raises:
    ///     RuntimeError: If function doesn't exist or call fails
    ///
    /// Example:
    ///     >>> rt = Runtime()
    ///     >>> rt.eval("def add(a: Int, b: Int) -> Int { a + b }")
    ///     >>> rt.call("add", 3, 4)
    ///     7
    #[pyo3(signature = (name, *args))]
    fn call(
        &self,
        py: Python<'_>,
        name: &str,
        args: Vec<Bound<'_, PyAny>>,
    ) -> PyResult<PyObject> {
        // Convert Python args to LatticeValues BEFORE acquiring lock
        let lattice_args: Vec<LatticeValue> = args
            .iter()
            .map(|arg| py_to_lattice_value(arg))
            .collect::<PyResult<Vec<_>>>()?;

        self.with_runtime(|rt| {
            match rt.call(name, lattice_args) {
                Ok(value) => lattice_value_to_py(py, &value),
                Err(e) => Err(to_py_err(e)),
            }
        })
    }
}
```

### 4.4 Global variable methods

**Reference from NIF (`lattice-nif/src/lib.rs:343-376`):**
```rust
#[rustler::nif]
fn get_global<'a>(...) -> NifResult<Term<'a>> { ... }

#[rustler::nif]
fn set_global<'a>(...) -> NifResult<Term<'a>> { ... }
```

**Add to `crates/lattice-py/src/runtime.rs`:**

```rust
#[pymethods]
impl Runtime {
    /// Get a global variable's value.
    ///
    /// Args:
    ///     name: Variable name
    ///
    /// Returns:
    ///     The variable's value, or None if not found
    fn get_global(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        self.with_runtime(|rt| {
            match rt.get_global(name) {
                Some(value) => lattice_value_to_py(py, &value),
                None => Ok(py.None()),
            }
        })
    }

    /// Set a global variable.
    ///
    /// Args:
    ///     name: Variable name
    ///     value: Value to set
    fn set_global(&self, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        let lattice_value = py_to_lattice_value(value)?;
        self.with_runtime(|rt| {
            rt.set_global(name, lattice_value);
            Ok(())
        })
    }

    /// Check if a function exists.
    ///
    /// Args:
    ///     name: Function name
    ///
    /// Returns:
    ///     True if function exists, False otherwise
    fn has_function(&self, name: &str) -> PyResult<bool> {
        self.with_runtime(|rt| Ok(rt.has_function(name)))
    }

    /// Reset the runtime, clearing all state.
    fn reset(&self) -> PyResult<()> {
        self.with_runtime(|rt| {
            rt.reset();
            Ok(())
        })
    }
}
```

---

## Phase 5: Type Schema Export

### 5.1 TypeSchema -> Python dict

**Reference from NIF (`lattice-nif/src/lib.rs:496-604`):**

The NIF converts TypeSchema to Elixir maps with `:type_schema` atom keys. For Python, we'll use dicts.

**File:** `crates/lattice-py/src/schema.rs`

```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;

use lattice::runtime::{TypeSchema, StructSchema, EnumSchema, FieldSchema};

/// Convert TypeSchema to Python dict
pub fn type_schema_to_py(py: Python<'_>, schema: &TypeSchema) -> PyResult<PyObject> {
    let dict = PyDict::new_bound(py);

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

            let fields_list: Vec<PyObject> = s.fields
                .iter()
                .map(|f| field_schema_to_py(py, f))
                .collect::<PyResult<Vec<_>>>()?;
            dict.set_item("fields", fields_list)?;
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

    Ok(dict.into())
}

fn field_schema_to_py(py: Python<'_>, field: &FieldSchema) -> PyResult<PyObject> {
    let dict = PyDict::new_bound(py);
    dict.set_item("name", &field.name)?;
    dict.set_item("type_schema", type_schema_to_py(py, &field.type_schema)?)?;
    dict.set_item("optional", field.optional)?;
    if let Some(desc) = &field.description {
        dict.set_item("description", desc)?;
    }
    Ok(dict.into())
}
```

### 5.2 get_types() method

**Add to `crates/lattice-py/src/runtime.rs`:**

```rust
use crate::schema::type_schema_to_py;

#[pymethods]
impl Runtime {
    /// Get all registered type schemas.
    ///
    /// Returns:
    ///     List of type schema dicts
    ///
    /// Example:
    ///     >>> rt = Runtime()
    ///     >>> rt.eval("type Person { name: String, age: Int }")
    ///     >>> rt.get_types()
    ///     [{'type': 'struct', 'name': 'Person', 'fields': [...]}]
    fn get_types(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        self.with_runtime(|rt| {
            rt.get_types()
                .iter()
                .map(|schema| type_schema_to_py(py, schema))
                .collect()
        })
    }
}
```

---

## Phase 6: Function Signatures

### 6.1 FunctionSignature -> Python dict

**Reference from NIF (`lattice-nif/src/lib.rs:607-638`):**

**Add to `crates/lattice-py/src/schema.rs`:**

```rust
use lattice::runtime::{FunctionSignature, ParameterSchema};

/// Convert FunctionSignature to Python dict
pub fn function_signature_to_py(py: Python<'_>, sig: &FunctionSignature) -> PyResult<PyObject> {
    let dict = PyDict::new_bound(py);

    dict.set_item("name", &sig.name)?;
    dict.set_item("is_llm", sig.is_llm)?;
    dict.set_item("is_async", sig.is_async)?;
    dict.set_item("return_type", type_schema_to_py(py, &sig.return_type)?)?;

    let params: Vec<PyObject> = sig.params
        .iter()
        .map(|p| param_schema_to_py(py, p))
        .collect::<PyResult<Vec<_>>>()?;
    dict.set_item("params", params)?;

    Ok(dict.into())
}

fn param_schema_to_py(py: Python<'_>, param: &ParameterSchema) -> PyResult<PyObject> {
    let dict = PyDict::new_bound(py);
    dict.set_item("name", &param.name)?;
    dict.set_item("type_schema", type_schema_to_py(py, &param.type_schema)?)?;
    Ok(dict.into())
}
```

### 6.2 get_function_signatures() method

**Add to `crates/lattice-py/src/runtime.rs`:**

```rust
use crate::schema::function_signature_to_py;

#[pymethods]
impl Runtime {
    /// Get all function signatures.
    ///
    /// Returns:
    ///     List of function signature dicts
    ///
    /// Example:
    ///     >>> rt = Runtime()
    ///     >>> rt.eval("def add(a: Int, b: Int) -> Int { a + b }")
    ///     >>> rt.get_function_signatures()
    ///     [{'name': 'add', 'params': [...], 'return_type': {'type': 'int'}, ...}]
    fn get_function_signatures(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        self.with_runtime(|rt| {
            rt.get_function_signatures()
                .iter()
                .map(|sig| function_signature_to_py(py, sig))
                .collect()
        })
    }
}
```

---

## Phase 7: Optional Features

### 7.1 SQL Feature

The SQL feature is conditionally compiled. Handle gracefully when not available:

```rust
#[cfg(feature = "sql")]
fn build_with_sql(builder: RuntimeBuilder) -> Result<RuntimeBuilder, ...> {
    builder.with_default_sql_provider()
}

#[cfg(not(feature = "sql"))]
fn build_with_sql(_builder: RuntimeBuilder) -> Result<RuntimeBuilder, ...> {
    Err(PyValueError::new_err("SQL support not compiled"))
}
```

### 7.2 LLM Debug Info

**Reference from NIF:** Not implemented in NIF, but available in LatticeRuntime.

**Add to `crates/lattice-py/src/runtime.rs`:**

```rust
#[pymethods]
impl Runtime {
    /// Get debug info from the last LLM call.
    ///
    /// Returns:
    ///     Dict with 'prompt', 'raw_response', 'function_name', 'return_type'
    ///     or None if no LLM call was made
    fn take_llm_debug(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.with_runtime(|rt| {
            match rt.take_llm_debug() {
                Some(debug) => {
                    let dict = PyDict::new_bound(py);
                    dict.set_item("prompt", &debug.prompt)?;
                    dict.set_item("raw_response", &debug.raw_response)?;
                    dict.set_item("function_name", &debug.function_name)?;
                    dict.set_item("return_type", &debug.return_type)?;
                    Ok(dict.into())
                }
                None => Ok(py.None()),
            }
        })
    }
}
```

---

## Phase 8: Python Enhancements

### 8.1 Magic methods for Pythonic API

```rust
#[pymethods]
impl Runtime {
    /// Support `rt["var"]` syntax for getting globals
    fn __getitem__(&self, py: Python<'_>, key: &str) -> PyResult<PyObject> {
        self.get_global(py, key)
    }

    /// Support `rt["var"] = value` syntax for setting globals
    fn __setitem__(&self, key: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.set_global(key, value)
    }

    /// Support `"var" in rt` syntax
    fn __contains__(&self, key: &str) -> PyResult<bool> {
        self.with_runtime(|rt| Ok(rt.get_global(key).is_some()))
    }

    /// Friendly repr for REPL
    fn __repr__(&self) -> PyResult<String> {
        self.with_runtime(|rt| {
            let funcs = rt.function_names().len();
            let llm_funcs = rt.llm_function_names().len();
            let types = rt.get_types().len();
            Ok(format!(
                "<Runtime: {} functions, {} LLM functions, {} types>",
                funcs, llm_funcs, types
            ))
        })
    }
}
```

### 8.2 Context manager support

```rust
#[pymethods]
impl Runtime {
    /// Enter context manager (returns self)
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Exit context manager (resets runtime)
    fn __exit__(
        &self,
        _exc_type: &Bound<'_, PyAny>,
        _exc_val: &Bound<'_, PyAny>,
        _exc_tb: &Bound<'_, PyAny>,
    ) -> PyResult<bool> {
        self.reset()?;
        Ok(false)  // Don't suppress exceptions
    }
}
```

Usage:
```python
with Runtime() as rt:
    rt.eval("let x = 42")
    print(rt["x"])
# Runtime is reset after the block
```

### 8.3 Type stubs for IDE support

**File:** `crates/lattice-py/lattice/__init__.pyi`

```python
from typing import Any, Dict, List, Optional, Union
from pathlib import Path

LatticeValue = Union[None, bool, int, float, str, Path, List["LatticeValue"], Dict[str, "LatticeValue"]]

class Runtime:
    def __init__(self, *, llm: bool = False, sql: bool = False) -> None: ...
    def eval(self, source: str, *, bindings: Optional[Dict[str, LatticeValue]] = None) -> LatticeValue: ...
    def eval_file(self, path: str) -> LatticeValue: ...
    def call(self, name: str, *args: LatticeValue) -> LatticeValue: ...
    def get_global(self, name: str) -> Optional[LatticeValue]: ...
    def set_global(self, name: str, value: LatticeValue) -> None: ...
    def has_function(self, name: str) -> bool: ...
    def reset(self) -> None: ...
    def get_types(self) -> List[Dict[str, Any]]: ...
    def get_function_signatures(self) -> List[Dict[str, Any]]: ...
    def take_llm_debug(self) -> Optional[Dict[str, str]]: ...
    def __getitem__(self, key: str) -> Optional[LatticeValue]: ...
    def __setitem__(self, key: str, value: LatticeValue) -> None: ...
    def __contains__(self, key: str) -> bool: ...
    def __repr__(self) -> str: ...
    def __enter__(self) -> "Runtime": ...
    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> bool: ...
```

---

## Phase 9: Testing

### 9.1 Python tests

**File:** `crates/lattice-py/tests/test_runtime.py`

```python
import pytest
from pathlib import Path

from lattice import Runtime


class TestBasicEval:
    def test_arithmetic(self):
        rt = Runtime()
        assert rt.eval("1 + 2 * 3") == 7

    def test_string(self):
        rt = Runtime()
        assert rt.eval('"hello" + " world"') == "hello world"

    def test_list(self):
        rt = Runtime()
        assert rt.eval("[1, 2, 3]") == [1, 2, 3]

    def test_map(self):
        rt = Runtime()
        result = rt.eval('{"a": 1, "b": 2}')
        assert result == {"a": 1, "b": 2}

    def test_null(self):
        rt = Runtime()
        assert rt.eval("null") is None

    def test_bool(self):
        rt = Runtime()
        assert rt.eval("true") is True
        assert rt.eval("false") is False


class TestBindings:
    def test_eval_with_bindings(self):
        rt = Runtime()
        result = rt.eval("x + y", bindings={"x": 10, "y": 20})
        assert result == 30

    def test_bindings_with_list(self):
        rt = Runtime()
        result = rt.eval("items[0] + items[1]", bindings={"items": [3, 4]})
        assert result == 7


class TestGlobals:
    def test_set_get_global(self):
        rt = Runtime()
        rt.set_global("count", 42)
        assert rt.get_global("count") == 42

    def test_global_in_eval(self):
        rt = Runtime()
        rt.set_global("x", 100)
        assert rt.eval("x * 2") == 200

    def test_dict_syntax(self):
        rt = Runtime()
        rt["foo"] = 123
        assert rt["foo"] == 123

    def test_contains(self):
        rt = Runtime()
        rt["bar"] = 1
        assert "bar" in rt
        assert "baz" not in rt


class TestFunctions:
    def test_define_and_call(self):
        rt = Runtime()
        rt.eval("def add(a: Int, b: Int) -> Int { a + b }")
        assert rt.call("add", 3, 4) == 7

    def test_has_function(self):
        rt = Runtime()
        rt.eval("def foo() -> Int { 42 }")
        assert rt.has_function("foo")
        assert not rt.has_function("bar")

    def test_function_signatures(self):
        rt = Runtime()
        rt.eval("def greet(name: String) -> String { name }")
        sigs = rt.get_function_signatures()
        assert len(sigs) == 1
        assert sigs[0]["name"] == "greet"


class TestTypes:
    def test_struct(self):
        rt = Runtime()
        rt.eval("type Person { name: String, age: Int }")
        types = rt.get_types()
        assert len(types) == 1
        assert types[0]["type"] == "struct"
        assert types[0]["name"] == "Person"

    def test_enum(self):
        rt = Runtime()
        rt.eval("enum Color { Red, Green, Blue }")
        types = rt.get_types()
        assert len(types) == 1
        assert types[0]["type"] == "enum"
        assert types[0]["variants"] == ["Red", "Green", "Blue"]


class TestContextManager:
    def test_context_manager_reset(self):
        rt = Runtime()
        rt["x"] = 42
        with rt:
            assert rt["x"] == 42
        # After exiting, runtime is reset
        assert rt["x"] is None


class TestErrors:
    def test_syntax_error(self):
        rt = Runtime()
        with pytest.raises(RuntimeError):
            rt.eval("1 +")

    def test_undefined_variable(self):
        rt = Runtime()
        with pytest.raises(RuntimeError):
            rt.eval("undefined_var")

    def test_undefined_function(self):
        rt = Runtime()
        with pytest.raises(RuntimeError):
            rt.call("nonexistent")
```

### 9.2 Running tests

```bash
cd crates/lattice-py

# Build and install in development mode
maturin develop

# Run Python tests
pytest tests/

# Run with verbose output
pytest tests/ -v
```

---

## Phase 10: Distribution

### 10.1 Build wheels

```bash
# Build wheel for current platform
maturin build --release

# Build for multiple Python versions
maturin build --release --interpreter python3.8 python3.9 python3.10 python3.11 python3.12

# Output in target/wheels/
```

### 10.2 Cross-compilation (optional)

For distributing to multiple platforms:

```bash
# Install cross-compilation targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-apple-darwin

# Build for Linux
maturin build --release --target x86_64-unknown-linux-gnu

# Build for Apple Silicon
maturin build --release --target aarch64-apple-darwin
```

### 10.3 Publish to PyPI

```bash
# Publish to PyPI
maturin publish

# Publish to test PyPI first
maturin publish --repository testpypi
```

---

## Reference: Elixir NIF Patterns

### NIF Function Mapping

| NIF Function | Line | Python Equivalent |
|--------------|------|-------------------|
| `new_runtime` | 96-107 | `Runtime()` |
| `new_runtime_with_llm` | 112-125 | `Runtime(llm=True)` |
| `new_runtime_with_sql` | 131-144 | `Runtime(sql=True)` |
| `new_runtime_with_all` | 150-164 | `Runtime(llm=True, sql=True)` |
| `eval` | 169-184 | `runtime.eval(source)` |
| `eval_file` | 191-205 | `runtime.eval_file(path)` |
| `eval_with_base_path` | 211-227 | `runtime.eval(source, base_path=path)` |
| `eval_with_bindings` | 233-258 | `runtime.eval(source, bindings=dict)` |
| `call_function` | 263-285 | `runtime.call(name, *args)` |
| `get_types` | 290-304 | `runtime.get_types()` |
| `get_function_signatures` | 309-326 | `runtime.get_function_signatures()` |
| `has_function` | 330-337 | `runtime.has_function(name)` |
| `get_global` | 343-357 | `runtime.get_global(name)` or `runtime[name]` |
| `set_global` | 361-376 | `runtime.set_global(name, val)` or `runtime[name] = val` |
| `reset` | 380-388 | `runtime.reset()` |

### Key Differences from NIF

1. **No atoms**: Python doesn't have atoms; use strings/None directly
2. **Path handling**: Use `pathlib.Path` instead of tuple `{:path, "string"}`
3. **Error handling**: Raise Python exceptions instead of returning `{:error, reason}`
4. **Result tuples**: Return values directly instead of `{:ok, value}`
5. **Magic methods**: Add `__getitem__`, `__setitem__`, `__contains__` for Pythonic API
6. **Context manager**: Add `__enter__`/`__exit__` for resource management

---

## Implementation Checklist

- [x] **Phase 1: Project Setup**
  - [x] Create `crates/lattice-py/` directory
  - [x] Write `Cargo.toml`
  - [x] Write `pyproject.toml`
  - [x] Create initial `src/lib.rs`
  - [x] Add to workspace `Cargo.toml`

- [x] **Phase 2: Core Runtime Wrapper**
  - [x] Create `src/runtime.rs` with `Runtime` struct
  - [x] Implement `Runtime::new()` with llm/sql options
  - [x] Add `with_runtime()` helper method

- [x] **Phase 3: Value Conversion**
  - [x] Create `src/convert.rs`
  - [x] Implement `lattice_value_to_py()`
  - [x] Implement `py_to_lattice_value()`
  - [x] Handle Path -> pathlib.Path conversion

- [x] **Phase 4: Runtime Methods**
  - [x] Implement `eval()` with optional bindings
  - [x] Implement `eval_file()`
  - [x] Implement `call()`
  - [x] Implement `get_global()` / `set_global()`
  - [x] Implement `has_function()`
  - [x] Implement `reset()`

- [x] **Phase 5: Type Schema Export**
  - [x] Create `src/schema.rs`
  - [x] Implement `type_schema_to_py()`
  - [x] Implement `get_types()` method

- [x] **Phase 6: Function Signatures**
  - [x] Implement `function_signature_to_py()`
  - [x] Implement `get_function_signatures()` method

- [x] **Phase 7: Optional Features**
  - [x] Handle SQL feature flag
  - [x] Implement `take_llm_debug()`

- [x] **Phase 8: Python Enhancements**
  - [x] Add `__getitem__` / `__setitem__`
  - [x] Add `__contains__`
  - [x] Add `__repr__`
  - [x] Add context manager (`__enter__` / `__exit__`)
  - [x] Create type stubs (`.pyi` file)

- [x] **Phase 9: Testing**
  - [x] Write Python test suite (36 tests total)
  - [x] Test with `uv sync` + `pytest`
  - [x] SQL feature tests (6 tests):
    - Runtime creation with SQL
    - Basic SQL queries
    - Multiple rows
    - Aggregation functions
    - Boolean values
    - F-string interpolation
  - [x] LLM feature tests (3 tests):
    - Runtime creation with LLM
    - LLM function definition
    - Debug info retrieval
  - [x] LLM integration tests (5 tests, require OPENROUTER_API_KEY):
    - Simple string response
    - Enum response
    - Struct response
    - Debug info capture
    - List response

- [x] **Phase 10: Distribution**
  - [x] Build release wheels (Python 3.12, 3.13, 3.14 for macOS arm64)
  - [x] Test installation from wheel
  - [ ] (Optional) Publish to PyPI

---

## Quick Start Commands

```bash
# Setup (using uv for virtual environment management)
cd crates/lattice-py
uv sync --dev

# Test
uv run python -c "from lattice import Runtime; rt = Runtime(); print(rt.eval('1+2'))"

# Run tests
uv run pytest tests/ -v

# Build release
uv run maturin build --release
```

### Implementation Notes

- **PyO3 Version**: Updated to PyO3 0.27 for Python 3.14 support (both `lattice` and `lattice-py` crates)
- **Package Manager**: Uses `uv` for virtual environment and dependency management
- **Build Backend**: Maturin 1.x with `tool.uv.cache-keys` configured for automatic rebuilds
