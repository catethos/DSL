/// Example: Using OpenRouter with BAML
///
/// OpenRouter (https://openrouter.ai) provides unified access to multiple LLM providers
/// through an OpenAI-compatible API. This example shows how to use it with BAML.
///
/// Benefits of OpenRouter:
/// - Access to 100+ models from different providers
/// - Single API key for all providers
/// - Automatic failover and load balancing
/// - Pay-as-you-go pricing
///
/// To run this example:
/// ```bash
/// export OPENROUTER_API_KEY="your-openrouter-api-key"
/// cargo run --example openrouter_example
/// ```
///
/// Get your API key at: https://openrouter.ai/keys

use simplify_baml::*;
use simplify_baml_macros::{BamlSchema, BamlClient};
use std::collections::HashMap;
use std::env;

// Define our data types
#[derive(BamlSchema)]
#[baml(description = "Information about a person")]
struct Person {
    #[baml(description = "Full name of the person")]
    name: String,

    #[baml(description = "Age in years")]
    age: i64,

    #[baml(description = "Occupation or job title")]
    occupation: Option<String>,
}

// Define BAML function
#[baml_function(client = "openrouter")]
fn extract_person(
    #[baml(description = "Text containing person information")]
    text: String
) -> Person {
    r#"Extract the person's information from the following text:

{{ text }}

Please extract: name, age, and occupation (if mentioned)."#
}

// Configure OpenRouter client using the Custom provider
// You can use any model available on OpenRouter
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "https://openrouter.ai/api/v1",
    model = "anthropic/claude-3.5-sonnet"
)]
struct OpenRouterClient;

// Alternative: Create clients for different models
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "https://openrouter.ai/api/v1",
    model = "openai/gpt-4-turbo"
)]
struct OpenRouterGPT4;

#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "https://openrouter.ai/api/v1",
    model = "google/gemini-pro"
)]
struct OpenRouterGemini;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== OpenRouter Example ===\n");

    // Get API key from environment
    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("Error: OPENROUTER_API_KEY not set.");
        eprintln!("Get your API key at: https://openrouter.ai/keys");
        eprintln!("Then run: export OPENROUTER_API_KEY='your-key-here'");
        std::process::exit(1);
    });

    // Build IR from types
    let ir = BamlSchemaRegistry::new()
        .register::<Person>()
        .build_with_functions(vec![extract_person()]);

    println!("📊 Built IR with:");
    println!("   - {} classes", ir.classes.len());
    println!("   - {} functions\n", ir.functions.len());

    // Create OpenRouter client (using Claude 3.5 Sonnet via OpenRouter)
    let client = OpenRouterClient::new(api_key.clone());

    println!("🌐 Using OpenRouter with model: anthropic/claude-3.5-sonnet");
    println!("   Base URL: https://openrouter.ai/api/v1\n");

    // Build runtime
    let runtime = RuntimeBuilder::new()
        .ir(ir)
        .client("openrouter", client)
        .build();

    // Prepare test data
    let test_cases = vec![
        "Alice Johnson is 28 years old and works as a data scientist.",
        "Meet Bob Smith, a 45-year-old software architect with 20 years of experience.",
        "Dr. Emily Chen, age 35, is a renowned neuroscientist.",
    ];

    // Execute for each test case
    for (i, text) in test_cases.iter().enumerate() {
        println!("--- Test Case {} ---", i + 1);
        println!("Input: {}\n", text);

        let mut params = HashMap::new();
        params.insert("text".to_string(), BamlValue::String(text.to_string()));

        match runtime.execute("ExtractPerson", params).await {
            Ok(result) => {
                println!("✅ Extracted:");
                print_result(&result);
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
            }
        }
        println!();
    }

    println!("\n=== Available OpenRouter Models ===");
    println!("
You can use any model from OpenRouter by changing the 'model' attribute:

Anthropic:
  - anthropic/claude-3.5-sonnet
  - anthropic/claude-3-opus
  - anthropic/claude-3-haiku

OpenAI:
  - openai/gpt-4-turbo
  - openai/gpt-4
  - openai/gpt-3.5-turbo

Google:
  - google/gemini-pro
  - google/gemini-pro-vision

Meta:
  - meta-llama/llama-3-70b-instruct
  - meta-llama/llama-3-8b-instruct

And many more! See: https://openrouter.ai/models
");

    println!("\n=== Using Multiple Models ===");
    println!("
You can create multiple clients for different models:

```rust
#[derive(BamlClient)]
#[baml(
    provider = \"Custom\",
    base_url = \"https://openrouter.ai/api/v1\",
    model = \"anthropic/claude-3.5-sonnet\"
)]
struct ClaudeClient;

#[derive(BamlClient)]
#[baml(
    provider = \"Custom\",
    base_url = \"https://openrouter.ai/api/v1\",
    model = \"openai/gpt-4-turbo\"
)]
struct GPT4Client;

// Register both
let runtime = RuntimeBuilder::new()
    .ir(ir)
    .client(\"claude\", ClaudeClient::new(api_key.clone()))
    .client(\"gpt4\", GPT4Client::new(api_key.clone()))
    .build();

// Use different models for different functions
#[baml_function(client = \"claude\")]
fn with_claude(text: String) -> Person {{ /* ... */ }}

#[baml_function(client = \"gpt4\")]
fn with_gpt4(text: String) -> Person {{ /* ... */ }}
```
");

    Ok(())
}

/// Pretty print the result
fn print_result(value: &BamlValue) {
    match value {
        BamlValue::Map(map) => {
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
        }
        _ => println!("{:?}", value),
    }
}
