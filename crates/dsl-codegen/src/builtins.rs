use crate::rust_ast::*;

pub fn generate_builtins_module() -> Vec<RustItem> {
    vec![
        RustItem::Use("dsl_ir::Value".to_string()),
        RustItem::Use("anyhow::Result".to_string()),
        generate_ask(),
        generate_extract_as(),
        generate_extract_person(),
        generate_length(),
        generate_upper(),
        generate_lower(),
        generate_join(),
        generate_sql(),
        generate_par(),
        generate_not(),
        generate_render_markdown(),
    ]
}

fn generate_ask() -> RustItem {
    RustItem::Function(RustFunction {
        name: "ask".to_string(),
        params: vec![RustParam {
            name: "prompt".to_string(),
            ty: RustType::Reference(false, Box::new(RustType::Named("str".to_string()))),
            is_mut: false,
            is_ref: false,
        }],
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Named("String".to_string())],
        ),
        is_async: true,
        is_public: true,
        body: vec![
            RustStmt::Expr(RustExpr::Comment("Call dsl_interpreter::builtins::ask".to_string())),
            RustStmt::Expr(RustExpr::Macro {
                name: "unimplemented".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::String("ask".to_string()))],
            }),
        ],
    })
}

fn generate_extract_as() -> RustItem {
    RustItem::Function(RustFunction {
        name: "extract_as".to_string(),
        params: vec![
            RustParam {
                name: "prompt".to_string(),
                ty: RustType::Reference(false, Box::new(RustType::Named("str".to_string()))),
                is_mut: false,
                is_ref: false,
            },
            RustParam {
                name: "type_name".to_string(),
                ty: RustType::Reference(false, Box::new(RustType::Named("str".to_string()))),
                is_mut: false,
                is_ref: false,
            },
        ],
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Named("Value".to_string())],
        ),
        is_async: true,
        is_public: true,
        body: vec![
            RustStmt::Expr(RustExpr::Comment("Call dsl_interpreter::builtins::extract_as".to_string())),
            RustStmt::Expr(RustExpr::Macro {
                name: "unimplemented".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::String("extract_as".to_string()))],
            }),
        ],
    })
}

fn generate_extract_person() -> RustItem {
    RustItem::Function(RustFunction {
        name: "extract_person".to_string(),
        params: vec![RustParam {
            name: "prompt".to_string(),
            ty: RustType::Reference(false, Box::new(RustType::Named("str".to_string()))),
            is_mut: false,
            is_ref: false,
        }],
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Named("Value".to_string())],
        ),
        is_async: true,
        is_public: true,
        body: vec![
            RustStmt::Expr(RustExpr::Comment("Call dsl_interpreter::builtins::extract_person".to_string())),
            RustStmt::Expr(RustExpr::Macro {
                name: "unimplemented".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::String("extract_person".to_string()))],
            }),
        ],
    })
}

fn generate_length() -> RustItem {
    RustItem::Function(RustFunction {
        name: "length".to_string(),
        params: vec![RustParam {
            name: "value".to_string(),
            ty: RustType::Reference(false, Box::new(RustType::Named("Value".to_string()))),
            is_mut: false,
            is_ref: false,
        }],
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Named("i64".to_string())],
        ),
        is_async: false,
        is_public: true,
        body: vec![
            RustStmt::Expr(RustExpr::Match {
                expr: Box::new(RustExpr::Variable("value".to_string())),
                arms: vec![
                    RustMatchArm {
                        pattern: RustPattern::Struct {
                            name: "Value::String".to_string(),
                            fields: vec![("0".to_string(), RustPattern::Binding("s".to_string()))],
                        },
                        guard: None,
                        body: RustExpr::Call {
                            func: Box::new(RustExpr::Variable("Ok".to_string())),
                            args: vec![RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Variable("s".to_string())),
                                method: "len".to_string(),
                                args: vec![],
                            }],
                        },
                    },
                    RustMatchArm {
                        pattern: RustPattern::Struct {
                            name: "Value::List".to_string(),
                            fields: vec![("0".to_string(), RustPattern::Binding("l".to_string()))],
                        },
                        guard: None,
                        body: RustExpr::Call {
                            func: Box::new(RustExpr::Variable("Ok".to_string())),
                            args: vec![RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Variable("l".to_string())),
                                method: "len".to_string(),
                                args: vec![],
                            }],
                        },
                    },
                    RustMatchArm {
                        pattern: RustPattern::Wildcard,
                        guard: None,
                        body: RustExpr::Call {
                            func: Box::new(RustExpr::Variable("Err".to_string())),
                            args: vec![RustExpr::Macro {
                                name: "anyhow::anyhow".to_string(),
                                args: vec![RustExpr::Literal(RustLiteral::String(
                                    "Length requires string or list".to_string(),
                                ))],
                            }],
                        },
                    },
                ],
            }),
        ],
    })
}

