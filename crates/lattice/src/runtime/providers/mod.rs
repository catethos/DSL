//! Injectable provider traits for external capabilities
//!
//! This module defines traits for LLM, SQL, and other external capabilities
//! that can be injected into the Lattice runtime. This enables:
//!
//! - Custom implementations (e.g., Elixir callback for LLM calls)
//! - Disabling capabilities in embedded contexts
//! - Testing with mock providers

pub mod llm;
pub mod sql;

pub use llm::{
    BoxedLlmProvider, DefaultLlmProvider, LlmError, LlmMessage, LlmProvider, LlmRequest,
    LlmResponse, LlmUsage, NoLlmProvider, ProviderRouting,
};

pub use sql::{BoxedSqlProvider, NoSqlProvider, SqlError, SqlProvider, SqlResult, SqlRow};

#[cfg(feature = "sql")]
pub use sql::DuckDbProvider;
