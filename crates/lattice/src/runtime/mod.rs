//! Runtime module for embeddable Lattice
//!
//! This module provides FFI-safe types and APIs for embedding Lattice
//! in other languages via bindings (Rustler, PyO3, Neon, etc.).
//!
//! ## Key Components
//!
//! - [`LatticeValue`]: FFI-safe value type for cross-language marshaling
//! - [`providers`]: Injectable provider traits (LLM, SQL, etc.)
//! - [`RuntimeBuilder`]: Builder pattern for constructing isolated runtime instances
//!
//! ## Example
//!
//! ```ignore
//! use lattice::runtime::{RuntimeBuilder, LatticeValue};
//!
//! // Create a runtime with default providers
//! let runtime = RuntimeBuilder::new()
//!     .with_default_providers()?
//!     .build()?;
//!
//! // Create a minimal runtime without LLM/SQL
//! let minimal = RuntimeBuilder::new()
//!     .without_llm()
//!     .without_sql()
//!     .build()?;
//! ```

mod builder;
mod value;
pub mod providers;

pub use builder::{BuiltRuntime, RuntimeBuilder, RuntimeConfig};
pub use value::{ConversionError, LatticeValue};
pub use providers::{
    DefaultLlmProvider, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmUsage,
    NoLlmProvider, NoSqlProvider, SqlError, SqlProvider, SqlResult, SqlRow,
};

#[cfg(feature = "sql")]
pub use providers::DuckDbProvider;
