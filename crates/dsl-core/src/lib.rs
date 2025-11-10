//! DSL Core - Parser and Compiler
//!
//! This crate provides the core parsing and compilation functionality for the DSL language:
//! - Parsing DSL syntax into AST
//! - Compiling AST to IR
//! - Type definitions
//!
//! Note: Execution/evaluation has been moved to dsl-interpreter

pub mod compiler;
pub mod eval; // Keep for backward compatibility (re-exports only)
pub mod eval_helpers; // Shared REPL evaluation helpers
pub mod keywords;
pub mod parser;
pub mod resolver;
pub mod types;

#[cfg(test)]
mod parser_debug_test;

// Re-export commonly used items
pub use compiler::{
    compile_expr,
    compile_function,
    compile_function_group, // New resolver-based compilation
    compile_program,
    compile_program_to_ir,
    compile_program_to_ir_with_resolver, // New resolver workflow
    compile_program_with_resolver,
    compile_to_ir,
};
pub use eval_helpers::{parse_repl_input, ParsedInput};
pub use keywords::{boolean_info, keyword_info, type_info, BooleanInfo, KeywordInfo, TypeInfo};
pub use parser::{
    parse_enum_definition, parse_expr, parse_expr_with_binding, parse_function_definition,
    parse_program, parse_type_definition, Binding, Expr, FunctionDef, FunctionExecution, Program,
};
pub use resolver::{
    convert_function_def_to_clause, convert_pattern_clause_to_clause, resolve_program, Clause,
    FunctionBody, FunctionGroup, SymbolTable,
};
pub use types::{TypeRegistry, Value};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::compiler::{compile_function, compile_to_ir};
    pub use crate::parser::{Expr, FunctionDef};
    pub use crate::types::{TypeRegistry, Value};
}
