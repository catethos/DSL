use super::ast::*;
use super::expressions::{build_expr, build_pattern, parse_template_string};
use super::types::build_field_type;
use super::{DslParser, Rule};
use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashMap;

/// Build a block expression from the new block grammar: { block_item* }
/// Uses flat sequence parsing with post-validation
fn build_block_from_new_grammar(pair: Pair<Rule>) -> Result<Expr, String> {
    // pair is a `block` rule, collect all block_items
    let mut items = Vec::new();

    for inner_pair in pair.into_inner() {
        if inner_pair.as_rule() == Rule::block_item {
            items.push(inner_pair);
        }
    }

    // Post-validation: ensure block structure is valid
    if items.is_empty() {
        return Err("Block cannot be empty".to_string());
    }

    // Separate let statements from expressions
    let mut statements = Vec::new();
    let mut expressions = Vec::new();

    for item_pair in items {
        let inner = item_pair.into_inner().next().ok_or("Empty block item")?;

        match inner.as_rule() {
            Rule::let_statement => {
                // let_statement: "let" ~ (pattern | identifier) ~ "=" ~ expr
                let mut inner_parts = inner.into_inner();
                let pattern_or_id = inner_parts
                    .next()
                    .ok_or("Missing pattern in let statement")?;
                let value_expr = inner_parts.next().ok_or("Missing value in let statement")?;

                let binding = match pattern_or_id.as_rule() {
                    Rule::identifier => {
                        Some(Binding::Single(pattern_or_id.as_str().to_string()))
                    }
                    Rule::pattern => {
                        // Pattern could be pattern_variable (which wraps an identifier) or other patterns
                        // We need to extract the actual binding
                        let pattern_inner = pattern_or_id.into_inner().next()
                            .ok_or("Empty pattern")?;
                        match pattern_inner.as_rule() {
                            Rule::pattern_variable => {
                                // pattern_variable contains an identifier
                                let id = pattern_inner.into_inner().next()
                                    .ok_or("Empty pattern_variable")?;
                                Some(Binding::Single(id.as_str().to_string()))
                            }
                            Rule::pattern_list => {
                                // For list patterns like [a, b, c], extract variable names
                                // This is a simplified version - full pattern support would be more complex
                                return Err("List pattern bindings not yet fully supported in let statements".to_string());
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                };

                let value = build_expr(value_expr)?;
                statements.push(Expr::Parallel {
                    exprs: vec![value],
                    binding,
                });
            }
            Rule::expr => {
                // Expression (could be final or in the middle)
                expressions.push(build_expr(inner)?);
            }
            _ => {}
        }
    }

    // Post-validation: last item must be an expression
    if expressions.is_empty() {
        return Err(
            "Block must end with an expression (statements alone are not sufficient)".to_string(),
        );
    }

    // Validation: if we have multiple expressions, only the last is allowed
    if expressions.len() > 1 {
        return Err(format!(
            "Block has {} expressions without statement terminators. Only the last item can be an unterminated expression.",
            expressions.len()
        ));
    }

    let result = expressions.into_iter().next().unwrap();

    if statements.is_empty() {
        // Just a single expression, no block needed
        Ok(result)
    } else {
        // Create a Block expression
        Ok(Expr::Block {
            statements,
            result: Box::new(result),
        })
    }
}

/// Extract a string property value from properties HashMap
fn extract_string_property(
    properties: &HashMap<String, PropertyValue>,
    key: &str,
) -> Option<String> {
    properties.get(key).and_then(|v| match v {
        PropertyValue::String(s) => Some(s.clone()),
        PropertyValue::Template(segments) => {
            // For now, just concatenate template segments (no interpolation)
            let mut result = String::new();
            for seg in segments {
                match seg {
                    TemplateSegment::Text(t) => result.push_str(t),
                    TemplateSegment::Interpolation(i) => {
                        result.push_str("${");
                        result.push_str(i);
                        result.push('}');
                    }
                }
            }
            Some(result)
        }
        _ => None,
    })
}

/// Extract a float property value from properties HashMap
fn extract_float_property(properties: &HashMap<String, PropertyValue>, key: &str) -> Option<f64> {
    properties.get(key).and_then(|v| match v {
        PropertyValue::Float(f) => Some(*f),
        PropertyValue::Int(i) => Some(*i as f64),
        _ => None,
    })
}

/// Parse an HTTP block from the grammar
fn parse_http_block(pair: Pair<Rule>) -> Result<HttpConfig, String> {
    let mut inner = pair.into_inner();

    // First item should be the HTTP method in a string literal
    let method_pair = inner.next().ok_or("Missing HTTP method")?;
    let method = extract_string_content(method_pair)?;

    let mut url = String::new();
    let params = None;
    let headers = None;
    let body = None;

    // Parse remaining properties
    for item in inner {
        match item.as_rule() {
            Rule::string_literal | Rule::template_string => {
                // This is the URL
                url = extract_string_content(item)?;
            }
            _ => {}
        }
    }

    Ok((method, url, params, headers, body))
}

/// Parse a function definition
pub fn parse_function_definition(input: &str) -> Result<FunctionDef, String> {
    let pairs =
        DslParser::parse(Rule::function_decl, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| "No function definition found".to_string())?;

    build_function_definition(pair)
}

/// Build a traditional function from name, params, return type, and body
fn build_traditional_function(
    name: String,
    params: Vec<String>,
    return_type: Option<dsl_types::FieldType>,
    body_pair: Pair<Rule>,
) -> Result<FunctionOrClause, String> {
    let mut properties = HashMap::new();
    let mut prompt_value: Option<String> = None;
    let mut sql_value: Option<String> = None;
    let mut http_config: Option<HttpConfig> = None;

    // Parse properties and prompt/sql/http blocks
    for body_item in body_pair.into_inner() {
        match body_item.as_rule() {
            Rule::property => {
                let (key, value) = build_property(body_item)?;
                properties.insert(key, value);
            }
            Rule::prompt_block => {
                let prompt_inner = body_item.into_inner().next().ok_or("Empty prompt block")?;
                prompt_value = Some(extract_string_content(prompt_inner)?);
            }
            Rule::sql_block => {
                let sql_inner = body_item.into_inner().next().ok_or("Empty sql block")?;
                sql_value = Some(extract_string_content(sql_inner)?);
            }
            Rule::http_block => {
                http_config = Some(parse_http_block(body_item)?);
            }
            _ => {}
        }
    }

    // Determine execution type based on what blocks are present
    let has_prompt = prompt_value.is_some();
    let has_http = http_config.is_some();
    let has_sql = sql_value.is_some();

    let execution = match (has_prompt, has_http, has_sql) {
        (true, false, false) => {
            // LLM function
            FunctionExecution::LLM {
                prompt: prompt_value.unwrap(),
                model: extract_string_property(&properties, "model"),
                base_url: extract_string_property(&properties, "base_url"),
                api_key_env: extract_string_property(&properties, "api_key_env"),
                temperature: extract_float_property(&properties, "temperature"),
            }
        }
        (false, true, false) => {
            // Pure HTTP function
            let (method, mut url, params, headers, body) = http_config.unwrap();

            // Check if URL is in properties (since it's parsed separately as a property)
            if url.is_empty() {
                url = extract_string_property(&properties, "url").unwrap_or_default();
            }

            FunctionExecution::HTTP {
                method,
                url,
                params,
                headers,
                body,
            }
        }
        (false, false, true) => {
            // SQL function
            FunctionExecution::SQL {
                query: sql_value.unwrap(),
            }
        }
        (true, true, false) => {
            return Err(format!(
                "Function '{}' cannot have both 'http' and 'prompt' blocks. Use composition instead: call HTTP as a regular function, then pass the result to an LLM function.",
                name
            ));
        }
        (false, false, false) => {
            return Err(format!(
                "Function '{}' must have at least one of: prompt, http, or sql block",
                name
            ));
        }
        _ => {
            return Err(format!(
                "Function '{}' has an invalid combination of blocks. Use either: prompt only, http only, sql only, or http + prompt",
                name
            ));
        }
    };

    Ok(FunctionOrClause::Function(FunctionDef {
        name,
        params,
        return_type,
        properties,
        execution,
    }))
}

/// Build a function definition or pattern clause from AST
pub(super) fn build_function_or_clause(pair: Pair<Rule>) -> Result<FunctionOrClause, String> {
    let mut inner = pair.into_inner();

    // Get function name
    let name = inner
        .next()
        .ok_or("Missing function name")?
        .as_str()
        .to_string();

    // Get parameters - check if they're patterns or simple identifiers
    let params_pair = inner.next().ok_or("Missing parameters")?;
    let mut has_patterns = false;
    let mut param_patterns = Vec::new();
    let mut param_names = Vec::new();

    for p in params_pair.into_inner() {
        if p.as_rule() == Rule::parameter {
            let mut param_inner = p.into_inner();
            if let Some(first) = param_inner.next() {
                match first.as_rule() {
                    // NEW: Handle typed_param rule
                    Rule::typed_param => {
                        // typed_param: identifier ~ ":" ~ field_type
                        let mut typed_inner = first.into_inner();
                        let id_pair = typed_inner
                            .next()
                            .ok_or("Missing identifier in typed param")?;
                        let id_name = id_pair.as_str().to_string();
                        param_names.push(id_name.clone());
                        param_patterns.push(Pattern::Variable(id_name));
                        // Type info is parsed but not used here (for future type checking)
                    }
                    // NEW: Handle pattern_param rule
                    Rule::pattern_param => {
                        // pattern_param: pattern
                        let pattern_pair = first
                            .into_inner()
                            .next()
                            .ok_or("Missing pattern in pattern_param")?;
                        let pattern = build_pattern(pattern_pair)?;
                        // Only set has_patterns if it's a complex pattern (not just a variable)
                        if !matches!(pattern, Pattern::Variable(_)) {
                            has_patterns = true;
                        }
                        // Extract parameter name if it's a simple variable
                        if let Pattern::Variable(name) = &pattern {
                            param_names.push(name.clone());
                        }
                        param_patterns.push(pattern);
                    }
                    // OLD: Keep for backward compatibility (though shouldn't be reached)
                    Rule::pattern => {
                        // This is a pattern parameter
                        let pattern = build_pattern(first)?;
                        // Only set has_patterns if it's a complex pattern (not just a variable)
                        if !matches!(pattern, Pattern::Variable(_)) {
                            has_patterns = true;
                        }
                        // Extract parameter name if it's a simple variable
                        if let Pattern::Variable(name) = &pattern {
                            param_names.push(name.clone());
                        }
                        param_patterns.push(pattern);
                    }
                    Rule::identifier => {
                        // Could be: identifier : field_type OR just identifier
                        let id_name = first.as_str().to_string();
                        param_names.push(id_name.clone());
                        param_patterns.push(Pattern::Variable(id_name));
                        // Skip the type annotation if present
                    }
                    _ => {
                        return Err(format!("Unexpected parameter rule: {:?}", first.as_rule()));
                    }
                }
            }
        }
    }

    // Check for return type (optional)
    let mut return_type = None;
    let mut body_pair = inner.next();

    // If next element is return_type, extract it
    if let Some(ref pair) = body_pair {
        if pair.as_rule() == Rule::return_type {
            // return_type: "->" ~ field_type
            let field_type_pair = pair
                .clone()
                .into_inner()
                .next()
                .ok_or("Missing field type in return type")?;
            return_type = Some(build_field_type(field_type_pair)?);
            body_pair = inner.next();
        }
    }

    let body_pair = body_pair.ok_or("Missing function body")?;

    // Check the body type - now it's function_body (not function_decl_body)
    if body_pair.as_rule() != Rule::function_body {
        return Err(format!(
            "Expected function_body, got {:?}",
            body_pair.as_rule()
        ));
    }

    let body_content = body_pair.into_inner().next().ok_or("Empty function body")?;

    match body_content.as_rule() {
        Rule::expr => {
            // Arrow function: => expr
            let body_expr = build_expr(body_content)?;

            // If has patterns, return as Clause
            // If only simple parameters, return as Function
            if has_patterns {
                Ok(FunctionOrClause::Clause {
                    name,
                    clause: PatternFunctionClause {
                        param_patterns,
                        guard: None,
                        body: body_expr,
                    },
                    return_type,
                })
            } else {
                Ok(FunctionOrClause::Function(FunctionDef {
                    name,
                    params: param_names,
                    return_type,
                    properties: HashMap::new(),
                    execution: FunctionExecution::Expression {
                        body: Box::new(body_expr),
                    },
                }))
            }
        }
        Rule::block => {
            // Block function: { statement* ~ expr }
            let body_expr = build_block_from_new_grammar(body_content)?;

            // If has patterns, return as Clause
            // If only simple parameters, return as Function
            if has_patterns {
                Ok(FunctionOrClause::Clause {
                    name,
                    clause: PatternFunctionClause {
                        param_patterns,
                        guard: None,
                        body: body_expr,
                    },
                    return_type,
                })
            } else {
                Ok(FunctionOrClause::Function(FunctionDef {
                    name,
                    params: param_names,
                    return_type,
                    properties: HashMap::new(),
                    execution: FunctionExecution::Expression {
                        body: Box::new(body_expr),
                    },
                }))
            }
        }
        Rule::special_body => {
            // Special function with prompt/sql/http
            if has_patterns {
                return Err(
                    "Pattern parameters are only allowed in expression-based functions".to_string(),
                );
            }
            // Parse as traditional function
            build_traditional_function(name, param_names, return_type, body_content)
        }
        _ => Err(format!(
            "Unexpected function body rule: {:?}",
            body_content.as_rule()
        )),
    }
}

fn build_function_definition(pair: Pair<Rule>) -> Result<FunctionDef, String> {
    let mut inner = pair.into_inner();

    // Get function name
    let name = inner
        .next()
        .ok_or("Missing function name")?
        .as_str()
        .to_string();

    // Get parameters
    let params_pair = inner.next().ok_or("Missing parameters")?;
    let params: Vec<String> = params_pair
        .into_inner()
        .map(|p| {
            // Each def_parameter has an identifier and optional type
            // We only want the identifier (parameter name)
            let mut inner = p.into_inner();
            inner
                .next()
                .map(|id| id.as_str().to_string())
                .unwrap_or_default()
        })
        .collect();

    // Check for return type (optional)
    let mut return_type = None;
    let mut properties = HashMap::new();
    let mut prompt_value: Option<String> = None;
    let mut sql_value: Option<String> = None;
    let mut http_config: Option<HttpConfig> = None;

    for item in inner {
        match item.as_rule() {
            Rule::field_type => {
                return_type = Some(build_field_type(item)?);
            }
            Rule::function_body => {
                // Parse properties and prompt/sql/http blocks
                for body_item in item.into_inner() {
                    match body_item.as_rule() {
                        Rule::property => {
                            let (key, value) = build_property(body_item)?;
                            properties.insert(key, value);
                        }
                        Rule::prompt_block => {
                            let prompt_inner =
                                body_item.into_inner().next().ok_or("Empty prompt block")?;
                            prompt_value = Some(extract_string_content(prompt_inner)?);
                        }
                        Rule::sql_block => {
                            let sql_inner =
                                body_item.into_inner().next().ok_or("Empty sql block")?;
                            sql_value = Some(extract_string_content(sql_inner)?);
                        }
                        Rule::http_block => {
                            http_config = Some(parse_http_block(body_item)?);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Determine execution type based on what blocks are present
    let has_prompt = prompt_value.is_some();
    let has_http = http_config.is_some();
    let has_sql = sql_value.is_some();

    let execution = match (has_prompt, has_http, has_sql) {
        (true, false, false) => {
            // LLM function
            FunctionExecution::LLM {
                prompt: prompt_value.unwrap(),
                model: extract_string_property(&properties, "model"),
                base_url: extract_string_property(&properties, "base_url"),
                api_key_env: extract_string_property(&properties, "api_key_env"),
                temperature: extract_float_property(&properties, "temperature"),
            }
        }
        (false, true, false) => {
            // Pure HTTP function
            let (method, mut url, params, headers, body) = http_config.unwrap();

            // Check if URL is in properties (since it's parsed separately as a property)
            if url.is_empty() {
                url = extract_string_property(&properties, "url").unwrap_or_default();
            }

            FunctionExecution::HTTP {
                method,
                url,
                params,
                headers,
                body,
            }
        }
        (false, false, true) => {
            // SQL function
            FunctionExecution::SQL {
                query: sql_value.unwrap(),
            }
        }
        (true, true, false) => {
            return Err(format!(
                "Function '{}' cannot have both 'http' and 'prompt' blocks. Use composition instead: call HTTP as a regular function, then pass the result to an LLM function.",
                name
            ));
        }
        (false, false, false) => {
            return Err(format!(
                "Function '{}' must have at least one of: prompt, http, or sql block",
                name
            ));
        }
        _ => {
            return Err(format!(
                "Function '{}' has an invalid combination of blocks. Use either: prompt only, http only, sql only, or http + prompt",
                name
            ));
        }
    };

    Ok(FunctionDef {
        name,
        params,
        return_type,
        properties,
        execution,
    })
}

fn build_property(pair: Pair<Rule>) -> Result<(String, PropertyValue), String> {
    let mut inner = pair.into_inner();

    let key = inner
        .next()
        .ok_or("Missing property key")?
        .as_str()
        .to_string();

    let value_pair = inner.next().ok_or("Missing property value")?;

    let value = match value_pair.as_rule() {
        Rule::string_literal => {
            let inner = value_pair.into_inner().next().ok_or("Empty string")?;
            PropertyValue::String(inner.as_str().to_string())
        }
        Rule::template_string => {
            let segments = parse_template_string(value_pair)?;
            PropertyValue::Template(segments)
        }
        Rule::integer => {
            let val = value_pair
                .as_str()
                .parse::<i64>()
                .map_err(|e| format!("Invalid integer: {}", e))?;
            PropertyValue::Int(val)
        }
        Rule::float => {
            let val = value_pair
                .as_str()
                .parse::<f64>()
                .map_err(|e| format!("Invalid float: {}", e))?;
            PropertyValue::Float(val)
        }
        Rule::boolean => {
            let val = value_pair.as_str() == "true";
            PropertyValue::Bool(val)
        }
        _ => {
            return Err(format!(
                "Unsupported property value type: {:?}",
                value_pair.as_rule()
            ))
        }
    };

    Ok((key, value))
}

fn extract_string_content(pair: Pair<Rule>) -> Result<String, String> {
    match pair.as_rule() {
        Rule::triple_string_inner | Rule::plain_string_inner | Rule::single_string_inner => {
            Ok(pair.as_str().to_string())
        }
        Rule::template_string => {
            // For templates, reconstruct the string with ${} syntax
            let segments = parse_template_string(pair)?;
            let mut result = String::new();
            for segment in segments {
                match segment {
                    TemplateSegment::Text(text) => result.push_str(&text),
                    TemplateSegment::Interpolation(expr) => {
                        result.push_str("${");
                        result.push_str(&expr);
                        result.push('}');
                    }
                }
            }
            Ok(result)
        }
        _ => {
            let inner = pair.into_inner().next().ok_or("Empty string content")?;
            extract_string_content(inner)
        }
    }
}
