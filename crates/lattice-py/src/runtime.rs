//! Runtime wrapper for PyO3 bindings

#[cfg(not(feature = "sql"))]
use pyo3::exceptions::PyValueError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Mutex;

use lattice::runtime::{LatticeRuntime, LatticeValue, RuntimeBuilder};

use crate::convert::{lattice_value_to_py, py_to_lattice_value};
use crate::error::to_py_err;
use crate::schema::{function_signature_to_py, type_schema_to_py};

/// Python wrapper for LatticeRuntime
///
/// Thread-safe via internal Mutex, matching the NIF pattern.
#[pyclass]
pub struct Runtime {
    inner: Mutex<LatticeRuntime>,
}

impl Runtime {
    /// Helper to acquire lock and handle poisoned mutex
    fn with_runtime<F, R>(&self, f: F) -> PyResult<R>
    where
        F: FnOnce(&mut LatticeRuntime) -> PyResult<R>,
    {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("Runtime lock poisoned"))?;
        f(&mut guard)
    }
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
            builder = builder.with_default_llm_provider().map_err(to_py_err)?;
        } else {
            builder = builder.without_llm();
        }

        #[cfg(feature = "sql")]
        if sql {
            builder = builder.with_default_sql_provider().map_err(to_py_err)?;
        } else {
            builder = builder.without_sql();
        }

        #[cfg(not(feature = "sql"))]
        if sql {
            return Err(PyValueError::new_err(
                "SQL support not compiled in. Rebuild with 'sql' feature.",
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
    ) -> PyResult<Py<PyAny>> {
        // Convert Python dict to Vec<(String, LatticeValue)> BEFORE acquiring lock
        let lattice_bindings: Option<Vec<(String, LatticeValue)>> = if let Some(bindings_dict) =
            bindings
        {
            let bindings: Vec<(String, LatticeValue)> = bindings_dict
                .iter()
                .map(|(k, v)| {
                    let key: String = k.extract()?;
                    let value = py_to_lattice_value(&v)?;
                    Ok((key, value))
                })
                .collect::<PyResult<Vec<_>>>()?;
            Some(bindings)
        } else {
            None
        };

        self.with_runtime(|rt| {
            let result = if let Some(bindings) = lattice_bindings {
                rt.eval_with_bindings(source, bindings)
            } else {
                rt.eval(source)
            };

            match result {
                Ok(value) => lattice_value_to_py(py, &value),
                Err(e) => Err(to_py_err(e)),
            }
        })
    }

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
    fn eval_file(&self, py: Python<'_>, path: &str) -> PyResult<Py<PyAny>> {
        self.with_runtime(|rt| match rt.eval_file(std::path::Path::new(path)) {
            Ok(value) => lattice_value_to_py(py, &value),
            Err(e) => Err(to_py_err(e)),
        })
    }

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
    ) -> PyResult<Py<PyAny>> {
        // Convert Python args to LatticeValues BEFORE acquiring lock
        let lattice_args: Vec<LatticeValue> = args
            .iter()
            .map(|arg| py_to_lattice_value(arg))
            .collect::<PyResult<Vec<_>>>()?;

        self.with_runtime(|rt| match rt.call(name, lattice_args) {
            Ok(value) => lattice_value_to_py(py, &value),
            Err(e) => Err(to_py_err(e)),
        })
    }

    /// Get a global variable's value.
    ///
    /// Args:
    ///     name: Variable name
    ///
    /// Returns:
    ///     The variable's value, or None if not found
    fn get_global(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
        self.with_runtime(|rt| match rt.get_global(name) {
            Some(value) => lattice_value_to_py(py, &value),
            None => Ok(py.None()),
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
    fn get_types(&self, py: Python<'_>) -> PyResult<Vec<Py<PyAny>>> {
        self.with_runtime(|rt| {
            rt.get_types()
                .iter()
                .map(|schema| type_schema_to_py(py, schema))
                .collect()
        })
    }

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
    fn get_function_signatures(&self, py: Python<'_>) -> PyResult<Vec<Py<PyAny>>> {
        self.with_runtime(|rt| {
            rt.get_function_signatures()
                .iter()
                .map(|sig| function_signature_to_py(py, sig))
                .collect()
        })
    }

    /// Get debug info from the last LLM call.
    ///
    /// Returns:
    ///     Dict with 'prompt', 'raw_response', 'function_name', 'return_type'
    ///     or None if no LLM call was made
    fn take_llm_debug(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.with_runtime(|rt| match rt.take_llm_debug() {
            Some(debug) => {
                let dict = PyDict::new(py);
                dict.set_item("prompt", &debug.prompt)?;
                dict.set_item("raw_response", &debug.raw_response)?;
                dict.set_item("function_name", &debug.function_name)?;
                dict.set_item("return_type", &debug.return_type)?;
                Ok(dict.unbind().into_any())
            }
            None => Ok(py.None()),
        })
    }

    // ========================================
    // Magic methods for Pythonic API
    // ========================================

    /// Support `rt["var"]` syntax for getting globals
    fn __getitem__(&self, py: Python<'_>, key: &str) -> PyResult<Py<PyAny>> {
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

    // ========================================
    // Context manager support
    // ========================================

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
        Ok(false) // Don't suppress exceptions
    }
}
