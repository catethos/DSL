use crate::expressions::generate_expr;
use crate::functions::generate_function;
use crate::rust_ast::*;
use crate::types::{generate_enum, generate_struct};
use anyhow::Result;
use dsl_ir::IR;

pub fn generate_executable(ir: &IR) -> Result<String> {
    let mut items = Vec::new();

    items.push(RustItem::Use("anyhow::Result".to_string()));
    items.push(RustItem::Use("dsl_ir::Value".to_string()));
    items.push(RustItem::Use("indexmap::IndexMap".to_string()));

    for class in &ir.types {
        items.push(RustItem::Struct(generate_struct(class)));
    }

    for enum_def in &ir.enums {
        items.push(RustItem::Enum(generate_enum(enum_def)));
    }

    for func in &ir.functions {
        items.push(RustItem::Function(generate_function(func)?));
    }

    items.push(RustItem::Function(generate_main_function(ir)?));

    let mut code = String::new();
    for item in items {
        code.push_str(&item.pretty_print(0));
        code.push_str("\n\n");
    }

    Ok(code)
}

pub fn generate_library(ir: &IR) -> Result<String> {
    let mut items = Vec::new();

    items.push(RustItem::Use("anyhow::Result".to_string()));
    items.push(RustItem::Use("dsl_ir::Value".to_string()));
    items.push(RustItem::Use("indexmap::IndexMap".to_string()));

    for class in &ir.types {
        items.push(RustItem::Struct(generate_struct(class)));
    }

    for enum_def in &ir.enums {
        items.push(RustItem::Enum(generate_enum(enum_def)));
    }

    for func in &ir.functions {
        items.push(RustItem::Function(generate_function(func)?));
    }

    items.push(RustItem::Function(generate_init_function(ir)?));

    let mut code = String::new();
    for item in items {
        code.push_str(&item.pretty_print(0));
        code.push_str("\n\n");
    }

    Ok(code)
}

fn generate_main_function(ir: &IR) -> Result<RustFunction> {
    let mut body = Vec::new();

    body.push(RustStmt::Expr(RustExpr::Macro {
        name: "println".to_string(),
        args: vec![RustExpr::Literal(RustLiteral::String(
            "Starting DSL program...".to_string(),
        ))],
    }));

    let entry_expr = generate_expr(&ir.entry_expr)?;

    body.push(RustStmt::Let {
        name: "result".to_string(),
        ty: None,
        value: RustExpr::Await(Box::new(entry_expr)),
        is_mut: false,
    });

    body.push(RustStmt::Expr(RustExpr::Macro {
        name: "println".to_string(),
        args: vec![
            RustExpr::Literal(RustLiteral::String("Result: {:?}".to_string())),
            RustExpr::Variable("result".to_string()),
        ],
    }));

    body.push(RustStmt::Expr(RustExpr::Call {
        func: Box::new(RustExpr::Variable("Ok".to_string())),
        args: vec![RustExpr::Literal(RustLiteral::Unit)],
    }));

    Ok(RustFunction {
        name: "main".to_string(),
        params: vec![],
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Unit],
        ),
        is_async: true,
        is_public: false,
        body,
    })
}

fn generate_init_function(_ir: &IR) -> Result<RustFunction> {
    let mut body = Vec::new();

    body.push(RustStmt::Expr(RustExpr::Macro {
        name: "println".to_string(),
        args: vec![RustExpr::Literal(RustLiteral::String(
            "Initializing DSL library...".to_string(),
        ))],
    }));

    body.push(RustStmt::Expr(RustExpr::Call {
        func: Box::new(RustExpr::Variable("Ok".to_string())),
        args: vec![RustExpr::Literal(RustLiteral::Unit)],
    }));

    Ok(RustFunction {
        name: "init".to_string(),
        params: vec![],
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Unit],
        ),
        is_async: false,
        is_public: true,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsl_ir::{IRNode, Class, Enum};

    #[test]
    fn test_generate_simple_executable() {
        let ir = IR {
            version: "0.1.0".to_string(),
            types: vec![],
            enums: vec![],
            functions: vec![],
            agents: vec![],
            entry_expr: IRNode::Int(42),
        };

        let code = generate_executable(&ir).unwrap();
        assert!(code.contains("fn main"));
        assert!(code.contains("Result"));
    }

    #[test]
    fn test_generate_with_types() {
        let ir = IR {
            version: "0.1.0".to_string(),
            types: vec![Class {
                name: "Person".to_string(),
                description: None,
                fields: vec![],
            }],
            enums: vec![Enum {
                name: "Status".to_string(),
                description: None,
                values: vec!["Active".to_string(), "Inactive".to_string()],
            }],
            functions: vec![],
            agents: vec![],
            entry_expr: IRNode::String("hello".to_string()),
        };

        let code = generate_executable(&ir).unwrap();
        assert!(code.contains("struct Person"));
        assert!(code.contains("enum Status"));
        assert!(code.contains("fn main"));
    }
}
