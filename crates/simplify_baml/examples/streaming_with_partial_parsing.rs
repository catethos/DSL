/// Example demonstrating streaming with partial JSON parsing
///
/// This example shows how to use the partial parser to get incremental
/// results while streaming from an LLM.

use simplify_baml::*;

fn main() -> anyhow::Result<()> {
    println!("=== Streaming with Partial Parsing Example ===\n");

    // Build IR
    let mut ir = IR::new();

    ir.classes.push(Class {
        name: "Person".to_string(),
        description: Some("A person entity".to_string()),
        fields: vec![
            Field {
                name: "name".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Full name".to_string()),
            },
            Field {
                name: "age".to_string(),
                field_type: FieldType::Int,
                optional: false,
                description: Some("Age in years".to_string()),
            },
            Field {
                name: "occupation".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Job title".to_string()),
            },
        ],
    });

    let target_type = FieldType::Class("Person".to_string());

    println!("📡 Simulating streaming LLM response...\n");

    // Simulate streaming chunks coming from an LLM
    let chunks = vec![
        r#"Here's the person info:"#,
        r#"Here's the person info:
```json
{"#,
        r#"Here's the person info:
```json
{"name": "Joh"#,
        r#"Here's the person info:
```json
{"name": "John Doe", "ag"#,
        r#"Here's the person info:
```json
{"name": "John Doe", "age": 3"#,
        r#"Here's the person info:
```json
{"name": "John Doe", "age": 35, "occupation": "Soft"#,
        r#"Here's the person info:
```json
{"name": "John Doe", "age": 35, "occupation": "Software Engineer"#,
        r#"Here's the person info:
```json
{"name": "John Doe", "age": 35, "occupation": "Software Engineer"}
```"#,
    ];

    let mut accumulated = String::new();

    for (i, chunk) in chunks.iter().enumerate() {
        println!("📦 Chunk {}:", i + 1);
        println!("{}", "=".repeat(60));

        // In a real scenario, you'd be accumulating tokens from the stream
        accumulated = chunk.to_string();

        // Try to parse the partial response
        match try_parse_partial_response(&ir, &accumulated, &target_type)? {
            Some(value) => {
                println!("✅ Parsed partial result:");
                println!("{:#?}", value);

                // You can also display it in a more user-friendly way
                if let BamlValue::Map(map) = &value {
                    println!("\n📊 Current Data:");
                    println!("  Name: {:?}", map.get("name").and_then(|v| v.as_string()).unwrap_or("<incomplete>"));
                    println!("  Age: {:?}", map.get("age").and_then(|v| v.as_int()).map(|i| i.to_string()).unwrap_or("<incomplete>".to_string()));
                    println!("  Occupation: {:?}", map.get("occupation").and_then(|v| v.as_string()).unwrap_or("<incomplete>"));
                }
            }
            None => {
                println!("⏳ Not enough data to parse yet...");
            }
        }

        println!();
    }

    println!("{}", "=".repeat(60));
    println!("🎉 Streaming complete!\n");

    // Final parse with the complete response
    let final_result = parse_llm_response_with_ir(&ir, &accumulated, &target_type)?;
    println!("✅ Final parsed result:");
    println!("{:#?}", final_result);

    println!("\n=== Key Benefits ===");
    println!("✨ Show partial results to users in real-time");
    println!("✨ Auto-close incomplete JSON structures intelligently");
    println!("✨ Graceful handling of incomplete data");
    println!("✨ Works with any streaming LLM client");

    Ok(())
}
