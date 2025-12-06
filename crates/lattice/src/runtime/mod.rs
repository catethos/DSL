//! Runtime module for embeddable Lattice
//!
//! This module provides FFI-safe types and APIs for embedding Lattice
//! in other languages via bindings (Rustler, PyO3, Neon, etc.).
//!
//! ## Key Components
//!
//! - [`LatticeValue`]: FFI-safe value type for cross-language marshaling
//! - [`providers`]: Injectable provider traits (LLM, SQL, etc.)

mod value;
pub mod providers;

pub use value::{ConversionError, LatticeValue};
pub use providers::{
    DefaultLlmProvider, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmUsage,
    NoLlmProvider, NoSqlProvider, SqlError, SqlProvider, SqlResult, SqlRow,
};

#[cfg(feature = "sql")]
pub use providers::DuckDbProvider;
