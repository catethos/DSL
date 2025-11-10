/// Example: Complete BAML with All Macros
///
/// This example demonstrates the full power of BAML macros:
/// - `#[derive(BamlSchema)]` for automatic class/enum IR generation
/// - `#[baml_function]` for function definition
/// - `#[derive(BamlClient)]` for client configuration
///
/// Compare this with the manual approach to see the dramatic improvement!
///
/// To run this example:
/// ```bash
/// export OPENAI_API_KEY="your-api-key-here"
/// cargo run --example with_macros
/// ```

use simplify_baml::*;
use simplify_baml_macros::{BamlSchema, BamlClient};
use std::collections::HashMap;
use std::env;

// Step 1: Define types using derive macros
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

// Step 2: Define BAML function using macro
#[baml_function(client = "openai")]
fn extract_person(
    #[baml(description = "Text containing person information to extract")]
    text: String
) -> Person {
    r#"Extract the person's information from the following text:

{{ text }}

Please extract: name, age, birth month (if mentioned), and occupation (if mentioned)."#
}

// Step 3: Define client using derive macro
#[derive(BamlClient)]
#[baml(provider = "OpenAI", model = "gpt-4o-mini")]
struct OpenAIClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== BAML Complete Macro Example ===\n");
    println!("✨ Using all three macros for maximum productivity!\n");

    // Build IR automatically from types
    let ir = BamlSchemaRegistry::new()
        .register::<Month>()
        .register::<Person>()
        .build_with_functions(vec![extract_person()]);

    println!("📊 IR built with {} classes, {} enums, {} functions",
        ir.classes.len(),
        ir.enums.len(),
        ir.functions.len()
    );

    // Print what was generated
    for class in &ir.classes {
        println!("   - Class: {} ({} fields)", class.name, class.fields.len());
    }
    for enum_def in &ir.enums {
        println!("   - Enum: {} ({} variants)", enum_def.name, enum_def.values.len());
    }
    for func in &ir.functions {
        println!("   - Function: {} ({} inputs -> {})",
            func.name,
            func.inputs.len(),
            func.output.to_string()
        );
    }
    println!();

    // Configure the LLM client using macro-generated struct
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Warning: OPENAI_API_KEY not set. Using mock mode.");
        "mock".to_string()
    });

    let client = OpenAIClient::new(api_key.clone());

    // Build the runtime
    let runtime = RuntimeBuilder::new()
        .ir(ir)
        .client("openai", client)
        .build();

    // Prepare input parameters
    let mut params = HashMap::new();
    params.insert(
        "text".to_string(),
        BamlValue::String(
            "John Smith is 30 years old and was born in March. He works as a software engineer."
                .to_string(),
        ),
    );

    println!("Input text: {}", params.get("text").unwrap().as_string().unwrap());
    println!("\nExecuting BAML function 'ExtractPerson'...\n");

    // Execute the function
    match runtime.execute("ExtractPerson", params).await {
        Ok(result) => {
            println!("✅ Success! Parsed result:");
            print_result(&result);
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            eprintln!("\nNote: If you haven't set OPENAI_API_KEY, this is expected.");
            eprintln!("Set it with: export OPENAI_API_KEY='your-key-here'");
        }
    }

    println!("\n=== Code Comparison ===");
    println!("Manual approach:");
    println!("  - Class definition: ~15 lines");
    println!("  - Enum definition: ~15 lines");
    println!("  - Function definition: ~20 lines");
    println!("  - Client setup: ~3 lines");
    println!("  - Total: ~53 lines");
    println!();
    println!("With macros:");
    println!("  - #[derive(BamlSchema)] on struct: ~12 lines");
    println!("  - #[derive(BamlSchema)] on enum: ~10 lines");
    println!("  - #[baml_function]: ~8 lines");
    println!("  - #[derive(BamlClient)]: ~3 lines");
    println!("  - Total: ~33 lines");
    println!();
    println!("💡 Savings: ~38% less code, more readable, and type-safe!");
    println!("💡 All macros now use consistent #[derive(...)] syntax!");

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
