# Schema-Aware Streaming (Recommended Approach)

## The Problem with Basic Streaming

When streaming LLM responses, you typically face this problem:

```javascript
// Chunk 1: {"name": "John"
// UI renders: <div>Name: John</div>

// Chunk 2: {"name": "John", "age": 30
// UI renders: <div>Name: John<br/>Age: 30</div>

// Chunk 3: {"name": "John", "age": 30, "occupation": "Engineer"
// UI renders: <div>Name: John<br/>Age: 30<br/>Occupation: Engineer</div>
```

**Problems:**
- ❌ Structure changes on every update (layout shift)
- ❌ Components appear/disappear unpredictably
- ❌ Hard to show "loading" states for pending fields
- ❌ Animations/transitions break
- ❌ Accessibility issues (screen readers confused)

## The Solution: Schema-Aware Streaming

Since we already know the IR schema upfront, we can do better:

```javascript
// Initial state (before any data):
{
  "value": {"name": null, "age": null, "occupation": null},
  "state": "pending"
}

// Chunk 1: {"name": "John"
{
  "value": {"name": "John", "age": null, "occupation": null},
  "state": "partial"
}

// Chunk 2: {"name": "John", "age": 30
{
  "value": {"name": "John", "age": 30, "occupation": null},
  "state": "partial"
}

// Final:
{
  "value": {"name": "John", "age": 30, "occupation": "Engineer"},
  "state": "complete"
}
```

**Benefits:**
- ✅ Structure is **always consistent**
- ✅ No layout shift
- ✅ Easy to show loading states (`null` → loading spinner)
- ✅ Smooth animations
- ✅ Better accessibility
- ✅ Simpler React/Vue/etc code

## API

### 1. Create Skeleton

```rust
use simplify_baml::*;

let ir = /* your IR */;
let target_type = FieldType::Class("Person".to_string());

// Create skeleton with full structure from IR
let mut streaming = StreamingBamlValue::from_ir_skeleton(&ir, &target_type);

// Initial state - all fields are null, state is "pending"
println!("{}", serde_json::to_string_pretty(&streaming)?);
// {
//   "value": {
//     "name": null,
//     "age": null,
//     "occupation": null
//   },
//   "state": "pending"
// }
```

### 2. Update as Data Arrives

```rust
let mut accumulated = String::new();

while let Some(chunk) = stream.next().await {
    accumulated.push_str(&chunk?);

    // Update the streaming value in-place
    update_streaming_response(
        &mut streaming,
        &ir,
        &accumulated,
        &target_type,
        false  // not final yet
    )?;

    // Send to UI - always has full structure!
    send_to_ui(&streaming);
}

// Mark as complete
update_streaming_response(&mut streaming, &ir, &accumulated, &target_type, true)?;
```

### 3. Render in UI

The UI code is now trivial because the structure never changes:

```javascript
// React Example
function PersonCard({ streaming }) {
  return (
    <div className="card">
      <Field
        label="Name"
        value={streaming.value.name}
        isLoading={streaming.value.name === null}
        isComplete={streaming.state === "complete"}
      />
      <Field
        label="Age"
        value={streaming.value.age}
        isLoading={streaming.value.age === null}
        isComplete={streaming.state === "complete"}
      />
      <Field
        label="Occupation"
        value={streaming.value.occupation}
        isLoading={streaming.value.occupation === null}
        isComplete={streaming.state === "complete"}
      />
      <SubmitButton disabled={streaming.state !== "complete"} />
    </div>
  );
}
```

## Complete Example

```rust
use simplify_baml::*;
use std::collections::HashMap;

async fn streaming_example() -> anyhow::Result<()> {
    // 1. Build IR
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

    let target_type = FieldType::Class("Person".to_string());

    // 2. Generate prompt
    let template = "Extract person: {{ text }}";
    let mut params = HashMap::new();
    params.insert("text".to_string(), BamlValue::String("Alice is 30".to_string()));

    let prompt = generate_prompt_from_ir(&ir, template, &params, &target_type)?;

    // 3. Create skeleton
    let mut streaming = StreamingBamlValue::from_ir_skeleton(&ir, &target_type);

    // 4. Stream from any LLM client
    let mut stream = your_llm_client.stream(&prompt).await?;
    let mut accumulated = String::new();

    while let Some(chunk) = stream.next().await {
        accumulated.push_str(&chunk?);

        // Update skeleton with new data
        update_streaming_response(&mut streaming, &ir, &accumulated, &target_type, false)?;

        // Send to UI
        println!("{}", serde_json::to_string_pretty(&streaming)?);
    }

    // Mark complete
    update_streaming_response(&mut streaming, &ir, &accumulated, &target_type, true)?;

    Ok(())
}
```

