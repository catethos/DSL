# Examples Summary

## Created Examples

### 1. `explicit_partial_parsing.rs` - **Most Clear Example**

**Purpose:** Shows partial parsing results explicitly at each step

**What it demonstrates:**
- Clear display of partial results as they arrive
- Shows exactly what fields are parsed at each checkpoint
- Pretty-prints the incremental structure
- Parses every 5 chunks and displays results

**Key features:**
```rust
// Function to display partial results clearly
fn print_partial_result(label: &str, result: &Option<BamlValue>) {
    // Shows each field as it becomes available
    // Formatted output showing structure
}

// Parse every 5 chunks and show progress
if chunk_count % 5 == 0 {
    match try_parse_partial_response(&ir, &accumulated, &target_type)? {
        Some(parsed) => {
            print_partial_result("Partial Parse #X", &Some(parsed));
        }
        None => {
            // Cannot parse yet
        }
    }
}
```

**Run:**
```bash
export OPENAI_API_KEY=sk-...
cargo run --example explicit_partial_parsing
```

### 2. `rust_genai_streaming_example.rs` - Basic Integration

**Purpose:** Complete flow from schema to structured output

**What it demonstrates:**
- Schema definition with IR
- Prompt generation
- rust-genai streaming API
- Partial parsing during streaming
- Final structured output

**Key flow:**
1. Define Person schema (name, age, occupation, level, skills)
2. Generate prompt with schema injection
3. Stream from LLM using `ChatStreamEvent`
4. Parse incrementally
5. Extract final BamlValue

**Run:**
```bash
export OPENAI_API_KEY=sk-...
cargo run --example rust_genai_streaming_example
```

### 3. `advanced_streaming_flow.rs` - Complex Schema

**Purpose:** Shows streaming with nested structures and tracking

**What it demonstrates:**
- Complex nested schemas (Task with Assignee)
- Multiple enums (Priority, Status)
- Field-by-field tracking as data arrives
- Parse attempt statistics

**Schema:**
- Task class with 7 fields
- Nested Assignee class
- List and enum types
- Parse every 3 chunks

**Run:**
```bash
export OPENAI_API_KEY=sk-...
cargo run --example advanced_streaming_flow
```

## Comparison

| Example | Complexity | Best For | Partial Display |
|---------|-----------|----------|-----------------|
| `explicit_partial_parsing.rs` | Simple | Learning how partial parsing works | ⭐⭐⭐⭐⭐ Very clear |
| `rust_genai_streaming_example.rs` | Medium | Understanding full integration | ⭐⭐⭐ Good |
| `advanced_streaming_flow.rs` | Complex | Real-world nested schemas | ⭐⭐⭐⭐ Very good |

## Key Pattern: rust-genai Streaming

All examples use the same streaming pattern:

```rust
use genai::chat::{ChatMessage, ChatRequest, ChatStreamEvent};
use genai::Client;
use futures::StreamExt;

let client = Client::default();
let mut chat_stream = client.exec_chat_stream("gpt-4o-mini", chat_req, None).await?;

let mut accumulated = String::new();

while let Some(event_result) = chat_stream.stream.next().await {
    let event = event_result?;
    
    match event {
        ChatStreamEvent::Chunk(chunk) => {
            let text = &chunk.content;
            accumulated.push_str(text);
            
            // Try partial parsing
            if let Some(partial) = try_parse_partial_response(&ir, &accumulated, &target_type)? {
                // Process partial result
            }
        }
        _ => {}
    }
}
```

## Partial Parsing Function

Located in `src/partial_parser.rs`:

```rust
pub fn try_parse_partial_json(partial_json: &str) -> Result<Option<JsonValue>>
```

**Strategies:**
1. Extract from markdown code blocks (` ```json `)
2. Find JSON boundaries (`{` and `}`)
3. Auto-close open structures
4. Handle incomplete strings
5. Multiple completion attempts

**Example transformations:**
```
Input:  {"name": "Joh
Output: {"name": "Joh"}  ✅

Input:  {"name": "John", "age": 3
Output: {"name": "John", "age": 3}  ✅

Input:  {"person": {"name": "John"
Output: {"person": {"name": "John"}}  ✅
```

## Runtime API Functions

From `src/runtime.rs`:

### 1. Generate Prompt
```rust
pub fn generate_prompt_from_ir(
    ir: &IR,
    template: &str,
    params: &HashMap<String, BamlValue>,
    output_type: &FieldType,
) -> Result<String>
```

### 2. Parse Complete Response
```rust
pub fn parse_llm_response_with_ir(
    ir: &IR,
    raw_response: &str,
    target_type: &FieldType,
) -> Result<BamlValue>
```

### 3. Parse Partial Response
```rust
pub fn try_parse_partial_response(
    ir: &IR,
    partial_response: &str,
    target_type: &FieldType,
) -> Result<Option<BamlValue>>
```

Returns:
- `Ok(Some(value))` - Successfully parsed (may be incomplete)
- `Ok(None)` - Need more data
- `Err(...)` - Parse error

## Architecture Flow

```
User Input + Schema (IR)
        ↓
generate_prompt_from_ir()
        ↓
    Prompt Text
        ↓
rust-genai: exec_chat_stream()
        ↓
Stream: ChatStreamEvent::Chunk(text)
        ↓
Accumulate: accumulated_text += text
        ↓
try_parse_partial_response()  ← Every N chunks
        ↓
    BamlValue (partial)
        ↓
Display to user (real-time!)
        ↓
[Stream ends]
        ↓
parse_llm_response_with_ir()
        ↓
BamlValue (complete & validated)
```

## Dependencies

From `Cargo.toml`:
```toml
genai = "0.4.2"        # Multi-provider LLM client
futures = "0.3"        # For StreamExt trait
serde_json = "1.0"     # JSON parsing
minijinja = "2.0"      # Template rendering
anyhow = "1.0"         # Error handling
```

## Quick Start

1. **Set API key:**
```bash
export OPENAI_API_KEY=sk-your-key-here
```

2. **Run the clearest example:**
```bash
cargo run --example explicit_partial_parsing
```

3. **Watch the output:**
- See streaming text appear in real-time
- Observe partial parse results after every 5 chunks
- See how fields get populated incrementally
- Final structured output at the end

## Documentation

- **`CURRENT_FLOW_EXPLANATION.md`** - Complete architecture and patterns
- **`RUST_GENAI_GUIDE.md`** - rust-genai API reference
- **`examples/`** - All runnable examples
