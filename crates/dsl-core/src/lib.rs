//! DSL Core - Execution Engine and Parser
//!
//! This crate provides the core functionality for the DSL language:
//! - Parsing DSL syntax into AST
//! - Compiling AST to IR
//! - Evaluating expressions (sync & async)
//! - Type system and registry
//! - Built-in function implementations
//! - Runtime value representation

pub mod parser;
pub mod compiler;
pub mod eval;
pub mod types;

// Re-export commonly used items
pub use compiler::{compile_to_ir, compile_function, compile_expr};
pub use eval::Evaluator;
pub use types::{TypeRegistry, Value};
pub use parser::{
    parse_expr,
    parse_expr_with_binding,
    parse_type_definition,
    parse_enum_definition,
    parse_function_definition,
    Expr,
    FunctionDef,
    FunctionExecution,
    Binding,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::compiler::{compile_to_ir, compile_function};
    pub use crate::eval::Evaluator;
    pub use crate::types::{TypeRegistry, Value};
    pub use crate::parser::{Expr, FunctionDef};
}
