use super::ast::*;
use super::Rule;
use pest::iterators::Pair;

/// Unescape a string literal
fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(escaped) = chars.next() {
                match escaped {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '\'' => result.push('\''),
                    '$' => result.push('$'),
                    'b' => result.push('\u{0008}'),
                    'f' => result.push('\u{000C}'),
                    '/' => result.push('/'),
                    'u' => {
                        // Unicode escape sequence \uXXXX
                        let hex: String = chars.by_ref().take(4).collect();
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(unicode_char) = char::from_u32(code) {
                                result.push(unicode_char);
                            }
                        }
                    }
                    _ => {
                        result.push('\\');
                        result.push(escaped);
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Parse a binding pattern
pub(super) fn parse_binding(pair: Pair<Rule>) -> Result<Binding, String> {
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

/// Build pattern from Pest pair
pub(super) fn build_pattern(pair: Pair<Rule>) -> Result<Pattern, String> {
    match pair.as_rule() {
        Rule::pattern => {
            // Descend to the actual pattern type
            let inner = pair.into_inner().next().ok_or("Empty pattern")?;
            build_pattern(inner)
        }
        Rule::pattern_wildcard => Ok(Pattern::Any),
        Rule::pattern_literal => {
            let inner = pair.into_inner().next().ok_or("Empty literal pattern")?;
            let expr = build_expr(inner)?;
            Ok(Pattern::Literal(Box::new(expr)))
        }
        Rule::pattern_variable => {
            // Extract just the identifier (not the lookahead)
            let id_pair = pair
                .into_inner()
                .next()
                .ok_or("Missing identifier in pattern_variable")?;
            let var_name = id_pair.as_str().to_string();
            Ok(Pattern::Variable(var_name))
        }
        Rule::pattern_binding => {
            let mut inner = pair.into_inner();
            let var_name = inner
                .next()
                .ok_or("Missing variable in binding pattern")?
                .as_str()
                .to_string();
            let nested_pattern = inner.next().ok_or("Missing nested pattern")?;
            Ok(Pattern::Binding(
                var_name,
                Box::new(build_pattern(nested_pattern)?),
            ))
        }
        Rule::pattern_type => {
            let mut inner = pair.into_inner();
            let type_name = inner
                .next()
                .ok_or("Missing type name")?
                .as_str()
                .to_string();
            let inner_pattern = inner.next().map(|p| build_pattern(p)).transpose()?;
            Ok(Pattern::Type {
                type_name,
                inner: inner_pattern.map(Box::new),
            })
        }
        Rule::pattern_list => {
            // Get the string first before consuming pair
            let list_str = pair.as_str().to_string();

            let inner = pair.into_inner();
            let parts: Vec<Pair<Rule>> = inner.collect();

            if parts.is_empty() {
                // Empty list pattern []
                return Ok(Pattern::List {
                    patterns: vec![],
                    rest: None,
                });
            }

            // First, detect if there's a rest pattern in the original string
            let rest = if let Some(rest_start) = list_str.find("...") {
                // Extract the identifier after "..."
                let after_dots = &list_str[rest_start + 3..];
                let rest_name = if let Some(end) =
                    after_dots.find(|c: char| !c.is_alphanumeric() && c != '_')
                {
                    after_dots[..end].trim().to_string()
                } else {
                    after_dots.trim_end_matches(']').trim().to_string()
                };

                if !rest_name.is_empty() {
                    Some(rest_name)
                } else {
                    None
                }
            } else {
                None
            };

            let mut patterns = Vec::new();

            // Parse patterns, skipping bare identifiers that match the rest variable
            for part in parts {
                if part.as_rule() == Rule::identifier {
                    // This is a bare identifier from the grammar (after ...)
                    // Skip it if it matches the rest variable
                    if let Some(rest_name) = &rest {
                        if part.as_str() == rest_name {
                            continue;
                        }
                    }
                    // Otherwise, treat it as a variable pattern
                    patterns.push(Pattern::Variable(part.as_str().to_string()));
                } else {
                    patterns.push(build_pattern(part)?);
                }
            }

            Ok(Pattern::List { patterns, rest })
        }
        Rule::pattern_map => {
            let inner = pair.into_inner();
            let mut fields = Vec::new();

            for field_pair in inner {
                if field_pair.as_rule() == Rule::pattern_map_field {
                    let mut field_inner = field_pair.into_inner();
                    let field_name = field_inner
                        .next()
                        .ok_or("Missing field name")?
                        .as_str()
                        .to_string();

                    // Check if there's a pattern after the field name
                    let field_pattern = if let Some(p) = field_inner.next() {
                        build_pattern(p)?
                    } else {
                        // Shorthand: {x} means {x: x}
                        Pattern::Variable(field_name.clone())
                    };

                    fields.push((field_name, field_pattern));
                }
            }

            Ok(Pattern::Map {
                fields,
                strict: false,
            })
        }
        Rule::pattern_tuple => {
            let mut patterns = Vec::new();
            for inner_pattern in pair.into_inner() {
                patterns.push(build_pattern(inner_pattern)?);
            }
            Ok(Pattern::Tuple(patterns))
        }
        _ => Err(format!("Unexpected pattern rule: {:?}", pair.as_rule())),
    }
}

/// Build expression AST from Pest pair
pub(super) fn build_expr(pair: Pair<Rule>) -> Result<Expr, String> {
    match pair.as_rule() {
        Rule::expr => {
            // Descend to conditional (let_binding is no longer part of expr)
            let inner = pair.into_inner().next().ok_or("Empty expression")?;
            build_expr(inner)
        }
        // Note: let_binding is now handled as let_statement at the program/block level
        // For backward compatibility in REPL, we might need to handle it differently
        Rule::let_statement => {
            // Parse: let x = expr or let pattern = expr
            let mut inner = pair.into_inner();

            // First element is the binding pattern (identifier or pattern)
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
                Rule::pattern => {
                    // For patterns, we need to extract the binding
                    // For now, just treat as identifier if it's a variable pattern
                    return Err(
                        "Complex patterns in let statements not yet fully supported".to_string()
                    );
                }
                _ => {
                    return Err(format!(
                        "Unexpected binding pattern: {:?}",
                        binding_pair.as_rule()
                    ))
                }
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
            let parts: Vec<Pair<Rule>> = inner.collect();

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
            let parts: Vec<Pair<Rule>> = inner.collect();

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
            let parts: Vec<Pair<Rule>> = inner.collect();

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
            let parts: Vec<Pair<Rule>> = inner.collect();

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
            let parts: Vec<Pair<Rule>> = inner.collect();

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
            let parts: Vec<Pair<Rule>> = inner.collect();

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
            let parts: Vec<Pair<Rule>> = inner.collect();

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
            // String is now atomic (@), so we need to extract content from quotes
            let full_str = pair.as_str();
            if full_str.len() < 2 {
                return Err("Empty string".to_string());
            }

            // Remove surrounding quotes
            let content = &full_str[1..full_str.len() - 1];

            // Unescape the string content
            let unescaped = unescape_string(content);
            Ok(Expr::String(unescaped))
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
                            // Extract the string content without quotes (string is now atomic)
                            let full_str = key_pair.as_str();
                            if full_str.len() < 2 {
                                return Err("Empty string key".to_string());
                            }
                            let content = &full_str[1..full_str.len() - 1];
                            unescape_string(content)
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
        Rule::inline_block => {
            // An inline block in expression context is treated as an empty map
            // This can happen when {} is parsed as inline_block rather than map_literal
            // due to grammar ambiguity
            Ok(Expr::Map(Vec::new()))
        }
        Rule::match_expr => {
            let mut inner = pair.into_inner();

            // First element is the scrutinee expression
            let scrutinee = inner.next().ok_or("Missing scrutinee in match")?;
            let scrutinee_expr = Box::new(build_expr(scrutinee)?);

            // Remaining elements are match cases
            let mut cases = Vec::new();
            for case_pair in inner {
                if case_pair.as_rule() == Rule::match_case {
                    let mut case_inner = case_pair.into_inner();

                    // First is the pattern
                    let pattern =
                        build_pattern(case_inner.next().ok_or("Missing pattern in case")?)?;

                    // Check for optional guard (if condition)
                    let mut guard = None;
                    let mut body_pair = case_inner.next().ok_or("Missing body in case")?;

                    // If we have more than one element left, the first is the guard
                    if case_inner.peek().is_some() {
                        guard = Some(build_expr(body_pair)?);
                        body_pair = case_inner.next().ok_or("Missing body after guard")?;
                    } else if body_pair.as_str().starts_with("if ")
                        || body_pair.as_rule() != Rule::expr
                    {
                        // This might be a guard, check the next element
                        if let Some(next) = case_inner.next() {
                            guard = Some(build_expr(body_pair)?);
                            body_pair = next;
                        }
                    }

                    // Build the body
                    let body = build_expr(body_pair)?;

                    cases.push(MatchCase {
                        pattern,
                        guard,
                        body,
                    });
                }
            }

            if cases.is_empty() {
                return Err("Match expression must have at least one case".to_string());
            }

            Ok(Expr::Match {
                scrutinee: scrutinee_expr,
                cases,
            })
        }
        Rule::identifier => Ok(Expr::Variable(pair.as_str().to_string())),
        _ => Err(format!("Unexpected rule: {:?}", pair.as_rule())),
    }
}

/// Parse template string segments
pub(super) fn parse_template_string(pair: Pair<Rule>) -> Result<Vec<TemplateSegment>, String> {
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
