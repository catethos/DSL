/// Example: Extract Person Information
///
/// This example demonstrates how to use the simplified BAML runtime to:
/// 1. Define types (Class, Enum, Function)
/// 2. Set up an LLM client
/// 3. Execute a function that calls an LLM
/// 4. Parse and validate the response
///
/// To run this example:
/// ```bash
/// export OPENAI_API_KEY="your-api-key-here"
/// cargo run --example extract_person
/// ```
use simplify_baml::*;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Simplified BAML Example: Extract Person ===\n");

    // Step 1: Build the IR (Intermediate Representation)
    let ir = build_ir();

    // Step 2: Configure the LLM client
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Warning: OPENAI_API_KEY not set. Using mock response.");
        "mock".to_string()
    });

    let client = if api_key == "mock" {
        // For demo purposes without API key
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

    Ok(())
}

/// Build the IR with type definitions and function
fn build_ir() -> IR {
    let mut ir = IR::new();

    // Define Month enum
    ir.enums.push(Enum {
        name: "Month".to_string(),
        description: Some("Calendar month of the year".to_string()),
        values: vec![
            "January".to_string(),
            "February".to_string(),
            "March".to_string(),
            "April".to_string(),
            "May".to_string(),
            "June".to_string(),
            "July".to_string(),
            "August".to_string(),
            "September".to_string(),
            "October".to_string(),
            "November".to_string(),
            "December".to_string(),
        ],
    });

    // Define Person class
    ir.classes.push(Class {
        name: "Person".to_string(),
        description: Some("Information about a person".to_string()),
        fields: vec![
            Field {
                name: "name".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: Some("Full name of the person".to_string()),
            },
            Field {
                name: "age".to_string(),
                field_type: FieldType::Int,
                optional: false,
                description: Some("Age in years".to_string()),
            },
            Field {
                name: "birthMonth".to_string(),
                field_type: FieldType::Enum("Month".to_string()),
                optional: true,
                description: Some("Month of birth, if mentioned".to_string()),
            },
            Field {
                name: "occupation".to_string(),
                field_type: FieldType::String,
                optional: true,
                description: Some("Job title or profession, if mentioned".to_string()),
            },
        ],
    });

    // Define the extraction function
    ir.functions.push(Function {
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
    });

    ir
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
