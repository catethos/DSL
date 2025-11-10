# BAML Macros Documentation

This document describes the macros available in Simplified BAML that make defining LLM functions dramatically simpler.

## Overview

Instead of manually constructing IR structures, you can now use powerful macros with consistent syntax:

1. **`#[derive(BamlSchema)]`** - For class and enum definitions
2. **`#[baml_function]`** - For function definitions
3. **`#[derive(BamlClient)]`** - For client configuration

All macros use Rust's attribute syntax for a consistent, ergonomic API.

## 1. `#[derive(BamlSchema)]` - Type Definitions

Automatically generates BAML IR from Rust structs and enums.

### Basic Usage

```rust
use simplify_baml_macros::BamlSchema;

#[derive(BamlSchema)]
struct Person {
    name: String,
    age: i64,
}

#[derive(BamlSchema)]
enum Month {
    January,
    February,
    March,
    // ...
}
```

### Attributes

#### Type-level attributes

```rust
#[derive(BamlSchema)]
#[baml(description = "Information about a person")]
struct Person {
    name: String,
}
```

#### Field-level attributes

```rust
#[derive(BamlSchema)]
struct Person {
    #[baml(description = "Full name of the person")]
    name: String,

    #[baml(rename = "yearOfBirth")]
    year_of_birth: i64,

    #[baml(description = "Optional email address")]
    email: Option<String>,  // Automatically becomes optional
}
```

### Type Mapping

| Rust Type | BAML FieldType |
|-----------|----------------|
| `String` | `FieldType::String` |
| `i64`, `i32`, `i16`, `i8` | `FieldType::Int` |
| `f64`, `f32` | `FieldType::Float` |
| `bool` | `FieldType::Bool` |
| `Option<T>` | Makes field optional |
| `Vec<T>` | `FieldType::List(T)` |
| Custom type | `FieldType::Class("TypeName")` |

### Complex Example

```rust
#[derive(BamlSchema)]
enum Role {
    Engineer,
    Manager,
    Designer,
}

#[derive(BamlSchema)]
struct Employee {
    name: String,
    role: Role,
    skills: Vec<String>,
    mentor: Option<String>,
}

// Register in IR
let ir = BamlSchemaRegistry::new()
    .register::<Role>()
    .register::<Employee>()
    .build();
```

## 2. `#[baml_function]` - Function Definitions

Define BAML functions using natural Rust function syntax.

### Basic Usage

```rust
use simplify_baml::baml_function;

#[baml_function(client = "openai")]
fn extract_person(text: String) -> Person {
    "Extract person information from: {{ text }}"
}
```

### With Descriptions

```rust
#[baml_function(client = "openai")]
fn extract_person(
    #[baml(description = "Text containing person information to extract")]
    text: String
) -> Person {
    r#"Extract the person's information from the following text:

{{ text }}

Please extract: name, age, and occupation."#
}
```

### Multiple Parameters

```rust
#[baml_function(client = "openai")]
fn summarize_conversation(
    #[baml(description = "The conversation transcript")]
    transcript: String,

    #[baml(description = "Maximum number of sentences in summary")]
    max_sentences: i64
) -> Summary {
    r#"Summarize this conversation in at most {{ max_sentences }} sentences:

{{ transcript }}"#
}
```

### Key Features

- **Automatic naming**: Function name `extract_person` becomes `ExtractPerson` in IR
- **Type-safe**: Input and output types are validated at compile time
- **Jinja2 templates**: Function body is the prompt template (supports Jinja2 syntax)
- **Returns Function**: The macro generates a function that returns a `Function` struct

### Usage in IR

```rust
let ir = BamlSchemaRegistry::new()
    .register::<Person>()
    .build_with_functions(vec![
        extract_person(),
        extract_company(),
    ]);
```

## 3. `#[derive(BamlClient)]` - Client Configuration

Configure LLM clients using a derive macro with attributes - consistent with the BamlSchema pattern.

### OpenAI Client

```rust
use simplify_baml_macros::BamlClient;

#[derive(BamlClient)]
#[baml(provider = "OpenAI", model = "gpt-4o-mini")]
struct MyOpenAI;

// Use it
let client = MyOpenAI::new(std::env::var("OPENAI_API_KEY")?);
```

### Anthropic Client

```rust
#[derive(BamlClient)]
#[baml(provider = "Anthropic", model = "claude-3-sonnet-20240229")]
struct MyAnthropic;

let client = MyAnthropic::new(std::env::var("ANTHROPIC_API_KEY")?);
```

### Custom Endpoint

```rust
#[derive(BamlClient)]
#[baml(provider = "Custom", base_url = "https://api.example.com/v1", model = "my-model")]
struct MyCustom;

let client = MyCustom::new("your-api-key".to_string());
```

