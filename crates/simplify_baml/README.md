# Simplified BAML Runtime

A minimal, educational implementation of the BAML (Basically A Markup Language) runtime that demonstrates the core concepts by reducing the original ~50K line codebase to approximately ~5K lines.

**✨ New Features**:
- Automatic IR generation from native Rust types using `#[derive(BamlSchema)]` macros!
- Function definitions with `#[baml_function]` attribute macro
- Client configuration with `#[derive(BamlClient)]` derive macro
- **All macros use consistent attribute syntax for a unified API**

## What is BAML?

BAML is a language for defining and calling LLM functions with structured outputs. The runtime handles:
1. Converting type definitions into human-readable schemas
2. Injecting schemas into Jinja2 templates
3. Calling LLM APIs
4. Parsing and validating LLM responses

## Core Components

This simplified implementation consists of 7 key components:

### 1. IR (Intermediate Representation) - `src/ir.rs`
Defines the core types:
- `Class` - Structured types with fields
- `Enum` - Enumerated types
- `Function` - LLM functions
- `BamlValue` - Runtime values
- `BamlSchema` trait - For automatic IR generation

### 2. Macro System - `simplify_baml_macros/`
Procedural macros for automatic IR generation with consistent syntax:
- `#[derive(BamlSchema)]` - Automatically implement BamlSchema trait for structs and enums
- `#[derive(BamlClient)]` - Configure LLM clients (OpenAI, Anthropic, custom)
- `#[baml_function(client = "...")]` - Define BAML functions with type-safe syntax
- `#[baml(description = "...")]` - Add descriptions to types, fields, and parameters
- `#[baml(rename = "...")]` - Rename fields in the generated schema

### 3. Schema Registry - `src/registry.rs`
Collects types and builds IR:
- `BamlSchemaRegistry::new()` - Create registry
- `.register::<T>()` - Register types implementing BamlSchema
- `.build()` - Generate final IR

### 4. Schema Formatter - `src/schema.rs`
Converts IR types into human-readable schema strings:
```
Month
----
- January
- February
- March

Answer in JSON using this schema:
{
  name: string,
  age: int,
  birthMonth: Month,
}
```

### 5. Template Renderer - `src/renderer.rs`
Uses Jinja2 (via minijinja) to render prompts with automatic schema injection.

### 6. HTTP Client - `src/client.rs`
Simple wrapper for calling LLM APIs. Currently uses OpenAI-compatible API format, which works with:
- OpenAI (native support)
- OpenAI-compatible endpoints
- **OpenRouter** (recommended for accessing multiple providers including Claude, Gemini, Llama, etc.)

### 7. Parser - `src/parser.rs`
Lenient JSON parser with type coercion that handles:
- Markdown code blocks
- Type conversions (string to int, etc.)
- Enum validation
- Nested structures

### 8. Runtime - `src/runtime.rs`
Orchestrates all components to execute BAML functions.

## Quick Start

### Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
simplify_baml = { path = "/path/to/simplify_baml" }
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage (With Macros - Recommended!)

```rust
use simplify_baml::*;
use simplify_baml_macros::{BamlSchema, BamlClient};
use std::collections::HashMap;

// 1. Define types using derive macros - clean and type-safe!
#[derive(BamlSchema)]
#[baml(description = "Information about a person")]
struct Person {
    #[baml(description = "Full name of the person")]
    name: String,

    #[baml(description = "Age in years")]
    age: i64,
}

// 2. Define BAML function using macro
#[baml_function(client = "openai")]
fn extract_person(
    #[baml(description = "Text containing person information")]
    text: String
) -> Person {
    "Extract person info from: {{ text }}"
}

// 3. Configure client using derive macro
#[derive(BamlClient)]
#[baml(provider = "OpenAI", model = "gpt-4o-mini")]
struct OpenAIClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 4. Build IR automatically from types
    let ir = BamlSchemaRegistry::new()
        .register::<Person>()
        .build_with_functions(vec![extract_person()]);

    // 5. Create client
    let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY")?);

    // 6. Build runtime
    let runtime = RuntimeBuilder::new()
        .ir(ir)
        .client("openai", client)
        .build();

    // 7. Execute
    let mut params = HashMap::new();
    params.insert(
        "text".to_string(),
        BamlValue::String("John is 30 years old".to_string())
    );

    let result = runtime.execute("ExtractPerson", params).await?;

    println!("{:?}", result);
    Ok(())
}
```

<details>
<summary>Manual IR Building (Click to expand)</summary>

If you prefer to build IR manually without macros:

