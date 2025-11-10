// Test file to verify parser issues
use dsl_core::parser::parse_expr;

fn main() {
    println!("Testing parser edge cases...\n");

    // Issue 1: Unclosed string with unclosed parenthesis
    test_parse(r#"AnalyzeWithClaude("why the sky is blue"#, "Unclosed string + paren");

    // Issue 2: Unclosed string alone
    test_parse(r#""hello"#, "Unclosed string");

    // Issue 3: Unclosed single quote string
    test_parse(r#"'hello"#, "Unclosed single quote string");

    // Issue 4: Unclosed parenthesis
    test_parse("foo(1, 2", "Unclosed parenthesis");

    // Issue 5: Unclosed bracket
    test_parse("[1, 2, 3", "Unclosed bracket");

    // Issue 6: Unclosed brace
    test_parse("{x: 1, y: 2", "Unclosed brace");

    // Issue 7: Unclosed template string
    test_parse(r#""hello ${name"#, "Unclosed template string");

    // Issue 8: Unclosed template expression
    test_parse(r#""hello ${name + age"#, "Unclosed template expression");

    // Issue 9: Unclosed triple quoted string
    test_parse(r#""""hello world"#, "Unclosed triple quote");

    // Issue 10: Mismatched delimiters
    test_parse("[1, 2, 3}", "Mismatched delimiters");

    // Issue 11: Function call with unclosed nested call
    test_parse("foo(bar(1, 2)", "Nested unclosed call");

    // Issue 12: Multiple unclosed delimiters
    test_parse("foo([1, 2, {x: 3", "Multiple unclosed delimiters");

    // Issue 13: Empty function call - should work
    test_parse("foo()", "Empty function call");

    // Issue 14: Incomplete match expression
    test_parse("match x {", "Incomplete match");

    // Issue 15: Match with unclosed case
    test_parse("match x { 1 => 2", "Match with unclosed case");
}

fn test_parse(input: &str, description: &str) {
    println!("Test: {}", description);
    println!("Input: {}", input);
    match parse_expr(input) {
        Ok(expr) => println!("✗ PARSED SUCCESSFULLY (should fail): {:?}\n", expr),
        Err(e) => println!("✓ Failed as expected: {}\n", e),
    }
}
