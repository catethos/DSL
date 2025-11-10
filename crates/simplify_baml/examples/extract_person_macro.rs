/// Example: Extract Person Information with Macros
///
/// This example demonstrates how to use the `#[derive(BamlSchema)]` macro
/// to automatically generate IR from native Rust types. Compare this with
/// the manual approach in `extract_person.rs` to see how much simpler it is!
///
/// To run this example:
/// ```bash
/// export OPENAI_API_KEY="your-api-key-here"
/// cargo run --example extract_person_macro
/// ```

use simplify_baml::*;
use simplify_baml_macros::BamlSchema;
use std::collections::HashMap;
use std::env;

// Define types using derive macros - much cleaner than manual IR building!

#[derive(BamlSchema)]
#[baml(description = "Calendar month of the year")]
enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

#[derive(BamlSchema)]
#[baml(description = "Information about a person")]
struct Person {
    #[baml(description = "Full name of the person")]
    name: String,

    #[baml(description = "Age in years")]
    age: i64,

    #[baml(description = "Month of birth, if mentioned")]
    #[baml(rename = "birthMonth")]
    birth_month: Option<Month>,

    #[baml(description = "Job title or profession, if mentioned")]
    occupation: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== BAML Macro Example: Extract Person ===\n");
    println!("✨ Using #[derive(BamlSchema)] to automatically generate IR!\n");

    // Step 1: Build IR using the registry - automatically collects all schema definitions
    let ir = BamlSchemaRegistry::new()
        .register::<Month>()
        .register::<Person>()
        .build_with_functions(vec![Function {
            name: "ExtractPerson".to_string(),
            inputs: vec![Field {
                name: "text".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Text containing person information to extract".to_string()),
            }],
            output: FieldType::Class("Person".to_string()),
            prompt_template: r#"Extract the person's information from the following text:

{{ text }}

Please extract: name, age, birth month (if mentioned), and occupation (if mentioned)."#
                .to_string(),
            client: "openai".to_string(),
        }]);

    println!("📊 IR built successfully with {} classes and {} enums",
        ir.classes.len(),
        ir.enums.len()
    );

    // Print what was generated
    for class in &ir.classes {
        println!("   - Class: {} ({} fields)", class.name, class.fields.len());
    }
    for enum_def in &ir.enums {
        println!("   - Enum: {} ({} variants)", enum_def.name, enum_def.values.len());
    }
    println!();

    // Step 2: Configure the LLM client
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Warning: OPENAI_API_KEY not set. Using mock response.");
        "mock".to_string()
    });

    let client = if api_key == "mock" {
        println!("Using mock mode (no real API calls)\n");
        LLMClient::openai(api_key, "gpt-4o-mini".to_string())
    } else {
        LLMClient::openai(api_key, "gpt-4o-mini".to_string())
    };

    // Step 3: Build the runtime
    let runtime = RuntimeBuilder::new()
        .ir(ir)
        .client("openai", client)
        .build();

    // Step 4: Prepare input parameters
    let mut params = HashMap::new();
    params.insert(
        "text".to_string(),
        BamlValue::String(
            "John Smith is 30 years old and was born in March. He works as a software engineer."
                .to_string(),
        ),
    );

    println!(
        "Input text: {}",
        params.get("text").unwrap().as_string().unwrap()
    );
    println!("\nExecuting BAML function 'ExtractPerson'...\n");

    // Step 5: Execute the function
    match runtime.execute("ExtractPerson", params).await {
        Ok(result) => {
            println!("Success! Parsed result:");
            print_result(&result);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("\nNote: If you haven't set OPENAI_API_KEY, this is expected.");
            eprintln!("Set it with: export OPENAI_API_KEY='your-key-here'");
        }
    }

    println!("\n=== Comparison ===");
    println!("Without macros: ~70 lines of manual IR construction");
    println!("With macros: ~25 lines of struct/enum definitions");
    println!("Savings: Less tedious, more readable, and type-safe!");

    Ok(())
}

/// Pretty print the result
fn print_result(value: &BamlValue) {
    match value {
        BamlValue::Map(map) => {
            println!("{{");
            for (key, val) in map {
                print!("  {}: ", key);
                match val {
                    BamlValue::String(s) => println!("\"{}\"", s),
                    BamlValue::Int(i) => println!("{}", i),
                    BamlValue::Float(f) => println!("{}", f),
                    BamlValue::Bool(b) => println!("{}", b),
                    BamlValue::Null => println!("null"),
                    _ => println!("{:?}", val),
                }
            }
            println!("}}");
        }
        _ => println!("{:?}", value),
    }
}
