//! Lattice - A statically-typed language for structured LLM interactions
//!
//! Lattice provides first-class support for LLM function definitions and
//! data manipulation via DuckDB SQL integration.

pub mod types;
pub mod llm;
pub mod syntax;
pub mod compiler;
pub mod vm;
pub mod sql;
pub mod stdlib;
pub mod output;
pub mod error;
