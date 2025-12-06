//! Parser for Lattice
//!
//! Converts pest parse tree into AST nodes.

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::error::{LatticeError, Result};

use super::ast::*;

#[derive(Parser)]
#[grammar = "syntax/grammar.pest"]
pub struct LatticeParser;

/// Parse a Lattice program from source code
pub fn parse(source: &str) -> Result<Program> {
    let pairs = LatticeParser::parse(Rule::program, source)
        .map_err(|e| LatticeError::Parse(e.to_string()))?;

    let mut items = Vec::new();
    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                if let Some(item) = parse_item(inner)? {
                    items.push(item);
                }
            }
        }
    }

    Ok(Program { items })
}

/// Parse a single expression (useful for REPL)
pub fn parse_expression(source: &str) -> Result<Expr> {
    let pairs = LatticeParser::parse(Rule::expression, source)
        .map_err(|e| LatticeError::Parse(e.to_string()))?;

    let pair = pairs.into_iter().next().ok_or_else(|| {
        LatticeError::Parse("Expected expression".to_string())
    })?;

    parse_expr(pair)
}

// ============================================================================
// Item Parsing
// ============================================================================

fn parse_item(pair: Pair<Rule>) -> Result<Option<Item>> {
    match pair.as_rule() {
        Rule::item => {
            let inner = pair.into_inner().next().unwrap();
            parse_item(inner)
        }
        Rule::type_def => Ok(Some(Item::TypeDef(parse_type_def(pair)?))),
        Rule::enum_def => Ok(Some(Item::EnumDef(parse_enum_def(pair)?))),
        Rule::llm_config_decl => Ok(Some(Item::LlmConfigDecl(parse_llm_config_decl(pair)?))),
        Rule::function_def => Ok(Some(Item::FunctionDef(parse_function_def(pair)?))),
        Rule::statement => Ok(Some(Item::Statement(parse_statement(pair)?))),
        Rule::EOI => Ok(None),
        _ => Ok(None),
    }
}

// ============================================================================
// Type Definition Parsing
// ============================================================================

fn parse_type_def(pair: Pair<Rule>) -> Result<TypeDef> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let name_pair = inner.next().unwrap();
    let name = Spanned::new(name_pair.as_str().to_string(), make_span(&name_pair));

    let mut fields = Vec::new();
    for p in inner {
        if p.as_rule() == Rule::field_list {
            for field_pair in p.into_inner() {
                if field_pair.as_rule() == Rule::field {
                    fields.push(parse_field_def(field_pair)?);
                }
            }
        }
    }

    Ok(TypeDef { name, fields, span })
}

fn parse_field_def(pair: Pair<Rule>) -> Result<FieldDef> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let name_pair = inner.next().unwrap();
    let name = Spanned::new(name_pair.as_str().to_string(), make_span(&name_pair));

    let ty_pair = inner.next().unwrap();
    let ty = parse_type_annotation(ty_pair)?;

    let description = inner.next().and_then(|p| {
        if p.as_rule() == Rule::field_description {
            p.into_inner()
                .next()
                .map(|s| parse_string_content(s.as_str()))
        } else {
            None
        }
    });

    Ok(FieldDef {
        name,
        ty,
        description,
        span,
    })
}

fn parse_enum_def(pair: Pair<Rule>) -> Result<EnumDef> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let name_pair = inner.next().unwrap();
    let name = Spanned::new(name_pair.as_str().to_string(), make_span(&name_pair));

    let mut variants = Vec::new();
    for p in inner {
        if p.as_rule() == Rule::variant_list {
            for variant_pair in p.into_inner() {
                if variant_pair.as_rule() == Rule::identifier {
                    variants.push(Spanned::new(
                        variant_pair.as_str().to_string(),
                        make_span(&variant_pair),
                    ));
                }
            }
        }
    }

    Ok(EnumDef {
        name,
        variants,
        span,
    })
}

// ============================================================================
// LLM Config Declaration Parsing
// ============================================================================

fn parse_llm_config_decl(pair: Pair<Rule>) -> Result<LlmConfigDecl> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let name_pair = inner.next().unwrap();
    let name = Spanned::new(name_pair.as_str().to_string(), make_span(&name_pair));

    let mut config = LlmConfigDecl {
        name,
        base_url: None,
        model: None,
        api_key_env: None,
        temperature: None,
        max_tokens: None,
        provider: None,
        span,
    };

    for p in inner {
        if p.as_rule() == Rule::config_field {
            let field_inner = p.into_inner().next().unwrap();
            match field_inner.as_rule() {
                Rule::simple_config_field => {
                    let mut simple_inner = field_inner.into_inner();
                    let key = simple_inner.next().unwrap().as_str();
                    let value_pair = simple_inner.next().unwrap();
                    let value_str = extract_string_value(&value_pair)?;

                    match key {
                        "base_url" => config.base_url = Some(value_str),
                        "model" => config.model = Some(value_str),
                        "api_key_env" => config.api_key_env = Some(value_str),
                        "temperature" => config.temperature = value_str.parse().ok(),
                        "max_tokens" => config.max_tokens = value_str.parse().ok(),
                        _ => {}
                    }
                }
                Rule::provider_field => {
                    config.provider = Some(parse_provider_object(field_inner)?);
                }
                _ => {}
            }
        }
    }

    Ok(config)
}

fn parse_provider_object(pair: Pair<Rule>) -> Result<ProviderConfig> {
    let mut provider = ProviderConfig::default();

    // Navigate to provider_object
    let provider_obj = pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::provider_object)
        .unwrap();

    for field in provider_obj.into_inner() {
        if field.as_rule() == Rule::provider_config_field {
            let mut inner = field.into_inner();
            let key = inner.next().unwrap().as_str();
            let value_pair = inner.next().unwrap();

            match key {
                "order" => provider.order = Some(extract_string_list(&value_pair)?),
                "only" => provider.only = Some(extract_string_list(&value_pair)?),
                "ignore" => provider.ignore = Some(extract_string_list(&value_pair)?),
                "quantizations" => provider.quantizations = Some(extract_string_list(&value_pair)?),
                "allow_fallbacks" => provider.allow_fallbacks = extract_bool_value(&value_pair),
                "require_parameters" => provider.require_parameters = extract_bool_value(&value_pair),
                "zdr" => provider.zdr = extract_bool_value(&value_pair),
                "data_collection" => provider.data_collection = extract_string_value(&value_pair).ok(),
                "sort" => provider.sort = extract_string_value(&value_pair).ok(),
                _ => {}
            }
        }
    }

    Ok(provider)
}

fn extract_string_list(pair: &Pair<Rule>) -> Result<Vec<String>> {
    let mut result = Vec::new();

    fn collect_strings(pair: &Pair<Rule>, result: &mut Vec<String>) {
        match pair.as_rule() {
            Rule::list_literal => {
                for inner in pair.clone().into_inner() {
                    collect_strings(&inner, result);
                }
            }
            Rule::string_literal | Rule::raw_string_literal => {
                result.push(parse_string_content(pair.as_str()));
            }
            _ => {
                for inner in pair.clone().into_inner() {
                    collect_strings(&inner, result);
                }
            }
        }
    }

    collect_strings(pair, &mut result);
    Ok(result)
}

fn extract_bool_value(pair: &Pair<Rule>) -> Option<bool> {
    fn find_bool(pair: &Pair<Rule>) -> Option<bool> {
        match pair.as_rule() {
            Rule::bool_literal => match pair.as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            },
            _ => {
                for inner in pair.clone().into_inner() {
                    if let Some(b) = find_bool(&inner) {
                        return Some(b);
                    }
                }
                None
            }
        }
    }

    find_bool(pair)
}

// ============================================================================
// Type Annotation Parsing
// ============================================================================

fn parse_type_annotation(pair: Pair<Rule>) -> Result<TypeAnnotation> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let type_expr_pair = inner.next().unwrap();
    let ty = parse_type_expr(type_expr_pair)?;

    // Check for optional marker
    let optional = inner.next().is_some();

    Ok(TypeAnnotation { ty, optional, span })
}

