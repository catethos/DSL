/// Example: Parser Showcase - Handling Improperly Formatted Responses
///
/// This example demonstrates the robustness of the simplified BAML parser
/// by showing how it handles various improperly formatted LLM responses:
/// 1. JSON wrapped in markdown code blocks
/// 2. JSON with extra explanatory text
/// 3. Type coercion (strings to numbers, etc.)
/// 4. Case-insensitive enum matching
/// 5. Missing optional fields
///
/// To run this example:
/// ```bash
/// cargo run --example parser_showcase
/// ```

use simplify_baml::*;

fn main() {
    println!("=== BAML Parser Showcase ===\n");
    println!("Demonstrating how the parser handles various malformed responses\n");

    // Build IR with test types
    let ir = build_test_ir();
    let parser = simplify_baml::parser::Parser::new(&ir);

    // Test cases showing different parsing scenarios
    run_test_case(&parser, "1. JSON in Markdown Code Block", r#"
Sure, here's the person information you requested:

```json
{
    "name": "Alice Johnson",
    "age": 28,
    "status": "active",
    "score": 95.5
}
```

Hope this helps!
    "#);

    run_test_case(&parser, "2. JSON without Language Tag", r#"
Here's what I found:

```
{
    "name": "Bob Smith",
    "age": 35,
    "status": "ACTIVE",
    "score": 88.0
}
```
    "#);

    run_test_case(&parser, "3. Plain JSON with Extra Text", r#"
Based on the information provided, the person is:
{
    "name": "Charlie Brown",
    "age": 42,
    "status": "inactive",
    "score": 72.3
}
That's all I could extract from the text.
    "#);

    run_test_case(&parser, "4. Type Coercion - String to Int", r#"
{
    "name": "Diana Prince",
    "age": "29",
    "status": "active",
    "score": "91.5"
}
    "#);

    run_test_case(&parser, "5. Case-Insensitive Enum", r#"
{
    "name": "Eve Adams",
    "age": 31,
    "status": "ACTIVE",
    "score": 87.2
}
    "#);

    run_test_case(&parser, "6. Mixed Case Enum", r#"
{
    "name": "Frank Miller",
    "age": 45,
    "status": "InAcTiVe",
    "score": 65.0
}
    "#);

    run_test_case(&parser, "7. Missing Optional Field (email)", r#"
{
    "name": "Grace Hopper",
    "age": 50,
    "status": "active",
    "score": 99.9
}
    "#);

    run_test_case(&parser, "8. Number as String for Age", r#"
{
    "name": "Henry Ford",
    "age": "38",
    "status": "active",
    "score": 82.1
}
    "#);

    run_test_case(&parser, "9. Integer for Float Field", r#"
{
    "name": "Ivy League",
    "age": 22,
    "status": "active",
    "score": 100
}
    "#);

    run_test_case(&parser, "10. JSON with Comments (will fail gracefully)", r#"
{
    // This is a person
    "name": "Jack Ryan",
    "age": 33,
    "status": "active",
    "score": 78.5
}
    "#);

    println!("\n=== Summary ===");
    println!("The parser successfully handles:");
    println!("✓ Markdown code blocks (with or without language tags)");
    println!("✓ Extra text before/after JSON");
    println!("✓ Type coercion (string ↔ number)");
    println!("✓ Case-insensitive enum matching");
    println!("✓ Missing optional fields");
    println!("✓ Integer/float interchangeability");
    println!("\nIt fails gracefully on:");
    println!("✗ JSON with comments (not valid JSON)");
    println!("✗ Completely malformed structures");
}

fn run_test_case(parser: &simplify_baml::parser::Parser, title: &str, response: &str) {
    println!("─────────────────────────────────────────────");
    println!("{}", title);
    println!("─────────────────────────────────────────────");
    println!("Raw Response:");
    println!("{}", response.trim());
    println!();

    let result = parser.parse(response, &FieldType::Class("Person".to_string()));

    match result {
        Ok(value) => {
            println!("✅ Successfully Parsed:");
            print_person(&value);
        }
        Err(e) => {
            println!("❌ Failed to Parse:");
            println!("   Error: {}", e);
        }
    }
    println!();
}

fn print_person(value: &BamlValue) {
    if let BamlValue::Map(map) = value {
        println!("   Person {{");
        if let Some(name) = map.get("name") {
            println!("      name: {:?}", name.as_string().unwrap_or("N/A"));
        }
        if let Some(age) = map.get("age") {
            println!("      age: {}", age.as_int().unwrap_or(-1));
        }
        if let Some(status) = map.get("status") {
            println!("      status: {:?}", status.as_string().unwrap_or("N/A"));
        }
        if let Some(score) = map.get("score") {
            println!("      score: {}", score.as_float().unwrap_or(-1.0));
        }
        if let Some(email) = map.get("email") {
            println!("      email: {:?}", email.as_string());
        }
        println!("   }}");
    }
}

fn build_test_ir() -> IR {
    let mut ir = IR::new();

    // Define Status enum with two values
    ir.enums.push(Enum {
        name: "Status".to_string(),
        description: None,
        values: vec!["active".to_string(), "inactive".to_string()],
    });

    // Define Person class with various field types
    ir.classes.push(Class {
        name: "Person".to_string(),
        description: None,
        fields: vec![
            Field {
                name: "name".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: None,
            },
            Field {
                name: "age".to_string(),
                field_type: FieldType::Int,
                optional: false,
                description: None,
            },
            Field {
                name: "status".to_string(),
                field_type: FieldType::Enum("Status".to_string()),
                optional: false,
                description: None,
            },
            Field {
                name: "score".to_string(),
                field_type: FieldType::Float,
                optional: false,
                description: None,
            },
            Field {
                name: "email".to_string(),
                field_type: FieldType::String,
                optional: true, // Optional field
                description: None,
            },
        ],
    });

    ir
}
