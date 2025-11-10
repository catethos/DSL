//! Shared evaluation helpers for REPL implementations
//!
//! This module provides common functionality for parsing and compiling DSL input
//! in REPL contexts, handling the distinction between statements (like `let`)
//! and expressions.

use crate::parser::{parse_expr, parse_program};
use crate::compiler::compile_expr;
use dsl_ir::{IRBinding, IRNode};

/// Result of parsing input, containing the IR node and optional binding information
pub struct ParsedInput {
    /// The compiled IR node ready for evaluation
    pub ir_node: IRNode,
    /// Optional binding information (for let statements)
    pub binding: Option<IRBinding>,
}

/// Parse and compile input from a REPL context
///
/// This function handles both statements (like `let x = 2`) and expressions.
/// It automatically routes to the correct parser based on the input.
///
/// # Arguments
/// * `input` - The input string to parse
///
/// # Returns
/// * `Ok(ParsedInput)` - Successfully parsed and compiled input
/// * `Err(String)` - Parse or compile error message
pub fn parse_repl_input(input: &str) -> Result<ParsedInput, String> {
    let input = input.trim();

    // Check if this is a statement that requires parse_program
    if input.starts_with("let ")
        || input.starts_with("type ")
        || input.starts_with("enum ")
        || input.starts_with("def ")
        || input.starts_with("function ")
    {
        parse_as_program(input)
    } else {
        parse_as_expression(input)
    }
}

/// Parse input as a program (for statements like let, type, enum, def)
fn parse_as_program(input: &str) -> Result<ParsedInput, String> {
    let program = parse_program(input).map_err(|e| format!("Parse error: {}", e))?;

    // Get entry expression from the program
    if let Some(entry_expr) = program.entry_expr {
        let ir_node = compile_expr(&entry_expr).map_err(|e| format!("Compile error: {}", e))?;

        // Extract binding info from the IR node
        let binding = extract_binding(&ir_node);

        Ok(ParsedInput { ir_node, binding })
    } else {
        Err("Parse error: No entry expression in program".to_string())
    }
}

/// Parse input as an expression
fn parse_as_expression(input: &str) -> Result<ParsedInput, String> {
    let ast = parse_expr(input).map_err(|e| format!("Parse error: {}", e))?;
    let ir_node = compile_expr(&ast).map_err(|e| format!("Compile error: {}", e))?;

    // Extract binding info (for expressions with 'as' binding)
    let binding = extract_binding(&ir_node);

    Ok(ParsedInput { ir_node, binding })
}

/// Extract binding information from an IR node
fn extract_binding(ir_node: &IRNode) -> Option<IRBinding> {
    match ir_node {
        IRNode::Parallel {
            binding: Some(bind),
            ..
        } => Some(bind.clone()),
        IRNode::Sequential {
            binding: Some(bind),
            ..
        } => Some(bind.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_let_statement() {
        let result = parse_repl_input("let y = 2").unwrap();
        assert!(result.binding.is_some());
        if let Some(IRBinding::Single(name)) = result.binding {
            assert_eq!(name, "y");
        } else {
            panic!("Expected Single binding");
        }
    }

    #[test]
    fn test_parse_expression() {
        let result = parse_repl_input("2 + 3").unwrap();
        assert!(result.binding.is_none());
    }

    #[test]
    fn test_parse_expression_with_as_binding() {
        let result = parse_repl_input("5 as x").unwrap();
        assert!(result.binding.is_some());
        if let Some(IRBinding::Single(name)) = result.binding {
            assert_eq!(name, "x");
        } else {
            panic!("Expected Single binding");
        }
    }
}
