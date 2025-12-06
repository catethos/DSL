//! Desugaring pass for Lattice AST
//!
//! This module handles syntactic sugar transformations, including:
//! - `$field` syntax for implicit row accessors

use super::ast::*;

/// Check if an expression contains any DollarField nodes
pub fn contains_dollar_field(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::DollarField(_) => true,
        ExprKind::Literal(_) => false,
        ExprKind::Var(_) => false,
        ExprKind::EnumVariant { .. } => false,
        ExprKind::Binary { left, right, .. } => {
            contains_dollar_field(left) || contains_dollar_field(right)
        }
        ExprKind::Unary { operand, .. } => contains_dollar_field(operand),
        ExprKind::Field { object, .. } => contains_dollar_field(object),
        ExprKind::Index { object, index } => {
            contains_dollar_field(object) || contains_dollar_field(index)
        }
        ExprKind::Call { callee, args } => {
            contains_dollar_field(callee) || args.iter().any(contains_dollar_field)
        }
        ExprKind::List(items) => items.iter().any(contains_dollar_field),
        ExprKind::Map(pairs) => pairs.iter().any(|(_, v)| contains_dollar_field(v)),
        ExprKind::Struct { fields, .. } => fields.iter().any(|(_, v)| contains_dollar_field(v)),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            contains_dollar_field(condition)
                || block_contains_dollar(then_branch)
                || else_branch
                    .as_ref()
                    .map(if_else_contains_dollar)
                    .unwrap_or(false)
        }
        ExprKind::Match { scrutinee, arms } => {
            contains_dollar_field(scrutinee)
                || arms.iter().any(|arm| match &arm.body {
                    MatchArmBody::Expr(e) => contains_dollar_field(e),
                    MatchArmBody::Block(b) => block_contains_dollar(b),
                })
        }
        ExprKind::Lambda { body, .. } => match body.as_ref() {
            LambdaBody::Expr(e) => contains_dollar_field(e),
            LambdaBody::Block(b) => block_contains_dollar(b),
        },
        ExprKind::Parallel(exprs) => exprs.iter().any(contains_dollar_field),
        ExprKind::ParallelMap { collection, mapper } => {
            contains_dollar_field(collection) || contains_dollar_field(mapper)
        }
        ExprKind::MapColumn {
            table,
            input_col,
            output_col,
            mapper,
        } => {
            contains_dollar_field(table)
                || contains_dollar_field(input_col)
                || contains_dollar_field(output_col)
                || contains_dollar_field(mapper)
        }
        ExprKind::MapRow {
            table,
            output_col,
            mapper,
        } => {
            contains_dollar_field(table)
                || contains_dollar_field(output_col)
                || contains_dollar_field(mapper)
        }
        ExprKind::Explode {
            table,
            column,
            prefix,
        } => {
            contains_dollar_field(table)
                || contains_dollar_field(column)
                || prefix
                    .as_ref()
                    .map(|p| contains_dollar_field(p))
                    .unwrap_or(false)
        }
        ExprKind::Sql { query, .. } => contains_dollar_field(query),
        ExprKind::Grouped(inner) => contains_dollar_field(inner),
        ExprKind::Block(block) => block_contains_dollar(block),
        ExprKind::FString(parts) => parts.iter().any(|p| match p {
            FStringPart::Text(_) => false,
            FStringPart::Expr(e) => contains_dollar_field(e),
        }),
    }
}

fn block_contains_dollar(block: &Block) -> bool {
    block.stmts.iter().any(stmt_contains_dollar)
        || block
            .expr
            .as_ref()
            .map(|e| contains_dollar_field(e))
            .unwrap_or(false)
}