fn generate_upper() -> RustItem {
    RustItem::Function(RustFunction {
        name: "upper".to_string(),
        params: vec![RustParam {
            name: "s".to_string(),
            ty: RustType::Reference(false, Box::new(RustType::Named("str".to_string()))),
            is_mut: false,
            is_ref: false,
        }],
        return_type: RustType::Named("String".to_string()),
        is_async: false,
        is_public: true,
        body: vec![RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Variable("s".to_string())),
            method: "to_uppercase".to_string(),
            args: vec![],
        })],
    })
}

fn generate_lower() -> RustItem {
    RustItem::Function(RustFunction {
        name: "lower".to_string(),
        params: vec![RustParam {
            name: "s".to_string(),
            ty: RustType::Reference(false, Box::new(RustType::Named("str".to_string()))),
            is_mut: false,
            is_ref: false,
        }],
        return_type: RustType::Named("String".to_string()),
        is_async: false,
        is_public: true,
        body: vec![RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Variable("s".to_string())),
            method: "to_lowercase".to_string(),
            args: vec![],
        })],
    })
}

fn generate_join() -> RustItem {
    RustItem::Function(RustFunction {
        name: "join".to_string(),
        params: vec![
            RustParam {
                name: "list".to_string(),
                ty: RustType::Reference(
                    false,
                    Box::new(RustType::Generic(
                        "Vec".to_string(),
                        vec![RustType::Named("String".to_string())],
                    )),
                ),
                is_mut: false,
                is_ref: false,
            },
            RustParam {
                name: "separator".to_string(),
                ty: RustType::Reference(false, Box::new(RustType::Named("str".to_string()))),
                is_mut: false,
                is_ref: false,
            },
        ],
        return_type: RustType::Named("String".to_string()),
        is_async: false,
        is_public: true,
        body: vec![RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Variable("list".to_string())),
            method: "join".to_string(),
            args: vec![RustExpr::Variable("separator".to_string())],
        })],
    })
}

fn generate_sql() -> RustItem {
    RustItem::Function(RustFunction {
        name: "sql".to_string(),
        params: vec![RustParam {
            name: "query".to_string(),
            ty: RustType::Reference(false, Box::new(RustType::Named("str".to_string()))),
            is_mut: false,
            is_ref: false,
        }],
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Named("Value".to_string())],
        ),
        is_async: true,
        is_public: true,
        body: vec![
            RustStmt::Expr(RustExpr::Comment("Call dsl_interpreter::sql::execute".to_string())),
            RustStmt::Expr(RustExpr::Macro {
                name: "unimplemented".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::String("sql".to_string()))],
            }),
        ],
    })
}

fn generate_par() -> RustItem {
    RustItem::Function(RustFunction {
        name: "par".to_string(),
        params: vec![RustParam {
            name: "tasks".to_string(),
            ty: RustType::Generic(
                "Vec".to_string(),
                vec![RustType::Named("Value".to_string())],
            ),
            is_mut: false,
            is_ref: false,
        }],
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Generic(
                "Vec".to_string(),
                vec![RustType::Named("Value".to_string())],
            )],
        ),
        is_async: true,
        is_public: true,
        body: vec![
            RustStmt::Expr(RustExpr::Comment("Execute tasks in parallel using tokio::join!".to_string())),
            RustStmt::Expr(RustExpr::Macro {
                name: "unimplemented".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::String("par".to_string()))],
            }),
        ],
    })
}

fn generate_not() -> RustItem {
    RustItem::Function(RustFunction {
        name: "not".to_string(),
        params: vec![RustParam {
            name: "value".to_string(),
            ty: RustType::Named("bool".to_string()),
            is_mut: false,
            is_ref: false,
        }],
        return_type: RustType::Named("bool".to_string()),
        is_async: false,
        is_public: true,
        body: vec![RustStmt::Expr(RustExpr::UnaryOp {
            op: "!".to_string(),
            expr: Box::new(RustExpr::Variable("value".to_string())),
        })],
    })
}

fn generate_render_markdown() -> RustItem {
    RustItem::Function(RustFunction {
        name: "render_markdown".to_string(),
        params: vec![RustParam {
            name: "content".to_string(),
            ty: RustType::Named("String".to_string()),
            is_mut: false,
            is_ref: false,
        }],
        return_type: RustType::Named("Value".to_string()),
        is_async: false,
        is_public: true,
        body: vec![RustStmt::Expr(RustExpr::Call {
            func: Box::new(RustExpr::Variable("Value::Markdown".to_string())),
            args: vec![RustExpr::Variable("content".to_string())],
        })],
    })
}

impl RustExpr {
    pub fn Comment(s: String) -> RustExpr {
        RustExpr::Literal(RustLiteral::String(format!("// {}", s)))
    }
}