fn parse_type_expr(pair: Pair<Rule>) -> Result<TypeExpr> {
    match pair.as_rule() {
        Rule::type_expr => {
            let inner = pair.into_inner().next().unwrap();
            parse_type_expr(inner)
        }
        Rule::primitive_type => {
            let ty = match pair.as_str() {
                "String" => PrimitiveType::String,
                "Int" => PrimitiveType::Int,
                "Float" => PrimitiveType::Float,
                "Bool" => PrimitiveType::Bool,
                "Null" => PrimitiveType::Null,
                "Path" => PrimitiveType::Path,
                _ => return Err(LatticeError::Parse(format!("Unknown primitive type: {}", pair.as_str()))),
            };
            Ok(TypeExpr::Primitive(ty))
        }
        Rule::named_type => {
            let name = pair.into_inner().next().unwrap().as_str().to_string();
            Ok(TypeExpr::Named(name))
        }
        Rule::list_type => {
            let inner_type = parse_type_annotation(pair.into_inner().next().unwrap())?;
            Ok(TypeExpr::List(Box::new(inner_type)))
        }
        Rule::map_type => {
            let mut inner = pair.into_inner();
            let key_type = parse_type_annotation(inner.next().unwrap())?;
            let value_type = parse_type_annotation(inner.next().unwrap())?;
            Ok(TypeExpr::Map(Box::new(key_type), Box::new(value_type)))
        }
        Rule::result_type => {
            let mut inner = pair.into_inner();
            let ok_type = parse_type_annotation(inner.next().unwrap())?;
            let err_type = parse_type_annotation(inner.next().unwrap())?;
            Ok(TypeExpr::Result(Box::new(ok_type), Box::new(err_type)))
        }
        _ => Err(LatticeError::Parse(format!("Unexpected type rule: {:?}", pair.as_rule()))),
    }
}

// ============================================================================
// Function Definition Parsing
// ============================================================================

fn parse_function_def(pair: Pair<Rule>) -> Result<FunctionDef> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let name_pair = inner.next().unwrap();
    let name = Spanned::new(name_pair.as_str().to_string(), make_span(&name_pair));

    let mut params = Vec::new();
    let mut return_type = None;
    let mut body = None;

    for p in inner {
        match p.as_rule() {
            Rule::param_list => {
                for param_pair in p.into_inner() {
                    if param_pair.as_rule() == Rule::param {
                        params.push(parse_param_def(param_pair)?);
                    }
                }
            }
            Rule::type_annotation => {
                return_type = Some(parse_type_annotation(p)?);
            }
            Rule::function_body => {
                body = Some(parse_function_body(p)?);
            }
            _ => {}
        }
    }

    let body = body.ok_or_else(|| LatticeError::Parse("Function missing body".to_string()))?;

    Ok(FunctionDef {
        name,
        params,
        return_type,
        body,
        span,
    })
}

fn parse_param_def(pair: Pair<Rule>) -> Result<ParamDef> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let name_pair = inner.next().unwrap();
    let name = Spanned::new(name_pair.as_str().to_string(), make_span(&name_pair));

    let ty_pair = inner.next().unwrap();
    let ty = parse_type_annotation(ty_pair)?;

    Ok(ParamDef { name, ty, span })
}

fn parse_function_body(pair: Pair<Rule>) -> Result<FunctionBody> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::llm_config => Ok(FunctionBody::LlmConfig(Box::new(parse_llm_config(inner)?))),
        Rule::block_contents => Ok(FunctionBody::Block(parse_block_contents(inner)?)),
        _ => Err(LatticeError::Parse(format!(
            "Unexpected function body rule: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_llm_config(pair: Pair<Rule>) -> Result<LlmConfig> {
    let span = make_span(&pair);
    // Create a placeholder prompt - will be replaced when we parse prompt_field
    let placeholder_prompt = Expr {
        kind: ExprKind::Literal(Literal::String(String::new())),
        span,
    };

    let mut config = LlmConfig {
        use_config: None,
        base_url: None,
        model: None,
        api_key_env: None,
        temperature: None,
        max_tokens: None,
        provider: None,
        prompt: placeholder_prompt,
        span,
    };

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::use_config => {
                // Parse: use: identifier
                let ident = p.into_inner().next().unwrap();
                config.use_config = Some(ident.as_str().to_string());
            }
            Rule::config_field => {
                let field_inner = p.into_inner().next().unwrap();
                match field_inner.as_rule() {
                    Rule::simple_config_field => {
                        let mut simple_inner = field_inner.into_inner();
                        let key = simple_inner.next().unwrap().as_str();
                        let value_pair = simple_inner.next().unwrap();
                        let value_str = extract_string_value(&value_pair)?;

                        match key {
                            "base_url" => config.base_url = Some(value_str),
                            "model" => config.model = Some(value_str),
                            "api_key_env" => config.api_key_env = Some(value_str),
                            "temperature" => {
                                config.temperature = value_str.parse().ok();
                            }
                            "max_tokens" => {
                                config.max_tokens = value_str.parse().ok();
                            }
                            _ => {}
                        }
                    }
                    Rule::provider_field => {
                        config.provider = Some(parse_provider_object(field_inner)?);
                    }
                    _ => {}
                }
            }
            Rule::prompt_field => {
                let string_pair = p.into_inner().next().unwrap();
                // Parse as expression - can be string_literal, raw_string_literal, or fstring_literal
                config.prompt = parse_literal_expr(string_pair)?;
            }
            _ => {}
        }
    }

    Ok(config)
}

fn extract_string_value(pair: &Pair<Rule>) -> Result<String> {
    // Navigate to find the string literal
    fn find_string(pair: &Pair<Rule>) -> Option<String> {
        match pair.as_rule() {
            Rule::string_literal | Rule::raw_string_literal => {
                Some(parse_string_content(pair.as_str()))
            }
            Rule::float_literal | Rule::int_literal => Some(pair.as_str().to_string()),
            _ => {
                for inner in pair.clone().into_inner() {
                    if let Some(s) = find_string(&inner) {
                        return Some(s);
                    }
                }
                None
            }
        }
    }

    find_string(pair).ok_or_else(|| LatticeError::Parse("Expected string value".to_string()))
}

// ============================================================================
// Statement Parsing
// ============================================================================

fn parse_statement(pair: Pair<Rule>) -> Result<Stmt> {
    let span = make_span(&pair);

    let inner = match pair.as_rule() {
        Rule::statement => pair.into_inner().next().unwrap(),
        _ => pair,
    };

    let kind = match inner.as_rule() {
        Rule::let_statement => parse_let_statement(inner)?,
        Rule::assign_statement => parse_assign_statement(inner)?,
        Rule::if_statement => parse_if_statement(inner)?,
        Rule::while_statement => parse_while_statement(inner)?,
        Rule::for_statement => parse_for_statement(inner)?,
        Rule::return_statement => parse_return_statement(inner)?,
        Rule::expression_statement => {
            let expr = parse_expr(inner.into_inner().next().unwrap())?;
            StmtKind::Expr { expr }
        }
        _ => {
            return Err(LatticeError::Parse(format!(
                "Unexpected statement rule: {:?}",
                inner.as_rule()
            )))
        }
    };

    Ok(Stmt { kind, span })
}

fn parse_let_statement(pair: Pair<Rule>) -> Result<StmtKind> {
    let mut inner = pair.into_inner();

    let name_pair = inner.next().unwrap();
    let name = Spanned::new(name_pair.as_str().to_string(), make_span(&name_pair));

    let mut ty = None;
    let mut value = None;

    for p in inner {
        match p.as_rule() {
            Rule::type_annotation => {
                ty = Some(parse_type_annotation(p)?);
            }
            Rule::expression | Rule::or_expr => {
                value = Some(parse_expr(p)?);
            }
            _ => {}
        }
    }

    let value = value.ok_or_else(|| LatticeError::Parse("Let statement missing value".to_string()))?;

    Ok(StmtKind::Let { name, ty, value })
}

fn parse_assign_statement(pair: Pair<Rule>) -> Result<StmtKind> {
    let mut inner = pair.into_inner();

    let target_pair = inner.next().unwrap();
    let target = parse_assign_target(target_pair)?;

    let value = parse_expr(inner.next().unwrap())?;

    Ok(StmtKind::Assign { target, value })
}

fn parse_assign_target(pair: Pair<Rule>) -> Result<AssignTarget> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let base_pair = inner.next().unwrap();
    let base = Spanned::new(base_pair.as_str().to_string(), make_span(&base_pair));

    let mut accessors = Vec::new();
    for p in inner {
        match p.as_rule() {
            Rule::field_access => {
                let field_name = p.into_inner().next().unwrap().as_str().to_string();
                accessors.push(Accessor::Field(field_name));
            }
            Rule::index_access => {
                let index_expr = parse_expr(p.into_inner().next().unwrap())?;
                accessors.push(Accessor::Index(index_expr));
            }
            _ => {}
        }
    }

    Ok(AssignTarget {
        base,
        accessors,
        span,
    })
}

fn parse_if_statement(pair: Pair<Rule>) -> Result<StmtKind> {
    let mut inner = pair.into_inner();

    let condition = parse_expr(inner.next().unwrap())?;
    let then_branch = parse_block(inner.next().unwrap())?;

    let else_branch = inner.next().map(|p| parse_else_clause(p)).transpose()?;

    Ok(StmtKind::If {
        condition,
        then_branch,
        else_branch,
    })
}

