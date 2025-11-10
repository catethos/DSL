use anyhow::Result;
use futures::StreamExt;
use genai::chat::{ChatMessage, ChatRequest, ChatStreamEvent};
use genai::Client;
/// Example showing partial parsing results explicitly at each step
///
/// This demonstrates how partial parsing progressively builds up structured data
/// as chunks arrive from the streaming LLM response.
use simplify_baml::*;
use std::collections::HashMap;

fn print_partial_result(label: &str, result: &Option<BamlValue>) {
    println!("\n{}", "─".repeat(70));
    println!("📦 {}", label);
    println!("{}", "─".repeat(70));

    match result {
        Some(value) => {
            if let BamlValue::Map(map) = value {
                println!("Parsed fields:");
                for (key, val) in map {
                    match val {
                        BamlValue::String(s) => println!("  • {}: \"{}\"", key, s),
                        BamlValue::Int(i) => println!("  • {}: {}", key, i),
                        BamlValue::Float(f) => println!("  • {}: {}", key, f),
                        BamlValue::List(items) => {
                            println!("  • {}: [", key);
                            for item in items {
                                if let BamlValue::String(s) = item {
                                    println!("      \"{}\",", s);
                                }
                            }
                            println!("    ]");
                        }
                        BamlValue::Map(nested) => {
                            println!("  • {}: {{", key);
                            for (k, v) in nested {
                                if let BamlValue::String(s) = v {
                                    println!("      {}: \"{}\"", k, s);
                                } else {
                                    println!("      {}: {:?}", k, v);
                                }
                            }
                            println!("    }}");
                        }
                        _ => println!("  • {}: {:?}", key, val),
                    }
                }
            } else {
                println!("{:#?}", value);
            }
        }
        None => {
            println!("❌ Cannot parse yet - need more data");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n{}", "=".repeat(70));
    println!("  Explicit Partial Parsing Example");
    println!("{}", "=".repeat(70));
    println!();

    // ========================================
    // STEP 1: Define Schema
    // ========================================
    let mut ir = IR::new();

    ir.classes.push(Class {
        name: "Person".to_string(),
        description: Some("A person with contact information".to_string()),
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
                name: "email".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Email address".to_string()),
            },
            Field {
                name: "hobbies".to_string(),
                field_type: FieldType::List(Box::new(FieldType::String)),
                optional: false,
                description: Some("List of hobbies".to_string()),
            },
        ],
    });

    let target_type = FieldType::Class("Person".to_string());

    println!("✅ Defined Person schema with 4 fields: name, age, email, hobbies\n");

    // ========================================
    // STEP 2: Generate Prompt
    // ========================================
    let mut params = HashMap::new();
    params.insert(
        "input".to_string(),
        BamlValue::String("Extract: Alex Thompson, 28 years old, contact alex.t@email.com, enjoys hiking, photography, and cooking.".to_string())
    );

    let prompt = generate_prompt_from_ir(&ir, "{{ input }}", &params, &target_type)?;

    // ========================================
    // STEP 3: Stream and Parse Incrementally
    // ========================================
    println!("🌊 Streaming LLM response...\n");
    println!("{}", "=".repeat(70));

    let client = Client::default();
    let chat_req = ChatRequest::new(vec![
        ChatMessage::system("You are a data extraction assistant. Always respond with valid JSON matching the schema."),
        ChatMessage::user(prompt),
    ]);

    let mut chat_stream = client
        .exec_chat_stream("gpt-4o-mini", chat_req, None)
        .await?;

    let mut accumulated = String::new();
    let mut chunk_count = 0;
    let mut last_successful_parse: Option<BamlValue> = None;

    println!("Streaming chunks:");
    println!();

    while let Some(event_result) = chat_stream.stream.next().await {
        let event = event_result?;

        match event {
            ChatStreamEvent::Chunk(chunk) => {
                let text = &chunk.content;
                chunk_count += 1;
                accumulated.push_str(text);

                // Print the new text
                print!("{}", text);
                std::io::Write::flush(&mut std::io::stdout())?;

                // Try to parse every 5 chunks
                if chunk_count % 5 == 0 {
                    match try_parse_partial_response(&ir, &accumulated, &target_type)? {
                        Some(parsed) => {
                            last_successful_parse = Some(parsed.clone());
                            print_partial_result(
                                &format!(
                                    "Partial Parse #{} (after chunk {})",
                                    chunk_count / 5,
                                    chunk_count
                                ),
                                &Some(parsed),
                            );
                        }
                        None => {
                            // Optionally show parse failures
                            // print_partial_result(&format!("Parse attempt after chunk {}", chunk_count), &None);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    println!("\n\n{}", "=".repeat(70));
    println!("✅ Streaming complete!");
    println!("{}", "=".repeat(70));

    // ========================================
    // STEP 4: Final Parse
    // ========================================
    println!("\n🎯 Final Parse:");
    println!("{}", "=".repeat(70));

    let final_value = parse_llm_response_with_ir(&ir, &accumulated, &target_type)?;

    if let BamlValue::Map(map) = &final_value {
        println!("\n✅ Complete structured data:\n");
        println!("Person {{");
        println!(
            "  name: \"{}\"",
            map.get("name").and_then(|v| v.as_string()).unwrap_or("")
        );
        println!(
            "  age: {}",
            map.get("age").and_then(|v| v.as_int()).unwrap_or(0)
        );
        println!(
            "  email: \"{}\"",
            map.get("email").and_then(|v| v.as_string()).unwrap_or("")
        );

        if let Some(BamlValue::List(hobbies)) = map.get("hobbies") {
            println!("  hobbies: [");
            for hobby in hobbies {
                if let Some(h) = hobby.as_string() {
                    println!("    \"{}\",", h);
                }
            }
            println!("  ]");
        }
        println!("}}");
    }

    println!("\n{}", "=".repeat(70));
    println!("Total chunks processed: {}", chunk_count);
    println!("Successful partial parses: {}", chunk_count / 5);
    println!("{}", "=".repeat(70));

    Ok(())
}
