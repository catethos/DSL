//! Language syntax module
//!
//! Contains the Pest grammar, AST node definitions, parser, desugaring, import resolution,
//! and markdown LLM file parsing.

pub mod ast;
pub mod desugar;
pub mod imports;
pub mod markdown;
pub mod parser;

pub use ast::*;
pub use desugar::{contains_dollar_field, replace_dollar_fields, wrap_dollar_expr_in_lambda};
pub use imports::resolve_imports;
pub use markdown::{parse_markdown_llm, MarkdownError, MarkdownLlmDef, InputDef, OutputDef, ProviderDef};
pub use parser::{parse, parse_expression};
