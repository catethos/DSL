use crate::rust_ast::*;
use anyhow::{anyhow, Result};
use dsl_ir::{IRBinding, IRNode, IRTemplateSegment};

pub fn generate_expr(node: &IRNode) -> Result<RustExpr> {
    match node {
        IRNode::String(s) => Ok(RustExpr::Literal(RustLiteral::String(s.clone()))),

        IRNode::TemplateString(segments) => generate_template_string(segments),

        IRNode::Int(i) => Ok(RustExpr::Literal(RustLiteral::Int(*i))),
        IRNode::Float(f) => Ok(RustExpr::Literal(RustLiteral::Float(*f))),
        IRNode::Bool(b) => Ok(RustExpr::Literal(RustLiteral::Bool(*b))),

        IRNode::List(items) => {
            let item_exprs = items
                .iter()
                .map(generate_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(RustExpr::Vec(item_exprs))
        }

        IRNode::Map(entries) => generate_map(entries),

        IRNode::Variable(name) => Ok(RustExpr::Variable(name.clone())),

        IRNode::FunctionCall { name, args } => {
            let arg_exprs = args
                .iter()
                .map(generate_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(RustExpr::Call {
                func: Box::new(RustExpr::Variable(name.clone())),
                args: arg_exprs,
            })
        }

        IRNode::TypeInstantiation { type_name, fields } => {
            let field_exprs = fields
                .iter()
                .map(|(k, v)| Ok((k.clone(), generate_expr(v)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok(RustExpr::Struct {
                name: type_name.clone(),
                fields: field_exprs,
            })
        }

        IRNode::FieldAccess { base, field } => Ok(RustExpr::FieldAccess {
            base: Box::new(generate_expr(base)?),
            field: field.clone(),
        }),

        IRNode::IndexAccess { base, index } => Ok(RustExpr::IndexAccess {
            base: Box::new(generate_expr(base)?),
            index: Box::new(generate_expr(index)?),
        }),

        IRNode::BinaryOp { left, op, right } => Ok(RustExpr::BinaryOp {
            left: Box::new(generate_expr(left)?),
            op: op.clone(),
            right: Box::new(generate_expr(right)?),
        }),

        IRNode::Conditional {
            condition,
            then_expr,
            else_expr,
        } => generate_conditional(condition, then_expr, else_expr),

        IRNode::Sequential {
            left,
            right,
            binding,
        } => generate_sequential(left, right, binding),

        IRNode::Parallel { exprs, binding } => generate_parallel(exprs, binding),

        IRNode::Loop { body } => {
            let body_expr = generate_expr(body)?;
            let body_stmts = vec![RustStmt::Expr(body_expr)];
            Ok(RustExpr::Loop { body: body_stmts })
        }

        IRNode::While { condition, body } => {
            let cond_expr = generate_expr(condition)?;
            let body_expr = generate_expr(body)?;
            let body_stmts = vec![RustStmt::Expr(body_expr)];
            Ok(RustExpr::While {
                condition: Box::new(cond_expr),
                body: body_stmts,
            })
        }

        IRNode::For { var, iterable, body } => {
            let iter_expr = generate_expr(iterable)?;
            let body_expr = generate_expr(body)?;
            let body_stmts = vec![RustStmt::Expr(body_expr)];
            Ok(RustExpr::For {
                var: var.clone(),
                iter: Box::new(iter_expr),
                body: body_stmts,
            })
        }

        IRNode::Break { value } => {
            if let Some(v) = value {
                Ok(RustExpr::Break(Some(Box::new(generate_expr(v)?))))
            } else {
                Ok(RustExpr::Break(None))
            }
        }

        IRNode::Continue => Ok(RustExpr::Continue),

        IRNode::TryBlock {
            body,
            catch_var,
            catch_body,
        } => generate_try_block(body, catch_var, catch_body),

        IRNode::Throw { error } => {
            let error_expr = generate_expr(error)?;
            Ok(RustExpr::Call {
                func: Box::new(RustExpr::Variable("Err".to_string())),
                args: vec![error_expr],
            })
        }

        _ => Err(anyhow!("Unsupported IR node: {:?}", node)),
    }
}

fn generate_template_string(segments: &[IRTemplateSegment]) -> Result<RustExpr> {
    if segments.is_empty() {
        return Ok(RustExpr::Literal(RustLiteral::String(String::new())));
    }

    let mut format_str = String::new();
    let mut args = Vec::new();

    for segment in segments {
        match segment {
            IRTemplateSegment::Text(t) => {
                format_str.push_str(&t.replace('{', "{{").replace('}', "}}"));
            }
            IRTemplateSegment::Interpolation(var_name) => {
                format_str.push_str("{}");
                args.push(RustExpr::Variable(var_name.clone()));
            }
        }
    }

    let mut format_args = vec![RustExpr::Literal(RustLiteral::String(format_str))];
    format_args.extend(args);

    Ok(RustExpr::Macro {
        name: "format".to_string(),
        args: format_args,
    })
}

fn generate_map(entries: &[(String, IRNode)]) -> Result<RustExpr> {
    let mut stmts = Vec::new();

    let map_var = "_map";
    stmts.push(RustStmt::Let {
        name: map_var.to_string(),
        ty: Some(RustType::Named("indexmap::IndexMap<String, Value>".to_string())),
        value: RustExpr::Call {
            func: Box::new(RustExpr::Variable("indexmap::IndexMap::new".to_string())),
            args: vec![],
        },
        is_mut: true,
    });

    for (key, value) in entries {
        let value_expr = generate_expr(value)?;
        stmts.push(RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Variable(map_var.to_string())),
            method: "insert".to_string(),
            args: vec![
                RustExpr::Call {
                    func: Box::new(RustExpr::Variable("String::from".to_string())),
                    args: vec![RustExpr::Literal(RustLiteral::String(key.clone()))],
                },
                value_expr,
            ],
        }));
    }

    Ok(RustExpr::Block(
        stmts,
        Some(Box::new(RustExpr::Variable(map_var.to_string()))),
    ))
}

fn generate_conditional(
    condition: &IRNode,
    then_expr: &IRNode,
    else_expr: &IRNode,
) -> Result<RustExpr> {
    let cond = generate_expr(condition)?;
    let then_result = generate_expr(then_expr)?;
    let else_result = generate_expr(else_expr)?;

    Ok(RustExpr::If {
        condition: Box::new(cond),
        then_block: vec![RustStmt::Expr(then_result)],
        else_block: Some(vec![RustStmt::Expr(else_result)]),
    })
}

fn generate_sequential(
    left: &IRNode,
    right: &IRNode,
    binding: &Option<IRBinding>,
) -> Result<RustExpr> {
    let mut stmts = Vec::new();

    let left_expr = generate_expr(left)?;

    if let Some(binding) = binding {
        match binding {
            IRBinding::Single(name) => {
                stmts.push(RustStmt::Let {
                    name: name.clone(),
                    ty: None,
                    value: RustExpr::Await(Box::new(left_expr)),
                    is_mut: false,
                });
            }
            IRBinding::List(names) => {
                stmts.push(RustStmt::Let {
                    name: format!("({})", names.join(", ")),
                    ty: None,
                    value: RustExpr::Await(Box::new(left_expr)),
                    is_mut: false,
                });
            }
        }
    } else {
        stmts.push(RustStmt::Let {
            name: "_".to_string(),
            ty: None,
            value: RustExpr::Await(Box::new(left_expr)),
            is_mut: false,
        });
    }

    let right_expr = generate_expr(right)?;

    Ok(RustExpr::Block(stmts, Some(Box::new(right_expr))))
}

fn generate_parallel(exprs: &[IRNode], binding: &Option<IRBinding>) -> Result<RustExpr> {
    let expr_futures = exprs
        .iter()
        .map(generate_expr)
        .collect::<Result<Vec<_>>>()?;

    let join_expr = RustExpr::Macro {
        name: "tokio::join".to_string(),
        args: expr_futures,
    };

    if let Some(binding) = binding {
        let mut stmts = Vec::new();
        match binding {
            IRBinding::Single(name) => {
                stmts.push(RustStmt::Let {
                    name: name.clone(),
                    ty: None,
                    value: join_expr,
                    is_mut: false,
                });
            }
            IRBinding::List(names) => {
                stmts.push(RustStmt::Let {
                    name: format!("({})", names.join(", ")),
                    ty: None,
                    value: join_expr,
                    is_mut: false,
                });
            }
        }

        Ok(RustExpr::Block(
            stmts,
            Some(Box::new(RustExpr::Variable(match binding {
                IRBinding::Single(name) => name.clone(),
                IRBinding::List(names) => names[0].clone(),
            }))),
        ))
    } else {
        Ok(join_expr)
    }
}

fn generate_try_block(
    body: &IRNode,
    catch_var: &str,
    catch_body: &IRNode,
) -> Result<RustExpr> {
    let body_expr = generate_expr(body)?;
    let catch_expr = generate_expr(catch_body)?;

    let match_arms = vec![
        RustMatchArm {
            pattern: RustPattern::Struct {
                name: "Ok".to_string(),
                fields: vec![("0".to_string(), RustPattern::Binding("_val".to_string()))],
            },
            guard: None,
            body: RustExpr::Variable("_val".to_string()),
        },
        RustMatchArm {
            pattern: RustPattern::Struct {
                name: "Err".to_string(),
                fields: vec![("0".to_string(), RustPattern::Binding(catch_var.to_string()))],
            },
            guard: None,
            body: catch_expr,
        },
    ];

    Ok(RustExpr::Match {
        expr: Box::new(body_expr),
        arms: match_arms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_string_literal() {
        let node = IRNode::String("hello".to_string());
        let expr = generate_expr(&node).unwrap();
        assert_eq!(
            expr,
            RustExpr::Literal(RustLiteral::String("hello".to_string()))
        );
    }

    #[test]
    fn test_generate_int_literal() {
        let node = IRNode::Int(42);
        let expr = generate_expr(&node).unwrap();
        assert_eq!(expr, RustExpr::Literal(RustLiteral::Int(42)));
    }

    #[test]
    fn test_generate_list() {
        let node = IRNode::List(vec![IRNode::Int(1), IRNode::Int(2), IRNode::Int(3)]);
        let expr = generate_expr(&node).unwrap();
        match expr {
            RustExpr::Vec(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected Vec expression"),
        }
    }

    #[test]
    fn test_generate_function_call() {
        let node = IRNode::FunctionCall {
            name: "foo".to_string(),
            args: vec![IRNode::Int(1), IRNode::Int(2)],
        };
        let expr = generate_expr(&node).unwrap();
        match expr {
            RustExpr::Call { func: _, args } => {
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_generate_binary_op() {
        let node = IRNode::BinaryOp {
            left: Box::new(IRNode::Int(1)),
            op: "+".to_string(),
            right: Box::new(IRNode::Int(2)),
        };
        let expr = generate_expr(&node).unwrap();
        match expr {
            RustExpr::BinaryOp { left: _, op, right: _ } => {
                assert_eq!(op, "+");
            }
            _ => panic!("Expected BinaryOp expression"),
        }
    }
}