## Key Types

### `StreamingBamlValue`

Wraps a `BamlValue` with completion state:

```rust
pub struct StreamingBamlValue {
    pub value: BamlValue,
    pub completion_state: CompletionState,
}
```

**Serializes to:**
```json
{
  "value": { /* your data */ },
  "state": "pending" | "partial" | "complete"
}
```

### `CompletionState`

```rust
pub enum CompletionState {
    Pending,   // No data yet
    Partial,   // Some data received
    Complete,  // All data received
}
```

## Comparison

### Old Way (Unpredictable Structure)

```rust
// Just parse whatever we have
let result = try_parse_partial_response(&ir, &partial, &target_type)?;

match result {
    Some(value) => send_to_ui(value),  // Structure changes each time!
    None => {}
}
```

**Problems:**
- Structure grows over time
- UI must handle all variations
- Layout shifts constantly

### New Way (Schema-Aware)

```rust
// Create skeleton once
let mut streaming = StreamingBamlValue::from_ir_skeleton(&ir, &target_type);

// Update in-place
update_streaming_response(&mut streaming, &ir, &partial, &target_type, false)?;

// Always send full structure
send_to_ui(&streaming);
```

**Benefits:**
- Structure never changes
- UI code is simple
- Better UX

## Implementation Details

### How Skeletons Are Created

```rust
// For primitives: null
FieldType::String => BamlValue::Null

// For classes: object with all fields as null
FieldType::Class("Person") => BamlValue::Map({
    "name": BamlValue::Null,
    "age": BamlValue::Null,
    "occupation": BamlValue::Null,
})

// For lists: empty array
FieldType::List(_) => BamlValue::List([])

// For optional fields: null
optional: true => BamlValue::Null
```

### How Updates Are Merged

```rust
// Target (skeleton):  {"name": null, "age": null, "occupation": null}
// Source (partial):   {"name": "John", "age": 30}
// Result:             {"name": "John", "age": 30, "occupation": null}
//                                       ↑          ↑
//                                    updated    preserved
```

The merge is **field-aware** - it only updates fields that exist in the source, preserving the rest.

## Run the Examples

```bash
# Schema-aware streaming (RECOMMENDED)
cargo run --example streaming_with_schema_structure

# Basic partial parsing (also works)
cargo run --example streaming_with_partial_parsing

# Standalone functions demo
cargo run --example standalone_functions
```

## When to Use Which

### Use `StreamingBamlValue` (Schema-Aware) When:
- ✅ Building UIs (React, Vue, etc.)
- ✅ Need consistent structure
- ✅ Want to show loading states
- ✅ Need to disable buttons until complete
- ✅ **This is the recommended approach!**

### Use `try_parse_partial_response()` (Basic) When:
- ✅ Backend-only processing
- ✅ Don't care about structure consistency
- ✅ Just want any valid data ASAP
- ✅ Building CLI tools

### Use Full Accumulation When:
- ✅ Simplest approach
- ✅ Don't need intermediate updates
- ✅ Small responses

## Testing

All streaming functionality is tested:

```bash
# Test streaming value creation and updates
cargo test streaming_value

# Test partial JSON parsing
cargo test partial_parser

# Run all tests
cargo test
```

Current test count: **30 tests passing**

## Code Size

The entire streaming implementation is remarkably small:

- `streaming_value.rs`: ~330 lines (skeleton creation + merging)
- `partial_parser.rs`: ~310 lines (incomplete JSON handling)
- `runtime.rs` additions: ~50 lines (integration)

**Total: ~690 lines** for production-ready streaming support!

Compare this to full BAML's `jsonish` library: **5,000+ lines** across multiple crates.

## Summary

Schema-aware streaming gives you:

1. **Predictable UX** - Structure never changes
2. **Simple code** - UI components are straightforward
3. **Better accessibility** - Screen readers aren't confused
4. **Professional polish** - No layout shifts or flashing content
5. **Small footprint** - Just ~690 lines of code

This is the **recommended approach** for any UI that displays streaming LLM results!
