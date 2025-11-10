use anyhow::Result;
use futures::StreamExt;
use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;
/// Example demonstrating rust-genai integration with partial parsing
///
/// This example shows the complete flow:
/// 1. Define schema using simplify_baml IR
/// 2. Generate prompt with schema instructions
/// 3. Stream LLM response using rust-genai
/// 4. Parse partial JSON incrementally as it streams
/// 5. Get structured BamlValue output
use simplify_baml::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Rust-Genai Streaming with Partial Parsing Example ===\n");

    // ========================================
    // STEP 1: Define Schema using IR
    // ========================================
    println!("📋 Step 1: Define Schema\n");

    let mut ir = IR::new();

    // Define an enum for job level
    ir.enums.push(Enum {
        name: "JobLevel".to_string(),
        description: Some("Experience level".to_string()),
        values: vec![
            "Junior".to_string(),
            "Mid".to_string(),
            "Senior".to_string(),
            "Staff".to_string(),
        ],
    });

    // Define a Person class with multiple fields
    ir.classes.push(Class {
        name: "Person".to_string(),
        description: Some("A person with professional information".to_string()),
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
            Field {
                name: "level".to_string(),
                field_type: FieldType::Enum("JobLevel".to_string()),
                optional: false,
                description: Some("Experience level".to_string()),
            },
            Field {
                name: "skills".to_string(),
                field_type: FieldType::List(Box::new(FieldType::String)),
                optional: false,
                description: Some("List of skills".to_string()),
            },
        ],
    });

    let target_type = FieldType::Class("Person".to_string());

    println!("✅ Schema defined: Person class with JobLevel enum\n");

    // ========================================
    // STEP 2: Generate Prompt with Schema
    // ========================================
    println!("📝 Step 2: Generate Prompt\n");

    // Create parameters for template rendering
    let mut params = HashMap::new();
    params.insert(
        "input".to_string(),
        BamlValue::String("Extract information about: Sarah Chen, a 32-year-old Staff Software Engineer specializing in Rust, distributed systems, and cloud architecture.".to_string())
    );

    let prompt = generate_prompt_from_ir(&ir, "{{ input }}", &params, &target_type)?;

    println!("Generated prompt:");
    println!("{}", "-".repeat(60));
    println!("{}", prompt);
    println!("{}", "-".repeat(60));
    println!();

    // ========================================
    // STEP 3: Setup rust-genai Client
    // ========================================
    println!("🔌 Step 3: Setup rust-genai Client\n");

    let client = Client::default();

    let chat_req = ChatRequest::new(vec![ChatMessage::user(prompt)]);

    println!("✅ Client configured to use model: gpt-4o-mini\n");

    // ========================================
    // STEP 4: Stream LLM Response
    // ========================================
    println!("📡 Step 4: Stream LLM Response with Partial Parsing\n");
    println!("{}", "=".repeat(60));

    // Execute streaming chat
    let mut chat_stream = client
        .exec_chat_stream("gpt-4o-mini", chat_req, None)
        .await?;

    let mut accumulated_text = String::new();
    let mut last_parsed_value: Option<BamlValue> = None;
    let mut chunk_count = 0;

    // Process each chunk from the stream
    use genai::chat::ChatStreamEvent;
    while let Some(event_result) = chat_stream.stream.next().await {
        let event = event_result?;

        match event {
            ChatStreamEvent::Chunk(chunk) => {
                let text = &chunk.content;
                chunk_count += 1;
                accumulated_text.push_str(text);

                // Print the streamed text (simulating real-time display)
                print!("{}", text);
                std::io::Write::flush(&mut std::io::stdout())?;

                // Try to parse the partial response every few chunks
                // (In production, you might do this every chunk or on a timer)
                if chunk_count % 5 == 0 {
                    if let Some(parsed) =
                        try_parse_partial_response(&ir, &accumulated_text, &target_type)?
                    {
                        last_parsed_value = Some(parsed);
                    }
                }
            }
            _ => {} // Ignore other events for this example
        }
    }

    println!("\n{}", "=".repeat(60));
    println!();

    // ========================================
    // STEP 5: Parse Final Result
    // ========================================
    println!("✅ Step 5: Parse Final Structured Result\n");

    let final_value = parse_llm_response_with_ir(&ir, &accumulated_text, &target_type)?;

    println!("Structured output:");
    println!("{:#?}", final_value);
    println!();

    // ========================================
    // STEP 6: Extract and Use Data
    // ========================================
    println!("📊 Step 6: Extract and Use Data\n");

    if let BamlValue::Map(map) = &final_value {
        println!("Extracted fields:");
        println!(
            "  Name: {}",
            map.get("name")
                .and_then(|v| v.as_string())
                .unwrap_or("<missing>")
        );
        println!(
            "  Age: {}",
            map.get("age")
                .and_then(|v| v.as_int())
                .map(|i| i.to_string())
                .unwrap_or("<missing>".to_string())
        );
        println!(
            "  Occupation: {}",
            map.get("occupation")
                .and_then(|v| v.as_string())
                .unwrap_or("<missing>")
        );
        println!(
            "  Level: {}",
            map.get("level")
                .and_then(|v| v.as_string())
                .unwrap_or("<missing>")
        );

        if let Some(BamlValue::List(skills)) = map.get("skills") {
            println!("  Skills:");
            for skill in skills {
                if let Some(s) = skill.as_string() {
                    println!("    - {}", s);
                }
            }
        }
    }

    println!();

    // ========================================
    // Summary
    // ========================================
    println!("{}", "=".repeat(60));
    println!("🎉 Example Complete!\n");
    println!("Key Features Demonstrated:");
    println!("  ✨ Schema definition with IR (classes + enums)");
    println!("  ✨ Automatic prompt generation with schema injection");
    println!("  ✨ Streaming LLM calls with rust-genai");
    println!("  ✨ Incremental partial JSON parsing");
    println!("  ✨ Type-safe structured output extraction");
    println!("  ✨ Real-time display of streaming content");

    Ok(())
}
