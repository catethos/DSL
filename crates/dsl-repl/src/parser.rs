use pest::Parser;
use pest_derive::Parser;
use simplify_baml::{Class, Enum, Field, FieldType};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct DslParser;

/// Parsed expression AST
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expr {
    /// String literal
    String(String),
    /// Template string with interpolation
    TemplateString(Vec<TemplateSegment>),
    /// Integer literal
    Int(i64),
    /// Float literal
    Float(f64),
    /// Boolean literal
    Bool(bool),
    /// List literal
    List(Vec<Expr>),
    /// Map literal
    Map(Vec<(String, Expr)>),
    /// Variable reference
    Variable(String),
    /// Function call: name(args)
    FunctionCall { name: String, args: Vec<Expr> },
    /// Field access: expr.field
    FieldAccess { base: Box<Expr>, field: String },
    /// Index access: expr[index]
    IndexAccess { base: Box<Expr>, index: Box<Expr> },
    /// Binary operation: left op right
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    /// Conditional: condition ? then_expr : else_expr
    Conditional {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    /// Sequential composition: left >> right
    Sequential {
        left: Box<Expr>,
        right: Box<Expr>,
        binding: Option<Binding>,
    },
    /// Parallel composition: left || right
    Parallel {
        exprs: Vec<Expr>,
        binding: Option<Binding>,
    },
}

/// Template string segments
#[derive(Debug, Clone)]
pub enum TemplateSegment {
    /// Plain text
    Text(String),
    /// Interpolated expression ${expr}
    Interpolation(String), // Store as string to be parsed later
}

/// Variable binding patterns
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Binding {
    /// Single variable: as name
    Single(String),
    /// Destructuring: as [a, b, c]
    List(Vec<String>),
}

/// Function execution modes
#[derive(Debug, Clone)]
pub enum FunctionExecution {
    /// LLM-based execution with prompt
    LLM {
        prompt: String,
        model: Option<String>,
        temperature: Option<f64>,
    },
    /// HTTP request execution
    HTTP {
        method: String,
        url: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
    },
    /// SQL query execution
    SQL {
        query: String,
    },
    /// Hybrid: HTTP then LLM processing
    HTTPWithLLM {
        http_method: String,
        http_url: String,
        http_params: Option<HashMap<String, String>>,
        http_headers: Option<HashMap<String, String>>,
        llm_prompt: String,
        llm_model: Option<String>,
        llm_temperature: Option<f64>,
    },
}

/// Function definition
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub return_type: Option<FieldType>,
    pub properties: HashMap<String, PropertyValue>,
    pub execution: FunctionExecution,
}

/// Property values in function definitions
#[derive(Debug, Clone)]
pub enum PropertyValue {
    String(String),
    Template(Vec<TemplateSegment>),
    Int(i64),
    Float(f64),
    Bool(bool),
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
    // Bindings are handled within the expression tree, not at the top level
    Ok((expr, None))
}

/// Parse a binding pattern
fn parse_binding(pair: pest::iterators::Pair<Rule>) -> Result<Binding, String> {
    let mut inner = pair.into_inner();
    let first = inner.next().ok_or("Empty binding")?;

    match first.as_rule() {
        Rule::identifier => Ok(Binding::Single(first.as_str().to_string())),
        Rule::list_binding => {
            let vars: Vec<String> = first.into_inner().map(|p| p.as_str().to_string()).collect();
            Ok(Binding::List(vars))
        }
        _ => Err(format!("Unexpected binding rule: {:?}", first.as_rule())),
    }
}

