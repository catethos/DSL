//! Language syntax module
//!
//! Contains the Pest grammar, AST node definitions, parser, desugaring, and import resolution.

pub mod ast;
pub mod desugar;
pub mod imports;
pub mod parser;

pub use ast::*;
pub use desugar::{contains_dollar_field, replace_dollar_fields, wrap_dollar_expr_in_lambda};
pub use imports::resolve_imports;
pub use parser::{parse, parse_expression};
