use pest::Parser;
use pest_derive::Parser;
use simplify_baml::{Class, Enum, Field, FieldType};
use std::collections::HashMap;

/// Type alias for HTTP configuration: (method, url, params, headers, body)
type HttpConfig = (
    String,
    String,
    Option<HashMap<String, String>>,
    Option<HashMap<String, String>>,
    Option<String>,
);

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
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
    /// Type instantiation: TypeName { field: value, ... }
    TypeInstantiation {
        type_name: String,
        fields: Vec<(String, Expr)>,
    },
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
    /// Sequential composition: left |> right
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
        base_url: Option<String>,
        api_key_env: Option<String>,
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
    SQL { query: String },
    /// Hybrid: HTTP then LLM processing
    HTTPWithLLM {
        http_method: String,
        http_url: String,
        http_params: Option<HashMap<String, String>>,
        http_headers: Option<HashMap<String, String>>,
        llm_prompt: String,
        llm_model: Option<String>,
        llm_base_url: Option<String>,
        llm_api_key_env: Option<String>,
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

    // Extract binding from top-level Sequential or Parallel expression
    let binding = match &expr {
        Expr::Sequential { binding, .. } => binding.clone(),
        Expr::Parallel { binding, .. } => binding.clone(),
        _ => None,
    };

    Ok((expr, binding))
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
            // Descend to let_binding or conditional
            let inner = pair.into_inner().next().ok_or("Empty expression")?;
            build_expr(inner)
        }
        Rule::let_binding => {
            // Parse: let x = expr or let [a, b] = expr
            let mut inner = pair.into_inner();

            // First element is the binding pattern (identifier or list_binding)
            let binding_pair = inner.next().ok_or("Missing binding pattern in let")?;
            let binding = match binding_pair.as_rule() {
                Rule::identifier => Binding::Single(binding_pair.as_str().to_string()),
                Rule::list_binding => {
                    let vars: Vec<String> = binding_pair
                        .into_inner()
                        .map(|p| p.as_str().to_string())
                        .collect();
                    Binding::List(vars)
                }
                _ => return Err(format!("Unexpected binding pattern: {:?}", binding_pair.as_rule())),
            };

            // Second element is the expression
            let expr_pair = inner.next().ok_or("Missing expression in let")?;
            let expr = build_expr(expr_pair)?;

            // Wrap in a Parallel node with a single expression to handle the binding
            // This is the same representation as "expr as x"
            Ok(Expr::Parallel {
                exprs: vec![expr],
                binding: Some(binding),
            })
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

            // Grammar: logical_or ~ (binding)? ~ ("|>" ~ logical_or ~ (binding)?)*
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

            // If no |> operators, return the expression (possibly with binding)
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

            // Process remaining "|>" chains
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
        Rule::logical_or => {
            let inner = pair.into_inner();
            let parts: Vec<pest::iterators::Pair<Rule>> = inner.collect();

            if parts.is_empty() {
                return Err("Empty logical_or expression".to_string());
            }

            if parts.len() == 1 {
                return build_expr(parts[0].clone());
            }

            // Build left-associative binary operations
            let mut result = build_expr(parts[0].clone())?;
            let mut i = 1;

            while i < parts.len() {
                let right = build_expr(parts[i].clone())?;

                result = Expr::BinaryOp {
                    left: Box::new(result),
                    op: "||".to_string(),
                    right: Box::new(right),
                };

                i += 1;
            }

            Ok(result)
        }
        Rule::logical_and => {
            let inner = pair.into_inner();
            let parts: Vec<pest::iterators::Pair<Rule>> = inner.collect();

            if parts.is_empty() {
                return Err("Empty logical_and expression".to_string());
            }

            if parts.len() == 1 {
                return build_expr(parts[0].clone());
            }

            // Build left-associative binary operations
            let mut result = build_expr(parts[0].clone())?;
            let mut i = 1;

            while i < parts.len() {
                let right = build_expr(parts[i].clone())?;

                result = Expr::BinaryOp {
                    left: Box::new(result),
                    op: "&&".to_string(),
                    right: Box::new(right),
                };

                i += 1;
            }

            Ok(result)
        }
        Rule::comparison => {
            let inner = pair.into_inner();
            let parts: Vec<pest::iterators::Pair<Rule>> = inner.collect();

            if parts.is_empty() {
                return Err("Empty comparison expression".to_string());
            }

            if parts.len() == 1 {
                return build_expr(parts[0].clone());
            }

            // Should have exactly 3 parts: left, op, right
            if parts.len() == 3 {
                let left = build_expr(parts[0].clone())?;
                let op = parts[1].as_str().to_string();
                let right = build_expr(parts[2].clone())?;

                Ok(Expr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                })
            } else {
                Err("Invalid comparison expression".to_string())
            }
        }
        Rule::comparison_op => {
            // This shouldn't be called directly
            Err("Comparison operator should not be built directly".to_string())
        }
        Rule::unary => {
            let inner = pair.into_inner();
            let parts: Vec<pest::iterators::Pair<Rule>> = inner.collect();

            if parts.is_empty() {
                return Err("Empty unary expression".to_string());
            }

            if parts.len() == 1 {
                return build_expr(parts[0].clone());
            }

            // Unary operator: op ~ expr
            if parts.len() == 2 {
                let op = parts[0].as_str().to_string();
                let operand = build_expr(parts[1].clone())?;

                // Convert unary minus to BinaryOp: 0 - operand
                // Convert unary ! to separate UnaryOp
                if op == "-" {
                    Ok(Expr::BinaryOp {
                        left: Box::new(Expr::Int(0)),
                        op: "-".to_string(),
                        right: Box::new(operand),
                    })
                } else if op == "!" {
                    // We'll represent ! as a special unary operation
                    // For now, use a function call-like structure
                    Ok(Expr::FunctionCall {
                        name: "not".to_string(),
                        args: vec![operand],
                    })
                } else {
                    Err(format!("Unknown unary operator: {}", op))
                }
            } else {
                Err("Invalid unary expression".to_string())
            }
        }
        Rule::unary_op => {
            // This shouldn't be called directly
            Err("Unary operator should not be built directly".to_string())
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
        Rule::type_instantiation => {
            let mut inner = pair.into_inner();

            // First element is the type name
            let type_name = inner
                .next()
                .ok_or("Missing type name")?
                .as_str()
                .to_string();

            // Remaining elements are field assignments
            let mut fields = Vec::new();

            for field_pair in inner {
                if field_pair.as_rule() == Rule::type_field_assignment {
                    let mut field_inner = field_pair.into_inner();
                    let field_name = field_inner
                        .next()
                        .ok_or("Missing field name")?
                        .as_str()
                        .to_string();
                    let field_value = field_inner.next().ok_or("Missing field value")?;
                    let value_expr = build_expr(field_value)?;
                    fields.push((field_name, value_expr));
                }
            }

            Ok(Expr::TypeInstantiation { type_name, fields })
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
        Rule::map_literal => {
            let mut entries = Vec::new();
            for entry_pair in pair.into_inner() {
                if entry_pair.as_rule() == Rule::map_entry {
                    let mut entry_inner = entry_pair.into_inner();

                    // Get the key (either identifier or string_literal)
                    let key_pair = entry_inner.next().ok_or("Missing map key")?;
                    let key = match key_pair.as_rule() {
                        Rule::identifier => key_pair.as_str().to_string(),
                        Rule::string_literal => {
                            // Extract the string content without quotes
                            let inner = key_pair.into_inner().next().ok_or("Empty string key")?;
                            inner.as_str().to_string()
                        }
                        _ => return Err(format!("Invalid map key type: {:?}", key_pair.as_rule())),
                    };

                    // Get the value
                    let value_pair = entry_inner.next().ok_or("Missing map value")?;
                    let value = build_expr(value_pair)?;

                    entries.push((key, value));
                }
            }
            Ok(Expr::Map(entries))
        }
        Rule::block => {
            // A block in expression context is treated as an empty map
            // This can happen when {} is parsed as a block rather than map_literal
            // due to grammar ambiguity
            Ok(Expr::Map(Vec::new()))
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
fn parse_http_block(pair: pest::iterators::Pair<Rule>) -> Result<HttpConfig, String> {
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
            // Hybrid: HTTP then LLM
            let (method, mut url, params, headers, _body) = http_config.unwrap();

            // Check if URL is in properties (since it's parsed separately as a property)
            if url.is_empty() {
                url = extract_string_property(&properties, "url").unwrap_or_default();
            }

            FunctionExecution::HTTPWithLLM {
                http_method: method,
                http_url: url,
                http_params: params,
                http_headers: headers,
                llm_prompt: prompt_value.unwrap(),
                llm_model: extract_string_property(&properties, "model"),
                llm_base_url: extract_string_property(&properties, "base_url"),
                llm_api_key_env: extract_string_property(&properties, "api_key_env"),
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
        // Strings can be parsed as TemplateString (with no interpolation) or String
        assert!(matches!(result, Expr::String(_) | Expr::TemplateString(_)));
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
        let result = parse_expr("5 |> Length(_)").unwrap();
        assert!(matches!(result, Expr::Sequential { .. }));

        // Test chained sequential
        let result = parse_expr("1 |> _ * 2 |> _ + 3").unwrap();
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
        // Test parallel with par() function
        let result = parse_expr("par(Ask(\"a\"), Ask(\"b\"))").unwrap();
        if let Expr::FunctionCall { name, args } = result {
            assert_eq!(name, "par");
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected function call to par, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_comparison_operators() {
        // Test equality
        let result = parse_expr("5 == 5").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, "==");
        } else {
            panic!("Expected comparison expression");
        }

        // Test inequality
        let result = parse_expr("5 != 3").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, "!=");
        } else {
            panic!("Expected comparison expression");
        }

        // Test less than
        let result = parse_expr("3 < 5").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, "<");
        } else {
            panic!("Expected comparison expression");
        }

        // Test greater than or equal
        let result = parse_expr("5 >= 3").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, ">=");
        } else {
            panic!("Expected comparison expression");
        }
    }

    #[test]
    fn test_parse_logical_operators() {
        // Test AND
        let result = parse_expr("true && false").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, "&&");
        } else {
            panic!("Expected logical AND expression");
        }

        // Test OR
        let result = parse_expr("true || false").unwrap();
        if let Expr::BinaryOp { op, .. } = result {
            assert_eq!(op, "||");
        } else {
            panic!("Expected logical OR expression");
        }

        // Test NOT
        let result = parse_expr("!true").unwrap();
        if let Expr::FunctionCall { name, args } = result {
            assert_eq!(name, "not");
            assert_eq!(args.len(), 1);
        } else {
            panic!("Expected NOT expression (as function call)");
        }
    }

    #[test]
    fn test_parse_complex_conditional() {
        // Test conditional with comparison
        let result = parse_expr("age >= 18 ? \"adult\" : \"minor\"").unwrap();
        if let Expr::Conditional { condition, .. } = result {
            // Condition should be a comparison
            if let Expr::BinaryOp { op, .. } = *condition {
                assert_eq!(op, ">=");
            } else {
                panic!("Expected comparison in condition");
            }
        } else {
            panic!("Expected conditional expression");
        }

        // Test conditional with logical operators
        let result = parse_expr("age >= 18 && verified ? \"proceed\" : \"reject\"").unwrap();
        if let Expr::Conditional { condition, .. } = result {
            // Condition should be a logical AND
            if let Expr::BinaryOp { op, .. } = *condition {
                assert_eq!(op, "&&");
            } else {
                panic!("Expected logical AND in condition");
            }
        } else {
            panic!("Expected conditional expression");
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
        // Regression test for: [1, 2, 3] as numbers |> Length(numbers)
        // This previously failed with "Invalid variable name" error
        let result = parse_expr("[1, 2, 3] as numbers |> Length(numbers)").unwrap();

        // Should be: Sequential {
        //   left: Parallel { exprs: [[1,2,3]], binding: Some("numbers") },
        //   right: Length(numbers),
        //   binding: None
        // }
        if let Expr::Sequential {
            left,
            right,
            binding,
        } = result
        {
            // Left should be Parallel (single expr with binding) or Sequential
            match *left {
                Expr::Parallel {
                    binding: left_binding,
                    ..
                } => {
                    if let Some(Binding::Single(name)) = left_binding {
                        assert_eq!(name, "numbers");
                    } else {
                        panic!("Expected binding 'numbers' on left");
                    }
                }
                Expr::Sequential {
                    binding: left_binding,
                    ..
                } => {
                    if let Some(Binding::Single(name)) = left_binding {
                        assert_eq!(name, "numbers");
                    } else {
                        panic!("Expected binding 'numbers' on left Sequential");
                    }
                }
                _ => panic!(
                    "Expected left to be Parallel or Sequential with binding, got: {:?}",
                    left
                ),
            }

            // Right should be function call
            assert!(matches!(*right, Expr::FunctionCall { .. }));

            // No binding on the outer sequential
            assert!(binding.is_none());
        } else {
            panic!("Expected sequential expression, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_type_instantiation() {
        // Test basic type instantiation
        let result = parse_expr(r#"Person { name: "Alice", age: 30 }"#).unwrap();
        if let Expr::TypeInstantiation { type_name, fields } = result {
            assert_eq!(type_name, "Person");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "name");
            assert_eq!(fields[1].0, "age");
            // Strings are parsed as TemplateString, not String
            assert!(matches!(
                fields[0].1,
                Expr::String(_) | Expr::TemplateString(_)
            ));
            assert!(matches!(fields[1].1, Expr::Int(30)));
        } else {
            panic!("Expected type instantiation, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_type_instantiation_with_optional_fields() {
        // Test type instantiation with optional fields
        let result =
            parse_expr(r#"Person { name: "Bob", age: 25, email: "bob@example.com" }"#).unwrap();
        if let Expr::TypeInstantiation { type_name, fields } = result {
            assert_eq!(type_name, "Person");
            assert_eq!(fields.len(), 3);
            assert_eq!(fields[0].0, "name");
            assert_eq!(fields[1].0, "age");
            assert_eq!(fields[2].0, "email");
        } else {
            panic!("Expected type instantiation, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_type_instantiation_with_expressions() {
        // Test type instantiation with complex expressions
        let result = parse_expr(r#"Point { x: 1 + 2, y: 3 * 4 }"#).unwrap();
        if let Expr::TypeInstantiation { type_name, fields } = result {
            assert_eq!(type_name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "x");
            assert_eq!(fields[1].0, "y");
            assert!(matches!(fields[0].1, Expr::BinaryOp { .. }));
            assert!(matches!(fields[1].1, Expr::BinaryOp { .. }));
        } else {
            panic!("Expected type instantiation, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_type_instantiation_nested() {
        // Test nested type instantiation
        let result =
            parse_expr(r#"Employee { person: Person { name: "Alice", age: 30 }, id: 123 }"#)
                .unwrap();
        if let Expr::TypeInstantiation { type_name, fields } = result {
            assert_eq!(type_name, "Employee");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "person");
            assert_eq!(fields[1].0, "id");

            // Check that the first field is itself a type instantiation
            assert!(matches!(fields[0].1, Expr::TypeInstantiation { .. }));
        } else {
            panic!("Expected type instantiation, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_empty_type_instantiation() {
        // Test empty type instantiation
        let result = parse_expr("Empty {}").unwrap();
        if let Expr::TypeInstantiation { type_name, fields } = result {
            assert_eq!(type_name, "Empty");
            assert_eq!(fields.len(), 0);
        } else {
            panic!("Expected type instantiation, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_type_instantiation_syntax_error() {
        // Test syntax error - missing comma between fields
        let result = parse_expr(r#"Person { name: "Alice", age: 30 email: "ada@gmail.com"}"#);
        println!("Parse result: {:?}", result);

        // Let's see what it actually parses to
        if let Ok(expr) = result {
            println!("Parsed as: {:?}", expr);
            // It probably parses as: Person { name: "Alice", age: 30 }
            // with "email: ..." left unparsed or treated as something else
        }
    }

    #[test]
    fn test_parse_map_literal() {
        // Test empty map
        let result = parse_expr("{}").unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 0);
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }

        // Test simple map with identifier keys
        let result = parse_expr(r#"{ name: "Alice", age: 30 }"#).unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "name");
            assert_eq!(entries[1].0, "age");
            assert!(matches!(
                entries[0].1,
                Expr::String(_) | Expr::TemplateString(_)
            ));
            assert!(matches!(entries[1].1, Expr::Int(30)));
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }

        // Test map with string literal keys
        let result = parse_expr(
            r#"{ "Authorization": "Bearer token123", "Content-Type": "application/json" }"#,
        )
        .unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "Authorization");
            assert_eq!(entries[1].0, "Content-Type");
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }

        // Test nested map
        let result =
            parse_expr(r#"{ config: { timeout: 30, retries: 3 }, enabled: true }"#).unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "config");
            assert_eq!(entries[1].0, "enabled");
            assert!(matches!(entries[0].1, Expr::Map(_)));
            assert!(matches!(entries[1].1, Expr::Bool(true)));
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }

        // Test map with mixed value types
        let result =
            parse_expr(r#"{ str: "hello", num: 42, float: 3.14, bool: true, list: [1, 2, 3] }"#)
                .unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 5);
            assert_eq!(entries[0].0, "str");
            assert_eq!(entries[1].0, "num");
            assert_eq!(entries[2].0, "float");
            assert_eq!(entries[3].0, "bool");
            assert_eq!(entries[4].0, "list");
            assert!(matches!(entries[4].1, Expr::List(_)));
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }

        // Test map with trailing comma
        let result = parse_expr(r#"{ name: "Bob", age: 25, }"#).unwrap();
        if let Expr::Map(entries) = result {
            assert_eq!(entries.len(), 2);
        } else {
            panic!("Expected map literal, got: {:?}", result);
        }
    }

    #[test]
    fn test_parse_let_binding() {
        // Test basic let binding: let x = 5
        let result = parse_expr("let x = 5").unwrap();
        if let Expr::Parallel { exprs, binding } = result {
            assert_eq!(exprs.len(), 1);
            assert!(matches!(exprs[0], Expr::Int(5)));
            assert!(matches!(binding, Some(Binding::Single(ref name)) if name == "x"));
        } else {
            panic!("Expected Parallel with binding, got: {:?}", result);
        }

        // Test let binding with expression
        let result = parse_expr("let result = 1 + 2").unwrap();
        if let Expr::Parallel { exprs, binding } = result {
            assert_eq!(exprs.len(), 1);
            assert!(matches!(exprs[0], Expr::BinaryOp { .. }));
            assert!(matches!(binding, Some(Binding::Single(ref name)) if name == "result"));
        } else {
            panic!("Expected Parallel with binding, got: {:?}", result);
        }

        // Test let binding with list destructuring
        let result = parse_expr("let [a, b, c] = [1, 2, 3]").unwrap();
        if let Expr::Parallel { exprs, binding } = result {
            assert_eq!(exprs.len(), 1);
            assert!(matches!(exprs[0], Expr::List(_)));
            if let Some(Binding::List(vars)) = binding {
                assert_eq!(vars.len(), 3);
                assert_eq!(vars[0], "a");
                assert_eq!(vars[1], "b");
                assert_eq!(vars[2], "c");
            } else {
                panic!("Expected list binding");
            }
        } else {
            panic!("Expected Parallel with binding, got: {:?}", result);
        }
    }

    #[test]
    fn test_let_vs_as_equivalence() {
        // Test that "let x = 5" and "5 as x" produce the same IR structure
        let let_result = parse_expr("let x = 5").unwrap();
        let as_result = parse_expr("5 as x").unwrap();

        // Both should be Parallel with single expression and binding
        match (let_result, as_result) {
            (
                Expr::Parallel {
                    exprs: let_exprs,
                    binding: let_binding,
                },
                Expr::Parallel {
                    exprs: as_exprs,
                    binding: as_binding,
                },
            ) => {
                assert_eq!(let_exprs.len(), as_exprs.len());
                assert!(matches!(let_exprs[0], Expr::Int(5)));
                assert!(matches!(as_exprs[0], Expr::Int(5)));
                assert!(matches!(let_binding, Some(Binding::Single(ref name)) if name == "x"));
                assert!(matches!(as_binding, Some(Binding::Single(ref name)) if name == "x"));
            }
            _ => panic!("Expected both to be Parallel expressions with bindings"),
        }

        // Test list destructuring equivalence
        let let_result = parse_expr("let [a, b] = [1, 2]").unwrap();
        let as_result = parse_expr("[1, 2] as [a, b]").unwrap();

        match (let_result, as_result) {
            (
                Expr::Parallel {
                    binding: let_binding,
                    ..
                },
                Expr::Parallel {
                    binding: as_binding,
                    ..
                },
            ) => {
                if let (Some(Binding::List(let_vars)), Some(Binding::List(as_vars))) =
                    (let_binding, as_binding)
                {
                    assert_eq!(let_vars, as_vars);
                } else {
                    panic!("Expected both to have list bindings");
                }
            }
            _ => panic!("Expected both to be Parallel expressions"),
        }
    }
}
