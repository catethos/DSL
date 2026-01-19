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

    // SQL on Lattice data errors
    #[error("Table '{name}' not found. {hint}")]
    SqlTableNotFound {
        name: String,
        hint: &'static str,
    },

    #[error("Variable '{name}' has wrong type for SQL query. Expected {expected}, found {found}")]
    SqlWrongType {
        name: String,
        expected: &'static str,
        found: String,
    },

    #[error("SQL construct '{construct}' not supported. {hint}")]
    SqlUnsupportedConstruct {
        construct: &'static str,
        hint: &'static str,
    },

    #[error("Schema inference failed for column '{column}': {message}")]
    SqlSchemaInference { column: String, message: String },
}

pub type Result<T> = std::result::Result<T, LatticeError>;
