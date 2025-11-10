/// Example: Parser Edge Cases and Error Handling
///
/// This example demonstrates:
/// 1. Edge cases that the parser successfully handles
/// 2. Cases where the parser fails gracefully with clear error messages
/// 3. Boundary conditions (empty strings, null values, etc.)
///
/// To run this example:
/// ```bash
/// cargo run --example edge_cases
/// ```

use simplify_baml::*;

fn main() {
    println!("=== Parser Edge Cases ===\n");

    let ir = build_test_ir();
    let parser = simplify_baml::parser::Parser::new(&ir);

    println!("🟢 Cases that SUCCEED (graceful handling)\n");

    test_case(&parser, "Empty String Fields", r#"
{
    "title": "",
    "count": 0,
    "active": true,
    "rating": 0.0
}
    "#);

    test_case(&parser, "Whitespace in JSON", r#"
{
    "title"   :   "Test"   ,
    "count"   :   42   ,
    "active"  :   true  ,
    "rating"  :   3.14
}
    "#);

    test_case(&parser, "Extreme Numbers", r#"
{
    "title": "Test",
    "count": 9999999999,
    "active": true,
    "rating": 999999.999999
}
    "#);

    test_case(&parser, "Boolean as String (lenient)", r#"
{
    "title": "Test",
    "count": 10,
    "active": "true",
    "rating": 5.0
}
    "#);

    test_case(&parser, "Number with Decimals for Int (truncates)", r#"
{
    "title": "Test",
    "count": 42.9,
    "active": true,
    "rating": 3.14
}
    "#);

    test_case(&parser, "Unicode Characters", r#"
{
    "title": "Hello 世界 🌍",
    "count": 42,
    "active": true,
    "rating": 4.5
}
    "#);

    test_case(&parser, "Escaped Characters", r#"
{
    "title": "Line 1\nLine 2\tTabbed",
    "count": 1,
    "active": true,
    "rating": 1.0
}
    "#);

    test_case(&parser, "JSON Buried in Text", r#"
The system analyzed the data and found the following result. After processing
multiple iterations and considering various factors, the final output is:

{"title": "Result", "count": 100, "active": true, "rating": 9.5}

This represents the best match given the input parameters and constraints.
    "#);

    test_case(&parser, "Multiple JSON Objects (takes first)", r#"
{"title": "First", "count": 1, "active": true, "rating": 1.0}
{"title": "Second", "count": 2, "active": false, "rating": 2.0}
    "#);

    test_case(&parser, "JSON in Code Block with Backticks", r#"
Here's the data:
```
{"title": "Data", "count": 5, "active": true, "rating": 3.5}
```
    "#);

    println!("\n🔴 Cases that FAIL (expected failures)\n");

    test_case(&parser, "Missing Required Field", r#"
{
    "title": "Incomplete",
    "count": 10,
    "active": true
}
    "#);

    test_case(&parser, "Wrong Type (cannot coerce)", r#"
{
    "title": "Test",
    "count": "not-a-number",
    "active": true,
    "rating": 5.0
}
    "#);

    test_case(&parser, "Invalid JSON Syntax", r#"
{
    "title": "Broken",
    "count": 10,
    "active": true,
    "rating": 5.0,
}
    "#);

    test_case(&parser, "JSON with Comments", r#"
{
    // This is a comment
    "title": "Test",
    "count": 10,
    "active": true,
    "rating": 5.0
}
    "#);

    test_case(&parser, "Non-JSON Response", r#"
This is just plain text without any JSON structure.
The parser should fail gracefully on this.
    "#);

    test_case(&parser, "Null for Required Field", r#"
{
    "title": null,
    "count": 10,
    "active": true,
    "rating": 5.0
}
    "#);

    test_case(&parser, "Array Instead of Object", r#"
["title", 10, true, 5.0]
    "#);

    test_case(&parser, "Object Instead of Primitive", r#"
{
    "title": {"nested": "value"},
    "count": 10,
    "active": true,
    "rating": 5.0
}
    "#);

    println!("\n=== Summary ===");
    println!("\n✅ Parser handles gracefully:");
    println!("  • Empty strings");
    println!("  • Extra whitespace");
    println!("  • Type coercion (string↔number, bool↔string)");
    println!("  • Unicode and escaped characters");
    println!("  • JSON buried in text");
    println!("  • JSON in markdown blocks");
    println!("  • Decimal→int conversion (truncation)");
    println!("  • Multiple objects (takes first)");
    println!("\n❌ Parser fails gracefully on:");
    println!("  • Missing required fields");
    println!("  • Invalid JSON syntax");
    println!("  • JSON comments");
    println!("  • Non-coercible types");
    println!("  • Null for required fields");
    println!("  • Type mismatches (object vs primitive)");
    println!("  • Pure text without JSON");
}

fn test_case(parser: &simplify_baml::parser::Parser, title: &str, response: &str) {
    println!("──────────────────────────────────────");
    println!("📋 {}", title);
    println!("──────────────────────────────────────");

    let result = parser.parse(response, &FieldType::Class("Item".to_string()));

    match result {
        Ok(value) => {
            println!("✅ SUCCESS");
            if let BamlValue::Map(map) = value {
                println!("   {{");
                if let Some(title) = map.get("title") {
                    println!("     title: {:?}", title.as_string().unwrap_or("N/A"));
                }
                if let Some(count) = map.get("count") {
                    println!("     count: {}", count.as_int().unwrap_or(-1));
                }
                if let Some(active) = map.get("active") {
                    println!("     active: {}", active.as_bool().unwrap_or(false));
                }
                if let Some(rating) = map.get("rating") {
                    println!("     rating: {}", rating.as_float().unwrap_or(-1.0));
                }
                println!("   }}");
            }
        }
        Err(e) => {
            println!("❌ FAILED");
            println!("   Error: {}", e);
        }
    }
    println!();
}

fn build_test_ir() -> IR {
    let mut ir = IR::new();

    ir.classes.push(Class {
        name: "Item".to_string(),
        description: None,
        fields: vec![
            Field {
                name: "title".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: None,
            },
            Field {
                name: "count".to_string(),
                field_type: FieldType::Int,
                optional: false,
                description: None,
            },
            Field {
                name: "active".to_string(),
                field_type: FieldType::Bool,
                optional: false,
                description: None,
            },
            Field {
                name: "rating".to_string(),
                field_type: FieldType::Float,
                optional: false,
                description: None,
            },
        ],
    });

    ir
}