```rust
use simplify_baml::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Manually build IR
    let mut ir = IR::new();

    ir.classes.push(Class {
        name: "Person".to_string(),
        description: None,
        fields: vec![
            Field {
                name: "name".to_string(),
                field_type: FieldType::String,
                optional: false,
                description: None,
            },
            Field {
                name: "age".to_string(),
                field_type: FieldType::Int,
                optional: false,
                description: None,
            },
        ],
    });

    ir.functions.push(Function {
        name: "ExtractPerson".to_string(),
        inputs: vec![Field {
            name: "text".to_string(),
            field_type: FieldType::String,
            optional: false,
            description: None,
        }],
        output: FieldType::Class("Person".to_string()),
        prompt_template: "Extract person info from: {{ text }}".to_string(),
        client: "openai".to_string(),
    });

    // Rest is the same...
    Ok(())
}
```

</details>

## Running Examples

```bash
# Set your OpenAI API key
export OPENAI_API_KEY="your-key-here"

# Run the complete macro example (recommended - shows all 3 macros!)
cargo run --example with_macros

# Run the class/enum macro example
cargo run --example extract_person_macro

# Run the manual IR building example
cargo run --example extract_person

# Run nested structures with macros
cargo run --example nested_macro

# Run with OpenRouter (access 100+ models with one API key)
export OPENROUTER_API_KEY="your-openrouter-key"
cargo run --example openrouter_example
```

## Running Tests

```bash
cargo test
```

## Macro System Features

BAML provides three powerful macros that dramatically simplify development:

### 1. `#[derive(BamlSchema)]` - Type Definitions

The derive macro makes IR generation dramatically simpler and more maintainable:

### Type Mapping
- `String` → `FieldType::String`
- `i64`, `i32`, `i16`, `i8` → `FieldType::Int`
- `f64`, `f32` → `FieldType::Float`
- `bool` → `FieldType::Bool`
- `Option<T>` → Makes field optional
- `Vec<T>` → `FieldType::List(T)`
- Custom types → Automatically detected as `Class` or `Enum`

### Attributes
- `#[baml(description = "...")]` - Add descriptions to types and fields
- `#[baml(rename = "field_name")]` - Rename fields in generated schema

### Example: Complex Nested Structures

```rust
use simplify_baml_macros::BamlSchema;

#[derive(BamlSchema)]
enum Role {
    Engineer,
    Manager,
    Designer,
}

#[derive(BamlSchema)]
struct Address {
    street: String,
    city: String,
    #[baml(rename = "zipCode")]
    zip_code: String,
}

#[derive(BamlSchema)]
struct Employee {
    name: String,
    age: i64,
    role: Role,  // Enum reference
}

#[derive(BamlSchema)]
struct Company {
    name: String,
    employees: Vec<Employee>,  // List of structs
    address: Address,          // Nested struct
}

// Automatic IR generation - handles all nesting!
let ir = BamlSchemaRegistry::new()
    .register::<Role>()
    .register::<Address>()
    .register::<Employee>()
    .register::<Company>()
    .build();
```

**Benefits:**
- ✅ ~55% less code compared to manual IR building
- ✅ Type-safe - catches errors at compile time
- ✅ More readable and maintainable
- ✅ Automatic handling of nested structures
- ✅ Field renaming and descriptions

### 2. `#[baml_function]` - Function Definitions

Define BAML functions using natural Rust syntax instead of verbose struct construction:

```rust
#[baml_function(client = "openai")]
fn extract_person(
    #[baml(description = "Text containing person information")]
    text: String
) -> Person {
    r#"Extract the person's information from: {{ text }}"#
}

// Use it in your IR
let ir = BamlSchemaRegistry::new()
    .register::<Person>()
    .build_with_functions(vec![extract_person()]);
```

**Key Features:**
- ✅ Function name automatically converted to PascalCase (extract_person → ExtractPerson)
- ✅ Type-safe input/output definitions
- ✅ Parameter descriptions via `#[baml(description)]`
- ✅ Jinja2 template as function body
- ✅ Generates a function that returns `Function` struct

### 3. `#[derive(BamlClient)]` - Client Configuration

Configure LLM clients using a derive macro - consistent with the BamlSchema pattern:

```rust
use simplify_baml_macros::BamlClient;

// OpenAI client
#[derive(BamlClient)]
#[baml(provider = "OpenAI", model = "gpt-4o-mini")]
struct OpenAIClient;

// Anthropic client
#[derive(BamlClient)]
#[baml(provider = "Anthropic", model = "claude-3-sonnet")]
struct AnthropicClient;

// Custom endpoint
#[derive(BamlClient)]
#[baml(provider = "Custom", base_url = "https://api.example.com/v1", model = "my-model")]
struct CustomClient;

// Use it
let client = OpenAIClient::new(std::env::var("OPENAI_API_KEY")?);
```

