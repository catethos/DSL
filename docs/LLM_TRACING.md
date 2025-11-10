# LLM Tracing Implementation

This document describes the LLM tracing capability added to the DSL interpreter, which captures prompts sent to and responses received from LLM providers.

## Overview

The tracing system now captures:
- **Prompts**: The exact text sent to the LLM (including interpolated variables and schema injection)
- **Responses**: The raw text received from the LLM (before any processing/parsing)

This works for:
1. **Builtin LLM functions**: `Ask()`, `ExtractAs()`, `ExtractPerson()`
2. **User-defined LLM functions**: Functions with `prompt:` attribute
3. **HTTP+LLM functions**: Functions that fetch data via HTTP then process with LLM

## Implementation Details

### Architecture

The implementation follows a simple, non-invasive pattern:

1. **BuiltinFunctions** stores `last_prompt` and `last_response` after each LLM call
2. **TraceEvent** has optional `llm_prompt` and `llm_response` fields
3. **TracingInterpreter** captures these fields when tracing LLM function calls

### Code Changes

#### 1. BuiltinFunctions (builtins.rs)

Added `last_response` field to capture raw LLM responses:

```rust
pub struct BuiltinFunctions {
    // ... existing fields
    pub last_prompt: Option<String>,
    pub last_response: Option<String>,  // NEW
}
```

Updated all LLM methods to capture both prompt and response:
- `ask_with_config()`
- `extract_as_with_config()`
- `ask()`
- `extract_person()`
- `extract_as()`

Example:
```rust
pub async fn ask_with_config(...) -> Result<Value> {
    self.last_prompt = Some(prompt.clone());
    
    let client = self.create_client(...)?;
    let response = client.call(&prompt).await?;
    
    self.last_response = Some(response.clone());  // NEW
    
    Ok(Value::String(response))
}
```

#### 2. TraceEvent (tracing.rs)

Added optional LLM fields to trace events:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    // ... existing fields
    
    /// LLM prompt sent (for LLM function calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_prompt: Option<String>,
    
    /// LLM response received (for LLM function calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_response: Option<String>,
}
```

The `skip_serializing_if` ensures non-LLM events don't bloat the JSON export.

#### 3. TracingInterpreter (tracing.rs)

Added transient fields to capture LLM data:

```rust
pub struct TracingInterpreter {
    // ... existing fields
    current_llm_prompt: Option<String>,
    current_llm_response: Option<String>,
}
```

Updated `eval()` to attach LLM data to events:

```rust
let event = TraceEvent {
    // ... existing fields
    llm_prompt: self.current_llm_prompt.take(),
    llm_response: self.current_llm_response.take(),
};
```

Updated `eval_recursive()` to capture LLM data from both builtin and user functions:

```rust
// For builtin LLM functions
if lname == "ask" || lname == "extractas" || lname == "extractperson" {
    self.current_llm_prompt = self.interpreter.builtins.last_prompt.clone();
    self.current_llm_response = self.interpreter.builtins.last_response.clone();
}

// For user-defined LLM functions
if self.interpreter.builtins.last_prompt.is_some() {
    self.current_llm_prompt = self.interpreter.builtins.last_prompt.clone();
    self.current_llm_response = self.interpreter.builtins.last_response.clone();
}
```

## Usage Examples

### Basic Tracing

```rust
use dsl_interpreter::TracingInterpreter;

let source = r#"Ask("What is 2+2?")"#;
let ast = parse_program(source)?;
let ir = compile_program_to_ir(&ast)?;

let mut tracer = TracingInterpreter::from_ir(&ir)?;
let result = tracer.eval(&ir.entry_expr).await?;

// Access LLM trace data
for event in &tracer.trace.events {
    if let Some(prompt) = &event.llm_prompt {
        println!("Sent: {}", prompt);
    }
    if let Some(response) = &event.llm_response {
        println!("Received: {}", response);
    }
}
```

### Export to JSON

```rust
tracer.export_trace("trace.json")?;
```

The JSON output includes LLM data:

```json
{
  "events": [
    {
      "step": 0,
      "node_type": "FunctionCall",
      "description": "Call Ask(...) with 1 args",
      "duration_micros": 1234567,
      "llm_prompt": "What is 2+2?",
      "llm_response": "2 plus 2 equals 4."
    }
  ]
}
```

### Filtering LLM Calls

```rust
let llm_events: Vec<_> = tracer.trace.events.iter()
    .filter(|e| e.llm_prompt.is_some())
    .collect();

println!("Total LLM calls: {}", llm_events.len());
```

### Performance Analysis

```rust
for event in &tracer.trace.events {
    if let Some(prompt) = &event.llm_prompt {
        println!("LLM call took {}μs", event.duration_micros);
        println!("  Prompt: {} chars", prompt.len());
        if let Some(resp) = &event.llm_response {
            println!("  Response: {} chars", resp.len());
        }
    }
}
```

## Benefits

1. **Debugging**: See exactly what was sent to the LLM and what came back
2. **Prompt Engineering**: Analyze how variable interpolation and schema injection affect prompts
3. **Performance Analysis**: Correlate prompt/response sizes with latency
4. **Cost Tracking**: Estimate token usage from character counts
5. **Quality Assurance**: Verify LLM outputs before parsing/validation
6. **Auditing**: Log all LLM interactions for compliance/review

## Limitations

1. **Memory Usage**: Storing full prompts/responses can consume significant memory for long traces. Consider:
   - Using `TraceConfig::max_events` to limit buffer size
   - Streaming traces to files instead of holding in memory
   - Adding value truncation (future enhancement)

2. **No Streaming Support**: Currently captures only final responses, not streaming chunks

3. **Single LLM per Function**: If a user function makes multiple LLM calls internally, only the last one is captured

## Future Enhancements

Potential improvements identified by the oracle:

1. **Value Summarization**: Truncate large prompts/responses in traces with `max_value_preview` config
2. **Streaming Sink**: Write traces to NDJSON file incrementally for unbounded execution
3. **Parent/Child Relationships**: Add `parent_id` to reconstruct call trees
4. **Provider Metadata**: Capture model, provider, tokens, etc.
5. **Sampling**: Record only every Nth event for high-volume scenarios

## See Also

- [examples/llm_tracing_demo.rs](../crates/dsl-interpreter/examples/llm_tracing_demo.rs) - Complete working example
- [tracing.rs](../crates/dsl-interpreter/src/tracing.rs) - Implementation
- [INTERPRETER_PLAN.md](../crates/dsl-interpreter/INTERPRETER_PLAN.md) - Overall interpreter design
