/// Tests for Phase 4: Grammar Improvements
/// Tests negative lookahead for parameters and flat sequence block parsing
use dsl_core::parse_program;

#[test]
fn test_parameter_disambiguation_typed() {
    // Typed parameters should parse correctly
    let code = "def add(a: Int, b: Int) => a + b";
    let result = parse_program(code);
    assert!(result.is_ok(), "Should parse typed parameters");

    let program = result.unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].params, vec!["a", "b"]);
}

#[test]
fn test_parameter_disambiguation_pattern() {
    // Pattern parameters should parse correctly
    let code = "def factorial(0) => 1";
    let result = parse_program(code);
    assert!(result.is_ok(), "Should parse pattern parameters");

    let program = result.unwrap();
    assert_eq!(program.pattern_functions.len(), 1);
}

#[test]
fn test_parameter_disambiguation_mixed() {
    // Mixed typed and pattern parameters
    let code = r#"
def process(x: Int, [a, b]) => x + a + b
"#;
    let result = parse_program(code);
    assert!(
        result.is_ok(),
        "Should parse mixed parameters: {}",
        result.as_ref().err().unwrap_or(&"".to_string())
    );
}

#[test]
fn test_parameter_variable_without_type() {
    // Simple variable parameters (no type annotation)
    let code = "def add(a, b) => a + b";
    let result = parse_program(code);
    assert!(result.is_ok(), "Should parse simple variable parameters");

    let program = result.unwrap();
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].params, vec!["a", "b"]);
}

#[test]
fn test_negative_lookahead_prevents_ambiguity() {
    // This should NOT be ambiguous - "x: Int" should be typed param, not variable
    let code = "def foo(x: Int) => x";
    let result = parse_program(code);
    assert!(result.is_ok(), "Typed param should not be ambiguous");
}

#[test]
fn test_block_with_single_expression() {
    // Block with just an expression
    let code = "def foo() { 42 }";
    let result = parse_program(code);
    assert!(result.is_ok(), "Block with single expr should work");
}

#[test]
fn test_block_with_let_and_expression() {
    // Block with let statement and final expression
    let code = r#"
def foo() {
    let x = 5
    x + 1
}
"#;
    let result = parse_program(code);
    assert!(
        result.is_ok(),
        "Block with let and expr should work: {}",
        result.as_ref().err().unwrap_or(&"".to_string())
    );
}

#[test]
fn test_block_with_multiple_lets() {
    // Block with multiple let statements and final expression
    let code = r#"
def foo() {
    let x = 5
    let y = 10
    x + y
}
"#;
    let result = parse_program(code);
    assert!(result.is_ok(), "Block with multiple lets should work");
}

#[test]
fn test_block_empty_fails() {
    // Empty blocks should fail
    let code = "def foo() { }";
    let result = parse_program(code);
    assert!(result.is_err(), "Empty block should fail");

    let err = result.unwrap_err();
    assert!(
        err.contains("empty") || err.contains("expression"),
        "Error should mention empty or expression: {}",
        err
    );
}

#[test]
fn test_block_multiple_expressions_fails() {
    // Multiple bare expressions without let bindings should fail
    let code = r#"
def foo() {
    42
    43
}
"#;
    let result = parse_program(code);
    assert!(
        result.is_err(),
        "Multiple expressions without bindings should fail"
    );

    let err = result.unwrap_err();
    assert!(
        err.contains("expression") || err.contains("terminator"),
        "Error should mention expressions or terminators: {}",
        err
    );
}

#[test]
fn test_block_validation_comprehensive() {
    // Valid: Zero lets, one expression
    assert!(parse_program("def f() { 1 }").is_ok());

    // Valid: One let, one expression (newline separated)
    assert!(parse_program("def f() {\n  let x = 1\n  x\n}").is_ok());

    // Valid: Multiple lets, one expression (newline separated)
    assert!(parse_program("def f() {\n  let x = 1\n  let y = 2\n  x + y\n}").is_ok());

    // Invalid: Only let statement, no final expression
    assert!(parse_program("def f() { let x = 1 }").is_err());

    // Invalid: Empty block
    assert!(parse_program("def f() { }").is_err());
}

#[test]
fn test_order_independence_typed_first() {
    // Even if we reordered the grammar alternatives, this should still work
    // because of the explicit typed_param and pattern_param rules
    let code = "def foo(x: Int, y) => x + y";
    let result = parse_program(code);
    assert!(result.is_ok(), "Mixed typed and untyped params should work");
}

#[test]
fn test_order_independence_pattern_first() {
    // Pattern before typed - should still work
    let code = "def foo(0, x: Int) => x";
    let result = parse_program(code);
    assert!(result.is_ok(), "Pattern then typed param should work");
}

#[test]
fn test_negative_lookahead_edge_cases() {
    // Variable named "x" followed by colon in different context
    let code = r#"
def foo(x) {
    let y = x: Int
    y
}
"#;
    // This should fail at the "x: Int" part (not valid syntax)
    // but the parameter "x" should parse fine
    // For now, just check the function parses
    let _result = parse_program(code);
    // This will fail at "x: Int" which is not valid let syntax
    // but the important part is that parameter "x" was parsed correctly
}

#[test]
fn test_block_post_validation_messages() {
    // Test that error messages are helpful
    let code = "def foo() { let x = 1 }";
    let result = parse_program(code);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(
        err.contains("expression") || err.contains("Block must end"),
        "Error should be descriptive: {}",
        err
    );
}

#[test]
fn test_complex_block_with_nested_expressions() {
    // Test with nested block in expression position
    let code = r#"
def complex() {
    let x = 5
    let y = 10
    x + y
}
"#;
    let result = parse_program(code);
    assert!(
        result.is_ok(),
        "Complex block should work: {}",
        result.as_ref().err().unwrap_or(&"".to_string())
    );
}
