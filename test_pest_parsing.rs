use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar_inline = r#"
test_program = _{ SOI ~ item* ~ EOI }
item = { identifier | function_call }
function_call = { identifier ~ "(" ~ args ~ ")" }
args = { (expr ~ ("," ~ expr)*)? }
expr = { identifier | string }
string = { "\"" ~ inner ~ "\"" }
inner = @{ (!"\"" ~ ANY)* }
identifier = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
"#]
pub struct TestParser;

fn main() {
    println!("Test 1: Complete function call");
    match TestParser::parse(Rule::test_program, r#"foo("bar")"#) {
        Ok(_) => println!("  ✓ Parsed successfully"),
        Err(e) => println!("  ✗ Failed: {}", e),
    }

    println!("\nTest 2: Unclosed string in function call");
    match TestParser::parse(Rule::test_program, r#"foo("bar"#) {
        Ok(_) => println!("  ✗ Parsed successfully (SHOULD FAIL)"),
        Err(e) => println!("  ✓ Failed as expected: {}", e),
    }

    println!("\nTest 3: Unclosed parenthesis");
    match TestParser::parse(Rule::test_program, r#"foo("#) {
        Ok(_) => println!("  ✗ Parsed successfully (SHOULD FAIL)"),
        Err(e) => println!("  ✓ Failed as expected: {}", e),
    }

    println!("\nTest 4: Just identifier");
    match TestParser::parse(Rule::test_program, "foo") {
        Ok(_) => println!("  ✓ Parsed successfully"),
        Err(e) => println!("  ✗ Failed: {}", e),
    }
}
