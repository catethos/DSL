//! Error types for Lattice
//!
//! Defines error types for parsing, type checking, compilation, and runtime errors.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LatticeError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Type error: {0}")]
    Type(String),

    #[error("Compile error: {0}")]
    Compile(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("SQL error: {0}")]
    Sql(String),
}

pub type Result<T> = std::result::Result<T, LatticeError>;
