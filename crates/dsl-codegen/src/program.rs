use crate::expressions::generate_expr;
use crate::functions::generate_function;
use crate::rust_ast::*;
use crate::types::{generate_enum, generate_struct};
use anyhow::{Result, Context};
use dsl_ir::IR;

pub fn generate_executable(ir: &IR) -> Result<String> {
    // Instead of generating Rust code for each expression,
    // we embed the IR and use the interpreter at runtime
    let ir_json = serde_json::to_string_pretty(ir)
        .context("Failed to serialize IR to JSON")?;

    let code = format!(r##"use dsl_runtime::{{Result, anyhow}};
use dsl_interpreter::Interpreter;
use dsl_ir::{{IR, Value}};
use std::env;
use anyhow::Context;

#[tokio::main]
async fn main() -> Result<()> {{
    // Embedded IR
    let ir_json = r#"{}"#;

    // Parse the IR
    let ir: IR = serde_json::from_str(ir_json)
        .context("Failed to parse embedded IR")?;

    // Create interpreter
    let mut interpreter = Interpreter::new()?;

    // Load types and functions into the interpreter
    for class in &ir.types {{
        interpreter.runtime.types.register_class(class.clone());
    }}

    for enum_def in &ir.enums {{
        interpreter.runtime.types.register_enum(enum_def.clone());
    }}

    for func in &ir.functions {{
        interpreter.runtime.functions.insert(func.name.clone(), func.clone());
    }}

    // Inject command-line arguments as a variable
    let args: Vec<String> = env::args().skip(1).collect();
    let args_value = Value::List(
        args.iter().map(|s| Value::String(s.clone())).collect()
    );
    interpreter.runtime.vars.insert("args".to_string(), args_value);

    // Also inject individual positional arguments
    for (i, arg) in args.iter().enumerate() {{
        let var_name = format!("arg{{}}", i + 1);
        interpreter.runtime.vars.insert(var_name, Value::String(arg.clone()));
    }}

    // Execute the entry expression
    let result = interpreter.eval(&ir.entry_expr).await
        .map_err(|e| anyhow!(e))?;

    // Print the result
    println!("{{}}", result.display());

    Ok(())
}}
"##, ir_json);

    Ok(code)
}

pub fn generate_library(ir: &IR) -> Result<String> {
    let mut items = Vec::new();

    // Use dsl_runtime for all runtime support
    items.push(RustItem::Use("dsl_runtime::*".to_string()));

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
        value: entry_expr,
        is_mut: false,
    });

    body.push(RustStmt::Expr(RustExpr::Macro {
        name: "println".to_string(),
        args: vec![
            RustExpr::Literal(RustLiteral::String("Result: {:?}".to_string())),
            RustExpr::Variable("result".to_string()),
        ],
    }));

    body.push(RustStmt::Return(Some(RustExpr::Call {
        func: Box::new(RustExpr::Variable("Ok".to_string())),
        args: vec![RustExpr::Literal(RustLiteral::Unit)],
    })));

    Ok(RustFunction {
        name: "main".to_string(),
        params: vec![],
        return_type: RustType::Generic(
            "Result".to_string(),
            vec![RustType::Unit],
        ),
        is_async: true,
        is_public: false,
        attributes: vec!["tokio::main".to_string()],
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
        attributes: vec![],
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
        // With embedded interpreter, we check for IR JSON containing types
        assert!(code.contains("\"Person\""));
        assert!(code.contains("\"Status\""));
        assert!(code.contains("fn main"));
        assert!(code.contains("Interpreter::new"));
    }
}
