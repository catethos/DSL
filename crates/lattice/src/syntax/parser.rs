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
        Rule::llm_config => Ok(FunctionBody::LlmConfig(parse_llm_config(inner)?)),
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
        base_url: None,
        model: None,
        api_key_env: None,
        temperature: None,
        max_tokens: None,
        prompt: placeholder_prompt,
        span,
    };

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::config_field => {
                let mut inner = p.into_inner();
                let key = inner.next().unwrap().as_str();
                let value_pair = inner.next().unwrap();
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
    let span = make_span(&pair);

    match pair.as_rule() {
        Rule::expression | Rule::or_expr => parse_or_expr(pair),
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
                Err(LatticeError::Parse(format!(
                    "Unexpected expression rule"
                )))
            }
        }
    }
}

fn parse_or_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let mut left = parse_expr(inner.next().unwrap())?;

    while let Some(right_pair) = inner.next() {
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
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

    let mut left = parse_expr(inner.next().unwrap())?;

    while let Some(right_pair) = inner.next() {
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
    let span = make_span(&pair);
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
    let span = make_span(&pair);
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
    let span = make_span(&pair);
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
    let span = make_span(&pair);
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
    let span = make_span(&pair);
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
        Rule::sql_expr => parse_sql_expr(inner),
        Rule::lambda_expr => parse_lambda_expr(inner),
        Rule::list_literal => parse_list_literal(inner),
        Rule::map_literal => parse_map_literal(inner),
        Rule::struct_literal => parse_struct_literal(inner),
        Rule::literal => parse_literal_expr(inner),
        Rule::enum_constructor => parse_enum_constructor(inner),
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
    let mapper = parse_expr(inner.next().unwrap())?;

    Ok(Expr {
        kind: ExprKind::ParallelMap {
            collection: Box::new(collection),
            mapper: Box::new(mapper),
        },
        span,
    })
}

fn parse_sql_expr(pair: Pair<Rule>) -> Result<Expr> {
    let span = make_span(&pair);
    let mut inner = pair.into_inner();

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
    let mut inner = pair.into_inner();

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
            Rule::fstring_part => {
                let inner = p.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::fstring_text => {
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
            Rule::fstring_text => {
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
}
