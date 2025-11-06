//! DSL Runtime Support Library
//!
//! This crate provides runtime support for compiled DSL programs.
//! It re-exports types and functions needed by generated Rust code.

// Re-export Value type
pub use dsl_ir::Value;

// Re-export builtin functions
pub mod builtins;
pub use builtins::*;

// Re-export Runtime for stateful programs
pub use dsl_interpreter::Runtime;

// Re-export common types
pub use anyhow::{anyhow, Context, Result};
pub use indexmap::IndexMap;
pub use serde::{Deserialize, Serialize};
pub use serde_json;
pub use tokio;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_re_export() {
        let v = Value::String("test".to_string());
        assert_eq!(v, Value::String("test".to_string()));
    }

    #[test]
    fn test_runtime_re_export() {
        let _runtime = Runtime::new();
    }
}
