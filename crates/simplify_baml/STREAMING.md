# Streaming Support in Simplified BAML

This document explains how to use streaming with the simplified BAML runtime.

## Overview

Simplified BAML now supports **partial JSON parsing** for streaming LLM responses. Instead of copying the entire `jsonish` library from full BAML (which would add ~5,000+ lines and multiple dependencies), we've implemented a lightweight partial parser (~300 lines) that handles the most common streaming scenarios.

## Key Functions

### 1. `try_parse_partial_json()`

Low-level function that attempts to parse incomplete JSON:

```rust
use simplify_baml::try_parse_partial_json;

let partial = r#"{"name": "John", "age": 30"#;  // Missing closing }
let json = try_parse_partial_json(partial)?;

match json {
    Some(value) => println!("Parsed: {:?}", value),  // Auto-closed!
    None => println!("Need more data"),
}
```

**Features:**
- Auto-closes incomplete objects and arrays
- Handles incomplete strings
- Extracts JSON from markdown code blocks
- Returns `None` if too incomplete to parse

### 2. `try_parse_partial_response()`

High-level function that combines partial parsing with IR type coercion:

```rust
use simplify_baml::*;

let ir = /* your IR */;
let target_type = FieldType::Class("Person".to_string());

let partial = r#"{"name": "John", "age": "30"#;  // Missing }, wrong type

match try_parse_partial_response(&ir, partial, &target_type)? {
    Some(baml_value) => {
        // ✅ Auto-closed AND type-coerced!
        // "30" → 30 (string to int)
    }
    None => {
        // Need more data
    }
}
```

**Features:**
- Everything from `try_parse_partial_json()`
- Plus type coercion (string → int, etc.)
- Plus enum validation
- Plus nested structure handling

## Usage Patterns

### Pattern 1: Show Progress to Users

```rust
let mut accumulated = String::new();

while let Some(chunk) = stream.next().await {
    accumulated.push_str(&chunk?);

    // Try to parse after each chunk
    if let Some(result) = try_parse_partial_response(&ir, &accumulated, &target_type)? {
        // Show partial result to user in real-time!
        update_ui(result);
    }
}
```

### Pattern 2: Early Termination

```rust
while let Some(chunk) = stream.next().await {
    accumulated.push_str(&chunk?);

    if let Some(result) = try_parse_partial_response(&ir, &accumulated, &target_type)? {
        // If we have all required fields, stop early!
        if is_complete(&result) {
            break;
        }
    }
}
```

### Pattern 3: Use Any Streaming Client

```rust
use async_openai::{Client, types::*};
use futures::StreamExt;

async fn with_openai_streaming() -> anyhow::Result<()> {
    let client = Client::new();

    // 1. Generate prompt using BAML
    let prompt = generate_prompt_from_ir(&ir, template, &params, &output_type)?;

    // 2. Stream from OpenAI (or any other provider)
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4")
        .messages(vec![/* ... */])
        .build()?;

    let mut stream = client.chat().create_stream(request).await?;
    let mut accumulated = String::new();

    while let Some(chunk) = stream.next().await {
        if let Some(content) = chunk?.choices[0].delta.content {
            print!("{}", content);  // Show raw tokens
            accumulated.push_str(&content);

            // Try parsing partial
            if let Some(partial) = try_parse_partial_response(&ir, &accumulated, &output_type)? {
                println!("\n[Partial parse: {:?}]", partial);
            }
        }
    }

    // 3. Final parse
    let final_result = parse_llm_response_with_ir(&ir, &accumulated, &output_type)?;

    Ok(())
}
```

## How It Works

### Auto-Closing Strategy

The partial parser uses multiple strategies to complete incomplete JSON:

**Strategy 1: Count and Close**
```rust
// Input: {"name": "John", "age": 30
// Counts: 1 open brace, 0 close braces
// Output: {"name": "John", "age": 30}  ✅
```

**Strategy 2: Handle Incomplete Strings**
```rust
// Input: {"name": "Joh
// Detected: Incomplete string (no closing ")
// Output: {"name": "Joh"}  ✅
```

**Strategy 3: Remove Incomplete Fields**
```rust
// Input: {"name": "John", "age": 30, "job":
// Last complete field is "age"
// Output: {"name": "John", "age": 30}  ✅
```

**Strategy 4: Markdown Extraction**
```rust
// Input: Here's the data:\n```json\n{"name": "John"
// Extract: {"name": "John"
// Close: {"name": "John"}  ✅
```

## Comparison: Full BAML vs Simplified

| Feature | Full BAML (`jsonish`) | Simplified BAML |
|---------|----------------------|-----------------|
| **Size** | ~5,000+ lines, 5+ crates | ~300 lines, 0 extra deps |
| **Dependencies** | baml-types, internal-baml-core, internal-baml-jinja, etc. | Just anyhow, serde_json |
| **Partial parsing** | ✅ State machine parser | ✅ Heuristic parser |
| **Completion tracking** | ✅ Per-field state | ❌ Best-effort |
| **Streaming metadata** | ✅ Rich metadata | ❌ Simple Result |
| **Edge cases** | ✅ Handles everything | ✅ Handles 90% of cases |
| **Performance** | Optimized | Good enough |
| **Complexity** | High | Low |

## When to Use Which

### Use Simplified BAML's Partial Parser When:
- ✅ Learning how BAML works
- ✅ Prototyping
- ✅ Simple streaming use cases
- ✅ Want to keep dependencies minimal
- ✅ Streaming complete fields (not mid-word)

### Use Full BAML's `jsonish` When:
- ✅ Production applications
- ✅ Complex streaming scenarios
- ✅ Need per-field completion state
- ✅ Need rich streaming metadata
- ✅ Streaming word-by-word partial tokens

## Examples

See these examples for complete working code:

1. **`examples/streaming_with_partial_parsing.rs`** - Basic streaming demo
2. **`examples/standalone_functions.rs`** - Using the extracted functions independently

Run them:
```bash
cargo run --example streaming_with_partial_parsing
cargo run --example standalone_functions
```

## Testing

The partial parser has comprehensive tests:

```bash
# Run partial parser tests
cargo test partial_parser

# Run all tests
cargo test
```

## Future Improvements

Potential enhancements if needed:

1. **Token-by-token streaming** - Currently assumes mostly-complete fields
2. **Streaming state metadata** - Track which fields are complete
3. **Configurable strategies** - Let users choose auto-close behavior
4. **Error recovery** - Better handling of malformed JSON

For now, the simplified approach covers 90% of streaming use cases while keeping the codebase lean and educational.

## Summary

You now have **three options** for using BAML with streaming:

1. **Accumulate then parse** (simplest)
   ```rust
   let full_response = accumulate_stream().await;
   let result = parse_llm_response_with_ir(&ir, &full_response, &target_type)?;
   ```

2. **Partial parsing with simplified parser** (good balance)
   ```rust
   if let Some(partial) = try_parse_partial_response(&ir, &chunk, &target_type)? {
       show_to_user(partial);
   }
   ```

3. **Full BAML with jsonish** (production-ready)
   - Use the full BAML runtime for mission-critical applications

The simplified approach keeps your codebase minimal while still providing excellent streaming support! 🎉