fn parse_else_clause(pair: Pair<Rule>) -> Result<ElseClause> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::if_statement => {
            let if_stmt = Stmt {
                kind: parse_if_statement(inner)?,
                span: Span::default(),
            };
            Ok(ElseClause::ElseIf(Box::new(if_stmt)))
        }
        Rule::block => Ok(ElseClause::Else(parse_block(inner)?)),
        _ => Err(LatticeError::Parse(format!(
            "Unexpected else clause rule: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_while_statement(pair: Pair<Rule>) -> Result<StmtKind> {
    let mut inner = pair.into_inner();

    let condition = parse_expr(inner.next().unwrap())?;
    let body = parse_block(inner.next().unwrap())?;

    Ok(StmtKind::While { condition, body })
}

fn parse_for_statement(pair: Pair<Rule>) -> Result<StmtKind> {
    let mut inner = pair.into_inner();

    let var_pair = inner.next().unwrap();
    let var = Spanned::new(var_pair.as_str().to_string(), make_span(&var_pair));

    let iterable = parse_expr(inner.next().unwrap())?;
    let body = parse_block(inner.next().unwrap())?;

    Ok(StmtKind::For {
        var,
        iterable,
        body,
    })
}

fn parse_return_statement(pair: Pair<Rule>) -> Result<StmtKind> {
    let value = pair.into_inner().next().map(parse_expr).transpose()?;
    Ok(StmtKind::Return { value })
}

// ============================================================================
// Expression Parsing
// ============================================================================

fn parse_expr(pair: Pair<Rule>) -> Result<Expr> {
    let _span = make_span(&pair);

    match pair.as_rule() {
        Rule::expression => parse_pipe_expr(pair),
        Rule::pipe_expr => parse_pipe_expr(pair),
        Rule::or_expr => parse_or_expr(pair),
        Rule::and_expr => parse_and_expr(pair),
        Rule::equality_expr => parse_equality_expr(pair),
        Rule::comparison_expr => parse_comparison_expr(pair),
        Rule::additive_expr => parse_additive_expr(pair),
        Rule::multiplicative_expr => parse_multiplicative_expr(pair),
        Rule::unary_expr => parse_unary_expr(pair),
        Rule::postfix_expr => parse_postfix_expr(pair),
        Rule::primary_expr => parse_primary_expr(pair),
        _ => {
            // Try to find the actual expression inside
            if let Some(inner) = pair.into_inner().next() {
                parse_expr(inner)
            } else {
                Err(LatticeError::Parse("Unexpected expression rule".to_string()))
            }
        }
    }
}

/// Parse pipe expression: x |> f or x |> f(a, b)
/// Transforms x |> f into f(x) and x |> f(a, b) into f(x, a, b)
fn parse_pipe_expr(pair: Pair<Rule>) -> Result<Expr> {
    let _span = make_span(&pair);
    let mut inner = pair.into_inner();

    let first = inner.next().unwrap();
    let mut left = parse_expr(first)?;

    for right_pair in inner {
        let right = parse_expr(right_pair)?;
        let new_span = left.span.merge(right.span);

        // Transform the pipe: left |> right
        // If right is a Call, insert left as first argument
        // If right is a Var (function name), create Call with left as only argument
        left = match right.kind {
            ExprKind::Call { callee, args } => {
                // x |> f(a, b) => f(x, a, b)
                let mut new_args = vec![left];
                new_args.extend(args);
                Expr {
                    kind: ExprKind::Call {
                        callee,
                        args: new_args,
                    },
                    span: new_span,
                }
            }
            ExprKind::Var(_) => {
                // x |> f => f(x)
                Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(right),
                        args: vec![left],
                    },
                    span: new_span,
                }
            }
            ExprKind::Lambda { .. } => {
                // x |> |y| expr => (|y| expr)(x)
                Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(right),
                        args: vec![left],
                    },
                    span: new_span,
                }
            }
            ExprKind::MapColumn { table: _, input_col, output_col, mapper } => {
                // x |> map_column("in", "out", func) => map_column(x, "in", "out", func)
                Expr {
                    kind: ExprKind::MapColumn {
                        table: Box::new(left),
                        input_col,
                        output_col,
                        mapper,
                    },
                    span: new_span,
                }
            }
            ExprKind::ParallelMap { collection: _, mapper } => {
                // x |> parallel_map(func) => parallel_map(x, func)
                Expr {
                    kind: ExprKind::ParallelMap {
                        collection: Box::new(left),
                        mapper,
                    },
                    span: new_span,
                }
            }
            ExprKind::MapRow { table: _, output_col, mapper } => {
                // x |> map_row("out", func) => map_row(x, "out", func)
                Expr {
                    kind: ExprKind::MapRow {
                        table: Box::new(left),
                        output_col,
                        mapper,
                    },
                    span: new_span,
                }
            }
            ExprKind::Explode { table: _, column, prefix } => {
                // x |> explode("col") => explode(x, "col")
                // x |> explode("col", "prefix") => explode(x, "col", "prefix")
                Expr {
                    kind: ExprKind::Explode {
                        table: Box::new(left),
                        column,
                        prefix,
                    },
                    span: new_span,
                }
            }
            _ => {
                return Err(LatticeError::Parse(
                    "Pipe operator requires a function, function call, lambda, map_column, map_row, explode, or parallel_map on the right side".to_string()
                ));
            }
        };
    }

    Ok(left)
}

fn parse_or_expr(pair: Pair<Rule>) -> Result<Expr> {
    let _span = make_span(&pair);
    let mut inner = pair.into_inner();

    let mut left = parse_expr(inner.next().unwrap())?;

    for right_pair in inner {
        let right = parse_expr(right_pair)?;
        let new_span = left.span.merge(right.span);
        left = Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            },
            span: new_span,
        };
    }

    Ok(left)
}

fn parse_and_expr(pair: Pair<Rule>) -> Result<Expr> {
    let _span = make_span(&pair);
    let mut inner = pair.into_inner();

    let mut left = parse_expr(inner.next().unwrap())?;

    for right_pair in inner {
        let right = parse_expr(right_pair)?;
        let new_span = left.span.merge(right.span);
        left = Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            },
            span: new_span,
        };
    }

    Ok(left)
}

fn parse_equality_expr(pair: Pair<Rule>) -> Result<Expr> {
    let _span = make_span(&pair);
    let mut inner = pair.into_inner();

    let mut left = parse_expr(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "==" => BinaryOp::Eq,
            "!=" => BinaryOp::Ne,
            _ => {
                // This is actually the right operand, not an operator
                let right = parse_expr(op_pair)?;
                let new_span = left.span.merge(right.span);
                left = Expr {
                    kind: ExprKind::Binary {
                        left: Box::new(left),
                        op: BinaryOp::Eq, // default
                        right: Box::new(right),
                    },
                    span: new_span,
                };
                continue;
            }
        };

        let right = parse_expr(inner.next().unwrap())?;
        let new_span = left.span.merge(right.span);
        left = Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: new_span,
        };
    }

    Ok(left)
}

fn parse_comparison_expr(pair: Pair<Rule>) -> Result<Expr> {
    let _span = make_span(&pair);
    let mut inner = pair.into_inner();

    let mut left = parse_expr(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "<" => BinaryOp::Lt,
            "<=" => BinaryOp::Le,
            ">" => BinaryOp::Gt,
            ">=" => BinaryOp::Ge,
            _ => {
                let right = parse_expr(op_pair)?;
                let new_span = left.span.merge(right.span);
                left = Expr {
                    kind: ExprKind::Binary {
                        left: Box::new(left),
                        op: BinaryOp::Lt,
                        right: Box::new(right),
                    },
                    span: new_span,
                };
                continue;
            }
        };

        let right = parse_expr(inner.next().unwrap())?;
        let new_span = left.span.merge(right.span);
        left = Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: new_span,
        };
    }

    Ok(left)
}

fn parse_additive_expr(pair: Pair<Rule>) -> Result<Expr> {
    let _span = make_span(&pair);
    let mut inner = pair.into_inner();

    let mut left = parse_expr(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "+" => BinaryOp::Add,
            "-" => BinaryOp::Sub,
            _ => {
                let right = parse_expr(op_pair)?;
                let new_span = left.span.merge(right.span);
                left = Expr {
                    kind: ExprKind::Binary {
                        left: Box::new(left),
                        op: BinaryOp::Add,
                        right: Box::new(right),
                    },
                    span: new_span,
                };
                continue;
            }
        };

        let right = parse_expr(inner.next().unwrap())?;
        let new_span = left.span.merge(right.span);
        left = Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: new_span,
        };
    }

    Ok(left)
}

