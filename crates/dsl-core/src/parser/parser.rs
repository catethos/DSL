use super::ast::*;
use super::expressions::build_expr;
use super::functions::build_function_or_clause;
use super::types::{build_enum_definition, build_type_definition};
use super::{DslParser, Rule};
use pest::Parser;
use std::collections::HashMap;

/// Parse a complete program from input string
pub fn parse_program(input: &str) -> Result<Program, String> {
    let pairs = DslParser::parse(Rule::program, input)
        .map_err(|e| format!("Parse error: {}", e))?;

    let mut types = Vec::new();
    let mut enums = Vec::new();
    let mut functions = Vec::new();
    let mut pattern_functions_map: HashMap<String, Vec<PatternFunctionClause>> = HashMap::new();
    let mut entry_expr: Option<Expr> = None;

    for pair in pairs {
        for decl_pair in pair.into_inner() {
            match decl_pair.as_rule() {
                Rule::type_decl => {
                    types.push(build_type_definition(decl_pair)?);
                }
                Rule::enum_decl => {
                    enums.push(build_enum_definition(decl_pair)?);
                }
                Rule::function_decl => {
                    let func_or_clause = build_function_or_clause(decl_pair)?;
                    match func_or_clause {
                        FunctionOrClause::Function(func) => {
                            functions.push(func);
                        }
                        FunctionOrClause::Clause { name, clause, return_type: _ } => {
                            pattern_functions_map
                                .entry(name)
                                .or_default()
                                .push(clause);
                            // Note: We'll need to handle return_type consistency later
                        }
                    }
                }
                Rule::entry_expr => {
                    // Parse the entry expression
                    let expr_pair = decl_pair.into_inner().next().ok_or("Empty entry expression")?;
                    entry_expr = Some(build_expr(expr_pair)?);
                }
                Rule::expr => {
                    // Treat bare expr as entry expression (when it appears at program level)
                    entry_expr = Some(build_expr(decl_pair)?);
                }
                Rule::EOI => break,
                _ => {}
            }
        }
    }

    // Convert pattern function map to PatternFunctionDef list
    let mut pattern_functions = Vec::new();
    for (name, clauses) in pattern_functions_map {
        pattern_functions.push(PatternFunctionDef {
            name,
            clauses,
            return_type: None, // TODO: Infer or validate return types
        });
    }

    Ok(Program {
        types,
        enums,
        functions,
        pattern_functions,
        entry_expr,
    })
}

/// Parse an expression from input string
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    let pairs = DslParser::parse(Rule::expr, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| "No expression found".to_string())?;

    build_expr(pair)
}

/// Parse an expression and return any top-level binding
/// Note: Bindings are now handled within Sequential expression nodes
pub fn parse_expr_with_binding(input: &str) -> Result<(Expr, Option<Binding>), String> {
    let expr = parse_expr(input)?;

    // Extract binding from top-level Sequential or Parallel expression
    let binding = match &expr {
        Expr::Sequential { binding, .. } => binding.clone(),
        Expr::Parallel { binding, .. } => binding.clone(),
        _ => None,
    };

    Ok((expr, binding))
}

/// Check if input is a command (starts with :)
pub fn is_command(input: &str) -> bool {
    input.trim().starts_with(':')
}

/// Parse command name
pub fn parse_command(input: &str) -> Option<String> {
    if !is_command(input) {
        return None;
    }

    let trimmed = input.trim();
    Some(trimmed[1..].to_string())
}