/// Build expression AST from Pest pair
fn build_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, String> {
    match pair.as_rule() {
        Rule::expr => {
            // Descend to conditional
            let inner = pair.into_inner().next().ok_or("Empty expression")?;
            build_expr(inner)
        }
        Rule::conditional => {
            let mut inner = pair.into_inner();
            let first = inner.next().ok_or("Empty conditional")?;

            // Check if there's a ternary operator
            if let Some(then_part) = inner.next() {
                let else_part = inner.next().ok_or("Missing else branch")?;
                Ok(Expr::Conditional {
                    condition: Box::new(build_expr(first)?),
                    then_expr: Box::new(build_expr(then_part)?),
                    else_expr: Box::new(build_expr(else_part)?),
                })
            } else {
                // No ternary, just descend
                build_expr(first)
            }
        }
        Rule::sequential => {
            let inner = pair.into_inner();
            let parts: Vec<pest::iterators::Pair<Rule>> = inner.collect();

            if parts.is_empty() {
                return Err("Empty sequential expression".to_string());
            }

            // Grammar: parallel ~ (binding)? ~ (">>" ~ parallel ~ (binding)?)*
            let mut i = 0;

            // Get first parallel expression
            let first_expr = build_expr(parts[i].clone())?;
            i += 1;

            // Check for optional binding after first expression
            let first_binding = if i < parts.len() && parts[i].as_rule() == Rule::binding {
                let b = parse_binding(parts[i].clone())?;
                i += 1;
                Some(b)
            } else {
                None
            };

            // If no >> operators, return the expression (possibly with binding)
            if i >= parts.len() {
                if let Some(binding) = first_binding {
                    // Wrap in a Parallel node with a single expression to handle the binding
                    // This avoids double evaluation that would happen with Sequential
                    return Ok(Expr::Parallel {
                        exprs: vec![first_expr],
                        binding: Some(binding),
                    });
                } else {
                    return Ok(first_expr);
                }
            }

            // Start building the chain
            let mut result = if let Some(binding) = first_binding {
                // First expression has a binding - use Parallel to avoid double evaluation
                Expr::Parallel {
                    exprs: vec![first_expr],
                    binding: Some(binding),
                }
            } else {
                first_expr
            };

            // Process remaining ">>" chains
            while i < parts.len() {
                let right_expr = build_expr(parts[i].clone())?;
                i += 1;

                // Check for optional binding
                let binding = if i < parts.len() && parts[i].as_rule() == Rule::binding {
                    let b = parse_binding(parts[i].clone())?;
                    i += 1;
                    Some(b)
                } else {
                    None
                };

                result = Expr::Sequential {
                    left: Box::new(result),
                    right: Box::new(right_expr),
                    binding,
                };
            }

            Ok(result)
        }
        Rule::parallel => {
            let inner = pair.into_inner();
            let parts: Vec<pest::iterators::Pair<Rule>> = inner.collect();

            if parts.is_empty() {
                return Err("Empty parallel expression".to_string());
            }

            // Separate binding from expressions
            let (expr_parts, binding) = if parts.last().map(|p| p.as_rule()) == Some(Rule::binding)
            {
                let b = parse_binding(parts.last().unwrap().clone())?;
                (&parts[..parts.len() - 1], Some(b))
            } else {
                (&parts[..], None)
            };

            // Parse all expressions
            let exprs: Result<Vec<_>, _> =
                expr_parts.iter().map(|p| build_expr(p.clone())).collect();

            let expr_list = exprs?;

            if expr_list.is_empty() {
                return Err("Empty parallel expression".to_string());
            }

            // If only one expression and no binding, just return the expression
            // This prevents wrapping simple expressions like string literals in Parallel nodes
            if expr_list.len() == 1 && binding.is_none() {
                return Ok(expr_list.into_iter().next().unwrap());
            }

            // Multiple expressions (or single expression with binding) - create Parallel node
            // Note: Even a single expression with a binding becomes a Parallel node
            // to ensure the binding is handled without re-evaluating the expression
            Ok(Expr::Parallel {
                exprs: expr_list,
                binding,
            })
        }
        Rule::additive => {
            let inner = pair.into_inner();
            let parts: Vec<pest::iterators::Pair<Rule>> = inner.collect();

            if parts.is_empty() {
                return Err("Empty additive expression".to_string());
            }

            if parts.len() == 1 {
                return build_expr(parts[0].clone());
            }

            // Build left-associative binary operations
            let mut result = build_expr(parts[0].clone())?;
            let mut i = 1;

            while i < parts.len() - 1 {
                let op = parts[i].as_str().to_string();
                let right = build_expr(parts[i + 1].clone())?;

                result = Expr::BinaryOp {
                    left: Box::new(result),
                    op,
                    right: Box::new(right),
                };

                i += 2;
            }

            Ok(result)
        }
        Rule::multiplicative => {
            let inner = pair.into_inner();
            let parts: Vec<pest::iterators::Pair<Rule>> = inner.collect();

            if parts.is_empty() {
                return Err("Empty multiplicative expression".to_string());
            }

            if parts.len() == 1 {
                return build_expr(parts[0].clone());
            }

            // Build left-associative binary operations
            let mut result = build_expr(parts[0].clone())?;
            let mut i = 1;

            while i < parts.len() - 1 {
                let op = parts[i].as_str().to_string();
                let right = build_expr(parts[i + 1].clone())?;

                result = Expr::BinaryOp {
                    left: Box::new(result),
                    op,
                    right: Box::new(right),
                };

                i += 2;
            }

            Ok(result)
        }
        Rule::primary => {
            let inner = pair.into_inner().next().ok_or("Empty primary")?;
            build_expr(inner)
        }
        Rule::function_call => {
            let mut inner = pair.into_inner();
            let name = inner
                .next()
                .ok_or("Missing function name")?
                .as_str()
                .to_string();

            let args_pair = inner.next().ok_or("Missing arguments")?;
            let mut args = Vec::new();

            for arg_pair in args_pair.into_inner() {
                args.push(build_expr(arg_pair)?);
            }

            Ok(Expr::FunctionCall { name, args })
        }
        Rule::access_chain => {
            let mut inner = pair.into_inner();
            let base_name = inner
                .next()
                .ok_or("Missing base variable")?
                .as_str()
                .to_string();
            let mut expr = Expr::Variable(base_name);

            for access in inner {
                match access.as_rule() {
                    Rule::field_access => {
                        let field = access.into_inner().next().ok_or("Missing field name")?;
                        let field_name = field.as_str().to_string();
                        expr = Expr::FieldAccess {
                            base: Box::new(expr),
                            field: field_name,
                        };
                    }
                    Rule::index_access => {
                        let index_expr = access.into_inner().next().ok_or("Missing index")?;
                        let index = build_expr(index_expr)?;
                        expr = Expr::IndexAccess {
                            base: Box::new(expr),
                            index: Box::new(index),
                        };
                    }
                    _ => {}
                }
            }

            Ok(expr)
        }
        Rule::paren_expr => {
            let inner = pair.into_inner().next().ok_or("Empty parentheses")?;
            build_expr(inner)
        }
        Rule::string_literal => {
            let inner = pair.into_inner().next().ok_or("Empty string")?;
            Ok(Expr::String(inner.as_str().to_string()))
        }
        Rule::template_string => {
            let segments = parse_template_string(pair)?;
            Ok(Expr::TemplateString(segments))
        }
        Rule::integer => {
            let value = pair
                .as_str()
                .parse::<i64>()
                .map_err(|e| format!("Invalid integer: {}", e))?;
            Ok(Expr::Int(value))
        }
        Rule::float => {
            let value = pair
                .as_str()
                .parse::<f64>()
                .map_err(|e| format!("Invalid float: {}", e))?;
            Ok(Expr::Float(value))
        }
        Rule::boolean => {
            let value = pair.as_str() == "true";
            Ok(Expr::Bool(value))
        }
        Rule::list_literal => {
            let mut items = Vec::new();
            for item_pair in pair.into_inner() {
                items.push(build_expr(item_pair)?);
            }
            Ok(Expr::List(items))
        }
        Rule::identifier => Ok(Expr::Variable(pair.as_str().to_string())),
        _ => Err(format!("Unexpected rule: {:?}", pair.as_rule())),
    }
}