### Attributes

- **`provider`** (required): One of `"OpenAI"`, `"Anthropic"`, or `"Custom"`
- **`model`** (required): The model name to use
- **`base_url`** (required for Custom): The base URL for custom endpoints

### Key Features

- **Consistent syntax**: Uses `#[derive(...)]` like BamlSchema
- **Multiple providers**: OpenAI, Anthropic, or custom endpoints
- **Type-safe**: Generates a `new(api_key: String) -> LLMClient` method
- **Reusable**: Define once, use anywhere

## Complete Example

Here's a full example using all three macros:

```rust
use simplify_baml::*;
use simplify_baml_macros::{BamlSchema, BamlClient};
use std::collections::HashMap;

// 1. Define types
#[derive(BamlSchema)]
#[baml(description = "A person entity")]
struct Person {
    #[baml(description = "Full name")]
    name: String,

    #[baml(description = "Age in years")]
    age: i64,

    #[baml(description = "Occupation")]
    job: Option<String>,
}

// 2. Define function
#[baml_function(client = "openai")]
fn extract_person(
    #[baml(description = "Text to extract from")]
    text: String
) -> Person {
    "Extract person info from: {{ text }}"
}

// 3. Define client
#[derive(BamlClient)]
#[baml(provider = "OpenAI", model = "gpt-4o-mini")]
struct OpenAI;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Build IR
    let ir = BamlSchemaRegistry::new()
        .register::<Person>()
        .build_with_functions(vec![extract_person()]);

    // Create client
    let client = OpenAI::new(std::env::var("OPENAI_API_KEY")?);

    // Build runtime
    let runtime = RuntimeBuilder::new()
        .ir(ir)
        .client("openai", client)
        .build();

    // Execute
    let mut params = HashMap::new();
    params.insert("text".to_string(),
        BamlValue::String("John is 30 and works as an engineer".to_string()));

    let result = runtime.execute("ExtractPerson", params).await?;
    println!("{:?}", result);

    Ok(())
}
```

## Benefits Summary

| Feature | Without Macros | With Macros | Savings |
|---------|---------------|-------------|---------|
| Type definitions | ~15 lines/type | ~5 lines/type | ~67% |
| Function definitions | ~20 lines | ~8 lines | ~60% |
| Client setup | ~3 lines | ~3 lines | Same |
| **Overall** | **~53 lines** | **~33 lines** | **~38%** |

Additional benefits:
- ✅ **Type safety**: Catch errors at compile time
- ✅ **Readability**: Natural Rust syntax
- ✅ **Maintainability**: Changes propagate automatically
- ✅ **IDE support**: Full autocomplete and type hints
- ✅ **No DSL**: Everything is just Rust code
- ✅ **Consistent syntax**: All macros use `#[derive(...)]` or `#[attribute]` patterns

## Migration Guide

### From Manual IR Building

**Before:**
```rust
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
        // ... more fields
    ],
});
```

**After:**
```rust
#[derive(BamlSchema)]
#[baml(description = "A person entity")]
struct Person {
    #[baml(description = "Full name")]
    name: String,
    // ... more fields
}

let ir = BamlSchemaRegistry::new()
    .register::<Person>()
    .build();
```

### From Verbose Function Definition

**Before:**
```rust
let function = Function {
    name: "ExtractPerson".to_string(),
    inputs: vec![Field {
        name: "text".to_string(),
        field_type: FieldType::String,
        optional: false,
        description: Some("Text to extract from".to_string()),
    }],
    output: FieldType::Class("Person".to_string()),
    prompt_template: "Extract person info from: {{ text }}".to_string(),
    client: "openai".to_string(),
};
```

**After:**
```rust
#[baml_function(client = "openai")]
fn extract_person(
    #[baml(description = "Text to extract from")]
    text: String
) -> Person {
    "Extract person info from: {{ text }}"
}

let function = extract_person();
```

### From Declarative Client Macro

If you're migrating from the old `baml_client!` declarative macro:

**Before:**
```rust
baml_client!(openai_client, OpenAI, "gpt-4o-mini");

let client = openai_client(api_key);
```

**After:**
```rust
#[derive(BamlClient)]
#[baml(provider = "OpenAI", model = "gpt-4o-mini")]
struct OpenAIClient;

let client = OpenAIClient::new(api_key);
```

The new syntax provides:
- ✅ Consistent with `#[derive(BamlSchema)]` pattern
- ✅ More explicit and self-documenting
- ✅ Better IDE support for the generated type

## See Also

- `examples/with_macros.rs` - Complete example using all macros
- `examples/extract_person_macro.rs` - Class/enum macros example
- `examples/nested_macro.rs` - Complex nested structures
- `README.md` - Full project documentation