fn parse_multiplicative_expr(pair: Pair<Rule>) -> Result<Expr> {
    let _span = make_span(&pair);
    let mut inner = pair.into_inner();

    let mut left = parse_expr(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "*" => BinaryOp::Mul,
            "/" => BinaryOp::Div,
            "%" => BinaryOp::Mod,
            _ => {
                let right = parse_expr(op_pair)?;
                let new_span = left.span.merge(right.span);
                left = Expr {
                    kind: ExprKind::Binary {
                        left: Box::new(left),
                        op: BinaryOp::Mul,
                        right: Box::new(right),
                    },
                    span: new_span,
                };
                continue;
            }
        };

        let right = parse_expr(inner.next().unwrap())?;
        let new_span = left.span.merge(right.span);
        left = Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: new_span,
        };
    }

    Ok(left)
}

fn parse_unary_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::unary_op => {
            let op = match first.as_str() {
                "-" => UnaryOp::Neg,
                "!" => UnaryOp::Not,
                _ => return Err(LatticeError::Parse(format!("Unknown unary op: {}", first.as_str()))),
            };
            let operand = parse_expr(inner.next().unwrap())?;
            Ok(Expr {
                kind: ExprKind::Unary {
                    op,
                    operand: Box::new(operand),
                },
                span,
            })
        }
        _ => parse_expr(first),
    }
}

fn parse_postfix_expr(pair: Pair<Rule>) -> Result<Expr> {
    let _span = make_span(&pair);
    let mut inner = pair.into_inner();

    let mut expr = parse_expr(inner.next().unwrap())?;

    for p in inner {
        let new_span = expr.span.merge(make_span(&p));
        match p.as_rule() {
            Rule::field_access => {
                let field = p.into_inner().next().unwrap().as_str().to_string();
                expr = Expr {
                    kind: ExprKind::Field {
                        object: Box::new(expr),
                        field,
                    },
                    span: new_span,
                };
            }
            Rule::index_access => {
                let index = parse_expr(p.into_inner().next().unwrap())?;
                expr = Expr {
                    kind: ExprKind::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    },
                    span: new_span,
                };
            }
            Rule::call_args => {
                let mut args = Vec::new();
                for arg_pair in p.into_inner() {
                    if arg_pair.as_rule() == Rule::arg_list {
                        for a in arg_pair.into_inner() {
                            args.push(parse_expr(a)?);
                        }
                    }
                }
                expr = Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    },
                    span: new_span,
                };
            }
            _ => {}
        }
    }

    Ok(expr)
}

fn parse_primary_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::expression => {
            // Grouped expression
            let expr = parse_expr(inner)?;
            Ok(Expr {
                kind: ExprKind::Grouped(Box::new(expr)),
                span,
            })
        }
        Rule::if_expr => parse_if_expr(inner),
        Rule::match_expr => parse_match_expr(inner),
        Rule::parallel_block => parse_parallel_block(inner),
        Rule::parallel_map_expr => parse_parallel_map_expr(inner),
        Rule::map_column_expr => parse_map_column_expr(inner),
        Rule::map_row_expr => parse_map_row_expr(inner),
        Rule::explode_expr => parse_explode_expr(inner),
        Rule::sql_expr => parse_sql_expr(inner),
        Rule::lambda_expr => parse_lambda_expr(inner),
        Rule::list_literal => parse_list_literal(inner),
        Rule::map_literal => parse_map_literal(inner),
        Rule::struct_literal => parse_struct_literal(inner),
        Rule::literal => parse_literal_expr(inner),
        Rule::enum_constructor => parse_enum_constructor(inner),
        Rule::dollar_field => parse_dollar_field(inner),
        Rule::identifier => Ok(Expr {
            kind: ExprKind::Var(inner.as_str().to_string()),
            span,
        }),
        _ => Err(LatticeError::Parse(format!(
            "Unexpected primary expression rule: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_dollar_field(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let inner = pair.into_inner().next().unwrap();

    // The inner can be either an identifier ($field) or an expression ($["field"])
    let field_expr = match inner.as_rule() {
        Rule::identifier => {
            // $field -> convert to string literal for indexing
            Expr {
                kind: ExprKind::Literal(Literal::String(inner.as_str().to_string())),
                span: make_span(&inner),
            }
        }
        _ => {
            // $[expr] -> parse the expression
            parse_expr(inner)?
        }
    };

    Ok(Expr {
        kind: ExprKind::DollarField(Box::new(field_expr)),
        span,
    })
}

fn parse_enum_constructor(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let enum_name = inner.next().unwrap().as_str().to_string();
    let variant = inner.next().unwrap().as_str().to_string();

    Ok(Expr {
        kind: ExprKind::EnumVariant { enum_name, variant },
        span,
    })
}

fn parse_if_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let condition = parse_expr(inner.next().unwrap())?;
    let then_branch = parse_block(inner.next().unwrap())?;

    let else_branch = inner.next().map(|p| parse_else_expr_clause(p)).transpose()?;

    Ok(Expr {
        kind: ExprKind::If {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        },
        span,
    })
}

fn parse_else_expr_clause(pair: Pair<Rule>) -> Result<IfExprElse> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::if_expr => {
            let if_expr = parse_if_expr(inner)?;
            Ok(IfExprElse::ElseIf(Box::new(if_expr)))
        }
        Rule::block => Ok(IfExprElse::Else(parse_block(inner)?)),
        _ => Err(LatticeError::Parse(format!(
            "Unexpected else expression clause rule: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_match_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let scrutinee = parse_expr(inner.next().unwrap())?;

    let mut arms = Vec::new();
    for arm_pair in inner {
        if arm_pair.as_rule() == Rule::match_arm {
            arms.push(parse_match_arm(arm_pair)?);
        }
    }

    Ok(Expr {
        kind: ExprKind::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        },
        span,
    })
}

fn parse_match_arm(pair: Pair<Rule>) -> Result<MatchArm> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let pattern = parse_pattern(inner.next().unwrap())?;

    let body_pair = inner.next().unwrap();
    let body = match body_pair.as_rule() {
        Rule::block => MatchArmBody::Block(parse_block(body_pair)?),
        _ => MatchArmBody::Expr(parse_expr(body_pair)?),
    };

    Ok(MatchArm {
        pattern,
        body,
        span,
    })
}

fn parse_pattern(pair: Pair<Rule>) -> Result<Pattern> {
    let span = make_span(&pair);
    let inner = pair.into_inner().next().unwrap();

    let kind = match inner.as_rule() {
        Rule::result_pattern => {
            let mut parts = inner.into_inner();
            let variant = parts.next().unwrap().as_str();
            let is_ok = variant == "Ok";
            let binding = parts.next().unwrap().as_str().to_string();
            PatternKind::Result { is_ok, binding }
        }
        Rule::enum_pattern => {
            let mut parts = inner.into_inner();
            let enum_name = parts.next().unwrap().as_str().to_string();
            let variant = parts.next().unwrap().as_str().to_string();
            PatternKind::Enum { enum_name, variant }
        }
        Rule::literal_pattern => {
            let lit_pair = inner.into_inner().next().unwrap();
            PatternKind::Literal(parse_literal(lit_pair)?)
        }
        Rule::wildcard_pattern => PatternKind::Wildcard,
        Rule::binding_pattern => {
            PatternKind::Binding(inner.into_inner().next().unwrap().as_str().to_string())
        }
        _ => {
            return Err(LatticeError::Parse(format!(
                "Unexpected pattern rule: {:?}",
                inner.as_rule()
            )))
        }
    };

    Ok(Pattern { kind, span })
}

fn parse_parallel_block(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut exprs = Vec::new();

    for p in pair.into_inner() {
        exprs.push(parse_expr(p)?);
    }

    Ok(Expr {
        kind: ExprKind::Parallel(exprs),
        span,
    })
}

fn parse_parallel_map_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let collection = parse_expr(inner.next().unwrap())?;
    // The second argument is a lambda_expr, not a general expression
    let mapper = parse_lambda_expr(inner.next().unwrap())?;

    Ok(Expr {
        kind: ExprKind::ParallelMap {
            collection: Box::new(collection),
            mapper: Box::new(mapper),
        },
        span,
    })
}

