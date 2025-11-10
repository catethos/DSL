use dsl_interpreter::{Interpreter};
use dsl_ir::{IRNode, IRFunction, IRExecution, Value, IRBinding};

#[tokio::test]
async fn test_block_with_let_binding() {
    let mut interp = Interpreter::new().expect("Failed to create interpreter");

    // Create a function: def f(x) { let y = 2; x + y }
    let func = IRFunction {
        name: "f".to_string(),
        params: vec!["x".to_string()],
        return_type: None,
        properties: std::collections::HashMap::new(),
        execution: IRExecution::Expression {
            body: Box::new(IRNode::Block {
                statements: vec![
                    // let y = 2 (represented as Parallel with binding)
                    IRNode::Parallel {
                        exprs: vec![IRNode::Int(2)],
                        binding: Some(IRBinding::Single("y".to_string())),
                    }
                ],
                result: Box::new(IRNode::BinaryOp {
                    left: Box::new(IRNode::Variable("x".to_string())),
                    op: "+".to_string(),
                    right: Box::new(IRNode::Variable("y".to_string())),
                }),
            }),
        },
    };

    // Register the function
    interp.runtime.functions.insert("f".to_string(), func);

    // Call f(10) should return 12
    let call_node = IRNode::FunctionCall {
        name: "f".to_string(),
        args: vec![IRNode::Int(10)],
        effect_kind: None,
        source_span: None,
    };

    let result = interp.eval(&call_node).await;
    match result {
        Ok(val) => {
            assert_eq!(val, Value::Int(12), "Expected 12, got {:?}", val);
        }
        Err(e) => {
            panic!("Function call failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_block_with_multiple_let_bindings() {
    let mut interp = Interpreter::new().expect("Failed to create interpreter");

    // Create a function: def f(x) { let a = 2; let b = 3; x + a + b }
    let func = IRFunction {
        name: "f".to_string(),
        params: vec!["x".to_string()],
        return_type: None,
        properties: std::collections::HashMap::new(),
        execution: IRExecution::Expression {
            body: Box::new(IRNode::Block {
                statements: vec![
                    // let a = 2
                    IRNode::Parallel {
                        exprs: vec![IRNode::Int(2)],
                        binding: Some(IRBinding::Single("a".to_string())),
                    },
                    // let b = 3
                    IRNode::Parallel {
                        exprs: vec![IRNode::Int(3)],
                        binding: Some(IRBinding::Single("b".to_string())),
                    },
                ],
                result: Box::new(IRNode::BinaryOp {
                    left: Box::new(IRNode::BinaryOp {
                        left: Box::new(IRNode::Variable("x".to_string())),
                        op: "+".to_string(),
                        right: Box::new(IRNode::Variable("a".to_string())),
                    }),
                    op: "+".to_string(),
                    right: Box::new(IRNode::Variable("b".to_string())),
                }),
            }),
        },
    };

    // Register the function
    interp.runtime.functions.insert("f".to_string(), func);

    // Call f(10) should return 15
    let call_node = IRNode::FunctionCall {
        name: "f".to_string(),
        args: vec![IRNode::Int(10)],
        effect_kind: None,
        source_span: None,
    };

    let result = interp.eval(&call_node).await;
    match result {
        Ok(val) => {
            assert_eq!(val, Value::Int(15), "Expected 15, got {:?}", val);
        }
        Err(e) => {
            panic!("Function call failed: {:?}", e);
        }
    }
}