fn stmt_contains_dollar(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::Let { value, .. } => contains_dollar_field(value),
        StmtKind::Assign { value, .. } => contains_dollar_field(value),
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            contains_dollar_field(condition)
                || block_contains_dollar(then_branch)
                || else_branch
                    .as_ref()
                    .map(else_clause_contains_dollar)
                    .unwrap_or(false)
        }
        StmtKind::While { condition, body } => {
            contains_dollar_field(condition) || block_contains_dollar(body)
        }
        StmtKind::For { iterable, body, .. } => {
            contains_dollar_field(iterable) || block_contains_dollar(body)
        }
        StmtKind::Return { value } => value
            .as_ref()
            .map(contains_dollar_field)
            .unwrap_or(false),
        StmtKind::Expr { expr } => contains_dollar_field(expr),
    }
}

fn else_clause_contains_dollar(clause: &ElseClause) -> bool {
    match clause {
        ElseClause::ElseIf(stmt) => stmt_contains_dollar(stmt),
        ElseClause::Else(block) => block_contains_dollar(block),
    }
}

fn if_else_contains_dollar(clause: &IfExprElse) -> bool {
    match clause {
        IfExprElse::ElseIf(expr) => contains_dollar_field(expr),
        IfExprElse::Else(block) => block_contains_dollar(block),
    }
}

/// Transform an expression by replacing all DollarField nodes with row index access.
/// `$field` becomes `__row__["field"]`
/// `$[expr]` becomes `__row__[expr]`
pub fn replace_dollar_fields(expr: &Expr) -> Expr {
    let span = expr.span;
    let kind = match &expr.kind {
        ExprKind::DollarField(field_expr) => {
            // Transform $field to __row__[field]
            let row_var = Expr {
                kind: ExprKind::Var("__row__".to_string()),
                span,
            };
            ExprKind::Index {
                object: Box::new(row_var),
                index: Box::new(replace_dollar_fields(field_expr)),
            }
        }
        ExprKind::Literal(lit) => ExprKind::Literal(lit.clone()),
        ExprKind::Var(name) => ExprKind::Var(name.clone()),
        ExprKind::EnumVariant { enum_name, variant } => ExprKind::EnumVariant {
            enum_name: enum_name.clone(),
            variant: variant.clone(),
        },
        ExprKind::Binary { left, op, right } => ExprKind::Binary {
            left: Box::new(replace_dollar_fields(left)),
            op: *op,
            right: Box::new(replace_dollar_fields(right)),
        },
        ExprKind::Unary { op, operand } => ExprKind::Unary {
            op: *op,
            operand: Box::new(replace_dollar_fields(operand)),
        },
        ExprKind::Field { object, field } => ExprKind::Field {
            object: Box::new(replace_dollar_fields(object)),
            field: field.clone(),
        },
        ExprKind::Index { object, index } => ExprKind::Index {
            object: Box::new(replace_dollar_fields(object)),
            index: Box::new(replace_dollar_fields(index)),
        },
        ExprKind::Call { callee, args } => ExprKind::Call {
            callee: Box::new(replace_dollar_fields(callee)),
            args: args.iter().map(replace_dollar_fields).collect(),
        },
        ExprKind::List(items) => ExprKind::List(items.iter().map(replace_dollar_fields).collect()),
        ExprKind::Map(pairs) => ExprKind::Map(
            pairs
                .iter()
                .map(|(k, v)| (k.clone(), replace_dollar_fields(v)))
                .collect(),
        ),
        ExprKind::Struct { name, fields } => ExprKind::Struct {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(n, v)| (n.clone(), replace_dollar_fields(v)))
                .collect(),
        },
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => ExprKind::If {
            condition: Box::new(replace_dollar_fields(condition)),
            then_branch: replace_dollar_in_block(then_branch),
            else_branch: else_branch.as_ref().map(replace_dollar_in_if_else),
        },
        ExprKind::Match { scrutinee, arms } => ExprKind::Match {
            scrutinee: Box::new(replace_dollar_fields(scrutinee)),
            arms: arms
                .iter()
                .map(|arm| MatchArm {
                    pattern: arm.pattern.clone(),
                    body: match &arm.body {
                        MatchArmBody::Expr(e) => MatchArmBody::Expr(replace_dollar_fields(e)),
                        MatchArmBody::Block(b) => MatchArmBody::Block(replace_dollar_in_block(b)),
                    },
                    span: arm.span,
                })
                .collect(),
        },
        ExprKind::Lambda { params, body } => ExprKind::Lambda {
            params: params.clone(),
            body: Box::new(match body.as_ref() {
                LambdaBody::Expr(e) => LambdaBody::Expr(replace_dollar_fields(e)),
                LambdaBody::Block(b) => LambdaBody::Block(replace_dollar_in_block(b)),
            }),
        },
        ExprKind::Parallel(exprs) => {
            ExprKind::Parallel(exprs.iter().map(replace_dollar_fields).collect())
        }
        ExprKind::ParallelMap { collection, mapper } => ExprKind::ParallelMap {
            collection: Box::new(replace_dollar_fields(collection)),
            mapper: Box::new(replace_dollar_fields(mapper)),
        },
        ExprKind::MapColumn {
            table,
            input_col,
            output_col,
            mapper,
        } => ExprKind::MapColumn {
            table: Box::new(replace_dollar_fields(table)),
            input_col: Box::new(replace_dollar_fields(input_col)),
            output_col: Box::new(replace_dollar_fields(output_col)),
            mapper: Box::new(replace_dollar_fields(mapper)),
        },
        ExprKind::MapRow {
            table,
            output_col,
            mapper,
        } => ExprKind::MapRow {
            table: Box::new(replace_dollar_fields(table)),
            output_col: Box::new(replace_dollar_fields(output_col)),
            mapper: Box::new(replace_dollar_fields(mapper)),
        },
        ExprKind::Explode {
            table,
            column,
            prefix,
        } => ExprKind::Explode {
            table: Box::new(replace_dollar_fields(table)),
            column: Box::new(replace_dollar_fields(column)),
            prefix: prefix.as_ref().map(|p| Box::new(replace_dollar_fields(p))),
        },
        ExprKind::Sql { ty, query } => ExprKind::Sql {
            ty: ty.clone(),
            query: Box::new(replace_dollar_fields(query)),
        },
        ExprKind::Grouped(inner) => ExprKind::Grouped(Box::new(replace_dollar_fields(inner))),
        ExprKind::Block(block) => ExprKind::Block(replace_dollar_in_block(block)),
        ExprKind::FString(parts) => ExprKind::FString(
            parts
                .iter()
                .map(|p| match p {
                    FStringPart::Text(t) => FStringPart::Text(t.clone()),
                    FStringPart::Expr(e) => FStringPart::Expr(replace_dollar_fields(e)),
                })
                .collect(),
        ),
    };
    Expr { kind, span }
}