fn parse_map_column_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let inner: Vec<_> = pair.into_inner().collect();

    // Check if we have 4 arguments (full form) or 3 arguments (pipe form)
    if inner.len() == 4 {
        // Full form: map_column(table, input_col, output_col, mapper)
        let table = parse_expr(inner[0].clone())?;
        let input_col = parse_expr(inner[1].clone())?;
        let output_col = parse_expr(inner[2].clone())?;
        let mapper = parse_lambda_expr(inner[3].clone())?;

        Ok(Expr {
            kind: ExprKind::MapColumn {
                table: Box::new(table),
                input_col: Box::new(input_col),
                output_col: Box::new(output_col),
                mapper: Box::new(mapper),
            },
            span,
        })
    } else {
        // Pipe form: map_column(input_col, output_col, mapper) - table will be provided by pipe
        let input_col = parse_expr(inner[0].clone())?;
        let output_col = parse_expr(inner[1].clone())?;
        let mapper = parse_lambda_expr(inner[2].clone())?;

        // Use a placeholder Null expression for table - pipe will replace it
        let placeholder_table = Expr {
            kind: ExprKind::Literal(Literal::Null),
            span,
        };

        Ok(Expr {
            kind: ExprKind::MapColumn {
                table: Box::new(placeholder_table),
                input_col: Box::new(input_col),
                output_col: Box::new(output_col),
                mapper: Box::new(mapper),
            },
            span,
        })
    }
}

fn parse_map_row_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let inner: Vec<_> = pair.into_inner().collect();

    // Check if we have 3 arguments (full form) or 2 arguments (pipe form)
    if inner.len() == 3 {
        // Full form: map_row(table, output_col, mapper)
        let table = parse_expr(inner[0].clone())?;
        let output_col = parse_expr(inner[1].clone())?;
        let mapper = parse_map_row_mapper(inner[2].clone())?;

        Ok(Expr {
            kind: ExprKind::MapRow {
                table: Box::new(table),
                output_col: Box::new(output_col),
                mapper: Box::new(mapper),
            },
            span,
        })
    } else {
        // Pipe form: map_row(output_col, mapper) - table will be provided by pipe
        let output_col = parse_expr(inner[0].clone())?;
        let mapper = parse_map_row_mapper(inner[1].clone())?;

        // Use a placeholder Null expression for table - pipe will replace it
        let placeholder_table = Expr {
            kind: ExprKind::Literal(Literal::Null),
            span,
        };

        Ok(Expr {
            kind: ExprKind::MapRow {
                table: Box::new(placeholder_table),
                output_col: Box::new(output_col),
                mapper: Box::new(mapper),
            },
            span,
        })
    }
}

/// Parse a map_row mapper (can be a lambda or an expression with $field)
fn parse_map_row_mapper(pair: Pair<Rule>) -> Result<Expr> {
    match pair.as_rule() {
        Rule::map_row_mapper => {
            let inner = pair.into_inner().next().unwrap();
            parse_map_row_mapper(inner)
        }
        Rule::lambda_expr => parse_lambda_expr(pair),
        _ => parse_expr(pair),
    }
}

fn parse_explode_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let inner: Vec<_> = pair.into_inner().collect();

    match inner.len() {
        // Pipe form: explode("column")
        1 => {
            let column = parse_expr(inner[0].clone())?;

            // Use a placeholder Null expression for table - pipe will replace it
            let placeholder_table = Expr {
                kind: ExprKind::Literal(Literal::Null),
                span,
            };

            Ok(Expr {
                kind: ExprKind::Explode {
                    table: Box::new(placeholder_table),
                    column: Box::new(column),
                    prefix: None,
                },
                span,
            })
        }
        // Could be: explode(table, "column") OR explode("column", "prefix") in pipe
        // We treat 2 args as: explode(table, column)
        2 => {
            let table = parse_expr(inner[0].clone())?;
            let column = parse_expr(inner[1].clone())?;

            Ok(Expr {
                kind: ExprKind::Explode {
                    table: Box::new(table),
                    column: Box::new(column),
                    prefix: None,
                },
                span,
            })
        }
        // Full form: explode(table, "column", "prefix")
        3 => {
            let table = parse_expr(inner[0].clone())?;
            let column = parse_expr(inner[1].clone())?;
            let prefix = parse_expr(inner[2].clone())?;

            Ok(Expr {
                kind: ExprKind::Explode {
                    table: Box::new(table),
                    column: Box::new(column),
                    prefix: Some(Box::new(prefix)),
                },
                span,
            })
        }
        _ => Err(LatticeError::Parse(format!(
            "explode expects 1-3 arguments, got {}",
            inner.len()
        ))),
    }
}

fn parse_sql_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let inner = pair.into_inner();

    let mut ty = None;
    let mut query = None;

    for p in inner {
        match p.as_rule() {
            Rule::type_annotation => {
                ty = Some(parse_type_annotation(p)?);
            }
            _ => {
                query = Some(parse_expr(p)?);
            }
        }
    }

    let query = query.ok_or_else(|| LatticeError::Parse("SQL missing query".to_string()))?;

    Ok(Expr {
        kind: ExprKind::Sql {
            ty,
            query: Box::new(query),
        },
        span,
    })
}

fn parse_lambda_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let inner = pair.into_inner();

    let mut params = Vec::new();
    let mut body = None;

    for p in inner {
        match p.as_rule() {
            Rule::lambda_params => {
                for param_pair in p.into_inner() {
                    params.push(param_pair.as_str().to_string());
                }
            }
            Rule::block => {
                body = Some(LambdaBody::Block(parse_block(p)?));
            }
            _ => {
                body = Some(LambdaBody::Expr(parse_expr(p)?));
            }
        }
    }

    let body = body.ok_or_else(|| LatticeError::Parse("Lambda missing body".to_string()))?;

    Ok(Expr {
        kind: ExprKind::Lambda {
            params,
            body: Box::new(body),
        },
        span,
    })
}

fn parse_list_literal(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut elements = Vec::new();

    for p in pair.into_inner() {
        elements.push(parse_expr(p)?);
    }

    Ok(Expr {
        kind: ExprKind::List(elements),
        span,
    })
}

fn parse_map_literal(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut entries = Vec::new();

    for p in pair.into_inner() {
        if p.as_rule() == Rule::map_entry {
            let mut inner = p.into_inner();
            let key_pair = inner.next().unwrap();
            let key = match key_pair.as_rule() {
                Rule::string_literal => MapKey::String(parse_string_content(key_pair.as_str())),
                Rule::identifier => MapKey::Ident(key_pair.as_str().to_string()),
                _ => MapKey::String(key_pair.as_str().to_string()),
            };
            let value = parse_expr(inner.next().unwrap())?;
            entries.push((key, value));
        }
    }

    Ok(Expr {
        kind: ExprKind::Map(entries),
        span,
    })
}

fn parse_struct_literal(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    let mut fields = Vec::new();
    for p in inner {
        if p.as_rule() == Rule::struct_field {
            let mut field_inner = p.into_inner();
            let field_name = field_inner.next().unwrap().as_str().to_string();
            let field_value = parse_expr(field_inner.next().unwrap())?;
            fields.push((field_name, field_value));
        }
    }

    Ok(Expr {
        kind: ExprKind::Struct { name, fields },
        span,
    })
}

fn parse_literal_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);

    // Check if this is an f-string literal
    let inner = match pair.as_rule() {
        Rule::literal => pair.clone().into_inner().next().unwrap(),
        _ => pair.clone(),
    };

    if inner.as_rule() == Rule::fstring_literal {
        return parse_fstring_expr(inner);
    }

    let lit = parse_literal(pair)?;

    Ok(Expr {
        kind: ExprKind::Literal(lit),
        span,
    })
}

/// Parse an f-string (interpolated string) expression
fn parse_fstring_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut parts = Vec::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::fstring_single_quote | Rule::fstring_triple_quote => {
                // Delegate to the inner single or triple quote parsing
                return parse_fstring_inner(p, span);
            }
            Rule::fstring_part | Rule::fstring_triple_part => {
                let inner = p.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::fstring_text | Rule::fstring_triple_text => {
                        let text = parse_fstring_text(inner.as_str());
                        if !text.is_empty() {
                            parts.push(FStringPart::Text(text));
                        }
                    }
                    Rule::fstring_interpolation => {
                        let expr_pair = inner.into_inner().next().unwrap();
                        let expr = parse_expr(expr_pair)?;
                        parts.push(FStringPart::Expr(expr));
                    }
                    _ => {}
                }
            }
            Rule::fstring_text | Rule::fstring_triple_text => {
                let text = parse_fstring_text(p.as_str());
                if !text.is_empty() {
                    parts.push(FStringPart::Text(text));
                }
            }
            Rule::fstring_interpolation => {
                let expr_pair = p.into_inner().next().unwrap();
                let expr = parse_expr(expr_pair)?;
                parts.push(FStringPart::Expr(expr));
            }
            _ => {}
        }
    }

    Ok(Expr {
        kind: ExprKind::FString(parts),
        span,
    })
}

