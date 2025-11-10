# Current Flow: Structured LLM Output Generation

## Overview

This document explains how the current system generates structured results from LLM calls, with a focus on streaming and partial parsing capabilities.

## Architecture Components

```
┌─────────────────────────────────────────────────────────────┐
│                     USER CODE                                │
│  Define types, call runtime, receive structured results     │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                  IR (Intermediate Representation)            │
│  - Classes: Type definitions with fields                    │
│  - Enums: Allowed value sets                                │
│  - Fields: Name, type, optional flag, description           │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                  SCHEMA FORMATTER                            │
│  Converts IR to human-readable schema:                      │
│  - Enum definitions (name, values)                          │
│  - Class definitions (field names and types)                │
│  - "Answer in JSON using this schema:" instruction          │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                  PROMPT GENERATION                           │
│  Combines:                                                   │
│  - Schema instructions                                       │
│  - User input                                                │
│  - Optional template (Jinja2)                                │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                  LLM CLIENT (rust-genai)                     │
│  ┌─────────────────────┬─────────────────────┐             │
│  │   Non-Streaming     │     Streaming       │             │
│  │   exec_chat()       │  exec_chat_stream() │             │
│  └──────────┬──────────┴──────────┬──────────┘             │
│             │                     │                         │
│             ▼                     ▼                         │
│     Complete Response      Stream of Chunks                 │
└─────────────┬─────────────────────┬───────────────────────┘
              │                     │
              │                     ▼
              │          ┌──────────────────────┐
              │          │  PARTIAL PARSER      │
              │          │  - Accumulate chunks │
              │          │  - Extract JSON      │
              │          │  - Auto-close {}[]   │
              │          │  - Parse incomplete  │
              │          └──────────┬───────────┘
              │                     │
              ▼                     ▼
┌─────────────────────────────────────────────────────────────┐
│                      PARSER                                  │
│  1. Extract JSON (handle markdown, extra text)              │
│  2. Parse JSON                                               │
│  3. Coerce to target types (lenient)                         │
└───────────────────┬─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                   BamlValue                                  │
│  Type-safe structured data:                                 │
│  - String, Int, Float, Bool                                 │
│  - List, Map                                                │
│  - Nested structures                                         │
└─────────────────────────────────────────────────────────────┘
```

## Detailed Flow

### 1. Schema Definition (IR)

**Location:** `src/ir.rs`

Define your data structures using the IR types:

```rust
let mut ir = IR::new();

// Define an enum
ir.enums.push(Enum {
    name: "Priority".to_string(),
    values: vec!["Low".to_string(), "High".to_string()],
    description: Some("Task priority".to_string()),
});

// Define a class
ir.classes.push(Class {
    name: "Task".to_string(),
    fields: vec![
        Field {
            name: "title".to_string(),
            field_type: FieldType::String,
            optional: false,
            description: Some("Task title".to_string()),
        },
        Field {
            name: "priority".to_string(),
            field_type: FieldType::Enum("Priority".to_string()),
            optional: false,
            description: Some("Priority level".to_string()),
        },
    ],
    description: Some("A task item".to_string()),
});
```

### 2. Schema Formatting

**Location:** `src/schema.rs`

The `SchemaFormatter` converts IR types into human-readable text that the LLM can understand:

**Input:** IR with Task class and Priority enum

**Output:**
```
Priority
--------
- Low
- High

Answer in JSON using this schema:
{
  title: string, // Task title
  priority: Priority, // Priority level
}
```

### 3. Prompt Generation

**Location:** `src/runtime.rs` (`generate_prompt_from_ir`)

Combines the schema with user input:

```rust
let prompt = generate_prompt_from_ir(
    &ir,
    "Create a high priority task for fixing the bug",
    &FieldType::Class("Task".to_string()),
)?;
```

Result:
```
Priority
--------
- Low
- High

Answer in JSON using this schema:
{
  title: string,
  priority: Priority,
}

User input: Create a high priority task for fixing the bug
```

### 4. LLM Call with rust-genai

**Location:** External dependency `genai`

#### Non-Streaming (Simple)

```rust
use genai::chat::{ChatMessage, ChatRequest};
use genai::Client;

let client = Client::default();
let chat_req = ChatRequest::new(vec![
    ChatMessage::user(prompt),
]);

let response = client.exec_chat("gpt-4o-mini", chat_req, None).await?;
let text = response.content_text_as_str().unwrap();
```

