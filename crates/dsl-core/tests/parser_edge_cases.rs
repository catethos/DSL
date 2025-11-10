// Test file to verify parser edge cases and error handling
use dsl_core::parser::parse_expr;

#[test]
fn test_unclosed_string_with_unclosed_paren() {
    // The reported issue: should fail but might succeed
    let result = parse_expr(r#"AnalyzeWithClaude("why the sky is blue"#);
    println!("Result: {:?}", result);
    // This SHOULD be an error
    assert!(result.is_err(), "Should fail: unclosed string and parenthesis");
}

#[test]
fn test_unclosed_double_quote_string() {
    let result = parse_expr(r#""hello"#);
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: unclosed double quote string");
}

#[test]
fn test_unclosed_single_quote_string() {
    let result = parse_expr(r#"'hello"#);
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: unclosed single quote string");
}

#[test]
fn test_unclosed_parenthesis() {
    let result = parse_expr("foo(1, 2");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: unclosed parenthesis");
}

#[test]
fn test_unclosed_bracket() {
    let result = parse_expr("[1, 2, 3");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: unclosed bracket");
}

#[test]
fn test_unclosed_brace() {
    let result = parse_expr("{x: 1, y: 2");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: unclosed brace");
}

#[test]
fn test_unclosed_template_string() {
    let result = parse_expr(r#""hello ${name"#);
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: unclosed template string");
}

#[test]
fn test_unclosed_template_expression() {
    let result = parse_expr(r#""hello ${name + age"#);
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: unclosed template expression in string");
}

#[test]
fn test_unclosed_triple_quoted_string() {
    let result = parse_expr(r#""""hello world"#);
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: unclosed triple quoted string");
}

#[test]
fn test_mismatched_delimiters() {
    let result = parse_expr("[1, 2, 3}");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: mismatched delimiters");
}

#[test]
fn test_nested_unclosed_call() {
    let result = parse_expr("foo(bar(1, 2)");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: nested unclosed function call");
}

#[test]
fn test_multiple_unclosed_delimiters() {
    let result = parse_expr("foo([1, 2, {x: 3");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: multiple unclosed delimiters");
}

#[test]
fn test_incomplete_match() {
    let result = parse_expr("match x {");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: incomplete match expression");
}

#[test]
fn test_match_with_unclosed_case() {
    let result = parse_expr("match x { 1 => 2");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: match with unclosed case");
}

#[test]
fn test_empty_function_call() {
    // This should succeed
    let result = parse_expr("foo()");
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Should succeed: empty function call is valid");
}

#[test]
fn test_unclosed_type_instantiation() {
    let result = parse_expr("Person { name: \"Alice\"");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: unclosed type instantiation");
}

#[test]
fn test_incomplete_field_access() {
    let result = parse_expr("user.");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: incomplete field access");
}

#[test]
fn test_incomplete_index_access() {
    let result = parse_expr("arr[");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: incomplete index access");
}

#[test]
fn test_unclosed_index_access() {
    let result = parse_expr("arr[5");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: unclosed index access");
}

#[test]
fn test_incomplete_binary_op() {
    let result = parse_expr("5 +");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: incomplete binary operation");
}

#[test]
fn test_incomplete_ternary() {
    let result = parse_expr("x ? 1 :");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: incomplete ternary");
}

#[test]
fn test_incomplete_sequential() {
    let result = parse_expr("5 |>");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: incomplete sequential");
}

#[test]
fn test_dangling_comma_in_list() {
    // Trailing commas are allowed, but this is different
    let result = parse_expr("[1, 2, ]");
    println!("Result: {:?}", result);
    // This should actually be OK (trailing comma is allowed)
    assert!(result.is_ok(), "Should succeed: trailing comma in list is allowed");
}

#[test]
fn test_double_comma_in_list() {
    let result = parse_expr("[1,, 2]");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: double comma in list");
}

#[test]
fn test_missing_map_value() {
    let result = parse_expr("{x: }");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: missing map value");
}

#[test]
fn test_incomplete_pattern_match() {
    let result = parse_expr("match x { _ =>");
    println!("Result: {:?}", result);
    assert!(result.is_err(), "Should fail: incomplete pattern match case");
}