/// Parse the inner content of an f-string (single or triple quoted)
fn parse_fstring_inner(pair: Pair<Rule>, span: Span) -> Result<Expr> {
    let is_triple_quoted = pair.as_rule() == Rule::fstring_triple_quote;
    let mut parts = Vec::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::fstring_part | Rule::fstring_triple_part => {
                let inner = p.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::fstring_text | Rule::fstring_triple_text => {
                        let text = parse_fstring_text(inner.as_str());
                        if !text.is_empty() {
                            parts.push(FStringPart::Text(text));
                        }
                    }
                    Rule::fstring_interpolation => {
                        let expr_pair = inner.into_inner().next().unwrap();
                        let expr = parse_expr(expr_pair)?;
                        parts.push(FStringPart::Expr(expr));
                    }
                    _ => {}
                }
            }
            Rule::fstring_text | Rule::fstring_triple_text => {
                let text = parse_fstring_text(p.as_str());
                if !text.is_empty() {
                    parts.push(FStringPart::Text(text));
                }
            }
            Rule::fstring_interpolation => {
                let expr_pair = p.into_inner().next().unwrap();
                let expr = parse_expr(expr_pair)?;
                parts.push(FStringPart::Expr(expr));
            }
            _ => {}
        }
    }

    // Apply dedent to triple-quoted f-strings
    let parts = if is_triple_quoted {
        dedent_fstring_parts(parts)
    } else {
        parts
    };

    Ok(Expr {
        kind: ExprKind::FString(parts),
        span,
    })
}

