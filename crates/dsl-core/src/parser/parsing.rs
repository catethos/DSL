use super::ast::*;
use super::error::format_parse_error;
use super::expressions::build_expr;
use super::functions::build_function_or_clause;
use super::types::{build_enum_definition, build_type_definition};
use super::validation::validate_balanced_delimiters;
use super::{DslParser, Rule};
use pest::Parser;
use std::collections::HashMap;

/// Convert a simple function to a pattern function clause
fn convert_simple_function_to_clause(func: &FunctionDef) -> PatternFunctionClause {
    // Convert each parameter name to a Variable pattern
    let param_patterns: Vec<Pattern> = func
        .params
        .iter()
        .map(|param_name| Pattern::Variable(param_name.clone()))
        .collect();

    // Extract the body from the execution
    let body = match &func.execution {
        FunctionExecution::Expression { body } => (**body).clone(),
        _ => panic!("Cannot convert non-expression function to pattern function clause"),
    };

    PatternFunctionClause {
        param_patterns,
        guard: None,
        body,
    }
}

/// Parse a complete program from input string
pub fn parse_program(input: &str) -> Result<Program, String> {
    // First validate balanced delimiters for better error messages
    if let Err(validation_error) = validate_balanced_delimiters(input) {
        return Err(validation_error.format(input));
    }

    let pairs =
        DslParser::parse(Rule::program, input).map_err(|e| format_parse_error(&e, input))?;

    let mut imports = Vec::new();
    let mut types = Vec::new();
    let mut enums = Vec::new();
    let mut functions: Vec<FunctionDef> = Vec::new();
    let mut pattern_functions_map: HashMap<String, Vec<PatternFunctionClause>> = HashMap::new();
    let mut statements = Vec::new();

    for item_pair in pairs {
        match item_pair.as_rule() {
            Rule::item => {
                // An item can be an import, declaration, or a statement
                let inner = item_pair.into_inner().next().ok_or("Empty item")?;
                match inner.as_rule() {
                    Rule::import_stmt => {
                        let mut import_inner = inner.into_inner();
                        let path_pair = import_inner.next().ok_or("Missing import path")?;

                        // Extract string literal value
                        let path = path_pair
                            .as_str()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();

                        // Check for optional alias
                        let alias = if let Some(alias_pair) = import_inner.next() {
                            if alias_pair.as_rule() == Rule::import_alias {
                                let alias_id = alias_pair
                                    .into_inner()
                                    .next()
                                    .ok_or("Missing alias identifier")?;
                                Some(alias_id.as_str().to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        imports.push(Import { path, alias });
                    }
                    Rule::declaration => {
                        let decl = inner.into_inner().next().ok_or("Empty declaration")?;
                        match decl.as_rule() {
                            Rule::type_decl => {
                                types.push(build_type_definition(decl)?);
                            }
                            Rule::enum_decl => {
                                enums.push(build_enum_definition(decl)?);
                            }
                            Rule::function_decl => {
                                let func_or_clause = build_function_or_clause(decl)?;
                                match func_or_clause {
                                    FunctionOrClause::Function(func) => {
                                        let name = func.name.clone();

                                        // Check if this name already exists as function or pattern function
                                        use std::collections::hash_map::Entry;
                                        match pattern_functions_map.entry(name.clone()) {
                                            Entry::Occupied(mut entry) => {
                                                // Convert this simple function to a clause and add to existing pattern function
                                                let clause =
                                                    convert_simple_function_to_clause(&func);
                                                entry.get_mut().push(clause);
                                            }
                                            Entry::Vacant(entry) => {
                                                if let Some(existing_idx) =
                                                    functions.iter().position(|f| f.name == name)
                                                {
                                                    // Found existing simple function with same name
                                                    // Convert both to pattern function
                                                    let existing_func =
                                                        functions.remove(existing_idx);
                                                    let first_clause =
                                                        convert_simple_function_to_clause(
                                                            &existing_func,
                                                        );
                                                    let second_clause =
                                                        convert_simple_function_to_clause(&func);
                                                    entry.insert(vec![first_clause, second_clause]);
                                                } else {
                                                    // New simple function
                                                    functions.push(func);
                                                }
                                            }
                                        }
                                    }
                                    FunctionOrClause::Clause {
                                        name,
                                        clause,
                                        return_type: _,
                                    } => {
                                        // Check if a simple function with this name exists - convert it first
                                        if let Some(existing_idx) =
                                            functions.iter().position(|f| f.name == name)
                                        {
                                            let existing_func = functions.remove(existing_idx);
                                            let existing_clause =
                                                convert_simple_function_to_clause(&existing_func);
                                            pattern_functions_map
                                                .insert(name.clone(), vec![existing_clause]);
                                        }

                                        pattern_functions_map.entry(name).or_default().push(clause);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Rule::statement => {
                        // Statement can be let_statement or expr_statement
                        let stmt = inner.into_inner().next().ok_or("Empty statement")?;
                        let expr = match stmt.as_rule() {
                            Rule::let_statement => {
                                // let_statement: "let" ~ (pattern | identifier) ~ "=" ~ expr
                                let mut inner_pairs = stmt.into_inner();
                                let pattern_or_id = inner_pairs
                                    .next()
                                    .ok_or("Missing pattern in let statement")?;
                                let value_expr =
                                    inner_pairs.next().ok_or("Missing value in let statement")?;

                                // Build binding from pattern or identifier
                                let binding = if pattern_or_id.as_rule() == Rule::identifier {
                                    Some(Binding::Single(pattern_or_id.as_str().to_string()))
                                } else if pattern_or_id.as_rule() == Rule::pattern {
                                    // Check if it's a simple pattern_variable (which is just an identifier)
                                    let inner = pattern_or_id.into_inner().next();
                                    if let Some(pattern_inner) = inner {
                                        if pattern_inner.as_rule() == Rule::pattern_variable {
                                            // Extract the identifier from pattern_variable
                                            let id = pattern_inner
                                                .into_inner()
                                                .next()
                                                .ok_or("Missing identifier in pattern_variable")?;
                                            Some(Binding::Single(id.as_str().to_string()))
                                        } else {
                                            // For now, we'll handle list destructuring in let statements
                                            // Full pattern support can be added later
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                // Create a Parallel expression with binding (similar to old let_binding)
                                let value = build_expr(value_expr)?;
                                Expr::Parallel {
                                    exprs: vec![value],
                                    binding,
                                }
                            }
                            Rule::expr_statement => {
                                // expr_statement is just an expression
                                let expr_pair =
                                    stmt.into_inner().next().ok_or("Empty expr statement")?;
                                build_expr(expr_pair)?
                            }
                            _ => {
                                return Err(format!(
                                    "Unexpected statement rule: {:?}",
                                    stmt.as_rule()
                                ))
                            }
                        };
                        statements.push(expr);
                    }
                    _ => {}
                }
            }
            Rule::EOI => break,
            _ => {}
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

    // Convert multiple statements into a single entry expression
    // If we have statements, create a Block expression with all but the last as statements
    // and the last as the result
    let entry_expr = if statements.is_empty() {
        None
    } else if statements.len() == 1 {
        Some(statements.into_iter().next().unwrap())
    } else {
        // Multiple statements: wrap in a Block
        let result = statements.pop().unwrap();
        Some(Expr::Block {
            statements,
            result: Box::new(result),
        })
    };

    Ok(Program {
        imports,
        types,
        enums,
        functions,
        pattern_functions,
        entry_expr,
    })
}

/// Parse an expression from input string
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    // First validate balanced delimiters for better error messages
    if let Err(validation_error) = validate_balanced_delimiters(input) {
        return Err(validation_error.format(input));
    }

    let mut pairs =
        DslParser::parse(Rule::expr_with_eoi, input).map_err(|e| format_parse_error(&e, input))?;

    let pair = pairs
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

/// Parse REPL line (allows empty lines and comment-only lines)
pub fn parse_repl_line(input: &str) -> Result<Option<Expr>, String> {
    // First validate balanced delimiters for better error messages
    if let Err(validation_error) = validate_balanced_delimiters(input) {
        return Err(validation_error.format(input));
    }

    let mut pairs =
        DslParser::parse(Rule::repl_input, input).map_err(|e| format_parse_error(&e, input))?;

    // repl_input produces expr? (optional expr)
    // Because repl_input is silent, we get the inner expr directly (if present)
    let first_pair = pairs.next();

    if let Some(pair) = first_pair {
        // Check if this is an actual expr or just EOI/whitespace
        match pair.as_rule() {
            Rule::expr => {
                // It's an expression - build it
                Ok(Some(build_expr(pair)?))
            }
            _ => {
                // It's something else (like EOI) - treat as empty
                Ok(None)
            }
        }
    } else {
        // No expression found (empty or comment-only)
        Ok(None)
    }
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