/// Parse a type definition
pub fn parse_type_definition(input: &str) -> Result<Class, String> {
    let pairs =
        DslParser::parse(Rule::type_decl, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| "No type definition found".to_string())?;

    build_type_definition(pair)
}

fn build_type_definition(pair: pest::iterators::Pair<Rule>) -> Result<Class, String> {
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or("Missing type name")?
        .as_str()
        .to_string();

    let mut fields = Vec::new();

    for field_pair in inner {
        if field_pair.as_rule() == Rule::field {
            fields.push(build_field(field_pair)?);
        }
    }

    Ok(Class {
        name,
        description: None,
        fields,
    })
}

fn build_field(pair: pest::iterators::Pair<Rule>) -> Result<Field, String> {
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or("Missing field name")?
        .as_str()
        .to_string();

    // Check for optional marker
    let mut optional = false;
    let mut next = inner.next().ok_or("Missing field type")?;

    if next.as_rule() == Rule::optional {
        optional = true;
        next = inner
            .next()
            .ok_or("Missing field type after optional marker")?;
    }

    let field_type = build_field_type(next)?;

    // Check for description (optional)
    let description = inner.next().map(|p| {
        let desc_inner = p.into_inner().next().unwrap();
        desc_inner.as_str().to_string()
    });

    Ok(Field {
        name,
        field_type,
        optional,
        description,
    })
}