/// Parse f-string text content, handling escape sequences
fn parse_fstring_text(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    'n' => {
                        result.push('\n');
                        chars.next();
                    }
                    'r' => {
                        result.push('\r');
                        chars.next();
                    }
                    't' => {
                        result.push('\t');
                        chars.next();
                    }
                    '\\' => {
                        result.push('\\');
                        chars.next();
                    }
                    '"' => {
                        result.push('"');
                        chars.next();
                    }
                    '{' => {
                        result.push('{');
                        chars.next();
                    }
                    '}' => {
                        result.push('}');
                        chars.next();
                    }
                    _ => {
                        result.push(c);
                    }
                }
            } else {
                result.push(c);
            }
        } else if c == '{' {
            // Check for escaped brace {{
            if let Some(&'{') = chars.peek() {
                result.push('{');
                chars.next();
            }
        } else if c == '}' {
            // Check for escaped brace }}
            if let Some(&'}') = chars.peek() {
                result.push('}');
                chars.next();
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Dedent f-string parts for triple-quoted strings.
///
/// This applies a dedent algorithm similar to Python's textwrap.dedent():
/// 1. Skip the first line if it's empty/whitespace-only
/// 2. Find the minimum indentation of all non-empty lines
/// 3. Strip that common indentation from all lines
/// 4. Remove trailing whitespace-only final line
fn dedent_fstring_parts(parts: Vec<FStringPart>) -> Vec<FStringPart> {
    if parts.is_empty() {
        return parts;
    }

    // Concatenate text parts with placeholders to analyze line structure
    let mut full_text = String::new();
    for part in &parts {
        match part {
            FStringPart::Text(t) => full_text.push_str(t),
            FStringPart::Expr(_) => full_text.push('\x00'), // placeholder for expression
        }
    }

    // Find minimum indentation from the full text
    let mut min_indent: Option<usize> = None;
    let mut first_line = true;

    for line in full_text.split('\n') {
        if first_line {
            // Skip first line (content immediately after opening """)
            first_line = false;
            continue;
        }

        // Check if line has any non-whitespace content
        let trimmed = line.trim_start_matches([' ', '\t']);
        if trimmed.is_empty() {
            // Empty or whitespace-only line, skip for indent calculation
            continue;
        }

        // Count leading whitespace
        let indent = line.len() - trimmed.len();
        min_indent = Some(match min_indent {
            Some(current) => current.min(indent),
            None => indent,
        });
    }

    let min_indent = min_indent.unwrap_or(0);

    // Second pass: strip the minimum indentation from each text part
    let mut result = Vec::new();
    let mut at_line_start = true;
    let mut chars_to_skip = 0;

    for part in parts {
        match part {
            FStringPart::Text(text) => {
                let mut new_text = String::new();

                for c in text.chars() {
                    if c == '\n' {
                        new_text.push(c);
                        at_line_start = true;
                        chars_to_skip = min_indent;
                    } else if at_line_start && chars_to_skip > 0 && (c == ' ' || c == '\t') {
                        // Skip indentation character
                        chars_to_skip -= 1;
                    } else {
                        at_line_start = false;
                        chars_to_skip = 0;
                        new_text.push(c);
                    }
                }

                if !new_text.is_empty() {
                    result.push(FStringPart::Text(new_text));
                }
            }
            FStringPart::Expr(expr) => {
                at_line_start = false;
                chars_to_skip = 0;
                result.push(FStringPart::Expr(expr));
            }
        }
    }

    // Post-process: strip leading newline if first text part starts with one
    if let Some(FStringPart::Text(first)) = result.first_mut() {
        if first.starts_with('\n') {
            *first = first[1..].to_string();
        }
        if first.is_empty() {
            result.remove(0);
        }
    }

    // Strip trailing whitespace-only text
    if let Some(FStringPart::Text(last)) = result.last_mut() {
        let trimmed = last.trim_end();
        if trimmed.is_empty() {
            result.pop();
        } else {
            *last = trimmed.to_string();
        }
    }

    result
}

fn parse_literal(pair: Pair<Rule>) -> Result<Literal> {
    let inner = match pair.as_rule() {
        Rule::literal => pair.into_inner().next().unwrap(),
        _ => pair,
    };

    match inner.as_rule() {
        Rule::int_literal => {
            let value: i64 = inner
                .as_str()
                .parse()
                .map_err(|_| LatticeError::Parse(format!("Invalid integer: {}", inner.as_str())))?;
            Ok(Literal::Int(value))
        }
        Rule::float_literal => {
            let value: f64 = inner
                .as_str()
                .parse()
                .map_err(|_| LatticeError::Parse(format!("Invalid float: {}", inner.as_str())))?;
            Ok(Literal::Float(value))
        }
        Rule::string_literal | Rule::raw_string_literal => {
            Ok(Literal::String(parse_string_content(inner.as_str())))
        }
        Rule::bool_literal => Ok(Literal::Bool(inner.as_str() == "true")),
        Rule::null_literal => Ok(Literal::Null),
        Rule::fstring_literal => {
            // F-strings are handled separately as expressions, not literals
            // This branch shouldn't be reached in normal parsing
            Err(LatticeError::Parse("F-string should be parsed as expression".to_string()))
        }
        _ => Err(LatticeError::Parse(format!(
            "Unexpected literal rule: {:?}",
            inner.as_rule()
        ))),
    }
}

// ============================================================================
// Block Parsing
// ============================================================================

fn parse_block(pair: Pair<Rule>) -> Result<Block> {
    let span = make_span(&pair);

    let contents = pair.into_inner().next().unwrap();
    parse_block_contents_with_span(contents, span)
}

fn parse_block_contents(pair: Pair<Rule>) -> Result<Block> {
    let span = make_span(&pair);
    parse_block_contents_with_span(pair, span)
}

fn parse_block_contents_with_span(pair: Pair<Rule>, span: Span) -> Result<Block> {
    let mut stmts = Vec::new();
    let mut expr = None;

    let items: Vec<_> = pair.into_inner().collect();
    let len = items.len();

    for (i, p) in items.into_iter().enumerate() {
        let is_last = i == len - 1;
        match p.as_rule() {
            Rule::statement => {
                // Check if this is an expression_statement or if_statement that could be a trailing expression
                let inner = p.clone().into_inner().next().unwrap();
                if is_last && inner.as_rule() == Rule::expression_statement {
                    // Last item is an expression statement - use as trailing expression
                    let expr_inner = inner.into_inner().next().unwrap();
                    expr = Some(Box::new(parse_expr(expr_inner)?));
                } else if is_last && inner.as_rule() == Rule::if_statement {
                    // Last item is an if statement - convert to if expression
                    expr = Some(Box::new(convert_if_stmt_to_expr(inner)?));
                } else {
                    stmts.push(parse_statement(p)?);
                }
            }
            _ => {
                // Last item might be a trailing expression
                if is_last {
                    expr = Some(Box::new(parse_expr(p)?));
                } else {
                    // Wrap as expression statement
                    let e = parse_expr(p)?;
                    stmts.push(Stmt {
                        kind: StmtKind::Expr { expr: e.clone() },
                        span: e.span,
                    });
                }
            }
        }
    }

    Ok(Block { stmts, expr, span })
}

/// Convert an if_statement parse node to an if expression
fn convert_if_stmt_to_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let condition = parse_expr(inner.next().unwrap())?;
    let then_branch = parse_block(inner.next().unwrap())?;

    let else_branch = inner.next().map(|p| convert_else_clause_to_expr(p)).transpose()?;

    Ok(Expr {
        kind: ExprKind::If {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        },
        span,
    })
}

/// Convert an else_clause to an IfExprElse
fn convert_else_clause_to_expr(pair: Pair<Rule>) -> Result<IfExprElse> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::if_statement => {
            let if_expr = convert_if_stmt_to_expr(inner)?;
            Ok(IfExprElse::ElseIf(Box::new(if_expr)))
        }
        Rule::block => Ok(IfExprElse::Else(parse_block(inner)?)),
        _ => Err(LatticeError::Parse(format!(
            "Unexpected else clause rule in expression context: {:?}",
            inner.as_rule()
        ))),
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

fn make_span(pair: &Pair<Rule>) -> Span {
    let pest_span = pair.as_span();
    Span {
        start: pest_span.start(),
        end: pest_span.end(),
        line: pest_span.start_pos().line_col().0,
        column: pest_span.start_pos().line_col().1,
    }
}

/// Parse string content, handling escape sequences
fn parse_string_content(s: &str) -> String {
    // Remove quotes
    let s = if s.starts_with("\"\"\"") && s.ends_with("\"\"\"") {
        &s[3..s.len() - 3]
    } else if s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    };

    // Handle escape sequences
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    'n' => {
                        result.push('\n');
                        chars.next();
                    }
                    'r' => {
                        result.push('\r');
                        chars.next();
                    }
                    't' => {
                        result.push('\t');
                        chars.next();
                    }
                    '\\' => {
                        result.push('\\');
                        chars.next();
                    }
                    '"' => {
                        result.push('"');
                        chars.next();
                    }
                    '$' => {
                        result.push('$');
                        chars.next();
                    }
                    _ => {
                        result.push(c);
                    }
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int_literal() {
        let expr = parse_expression("42").unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_parse_float_literal() {
        let expr = parse_expression("3.14").unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::Float(f)) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float literal"),
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let expr = parse_expression("\"hello\"").unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::String(s)) if s == "hello"));
    }

    #[test]
    fn test_parse_bool_literal() {
        let expr = parse_expression("true").unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(true))));

        let expr = parse_expression("false").unwrap();
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Bool(false))));
    }

    #[test]
    fn test_parse_variable() {
        let expr = parse_expression("foo").unwrap();
        assert!(matches!(expr.kind, ExprKind::Var(s) if s == "foo"));
    }

    #[test]
    fn test_parse_binary_expr() {
        let expr = parse_expression("1 + 2").unwrap();
        match expr.kind {
            ExprKind::Binary { op, .. } => assert_eq!(op, BinaryOp::Add),
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_list_literal() {
        let expr = parse_expression("[1, 2, 3]").unwrap();
        match expr.kind {
            ExprKind::List(elements) => assert_eq!(elements.len(), 3),
            _ => panic!("Expected list literal"),
        }
    }

    #[test]
    fn test_parse_type_def() {
        let program = parse("type Person { name: String, age: Int }").unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::TypeDef(td) => {
                assert_eq!(td.name.node, "Person");
                assert_eq!(td.fields.len(), 2);
            }
            _ => panic!("Expected type definition"),
        }
    }

    #[test]
    fn test_parse_enum_def() {
        let program = parse("enum Color { Red, Green, Blue }").unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::EnumDef(ed) => {
                assert_eq!(ed.name.node, "Color");
                assert_eq!(ed.variants.len(), 3);
            }
            _ => panic!("Expected enum definition"),
        }
    }

    #[test]
    fn test_parse_function_def() {
        let program = parse("def add(a: Int, b: Int) -> Int { a + b }").unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::FunctionDef(fd) => {
                assert_eq!(fd.name.node, "add");
                assert_eq!(fd.params.len(), 2);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_let_statement() {
        let program = parse("let x = 42").unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Statement(stmt) => match &stmt.kind {
                StmtKind::Let { name, .. } => assert_eq!(name.node, "x"),
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected statement"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let program = parse("if x > 0 { y } else { z }").unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_function_call() {
        let expr = parse_expression("foo(1, 2)").unwrap();
        match expr.kind {
            ExprKind::Call { args, .. } => assert_eq!(args.len(), 2),
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_field_access() {
        let expr = parse_expression("obj.field").unwrap();
        match expr.kind {
            ExprKind::Field { field, .. } => assert_eq!(field, "field"),
            _ => panic!("Expected field access"),
        }
    }

    #[test]
    fn test_parse_index_access() {
        let expr = parse_expression("arr[0]").unwrap();
        assert!(matches!(expr.kind, ExprKind::Index { .. }));
    }

    #[test]
    fn test_precedence() {
        // Test that * binds tighter than +
        let expr = parse_expression("1 + 2 * 3").unwrap();
        match expr.kind {
            ExprKind::Binary { op, right, .. } => {
                assert_eq!(op, BinaryOp::Add);
                assert!(matches!(right.kind, ExprKind::Binary { op: BinaryOp::Mul, .. }));
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_fstring_simple() {
        let expr = parse_expression(r#"f"hello world""#).unwrap();
        match expr.kind {
            ExprKind::FString(parts) => {
                assert_eq!(parts.len(), 1);
                match &parts[0] {
                    FStringPart::Text(s) => assert_eq!(s, "hello world"),
                    _ => panic!("Expected text part"),
                }
            }
            _ => panic!("Expected f-string"),
        }
    }

    #[test]
    fn test_parse_fstring_with_interpolation() {
        let expr = parse_expression(r#"f"hello {name}""#).unwrap();
        match expr.kind {
            ExprKind::FString(parts) => {
                assert_eq!(parts.len(), 2);
                match &parts[0] {
                    FStringPart::Text(s) => assert_eq!(s, "hello "),
                    _ => panic!("Expected text part"),
                }
                match &parts[1] {
                    FStringPart::Expr(expr) => {
                        assert!(matches!(expr.kind, ExprKind::Var(ref n) if n == "name"));
                    }
                    _ => panic!("Expected expression part"),
                }
            }
            _ => panic!("Expected f-string"),
        }
    }

    #[test]
    fn test_parse_fstring_complex() {
        let expr = parse_expression(r#"f"The answer is {40 + 2}!""#).unwrap();
        match expr.kind {
            ExprKind::FString(parts) => {
                assert_eq!(parts.len(), 3);
                match &parts[0] {
                    FStringPart::Text(s) => assert_eq!(s, "The answer is "),
                    _ => panic!("Expected text part"),
                }
                match &parts[1] {
                    FStringPart::Expr(expr) => {
                        assert!(matches!(expr.kind, ExprKind::Binary { .. }));
                    }
                    _ => panic!("Expected expression part"),
                }
                match &parts[2] {
                    FStringPart::Text(s) => assert_eq!(s, "!"),
                    _ => panic!("Expected text part"),
                }
            }
            _ => panic!("Expected f-string"),
        }
    }

    #[test]
    fn test_parse_fstring_empty() {
        let expr = parse_expression(r#"f"""#).unwrap();
        match expr.kind {
            ExprKind::FString(parts) => {
                assert_eq!(parts.len(), 0);
            }
            _ => panic!("Expected f-string"),
        }
    }

    #[test]
    fn test_parse_triple_fstring_simple() {
        let expr = parse_expression(r#"f"""hello world""""#).unwrap();
        match expr.kind {
            ExprKind::FString(parts) => {
                assert_eq!(parts.len(), 1);
                match &parts[0] {
                    FStringPart::Text(s) => assert_eq!(s, "hello world"),
                    _ => panic!("Expected text part"),
                }
            }
            _ => panic!("Expected f-string"),
        }
    }

    #[test]
    fn test_parse_triple_fstring_with_interpolation() {
        let expr = parse_expression(r#"f"""hello {name}""""#).unwrap();
        match expr.kind {
            ExprKind::FString(parts) => {
                assert_eq!(parts.len(), 2);
                match &parts[0] {
                    FStringPart::Text(s) => assert_eq!(s, "hello "),
                    _ => panic!("Expected text part"),
                }
                match &parts[1] {
                    FStringPart::Expr(expr) => {
                        assert!(matches!(expr.kind, ExprKind::Var(ref n) if n == "name"));
                    }
                    _ => panic!("Expected expression part"),
                }
            }
            _ => panic!("Expected f-string"),
        }
    }

    #[test]
    fn test_parse_triple_fstring_multiline() {
        let expr = parse_expression("f\"\"\"line1\nline2\nline3\"\"\"").unwrap();
        match expr.kind {
            ExprKind::FString(parts) => {
                assert_eq!(parts.len(), 1);
                match &parts[0] {
                    FStringPart::Text(s) => assert_eq!(s, "line1\nline2\nline3"),
                    _ => panic!("Expected text part"),
                }
            }
            _ => panic!("Expected f-string"),
        }
    }

    #[test]
    fn test_parse_triple_fstring_with_quotes() {
        // Test with embedded double quote (not adjacent to closing triple quote)
        let expr = parse_expression(r#"f"""She said "hello" today""""#).unwrap();
        match expr.kind {
            ExprKind::FString(parts) => {
                assert_eq!(parts.len(), 1);
                match &parts[0] {
                    FStringPart::Text(s) => assert_eq!(s, r#"She said "hello" today"#),
                    _ => panic!("Expected text part"),
                }
            }
            _ => panic!("Expected f-string"),
        }
    }

    #[test]
    fn test_parse_triple_fstring_empty() {
        let expr = parse_expression(r#"f"""""""#).unwrap();
        match expr.kind {
            ExprKind::FString(parts) => {
                assert_eq!(parts.len(), 0);
            }
            _ => panic!("Expected f-string"),
        }
    }

    // ========================================================================
    // Pipe Operator Tests
    // ========================================================================

    #[test]
    fn test_parse_pipe_simple() {
        // x |> f => f(x)
        let expr = parse_expression("x |> f").unwrap();
        match expr.kind {
            ExprKind::Call { callee, args } => {
                assert!(matches!(callee.kind, ExprKind::Var(ref n) if n == "f"));
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0].kind, ExprKind::Var(ref n) if n == "x"));
            }
            _ => panic!("Expected call expression"),
        }
    }

    #[test]
    fn test_parse_pipe_with_args() {
        // x |> f(a, b) => f(x, a, b)
        let expr = parse_expression("x |> f(a, b)").unwrap();
        match expr.kind {
            ExprKind::Call { callee, args } => {
                assert!(matches!(callee.kind, ExprKind::Var(ref n) if n == "f"));
                assert_eq!(args.len(), 3);
                assert!(matches!(args[0].kind, ExprKind::Var(ref n) if n == "x"));
                assert!(matches!(args[1].kind, ExprKind::Var(ref n) if n == "a"));
                assert!(matches!(args[2].kind, ExprKind::Var(ref n) if n == "b"));
            }
            _ => panic!("Expected call expression"),
        }
    }

    #[test]
    fn test_parse_pipe_chain() {
        // x |> f |> g => g(f(x))
        let expr = parse_expression("x |> f |> g").unwrap();
        match expr.kind {
            ExprKind::Call { callee, args } => {
                assert!(matches!(callee.kind, ExprKind::Var(ref n) if n == "g"));
                assert_eq!(args.len(), 1);
                // The arg should be f(x)
                match &args[0].kind {
                    ExprKind::Call { callee: inner_callee, args: inner_args } => {
                        assert!(matches!(inner_callee.kind, ExprKind::Var(ref n) if n == "f"));
                        assert_eq!(inner_args.len(), 1);
                        assert!(matches!(inner_args[0].kind, ExprKind::Var(ref n) if n == "x"));
                    }
                    _ => panic!("Expected inner call"),
                }
            }
            _ => panic!("Expected call expression"),
        }
    }

    #[test]
    fn test_parse_pipe_with_lambda() {
        // x |> |y| y + 1 => (|y| y + 1)(x)
        let expr = parse_expression("x |> |y| y + 1").unwrap();
        match expr.kind {
            ExprKind::Call { callee, args } => {
                assert!(matches!(callee.kind, ExprKind::Lambda { .. }));
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0].kind, ExprKind::Var(ref n) if n == "x"));
            }
            _ => panic!("Expected call expression"),
        }
    }

    // ========================================================================
    // map_column Tests
    // ========================================================================

    #[test]
    fn test_parse_map_column_full() {
        let expr = parse_expression(r#"map_column(data, "input", "output", |x| x + 1)"#).unwrap();
        match expr.kind {
            ExprKind::MapColumn { table, input_col, output_col, mapper } => {
                assert!(matches!(table.kind, ExprKind::Var(ref n) if n == "data"));
                assert!(matches!(input_col.kind, ExprKind::Literal(Literal::String(ref s)) if s == "input"));
                assert!(matches!(output_col.kind, ExprKind::Literal(Literal::String(ref s)) if s == "output"));
                assert!(matches!(mapper.kind, ExprKind::Lambda { .. }));
            }
            _ => panic!("Expected MapColumn expression"),
        }
    }

    #[test]
    fn test_parse_map_column_pipe() {
        // data |> map_column("input", "output", |x| x + 1)
        let expr = parse_expression(r#"data |> map_column("input", "output", |x| x + 1)"#).unwrap();
        match expr.kind {
            ExprKind::MapColumn { table, input_col, output_col, mapper } => {
                assert!(matches!(table.kind, ExprKind::Var(ref n) if n == "data"));
                assert!(matches!(input_col.kind, ExprKind::Literal(Literal::String(ref s)) if s == "input"));
                assert!(matches!(output_col.kind, ExprKind::Literal(Literal::String(ref s)) if s == "output"));
                assert!(matches!(mapper.kind, ExprKind::Lambda { .. }));
            }
            _ => panic!("Expected MapColumn expression"),
        }
    }

    #[test]
    fn test_parse_map_column_with_function_call() {
        let expr = parse_expression(r#"map_column(data, "col", "out", |x| process(x))"#).unwrap();
        match expr.kind {
            ExprKind::MapColumn { mapper, .. } => {
                match mapper.kind {
                    ExprKind::Lambda { params, body } => {
                        assert_eq!(params.len(), 1);
                        assert_eq!(params[0], "x");
                        match body.as_ref() {
                            LambdaBody::Expr(expr) => {
                                assert!(matches!(expr.kind, ExprKind::Call { .. }));
                            }
                            _ => panic!("Expected expression body"),
                        }
                    }
                    _ => panic!("Expected lambda"),
                }
            }
            _ => panic!("Expected MapColumn expression"),
        }
    }

    // ========================================================================
    // map_row Tests
    // ========================================================================

    #[test]
    fn test_parse_map_row_full() {
        let expr = parse_expression(r#"map_row(data, "output", |row| row["a"] + row["b"])"#).unwrap();
        match expr.kind {
            ExprKind::MapRow { table, output_col, mapper } => {
                assert!(matches!(table.kind, ExprKind::Var(ref n) if n == "data"));
                assert!(matches!(output_col.kind, ExprKind::Literal(Literal::String(ref s)) if s == "output"));
                assert!(matches!(mapper.kind, ExprKind::Lambda { .. }));
            }
            _ => panic!("Expected MapRow expression"),
        }
    }

    #[test]
    fn test_parse_map_row_pipe() {
        // data |> map_row("output", |row| process(row))
        let expr = parse_expression(r#"data |> map_row("output", |row| process(row))"#).unwrap();
        match expr.kind {
            ExprKind::MapRow { table, output_col, mapper } => {
                assert!(matches!(table.kind, ExprKind::Var(ref n) if n == "data"));
                assert!(matches!(output_col.kind, ExprKind::Literal(Literal::String(ref s)) if s == "output"));
                match mapper.kind {
                    ExprKind::Lambda { params, .. } => {
                        assert_eq!(params.len(), 1);
                        assert_eq!(params[0], "row");
                    }
                    _ => panic!("Expected lambda"),
                }
            }
            _ => panic!("Expected MapRow expression"),
        }
    }

    #[test]
    fn test_parse_map_row_chain() {
        // data |> map_row("a", |r| r["x"]) |> map_row("b", |r| r["y"])
        let expr = parse_expression(r#"data |> map_row("a", |r| r["x"]) |> map_row("b", |r| r["y"])"#).unwrap();
        match expr.kind {
            ExprKind::MapRow { table, output_col, .. } => {
                assert!(matches!(output_col.kind, ExprKind::Literal(Literal::String(ref s)) if s == "b"));
                // The table should be another MapRow
                assert!(matches!(table.kind, ExprKind::MapRow { .. }));
            }
            _ => panic!("Expected MapRow expression"),
        }
    }

    #[test]
    fn test_parse_map_column_and_row_chain() {
        // data |> map_column("a", "b", |x| x) |> map_row("c", |r| r["b"])
        let expr = parse_expression(r#"data |> map_column("a", "b", |x| x) |> map_row("c", |r| r["b"])"#).unwrap();
        match expr.kind {
            ExprKind::MapRow { table, .. } => {
                // The table should be a MapColumn
                assert!(matches!(table.kind, ExprKind::MapColumn { .. }));
            }
            _ => panic!("Expected MapRow expression"),
        }
    }
}
