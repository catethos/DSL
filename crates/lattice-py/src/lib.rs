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