fn build_field_type(pair: pest::iterators::Pair<Rule>) -> Result<FieldType, String> {
    let inner = pair.into_inner().next().ok_or("Empty field type")?;

    match inner.as_rule() {
        Rule::primitive_type => match inner.as_str().to_lowercase().as_str() {
            "string" => Ok(FieldType::String),
            "int" => Ok(FieldType::Int),
            "float" => Ok(FieldType::Float),
            "bool" => Ok(FieldType::Bool),
            _ => Err(format!("Unknown primitive type: {}", inner.as_str())),
        },
        Rule::list_type => {
            let inner_type = inner.into_inner().next().ok_or("Missing list inner type")?;
            let element_type = build_field_type(inner_type)?;
            Ok(FieldType::List(Box::new(element_type)))
        }
        Rule::custom_type => {
            let type_name = inner.as_str().to_string();
            Ok(FieldType::Class(type_name))
        }
        _ => Err(format!("Unexpected field type rule: {:?}", inner.as_rule())),
    }
}

/// Parse an enum definition
pub fn parse_enum_definition(input: &str) -> Result<Enum, String> {
    let pairs =
        DslParser::parse(Rule::enum_decl, input).map_err(|e| format!("Parse error: {}", e))?;

    let pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| "No enum definition found".to_string())?;

    build_enum_definition(pair)
}