#### Streaming (Real-time)

```rust
use genai::chat::ChatStreamEvent;

let mut chat_stream = client.exec_chat_stream("gpt-4o-mini", chat_req, None).await?;

let mut accumulated = String::new();
while let Some(event_result) = chat_stream.stream.next().await {
    let event = event_result?;
    
    match event {
        ChatStreamEvent::Chunk(chunk) => {
            let text = &chunk.content;
            accumulated.push_str(text);
            // Process partial response here
        }
        _ => {} // Handle other events as needed
    }
}
```

### 5. Partial Parsing (Streaming Only)

**Location:** `src/partial_parser.rs`

The partial parser handles incomplete JSON from streaming responses:

```rust
use simplify_baml::try_parse_partial_response;

// As chunks arrive...
accumulated_text.push_str(new_chunk);

// Try parsing
match try_parse_partial_response(&ir, &accumulated_text, &target_type)? {
    Some(partial_value) => {
        // Got a parseable (possibly incomplete) result
        println!("Current progress: {:?}", partial_value);
    }
    None => {
        // Not enough data yet
    }
}
```

**How it works:**

1. **Extract from markdown:** Handle ` ```json ... ``` ` blocks
2. **Find JSON boundaries:** Look for `{` and `}` characters
3. **Auto-close structures:** Add missing `}` and `]` based on open count
4. **Handle incomplete strings:** Add closing `"` if needed
5. **Multiple strategies:** Try several completion approaches

Example progression:
```
Chunk 1: "```json\n{"                     -> None (too early)
Chunk 2: "```json\n{\"title\": \"Fix"     -> Some({"title": "Fix"})
Chunk 3: "```json\n{\"title\": \"Fix bug\"" -> Some({"title": "Fix bug"})
Chunk 4: "```json\n{\"title\": \"Fix bug\", \"priority\": \"Hig" 
         -> Some({"title": "Fix bug", "priority": "Hig"})
Chunk 5: "```json\n{\"title\": \"Fix bug\", \"priority\": \"High\"}"
         -> Some({"title": "Fix bug", "priority": "High"})
```

### 6. Final Parsing

**Location:** `src/parser.rs`

The full parser processes the complete response:

```rust
use simplify_baml::parse_llm_response_with_ir;

let final_value = parse_llm_response_with_ir(
    &ir, 
    &accumulated_text, 
    &target_type
)?;
```

**Process:**

1. **Extract JSON:** Remove markdown, find JSON boundaries
2. **Parse JSON:** Use `serde_json` 
3. **Coerce types:** Convert JSON values to expected BAML types
   - String → can come from string, number, or bool
   - Int → can parse from string, convert from float
   - Enum → case-insensitive matching with variants
   - Class → validate fields, check required vs optional
   - List → coerce each element
   - Map → coerce each value

### 7. Using the Result

**Output:** `BamlValue` enum

```rust
pub enum BamlValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<BamlValue>),
    Map(HashMap<String, BamlValue>),
}
```

**Access patterns:**

```rust
if let BamlValue::Map(map) = final_value {
    let title = map.get("title")
        .and_then(|v| v.as_string())
        .unwrap_or("No title");
    
    let priority = map.get("priority")
        .and_then(|v| v.as_string())
        .unwrap_or("Unknown");
    
    println!("Task: {} [{}]", title, priority);
}
```

## rust-genai Integration Benefits

### Why rust-genai?

1. **Unified API:** One interface for multiple providers (OpenAI, Anthropic, Gemini, etc.)
2. **Simple streaming:** Easy async/await based streaming
3. **No complex setup:** Auto-detects providers from model names
4. **Auto-authentication:** Uses environment variables
5. **Type-safe:** Strong Rust typing throughout

### Example: Multi-provider support

```rust
// Same code works for different providers
let models = vec![
    "gpt-4o-mini",              // OpenAI
    "claude-3-5-sonnet-20241022", // Anthropic
    "gemini-1.5-flash",          // Google
];

for model in models {
    let response = client.exec_chat(model, chat_req.clone(), None).await?;
    // Process response
}
```

### Example: Streaming with real-time UI updates

```rust
use genai::chat::ChatStreamEvent;
use std::time::{Duration, Instant};

let mut chat_stream = client.exec_chat_stream(model, chat_req, None).await?;