fn replace_dollar_in_block(block: &Block) -> Block {
    Block {
        stmts: block.stmts.iter().map(replace_dollar_in_stmt).collect(),
        expr: block.expr.as_ref().map(|e| Box::new(replace_dollar_fields(e))),
        span: block.span,
    }
}

fn replace_dollar_in_stmt(stmt: &Stmt) -> Stmt {
    let kind = match &stmt.kind {
        StmtKind::Let { name, ty, value } => StmtKind::Let {
            name: name.clone(),
            ty: ty.clone(),
            value: replace_dollar_fields(value),
        },
        StmtKind::Assign { target, value } => StmtKind::Assign {
            target: target.clone(),
            value: replace_dollar_fields(value),
        },
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => StmtKind::If {
            condition: replace_dollar_fields(condition),
            then_branch: replace_dollar_in_block(then_branch),
            else_branch: else_branch.as_ref().map(replace_dollar_in_else_clause),
        },
        StmtKind::While { condition, body } => StmtKind::While {
            condition: replace_dollar_fields(condition),
            body: replace_dollar_in_block(body),
        },
        StmtKind::For { var, iterable, body } => StmtKind::For {
            var: var.clone(),
            iterable: replace_dollar_fields(iterable),
            body: replace_dollar_in_block(body),
        },
        StmtKind::Return { value } => StmtKind::Return {
            value: value.as_ref().map(replace_dollar_fields),
        },
        StmtKind::Expr { expr } => StmtKind::Expr {
            expr: replace_dollar_fields(expr),
        },
    };
    Stmt {
        kind,
        span: stmt.span,
    }
}