**Benefits:**
- ✅ Consistent syntax with `#[derive(BamlSchema)]`
- ✅ Supports OpenAI, Anthropic, and custom endpoints
- ✅ Type-safe API
- ✅ Generates a `new(api_key: String) -> LLMClient` method

### Using OpenRouter (Recommended for Multi-Model Access)

[OpenRouter](https://openrouter.ai) provides access to 100+ models from different providers through a single API key. Since it uses an OpenAI-compatible API, it works perfectly with the Custom provider:

```rust
// Access Claude via OpenRouter
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "https://openrouter.ai/api/v1",
    model = "anthropic/claude-3.5-sonnet"
)]
struct ClaudeClient;

// Access GPT-4 via OpenRouter
#[derive(BamlClient)]
#[baml(
    provider = "Custom",
    base_url = "https://openrouter.ai/api/v1",
    model = "openai/gpt-4-turbo"
)]
struct GPT4Client;

// Use both in the same runtime
let runtime = RuntimeBuilder::new()
    .ir(ir)
    .client("claude", ClaudeClient::new(api_key.clone()))
    .client("gpt4", GPT4Client::new(api_key.clone()))
    .build();
```

**See `examples/openrouter_example.rs` for a complete working example.**

For comprehensive multi-provider setup including cost optimization strategies, see **[MULTI_PROVIDER.md](MULTI_PROVIDER.md)**.

## What's Different from Full BAML?

This simplified version focuses on the core execution path and omits:
- ❌ BAML language parser (use Rust macros instead)
- ❌ CLI tools
- ❌ WASM support
- ❌ Advanced tracing/telemetry
- ❌ Test runner
- ❌ Code generation for multiple languages
- ❌ VS Code extension integration
- ❌ Streaming support
- ❌ Complex retry policies and orchestration strategies
- ❌ Multiple prompt configs per function

What it keeps (and improves!):
- ✅ Core IR types (Class, Enum, Function)
- ✅ **Automatic IR generation from Rust types via `#[derive(BamlSchema)]`** 🆕
- ✅ **Function definitions via `#[baml_function]`** 🆕
- ✅ **Client configuration via `#[derive(BamlClient)]`** 🆕
- ✅ **Consistent attribute-based syntax across all macros** 🆕
- ✅ Schema formatting (types → human-readable strings)
- ✅ Jinja2 template rendering
- ✅ HTTP client for LLM calls
- ✅ Lenient JSON parsing with type coercion
- ✅ Basic runtime orchestration

## Key Insights

### How BAML Actually Works

1. **IR as a Bidirectional Contract**: The Intermediate Representation (IR) is the **single source of truth** that serves dual purposes:
   - **Outbound (Generation)**: IR → SchemaFormatter converts types into human-readable schemas that tell the LLM what structure to return
   - **Inbound (Parsing)**: IR → Parser validates and coerces the LLM's response back into typed values

   This bidirectional design ensures **type safety and consistency** - the same type definitions generate both the prompt instructions AND validate the results:
   ```
   ┌─────────────────────────────────────────────────────┐
   │ IR (Single Source of Truth)                         │
   │ - Classes, Enums, Functions                         │
   │ - Field types and structure                         │
   └──────────────┬──────────────────────┬────────────────┘
                  │                      │
                  │ (Generate)           │ (Parse)
                  ▼                      ▼
       ┌──────────────────┐   ┌──────────────────┐
       │ Schema Formatter │   │     Parser       │
       │ (src/schema.rs)  │   │ (src/parser.rs)  │
       └────────┬─────────┘   └────────▲─────────┘
                │                      │
                ▼                      │
         Human-readable             JSON from
         schema text                LLM response
                │                      │
                └──> Prompt to LLM ────┘
   ```

2. **Schema Auto-Generation**: BAML automatically converts your type definitions into human-readable schemas and appends them to prompts.

3. **Two-Stage Parsing**:
   - Stage 1: Lenient JSON extraction (handles markdown, extra text)
   - Stage 2: Type coercion (converts values to match expected types)

4. **Jinja2 Templates**: User prompts are Jinja2 templates with automatic `output_schema` variable injection.

5. **Simple Flow**:
   ```
   IR → Schema Formatter → Jinja2 → HTTP Client → Lenient Parser → Typed Result
   ```

## Transformation Pipeline Deep Dive

Understanding the complete transformation from Rust struct to LLM prompt and back is key to understanding BAML. Here's the full pipeline with concrete examples:

### **Step 1: Rust Struct Definition**

```rust
// Define your types using Rust syntax with derive macros
#[derive(BamlSchema)]
#[baml(description = "Calendar month of the year")]
enum Month {
    January,
    February,
    March,
    // ... other months
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
```

**Location**: User code (e.g., `examples/with_macros.rs:7-25`)

### **Step 2: IR (Intermediate Representation)**

The `#[derive(BamlSchema)]` macro automatically generates code that produces this IR:

```rust
// Generated IR representation
Class {
    name: "Person",
    description: Some("Information about a person"),
    fields: vec![
        Field {
            name: "name",
            field_type: FieldType::String,
            optional: false,
            description: Some("Full name of the person"),
        },
        Field {
            name: "age",
            field_type: FieldType::Int,
            optional: false,
            description: Some("Age in years"),
        },
        Field {
            name: "birthMonth",  // Renamed from birth_month
            field_type: FieldType::Enum("Month"),
            optional: true,  // Option<T> makes it optional
            description: Some("Month of birth, if mentioned"),
        },
        Field {
            name: "occupation",
            field_type: FieldType::String,
            optional: true,
            description: Some("Job title or profession, if mentioned"),
        },
    ],
}

Enum {
    name: "Month",
    description: Some("Calendar month of the year"),
    values: vec!["January", "February", "March", ...],
}
```

**Location**: `src/ir.rs:7-60` (IR types), `simplify_baml_macros/src/lib.rs:74-127` (macro generation)

**Key Transformations**:
- `String` → `FieldType::String`
- `i64` → `FieldType::Int`
- `Option<T>` → `optional: true`
- `Vec<T>` → `FieldType::List(T)`
- `#[baml(rename = "...")]` → Changes field name in schema
- Enum variants → List of string values

### **Step 3: Human-Readable Schema**

The `SchemaFormatter` converts IR to a human-readable format:

```text
Month
----
- January
- February
- March
- April
- May
- June
- July
- August
- September
- October
- November
- December

Answer in JSON using this schema:
{
  name: string,
  age: int,
  birthMonth: Month,
  occupation: string,
}
```

**Location**: `src/schema.rs:10-140`

**Process**:
1. Collect all dependencies (enums and nested classes)
2. Render enum definitions with list of values
3. Render the main schema in pseudo-JSON format
4. Add instruction: "Answer in JSON using this schema:"

### **Step 4: Final Prompt (Template + Schema)**

The `PromptRenderer` combines your Jinja2 template with the generated schema:

```text
Extract the person's information from the following text:

John Smith is 30 years old and was born in March. He works as a software engineer.

Please extract: name, age, birth month (if mentioned), and occupation (if mentioned).

Month
----
- January
- February
- March
- April
- May
- June
- July
- August
- September
- October
- November
- December

Answer in JSON using this schema:
{
  name: string,
  age: int,
  birthMonth: Month,
  occupation: string,
}
```

**Location**: `src/renderer.rs:9-70`

**Process**:
1. Generate schema from IR (via `SchemaFormatter`)
2. Create Jinja2 environment (using `minijinja`)
3. Render template with user parameters
4. Append schema if not already in template (automatic injection!)

### **Step 5: LLM Response**

The prompt is sent to the LLM via HTTP, and it returns a response:

```text
Here's the extracted information:
```json
{
  "name": "John Smith",
  "age": "30",
  "birthMonth": "march",
  "occupation": "software engineer"
}
```
```

**Location**: `src/client.rs:10-80` (HTTP client)

**Note**: The LLM may return:
- Extra text before/after JSON
- Markdown code blocks
- Incorrect types (e.g., `"30"` as string instead of int)
- Different casing (e.g., `"march"` instead of `"March"`)

### **Step 6: Parsed and Coerced Result**

The `Parser` extracts JSON and coerces types to match the schema:

```rust
BamlValue::Map({
    "name": BamlValue::String("John Smith"),
    "age": BamlValue::Int(30),                    // "30" → 30 (string to int coercion)
    "birthMonth": BamlValue::String("March"),     // "march" → "March" (enum normalization)
    "occupation": BamlValue::String("software engineer"),
})
```

**Location**: `src/parser.rs:15-200`

**Process**:
1. **Extract JSON**: Find JSON in response (handles markdown blocks, extra text)
   - Look for ` ```json` blocks
   - Look for plain ` ``` ` blocks
   - Find `{ ... }` boundaries
2. **Parse JSON**: Use `serde_json` to parse string
3. **Type Coercion**: Convert values to match target types
   - String to Int: Parse `"30"` → `30`
   - String to Float: Parse `"3.14"` → `3.14`
   - Enum: Case-insensitive matching (`"march"` → `"March"`)
   - Nested structures: Recursively validate and coerce

### **Complete Flow Diagram**

```
┌─────────────────────────────────────────────────────────────┐
│ 1. RUST STRUCT                                              │
│    #[derive(BamlSchema)]                                    │
│    struct Person { name: String, age: i64, ... }            │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ Procedural Macro
                     │ (simplify_baml_macros/src/lib.rs)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. IR (Intermediate Representation)                         │
│    Class { name: "Person", fields: [...] }                  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ SchemaFormatter::render()
                     │ (src/schema.rs)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. HUMAN-READABLE SCHEMA                                    │
│    Answer in JSON using this schema:                        │
│    { name: string, age: int, birthMonth: Month, ... }       │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ PromptRenderer::render()
                     │ (src/renderer.rs + minijinja)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. FINAL PROMPT                                             │
│    Extract info from: {{ text }}                            │
│    [schema appended automatically]                          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ LLMClient::call()
                     │ (src/client.rs - HTTP request)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. LLM RESPONSE                                             │
│    ```json                                                  │
│    { "name": "John", "age": "30", ... }                     │
│    ```                                                      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ Parser::parse()
                     │ (src/parser.rs - extract + coerce)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. TYPED BAMLVALUE                                          │
│    BamlValue::Map({                                         │
│      "name": String("John"),                                │
│      "age": Int(30),  // Coerced from "30"                  │
│    })                                                       │
└─────────────────────────────────────────────────────────────┘
```

### **Runtime Orchestration**

All of this is orchestrated by `BamlRuntime::execute()`:

```rust
// src/runtime.rs:18-48
pub async fn execute(
    &self,
    function_name: &str,
    params: HashMap<String, BamlValue>,
) -> Result<BamlValue> {
    // 1. Find function in IR
    let function = self.ir.find_function(function_name)?;

    // 2. Get LLM client
    let client = self.clients.get(&function.client)?;

    // 3. Render prompt (template + schema)
    let renderer = PromptRenderer::new(&self.ir);
    let prompt = renderer.render(
        &function.prompt_template,
        &params,
        &function.output,
    )?;

    // 4. Call LLM
    let raw_response = client.call(&prompt).await?;

    // 5. Parse and coerce response
    let parser = Parser::new(&self.ir);
    let result = parser.parse(&raw_response, &function.output)?;

    Ok(result)
}
```

### **Key Design Decisions**

1. **Lenient Parsing**: The parser is intentionally lenient, handling markdown, type mismatches, and extra text. This makes it work reliably with real LLM outputs.

2. **Schema Injection**: Schemas are automatically appended to prompts, ensuring the LLM always knows the expected structure.

3. **Type Coercion**: Automatic conversion between compatible types (string ↔ number, case normalization for enums) reduces friction.

4. **Compile-Time Safety**: Using Rust macros instead of a custom DSL provides type safety at compile time.

5. **Minimal Dependencies**: The entire pipeline uses only essential dependencies: `minijinja` for templates, `reqwest` for HTTP, `serde_json` for JSON parsing.

## Architecture Comparison

### Original BAML (~50K lines)
- Full language parser and compiler
- Multi-language code generation
- Complex orchestration strategies
- Extensive tracing and telemetry
- WASM compilation
- CLI tools and VS Code integration

### Simplified BAML (~6K lines)
- **Rust macro-based IR generation** 🆕
  - `#[derive(BamlSchema)]` for types
  - `#[baml_function]` for functions
  - `#[derive(BamlClient)]` for clients
  - Consistent attribute-based syntax
- Single language (Rust)
- Basic orchestration
- Minimal logging
- Native only
- Library-only interface

**Innovation**: While the original BAML requires writing schemas in a custom DSL, Simplified BAML uses Rust's macro system to generate IR directly from native code. This provides:
- ✨ Compile-time type safety
- ✨ No separate DSL parser needed
- ✨ Natural Rust syntax for all definitions
- ✨ Consistent API across all macros
- ✨ ~60% less code overall

## License

This is an educational implementation demonstrating the core concepts of BAML.
For production use, please use the official BAML runtime.

## Learn More

- Original BAML: https://github.com/BoundaryML/baml
- BAML Documentation: https://docs.boundaryml.com