let mut accumulated = String::new();
let mut last_ui_update = Instant::now();

while let Some(event_result) = chat_stream.stream.next().await {
    let event = event_result?;
    
    match event {
        ChatStreamEvent::Chunk(chunk) => {
            let text = &chunk.content;
            accumulated.push_str(text);
            
            // Update UI every 100ms
            if last_ui_update.elapsed() > Duration::from_millis(100) {
                if let Some(partial) = try_parse_partial_response(&ir, &accumulated, &target_type)? {
                    update_ui(partial);
                }
                last_ui_update = Instant::now();
            }
        }
        _ => {}
    }
}
```

## Key Files Reference

| Component | File | Purpose |
|-----------|------|---------|
| IR | `src/ir.rs` | Type definitions (classes, enums, fields) |
| Schema Formatter | `src/schema.rs` | Convert IR to readable schema text |
| Parser | `src/parser.rs` | Parse complete JSON responses |
| Partial Parser | `src/partial_parser.rs` | Parse incomplete streaming JSON |
| Runtime | `src/runtime.rs` | Orchestrate the full flow |
| Client | `src/client.rs` | Simple HTTP client (legacy) |
| Streaming Value | `src/streaming_value.rs` | Track streaming state |

## Examples

See the `examples/` directory:

- **`rust_genai_streaming_example.rs`** - Complete flow with rust-genai
- **`advanced_streaming_flow.rs`** - Detailed walkthrough with visualization
- **`streaming_with_partial_parsing.rs`** - Simulated streaming chunks
- **`extract_person.rs`** - Simple non-streaming example
- **`extract_person_macro.rs`** - Using derive macros

## Running Examples

```bash
# Set your API key
export OPENAI_API_KEY=sk-...

# Run the rust-genai streaming example
cargo run --example rust_genai_streaming_example

# Run the advanced flow example
cargo run --example advanced_streaming_flow
```

## Advantages of This Architecture

1. **Type Safety:** Strong typing from IR to final output
2. **Provider Agnostic:** Works with any LLM via rust-genai
3. **Streaming First:** Built for real-time user experiences
4. **Lenient Parsing:** Handles imperfect LLM outputs
5. **Incremental Results:** Show progress during streaming
6. **Schema Validation:** Ensure output matches expected structure
7. **Easy to Extend:** Add new types, providers, or parsing strategies

## Common Patterns

### Pattern 1: Simple Extraction

```rust
let ir = /* define schema */;
let prompt = generate_prompt_from_ir(&ir, user_input, &target_type)?;
let response = client.exec_chat("gpt-4o-mini", ChatRequest::new(vec![
    ChatMessage::user(prompt)
]), None).await?;
let value = parse_llm_response_with_ir(&ir, response.content_text_as_str().unwrap(), &target_type)?;
```

### Pattern 2: Streaming with Progress

```rust
use genai::chat::ChatStreamEvent;

let mut chat_stream = client.exec_chat_stream(model, chat_req, None).await?;
let mut accumulated = String::new();

while let Some(event_result) = chat_stream.stream.next().await {
    let event = event_result?;
    
    match event {
        ChatStreamEvent::Chunk(chunk) => {
            accumulated.push_str(&chunk.content);
            
            if let Some(partial) = try_parse_partial_response(&ir, &accumulated, &target_type)? {
                show_progress(partial);
            }
        }
        _ => {}
    }
}

let final_value = parse_llm_response_with_ir(&ir, &accumulated, &target_type)?;
```

### Pattern 3: Multi-turn Conversation

```rust
let mut chat_req = ChatRequest::default()
    .with_system("You extract structured data.");

chat_req = chat_req.append_message(ChatMessage::user(first_prompt));
let response = client.exec_chat(model, chat_req.clone(), None).await?;
chat_req = chat_req.append_message(ChatMessage::assistant(response.content_text_as_str().unwrap()));

chat_req = chat_req.append_message(ChatMessage::user(follow_up));
let response = client.exec_chat(model, chat_req.clone(), None).await?;
```

## Summary

The current flow is a pipeline:

**IR → Schema → Prompt → LLM (rust-genai) → Partial Parser → Final Parser → BamlValue**

Each stage is independent and testable. rust-genai provides a clean abstraction over multiple LLM providers, making it easy to switch providers or use streaming without changing the core logic.
