/// Example demonstrating the use of the two extracted functions:
/// 1. generate_prompt_from_ir() - Convert IR to LLM prompt
/// 2. parse_llm_response_with_ir() - Parse LLM response using IR

use simplify_baml::*;
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    println!("=== Standalone Functions Example ===\n");

    // Build a simple IR
    let mut ir = IR::new();

    ir.enums.push(Enum {
        name: "Status".to_string(),
        description: Some("Task status".to_string()),
        values: vec!["pending".to_string(), "completed".to_string(), "failed".to_string()],
    });

    ir.classes.push(Class {
        name: "Task".to_string(),
        description: Some("A task with status".to_string()),
        fields: vec![
            Field {
                name: "title".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Task title".to_string()),
            },
            Field {
                name: "status".to_string(),
                field_type: FieldType::Enum("Status".to_string()),
                optional: false,
                description: Some("Current status".to_string()),
            },
            Field {
                name: "priority".to_string(),
                field_type: FieldType::Int,
                optional: false,
                description: Some("Priority level (1-5)".to_string()),
            },
        ],
    });

    // ===================================================================
    // Function 1: Generate prompt from IR
    // ===================================================================
    println!("1️⃣  Generating prompt from IR...\n");

    let template = r#"Extract the task information from the following text:

{{ input_text }}

Please identify the task title, status, and priority level."#;

    let mut params = HashMap::new();
    params.insert(
        "input_text".to_string(),
        BamlValue::String("Complete the documentation is pending with priority 5".to_string())
    );

    let prompt = generate_prompt_from_ir(
        &ir,
        template,
        &params,
        &FieldType::Class("Task".to_string())
    )?;

    println!("Generated Prompt:");
    println!("{}", "=".repeat(60));
    println!("{}", prompt);
    println!("{}\n", "=".repeat(60));

    // ===================================================================
    // Function 2: Parse LLM response using IR
    // ===================================================================
    println!("2️⃣  Parsing LLM response using IR...\n");

    // Simulate an LLM response (with markdown code block and type issues)
    let llm_response = r#"Based on the text, here's the extracted task information:

```json
{
  "title": "Complete the documentation",
  "status": "PENDING",
  "priority": "5"
}
```

The task is pending with high priority."#;

    println!("Raw LLM Response:");
    println!("{}", "=".repeat(60));
    println!("{}", llm_response);
    println!("{}\n", "=".repeat(60));

    let parsed_result = parse_llm_response_with_ir(
        &ir,
        llm_response,
        &FieldType::Class("Task".to_string())
    )?;

    println!("Parsed Result (with type coercion):");
    println!("{}", "=".repeat(60));
    println!("{:#?}", parsed_result);
    println!("{}\n", "=".repeat(60));

    // Verify the parsing worked correctly
    if let BamlValue::Map(map) = parsed_result {
        println!("✅ Verification:");
        println!("  - Title: {:?}", map.get("title").and_then(|v| v.as_string()));
        println!("  - Status: {:?} (case-normalized from 'PENDING')", map.get("status").and_then(|v| v.as_string()));
        println!("  - Priority: {:?} (coerced from string \"5\" to int)", map.get("priority").and_then(|v| v.as_int()));
    }

    println!("\n=== Key Benefits ===");
    println!("✨ These functions can be used independently:");
    println!("   - generate_prompt_from_ir() - No need for full runtime to generate prompts");
    println!("   - parse_llm_response_with_ir() - Parse responses from any LLM client");
    println!("✨ Easier testing of each phase separately");
    println!("✨ More flexible for custom workflows");

    Ok(())
}