fn replace_dollar_in_else_clause(clause: &ElseClause) -> ElseClause {
    match clause {
        ElseClause::ElseIf(stmt) => ElseClause::ElseIf(Box::new(replace_dollar_in_stmt(stmt))),
        ElseClause::Else(block) => ElseClause::Else(replace_dollar_in_block(block)),
    }
}

fn replace_dollar_in_if_else(clause: &IfExprElse) -> IfExprElse {
    match clause {
        IfExprElse::ElseIf(expr) => IfExprElse::ElseIf(Box::new(replace_dollar_fields(expr))),
        IfExprElse::Else(block) => IfExprElse::Else(replace_dollar_in_block(block)),
    }
}

/// Wrap an expression containing $field references into a lambda.
/// If the expression contains any $field, it becomes:
///   |__row__| <expression with $field replaced by __row__["field"]>
///
/// If the expression doesn't contain $field, it's returned unchanged.
pub fn wrap_dollar_expr_in_lambda(expr: &Expr) -> Expr {
    if !contains_dollar_field(expr) {
        return expr.clone();
    }

    let span = expr.span;
    let transformed = replace_dollar_fields(expr);

    Expr {
        kind: ExprKind::Lambda {
            params: vec!["__row__".to_string()],
            body: Box::new(LambdaBody::Expr(transformed)),
        },
        span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dollar_field(name: &str) -> Expr {
        Expr {
            kind: ExprKind::DollarField(Box::new(Expr {
                kind: ExprKind::Literal(Literal::String(name.to_string())),
                span: Span::default(),
            })),
            span: Span::default(),
        }
    }

    fn make_var(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Var(name.to_string()),
            span: Span::default(),
        }
    }

    #[test]
    fn test_contains_dollar_field() {
        let expr = make_dollar_field("name");
        assert!(contains_dollar_field(&expr));

        let var = make_var("x");
        assert!(!contains_dollar_field(&var));
    }

    #[test]
    fn test_contains_dollar_in_binary() {
        let dollar = make_dollar_field("a");
        let var = make_var("b");
        let binary = Expr {
            kind: ExprKind::Binary {
                left: Box::new(dollar),
                op: BinaryOp::Add,
                right: Box::new(var),
            },
            span: Span::default(),
        };
        assert!(contains_dollar_field(&binary));
    }

    #[test]
    fn test_replace_dollar_fields() {
        let dollar = make_dollar_field("name");
        let result = replace_dollar_fields(&dollar);

        // Should become __row__["name"]
        match result.kind {
            ExprKind::Index { object, index } => {
                match object.kind {
                    ExprKind::Var(ref n) => assert_eq!(n, "__row__"),
                    _ => panic!("Expected Var(__row__)"),
                }
                match index.kind {
                    ExprKind::Literal(Literal::String(ref s)) => assert_eq!(s, "name"),
                    _ => panic!("Expected string literal"),
                }
            }
            _ => panic!("Expected Index expression"),
        }
    }

    #[test]
    fn test_wrap_in_lambda() {
        let dollar = make_dollar_field("field");
        let wrapped = wrap_dollar_expr_in_lambda(&dollar);

        match wrapped.kind {
            ExprKind::Lambda { params, .. } => {
                assert_eq!(params, vec!["__row__".to_string()]);
            }
            _ => panic!("Expected Lambda"),
        }
    }

    #[test]
    fn test_no_wrap_when_no_dollar() {
        let var = make_var("x");
        let result = wrap_dollar_expr_in_lambda(&var);

        // Should return unchanged
        match result.kind {
            ExprKind::Var(ref n) => assert_eq!(n, "x"),
            _ => panic!("Expected unchanged Var"),
        }
    }
}
