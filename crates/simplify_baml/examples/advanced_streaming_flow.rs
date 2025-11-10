use anyhow::Result;
use futures::StreamExt;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest};
use genai::Client;
/// Advanced example showing detailed flow with partial parsing visualization
///
/// This demonstrates:
/// 1. The complete current flow for structured LLM output generation
/// 2. How partial parsing works at each streaming stage
/// 3. Real-world patterns for handling streaming responses
use simplify_baml::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n{}", "=".repeat(70));
    println!("  🚀 Advanced Streaming Flow: Detailed Walkthrough");
    println!("{}", "=".repeat(70));
    println!();

    println!("This example demonstrates streaming with partial parsing.");

    // ============================================================================
    // STEP 1: DEFINE COMPLEX SCHEMA
    // ============================================================================
    println!("📋 STEP 1: Define Complex Schema");
    println!("{}", "-".repeat(70));

    let mut ir = IR::new();

    // Define multiple enums
    ir.enums.push(Enum {
        name: "Priority".to_string(),
        description: Some("Task priority level".to_string()),
        values: vec![
            "Low".to_string(),
            "Medium".to_string(),
            "High".to_string(),
            "Critical".to_string(),
        ],
    });

    ir.enums.push(Enum {
        name: "Status".to_string(),
        description: Some("Task completion status".to_string()),
        values: vec![
            "NotStarted".to_string(),
            "InProgress".to_string(),
            "Completed".to_string(),
        ],
    });

    // Define nested class for Assignee
    ir.classes.push(Class {
        name: "Assignee".to_string(),
        description: Some("Person assigned to a task".to_string()),
        fields: vec![
            Field {
                name: "name".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Full name".to_string()),
            },
            Field {
                name: "email".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Email address".to_string()),
            },
        ],
    });

    // Define main Task class with nested types
    ir.classes.push(Class {
        name: "Task".to_string(),
        description: Some("A project task".to_string()),
        fields: vec![
            Field {
                name: "title".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Task title".to_string()),
            },
            Field {
                name: "description".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Detailed description".to_string()),
            },
            Field {
                name: "priority".to_string(),
                field_type: FieldType::Enum("Priority".to_string()),
                optional: false,
                description: Some("Priority level".to_string()),
            },
            Field {
                name: "status".to_string(),
                field_type: FieldType::Enum("Status".to_string()),
                optional: false,
                description: Some("Current status".to_string()),
            },
            Field {
                name: "assignee".to_string(),
                field_type: FieldType::Class("Assignee".to_string()),
                optional: false,
                description: Some("Assigned person".to_string()),
            },
            Field {
                name: "tags".to_string(),
                field_type: FieldType::List(Box::new(FieldType::String)),
                optional: false,
                description: Some("Tags for categorization".to_string()),
            },
            Field {
                name: "estimatedHours".to_string(),
                field_type: FieldType::Float,
                optional: false,
                description: Some("Estimated time in hours".to_string()),
            },
        ],
    });

    let target_type = FieldType::Class("Task".to_string());

    println!("✅ Defined schema:");
    println!("   - 2 enums: Priority, Status");
    println!("   - 2 classes: Assignee (nested), Task (main)");
    println!("   - 7 fields with various types");
    println!();

    // ============================================================================
    // STEP 2: GENERATE PROMPT
    // ============================================================================
    println!("📝 STEP 2: Generate Prompt with Schema");
    println!("{}", "-".repeat(70));

    let user_input = "Create a task for implementing user authentication with OAuth2. \
                      It should be high priority, assigned to Alice Johnson (alice@example.com), \
                      tagged with 'security' and 'backend', and estimated to take 16 hours.";

    // Create parameters for template rendering
    let mut params = HashMap::new();
    params.insert(
        "input".to_string(),
        BamlValue::String(user_input.to_string()),
    );

    let prompt = generate_prompt_from_ir(&ir, "{{ input }}", &params, &target_type)?;

    println!("Generated prompt preview:");
    println!("{}", "·".repeat(70));
    let preview: String = prompt.chars().take(400).collect();
    println!("{}", preview);
    if prompt.len() > 400 {
        println!("... (truncated, {} more chars)", prompt.len() - 400);
    }
    println!("{}", "·".repeat(70));
    println!();

    // ============================================================================
    // STEP 3: SETUP RUST-GENAI CLIENT
    // ============================================================================
    println!("🔌 STEP 3: Setup rust-genai Client");
    println!("{}", "-".repeat(70));

    let client = Client::default();

    // Configure options
    let options = ChatOptions::default()
        .with_temperature(0.7)
        .with_max_tokens(1000);

    let chat_req = ChatRequest::new(vec![
        ChatMessage::system("You are a helpful assistant that extracts structured data. Always respond with valid JSON."),
        ChatMessage::user(prompt),
    ]);

    println!("✅ Client configured:");
    println!("   - Model: gpt-4o-mini (auto-detected)");
    println!("   - Temperature: 0.7");
    println!("   - Max tokens: 1000");
    println!("   - Streaming: enabled");
    println!();

    // ============================================================================
    // STEP 4: STREAM WITH PARTIAL PARSING VISUALIZATION
    // ============================================================================
    println!("📡 STEP 4: Stream Response with Partial Parsing");
    println!("{}", "-".repeat(70));
    println!();

    let mut chat_stream = client
        .exec_chat_stream("gpt-4o-mini", chat_req, Some(&options))
        .await?;

    let mut accumulated_text = String::new();
    let mut chunk_count = 0;
    let mut parse_attempts = 0;
    let mut successful_parses = 0;

    println!("🌊 Streaming started...\n");
    println!("{}", "─".repeat(70));

    // Track what fields have been seen
    let mut seen_fields: Vec<String> = Vec::new();

    use genai::chat::ChatStreamEvent;
    while let Some(event_result) = chat_stream.stream.next().await {
        let event = event_result?;

        match event {
            ChatStreamEvent::Chunk(chunk) => {
                let text = &chunk.content;
                chunk_count += 1;
                accumulated_text.push_str(text);

                // Every 3 chunks, try partial parsing and show results
                if chunk_count % 3 == 0 {
                    parse_attempts += 1;

                    print!(
                        "\n📦 Chunk {} | Attempting parse #{}",
                        chunk_count, parse_attempts
                    );

                    match try_parse_partial_response(&ir, &accumulated_text, &target_type) {
                        Ok(Some(partial_value)) => {
                            successful_parses += 1;
                            print!(" ✅");

                            // Show incremental progress
                            if let BamlValue::Map(map) = &partial_value {
                                let current_fields: Vec<String> = map.keys().cloned().collect();
                                let new_fields: Vec<String> = current_fields
                                    .iter()
                                    .filter(|f| !seen_fields.contains(f))
                                    .cloned()
                                    .collect();

                                if !new_fields.is_empty() {
                                    print!(" [New: {}]", new_fields.join(", "));
                                    seen_fields.extend(new_fields);
                                }

                                println!("\n   Parsed fields so far: {:?}", current_fields);
                            }
                        }
                        Ok(None) => {
                            print!(" ⏳ (waiting for more data)");
                            println!();
                        }
                        Err(e) => {
                            print!(" ❌ ({})", e);
                            println!();
                        }
                    }
                }
            }
            _ => {} // Ignore other events
        }
    }

    println!("\n{}", "─".repeat(70));
    println!("🌊 Streaming complete!");
    println!();

    // ============================================================================
    // STEP 5: PARSE FINAL RESULT
    // ============================================================================
    println!("✅ STEP 5: Parse Final Structured Result");
    println!("{}", "-".repeat(70));

    let final_value = parse_llm_response_with_ir(&ir, &accumulated_text, &target_type)?;

    println!("\nFull structured output:");
    println!("{:#?}", final_value);
    println!();

    // ============================================================================
    // STEP 6: EXTRACT AND VALIDATE DATA
    // ============================================================================
    println!("📊 STEP 6: Extract and Validate Data");
    println!("{}", "-".repeat(70));

    if let BamlValue::Map(map) = &final_value {
        println!("\n✅ Successfully extracted Task:");
        println!();

        println!(
            "  📌 Title: {}",
            map.get("title")
                .and_then(|v| v.as_string())
                .unwrap_or("<missing>")
        );

        println!(
            "  📝 Description: {}",
            map.get("description")
                .and_then(|v| v.as_string())
                .unwrap_or("<missing>")
        );

        println!(
            "  ⚡ Priority: {}",
            map.get("priority")
                .and_then(|v| v.as_string())
                .unwrap_or("<missing>")
        );

        println!(
            "  📈 Status: {}",
            map.get("status")
                .and_then(|v| v.as_string())
                .unwrap_or("<missing>")
        );

        if let Some(BamlValue::Map(assignee)) = map.get("assignee") {
            println!("  👤 Assignee:");
            println!(
                "     - Name: {}",
                assignee
                    .get("name")
                    .and_then(|v| v.as_string())
                    .unwrap_or("<missing>")
            );
            println!(
                "     - Email: {}",
                assignee
                    .get("email")
                    .and_then(|v| v.as_string())
                    .unwrap_or("<missing>")
            );
        }

        if let Some(BamlValue::List(tags)) = map.get("tags") {
            print!("  🏷️  Tags: [");
            for (i, tag) in tags.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", tag.as_string().unwrap_or("<?>"));
            }
            println!("]");
        }

        println!(
            "  ⏱️  Estimated Hours: {}",
            map.get("estimatedHours")
                .and_then(|v| v.as_float())
                .map(|f| f.to_string())
                .unwrap_or("<missing>".to_string())
        );
    }

    println!();
    println!("{}", "=".repeat(70));
    println!("📊 Statistics:");
    println!("  Total chunks: {}", chunk_count);
    println!("  Parse attempts: {}", parse_attempts);
    println!("  Successful parses: {}", successful_parses);
    println!("{}", "=".repeat(70));

    Ok(())
}
