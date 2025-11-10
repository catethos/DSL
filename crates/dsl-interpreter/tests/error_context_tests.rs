use dsl_interpreter::Interpreter;
use dsl_ir::{IRNode, Span};

#[tokio::test]
async fn test_llm_error_includes_span() {
    let mut interpreter = Interpreter::new().unwrap();

    // Create a mock LLM function that will fail
    let span = Span {
        file: "test.dsl".to_string(),
        line: 42,
        column: 10,
    };

    // Create an intrinsic call with span info
    let node = IRNode::FunctionCall {
        name: "__llm_execute".to_string(),
        args: vec![
            IRNode::String("test prompt".to_string()),
            IRNode::Map(vec![]), // Empty config will cause an error due to missing API key
        ],
        effect_kind: Some(dsl_ir::EffectKind::LLM),
        source_span: Some(span.clone()),
    };

    // Execute and expect error
    let result = interpreter.eval(&node).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();

    // Error message should include span information
    assert!(error_msg.contains("test.dsl"), "Error should mention file: {}", error_msg);
    assert!(error_msg.contains("42"), "Error should mention line number: {}", error_msg);
}

#[tokio::test]
async fn test_http_error_includes_method_and_url() {
    let mut interpreter = Interpreter::new().unwrap();

    let span = Span {
        file: "http_test.dsl".to_string(),
        line: 10,
        column: 5,
    };

    // Create an intrinsic HTTP call with invalid method
    let node = IRNode::FunctionCall {
        name: "__http".to_string(),
        args: vec![
            IRNode::String("INVALID_METHOD".to_string()),
            IRNode::String("https://example.com".to_string()),
            IRNode::Map(vec![]), // params
            IRNode::Map(vec![]), // headers
            IRNode::String("".to_string()), // body
        ],
        effect_kind: Some(dsl_ir::EffectKind::HTTP),
        source_span: Some(span.clone()),
    };

    let result = interpreter.eval(&node).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();

    // Error should include method and URL
    assert!(error_msg.contains("INVALID_METHOD"), "Error should mention method: {}", error_msg);
    assert!(error_msg.contains("https://example.com"), "Error should mention URL: {}", error_msg);
    assert!(error_msg.contains("http_test.dsl"), "Error should mention file: {}", error_msg);
    assert!(error_msg.contains("10"), "Error should mention line: {}", error_msg);
}

#[tokio::test]
async fn test_type_error_includes_span() {
    let mut interpreter = Interpreter::new().unwrap();

    let span = Span {
        file: "type_test.dsl".to_string(),
        line: 20,
        column: 15,
    };

    // Create an intrinsic call with wrong argument type
    let node = IRNode::FunctionCall {
        name: "__llm_execute".to_string(),
        args: vec![
            IRNode::Int(123), // Wrong type - should be String
            IRNode::Map(vec![]),
        ],
        effect_kind: Some(dsl_ir::EffectKind::LLM),
        source_span: Some(span.clone()),
    };

    let result = interpreter.eval(&node).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();

    // Error should mention type mismatch and span
    assert!(error_msg.contains("Type"), "Error should be a type error: {}", error_msg);
    assert!(error_msg.contains("String"), "Error should mention expected type: {}", error_msg);
    assert!(error_msg.contains("type_test.dsl"), "Error should mention file: {}", error_msg);
    assert!(error_msg.contains("20"), "Error should mention line: {}", error_msg);
}

#[tokio::test]
async fn test_invalid_arguments_error_includes_span() {
    let mut interpreter = Interpreter::new().unwrap();

    let span = Span {
        file: "args_test.dsl".to_string(),
        line: 5,
        column: 8,
    };

    // Create an intrinsic call with wrong number of arguments
    let node = IRNode::FunctionCall {
        name: "__llm_execute".to_string(),
        args: vec![
            IRNode::String("only one arg".to_string()),
            // Missing second argument
        ],
        effect_kind: Some(dsl_ir::EffectKind::LLM),
        source_span: Some(span.clone()),
    };

    let result = interpreter.eval(&node).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();

    // Error should mention invalid arguments and span
    assert!(error_msg.contains("Invalid Arguments") || error_msg.contains("expects 2"),
            "Error should mention invalid arguments: {}", error_msg);
    assert!(error_msg.contains("args_test.dsl"), "Error should mention file: {}", error_msg);
    assert!(error_msg.contains("5"), "Error should mention line: {}", error_msg);
}

#[tokio::test]
async fn test_unknown_intrinsic_error_includes_span() {
    let mut interpreter = Interpreter::new().unwrap();

    let span = Span {
        file: "unknown_test.dsl".to_string(),
        line: 99,
        column: 1,
    };

    // Create a call to non-existent intrinsic
    let node = IRNode::FunctionCall {
        name: "__unknown_intrinsic".to_string(),
        args: vec![],
        effect_kind: None,
        source_span: Some(span.clone()),
    };

    let result = interpreter.eval(&node).await;
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();

    // Error should mention unknown intrinsic and span
    assert!(error_msg.contains("Unknown Intrinsic") || error_msg.contains("__unknown_intrinsic"),
            "Error should mention unknown intrinsic: {}", error_msg);
    assert!(error_msg.contains("unknown_test.dsl"), "Error should mention file: {}", error_msg);
    assert!(error_msg.contains("99"), "Error should mention line: {}", error_msg);
}
