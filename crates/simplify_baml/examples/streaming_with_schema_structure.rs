/// Example: Schema-Aware Streaming (RECOMMENDED for UIs)
///
/// This example shows the improved streaming UX where the UI always receives
/// the full schema structure, with fields filled in progressively.
///
/// This is much better for frontend rendering because:
/// - Structure never changes (no layout shift)
/// - Can show loading states per field
/// - Easy to render empty states
/// - Predictable component structure

use simplify_baml::*;

fn main() -> anyhow::Result<()> {
    println!("=== Schema-Aware Streaming Example ===\n");
    println!("💡 The UI always gets the FULL structure!\n");

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
            Field {
                name: "email".to_string(),
                field_type: FieldType::String,
                optional: true,
                description: Some("Email address".to_string()),
            },
        ],
    });

    let target_type = FieldType::Class("Person".to_string());

    // 1. Create skeleton from IR (UI knows structure upfront!)
    let mut streaming = StreamingBamlValue::from_ir_skeleton(&ir, &target_type);

    println!("📋 Initial skeleton (before any data):");
    println!("{}", serde_json::to_string_pretty(&streaming)?);
    println!();

    // Simulate streaming chunks
    let chunks = vec![
        (r#"```json"#, false),
        (r#"```json
{"name": "Al"#, false),
        (r#"```json
{"name": "Alice", "age": 2"#, false),
        (r#"```json
{"name": "Alice", "age": 28, "occupation": "Data Sci"#, false),
        (r#"```json
{"name": "Alice", "age": 28, "occupation": "Data Scientist"}"#, false),
        (r#"```json
{"name": "Alice", "age": 28, "occupation": "Data Scientist", "email": "alice@example.com"}
```"#, true),
    ];

    let mut accumulated = String::new();

    for (i, (chunk, is_final)) in chunks.iter().enumerate() {
        println!("📦 Chunk {}: {}", i + 1, if *is_final { "(FINAL)" } else { "" });
        println!("{}", "=".repeat(70));

        accumulated = chunk.to_string();

        // Update the streaming value
        update_streaming_response(&mut streaming, &ir, &accumulated, &target_type, *is_final)?;

        // Display as JSON (what the UI would receive)
        println!("{}", serde_json::to_string_pretty(&streaming)?);
        println!();

        // Show how a UI could render this
        render_ui(&streaming);
        println!();
    }

    println!("{}", "=".repeat(70));
    println!("🎉 Stream complete!\n");

    println!("=== Why This is Better for UIs ===");
    println!("✅ Structure is consistent from start to finish");
    println!("✅ No layout shift as fields appear");
    println!("✅ Can show loading spinners for null fields");
    println!("✅ Easy to map to React/Vue/etc components");
    println!("✅ Completion state lets you disable 'Submit' buttons");

    Ok(())
}

/// Example of how a UI might render the streaming value
fn render_ui(streaming: &StreamingBamlValue) {
    let state_emoji = match streaming.completion_state {
        CompletionState::Pending => "⏳",
        CompletionState::Partial => "🔄",
        CompletionState::Complete => "✅",
    };

    println!("🖥️  UI Rendering {} {}:", state_emoji, format!("{:?}", streaming.completion_state));

    if let BamlValue::Map(map) = &streaming.value {
        println!("┌─────────────────────────────────────┐");
        println!("│ Person Information                  │");
        println!("├─────────────────────────────────────┤");

        // Name
        print!("│ Name:       ");
        match map.get("name") {
            Some(BamlValue::String(s)) if !s.is_empty() => println!("{:<24} │", s),
            Some(BamlValue::Null) | None => println!("{:<24} │", "[Loading...]"),
            _ => println!("{:<24} │", "[Error]"),
        }

        // Age
        print!("│ Age:        ");
        match map.get("age") {
            Some(BamlValue::Int(i)) => println!("{:<24} │", i),
            Some(BamlValue::Null) | None => println!("{:<24} │", "[Loading...]"),
            _ => println!("{:<24} │", "[Error]"),
        }

        // Occupation
        print!("│ Occupation: ");
        match map.get("occupation") {
            Some(BamlValue::String(s)) if !s.is_empty() => println!("{:<24} │", s),
            Some(BamlValue::Null) | None => println!("{:<24} │", "[Loading...]"),
            _ => println!("{:<24} │", "[Error]"),
        }

        // Email (optional)
        print!("│ Email:      ");
        match map.get("email") {
            Some(BamlValue::String(s)) if !s.is_empty() => println!("{:<24} │", s),
            Some(BamlValue::Null) | None => println!("{:<24} │", "(optional)"),
            _ => println!("{:<24} │", "[Error]"),
        }

        println!("└─────────────────────────────────────┘");
    }
}
