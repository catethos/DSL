//! Language syntax module
//!
//! Contains the Pest grammar, AST node definitions, parser, and desugaring.

pub mod ast;
pub mod desugar;
pub mod parser;

pub use ast::*;
pub use desugar::{contains_dollar_field, replace_dollar_fields, wrap_dollar_expr_in_lambda};
pub use parser::{parse, parse_expression};