fn build_enum_definition(pair: pest::iterators::Pair<Rule>) -> Result<Enum, String> {
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or("Missing enum name")?
        .as_str()
        .to_string();

    let mut values = Vec::new();

    for variant_pair in inner {
        if variant_pair.as_rule() == Rule::enum_variant {
            let variant_name = variant_pair
                .into_inner()
                .next()
                .ok_or("Missing variant name")?
                .as_str()
                .to_string();
            values.push(variant_name);
        }
    }

    if values.is_empty() {
        return Err("Enum must have at least one value".to_string());
    }

    Ok(Enum {
        name,
        description: None,
        values,
    })
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

/// Extract a string property value from properties HashMap
fn extract_string_property(properties: &HashMap<String, PropertyValue>, key: &str) -> Option<String> {
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
fn parse_http_block(pair: pest::iterators::Pair<Rule>) -> Result<(String, String, Option<HashMap<String, String>>, Option<HashMap<String, String>>, Option<String>), String> {
    let mut inner = pair.into_inner();

    // First item should be the HTTP method in a string literal
    let method_pair = inner.next().ok_or("Missing HTTP method")?;
    let method = extract_string_content(method_pair)?;

    let mut url = String::new();
    let mut params = None;
    let mut headers = None;
    let mut body = None;

    // Parse remaining properties
    for item in inner {
        match item.as_rule() {
            Rule::string_literal => {
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

fn build_function_definition(pair: pest::iterators::Pair<Rule>) -> Result<FunctionDef, String> {
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
    let mut http_config: Option<(String, String, Option<HashMap<String, String>>, Option<HashMap<String, String>>, Option<String>)> = None;

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
                temperature: extract_float_property(&properties, "temperature"),
            }
        }
        (false, true, false) => {
            // Pure HTTP function
            let (method, url, params, headers, body) = http_config.unwrap();
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
            // Hybrid: HTTP then LLM
            let (method, url, params, headers, _body) = http_config.unwrap();
            FunctionExecution::HTTPWithLLM {
                http_method: method,
                http_url: url,
                http_params: params,
                http_headers: headers,
                llm_prompt: prompt_value.unwrap(),
                llm_model: extract_string_property(&properties, "model"),
                llm_temperature: extract_float_property(&properties, "temperature"),
            }
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

fn build_property(pair: pest::iterators::Pair<Rule>) -> Result<(String, PropertyValue), String> {
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

fn parse_template_string(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Vec<TemplateSegment>, String> {
    let inner = pair.into_inner().next().ok_or("Empty template string")?;

    let mut segments = Vec::new();

    for segment in inner.into_inner() {
        match segment.as_rule() {
            Rule::template_segment => {
                // Get the actual content (either template_text or template_expr)
                let content = segment.into_inner().next().ok_or("Empty segment")?;
                match content.as_rule() {
                    Rule::template_text => {
                        let text = content.as_str().to_string();
                        if !text.is_empty() {
                            segments.push(TemplateSegment::Text(text));
                        }
                    }
                    Rule::template_expr => {
                        let expr_str = content
                            .into_inner()
                            .next()
                            .ok_or("Empty template expr")?
                            .as_str()
                            .to_string();
                        segments.push(TemplateSegment::Interpolation(expr_str));
                    }
                    _ => {}
                }
            }
            Rule::template_text => {
                let text = segment.as_str().to_string();
                if !text.is_empty() {
                    segments.push(TemplateSegment::Text(text));
                }
            }
            Rule::template_expr => {
                let expr_str = segment
                    .into_inner()
                    .next()
                    .ok_or("Empty template expr")?
                    .as_str()
                    .to_string();
                segments.push(TemplateSegment::Interpolation(expr_str));
            }
            _ => {}
        }
    }

    Ok(segments)
}

fn extract_string_content(pair: pest::iterators::Pair<Rule>) -> Result<String, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_literal() {
        let result = parse_expr("42").unwrap();
        assert!(matches!(result, Expr::Int(42)));

        let result = parse_expr("3.14").unwrap();
        assert!(matches!(result, Expr::Float(_)));

        let result = parse_expr("true").unwrap();
        assert!(matches!(result, Expr::Bool(true)));

        let result = parse_expr(r#""hello""#).unwrap();
        assert!(matches!(result, Expr::String(_)));
    }

    #[test]
    fn test_parse_function_call() {
        let result = parse_expr("Length(\"test\")").unwrap();
        if let Expr::FunctionCall { name, args } = result {
            assert_eq!(name, "Length");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected function call");
        }
    }

    #[test]
    fn test_parse_access_chain() {
        let result = parse_expr("user.name").unwrap();
        if let Expr::FieldAccess { base, field } = result {
            assert!(matches!(*base, Expr::Variable(_)));
            assert_eq!(field, "name");
        } else {
            panic!("Expected field access");
        }
    }

    #[test]
    fn test_parse_type_definition() {
        let input = "type Person { name: string, age: int }";
        let result = parse_type_definition(input).unwrap();

        assert_eq!(result.name, "Person");
        assert_eq!(result.fields.len(), 2);
        assert_eq!(result.fields[0].name, "name");
        assert_eq!(result.fields[1].name, "age");
    }

    #[test]
    fn test_parse_enum_definition() {
        let input = "enum Status { Pending, InProgress, Completed }";
        let result = parse_enum_definition(input).unwrap();

        assert_eq!(result.name, "Status");
        assert_eq!(result.values.len(), 3);
        assert_eq!(result.values[0], "Pending");
    }

    #[test]
    fn test_parse_sequential() {
        // Test simple sequential
        let result = parse_expr("5 >> Length(_)").unwrap();
        assert!(matches!(result, Expr::Sequential { .. }));

        // Test chained sequential
        let result = parse_expr("1 >> _ * 2 >> _ + 3").unwrap();
        // Debug: print the actual structure
        println!("Parsed result: {:?}", result);

        if let Expr::Sequential { left, right, .. } = result {
            // The chain should be built left-to-right:
            // Sequential { left: Sequential { left: 1, right: _*2 }, right: _+3 }
            println!("Left: {:?}", left);
            println!("Right: {:?}", right);
            assert!(matches!(*left, Expr::Sequential { .. }));
            assert!(matches!(*right, Expr::BinaryOp { .. }));
        } else {
            panic!("Expected sequential expression, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_parallel() {
        // Test parallel with || operator
        let result = parse_expr("Ask(\"a\") || Ask(\"b\")").unwrap();
        if let Expr::Parallel { exprs, .. } = result {
            assert_eq!(exprs.len(), 2);
        } else {
            panic!("Expected parallel expression, got: {:?}", result);
        }

        // Test parallel with binding
        let result = parse_expr("Ask(\"a\") || Ask(\"b\") as results").unwrap();
        if let Expr::Parallel { exprs, binding } = result {
            assert_eq!(exprs.len(), 2);
            assert!(binding.is_some());
        } else {
            panic!(
                "Expected parallel expression with binding, got: {:?}",
                result
            );
        }
    }

    #[test]
    fn test_parse_conditional() {
        // Test ternary operator
        let result = parse_expr("true ? 1 : 2").unwrap();
        if let Expr::Conditional {
            condition,
            then_expr,
            else_expr,
        } = result
        {
            assert!(matches!(*condition, Expr::Bool(true)));
            assert!(matches!(*then_expr, Expr::Int(1)));
            assert!(matches!(*else_expr, Expr::Int(2)));
        } else {
            panic!("Expected conditional expression");
        }
    }

    #[test]
    fn test_parse_binding_with_sequential() {
        // Regression test for: [1, 2, 3] as numbers >> Length(numbers)
        // This previously failed with "Invalid variable name" error
        let result = parse_expr("[1, 2, 3] as numbers >> Length(numbers)").unwrap();

        // Should be: Sequential {
        //   left: Sequential { left: [1,2,3], right: [1,2,3], binding: Some("numbers") },
        //   right: Length(numbers),
        //   binding: None
        // }
        if let Expr::Sequential {
            left,
            right,
            binding,
        } = result
        {
            // Left should be another Sequential with the binding
            if let Expr::Sequential {
                binding: left_binding,
                ..
            } = *left
            {
                if let Some(Binding::Single(name)) = left_binding {
                    assert_eq!(name, "numbers");
                } else {
                    panic!("Expected binding 'numbers' on left");
                }
            } else {
                panic!("Expected left to be Sequential with binding");
            }

            // Right should be function call
            assert!(matches!(*right, Expr::FunctionCall { .. }));

            // No binding on the outer sequential
            assert!(binding.is_none());
        } else {
            panic!("Expected sequential expression, got: {:?}", result);
        }
    }
}
