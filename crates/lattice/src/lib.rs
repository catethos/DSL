//! Lattice - A statically-typed language for structured LLM interactions
//!
//! Lattice provides first-class support for LLM function definitions and
//! data manipulation via DuckDB SQL integration.

pub mod compiler;
pub mod error;
pub mod llm;
pub mod output;
pub mod runtime;
#[cfg(feature = "sql")]
pub mod sql;
pub mod stdlib;
pub mod syntax;
pub mod types;
pub mod vm;
