//! Error handling and conversion for PyO3 bindings

use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;

use lattice::error::LatticeError;

/// Convert a LatticeError to a Python exception
pub fn to_py_err(err: LatticeError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}
