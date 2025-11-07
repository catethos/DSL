//! DSL Core - Parser and Compiler
//!
//! This crate provides the core parsing and compilation functionality for the DSL language:
//! - Parsing DSL syntax into AST
//! - Compiling AST to IR
//! - Type definitions
//!
//! Note: Execution/evaluation has been moved to dsl-interpreter

pub mod parser;
pub mod compiler;
pub mod eval;  // Keep for backward compatibility (re-exports only)
pub mod types;

// Re-export commonly used items
pub use compiler::{compile_to_ir, compile_function, compile_expr, compile_program, compile_program_to_ir};
pub use types::{TypeRegistry, Value};
pub use parser::{
    parse_expr,
    parse_expr_with_binding,
    parse_type_definition,
    parse_enum_definition,
    parse_function_definition,
    parse_program,
    Expr,
    FunctionDef,
    FunctionExecution,
    Binding,
    Program,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::compiler::{compile_to_ir, compile_function};
    pub use crate::types::{TypeRegistry, Value};
    pub use crate::parser::{Expr, FunctionDef};
}
