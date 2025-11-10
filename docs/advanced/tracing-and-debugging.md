# Tracing and Debugging

The DSL provides comprehensive **execution tracing** for debugging, profiling, and optimization. The tracing system captures detailed execution information including timing, values, call stacks, and LLM interactions.

## Table of Contents

- [Quick Start](#quick-start)
- [Command-Line Usage](#command-line-usage)
- [Trace Features](#trace-features)
- [LLM Tracing](#llm-tracing)
- [Programmatic Usage](#programmatic-usage)
- [Trace Analysis](#trace-analysis)
- [Configuration Options](#configuration-options)
- [Performance Considerations](#performance-considerations)
- [Examples](#examples)

---

## Quick Start

The easiest way to use tracing is with the `--trace` flag:

```bash
# Basic tracing
dsl run myfile.dsl --trace

# Detailed trace with verbose output
dsl run myfile.dsl --trace --trace-verbose

# Export trace to JSON file
dsl run myfile.dsl --trace --trace-output trace.json

# Filter trace by node types
dsl run myfile.dsl --trace --trace-filter BinaryOp,FunctionCall

# Only show slow operations (> 1000 microseconds)
dsl run myfile.dsl --trace --trace-min-duration 1000

# Combine options
dsl run myfile.dsl --trace --trace-verbose --trace-output trace.json
```

### Example Output

```
Running test_trace.dsl...
Compiling to IR...
Executing...

Result:
19

=== Trace Summary ===
Total events: 1
Total duration: 80µs

Breakdown by node type:
  BinaryOp    1 calls  80µs

Detailed Trace:
Step   Node Type       Duration     Depth    Description
--------------------------------------------------------------------------------
0      BinaryOp                80μs 1        Binary operation: +

✓ Trace exported to trace.json
```

---

## Command-Line Usage

### Available Options

| Option | Description |
|--------|-------------|
| `--trace` | Enable execution tracing |
| `--trace-verbose` | Show detailed trace table |
| `--trace-output <file>` | Export trace to JSON file |
| `--trace-filter <types>` | Filter by node types (comma-separated) |
| `--trace-min-duration <μs>` | Only show events slower than N microseconds |

### Examples

**Basic tracing:**
```bash
dsl run examples/greet.dsl --trace
```

**Verbose output:**
```bash
dsl run examples/greet.dsl --trace --trace-verbose
```

**Filter specific operations:**
```bash
dsl run examples/greet.dsl --trace --trace-filter FunctionCall,LLMCall
```

**Performance profiling:**
```bash
# Find operations taking longer than 1ms
dsl run myfile.dsl --trace --trace-min-duration 1000
```

**Export for analysis:**
```bash
dsl run myfile.dsl --trace --trace-output trace.json
# Analyze trace.json with external tools
```

---

## Trace Features

### What is Captured

The `TracingInterpreter` records:

1. **Node Information**
   - Node type being executed
   - Description of operation
   - Call stack depth

2. **Values**
   - Input values
   - Output values
   - Optional variable snapshots

3. **Timing**
   - Execution duration (microseconds)
   - Total time breakdown by node type

4. **LLM Interactions** (see [LLM Tracing](#llm-tracing))
   - Prompts sent to LLM
   - Responses received from LLM

5. **Errors**
   - Error information with context

### Recursive Tracing

All child node evaluations are automatically traced:

```bash
$ dsl run examples/greet.dsl --trace
...
=== Trace Summary ===
Total events: 4
Total duration: 359µs

Breakdown by node type:
  Sequential      1 calls  200µs
  FunctionCall    1 calls   84µs
  TemplateString  2 calls   75µs
```

Notice how nested evaluations (template strings within function call within sequential operation) are all captured.

### Node Types

Common node types you'll see in traces:

- **BinaryOp** - Binary operations (+, -, *, /, etc.)
- **FunctionCall** - Function invocations
- **TemplateString** - String interpolation
- **Sequential** - Sequential composition (>>)
- **Parallel** - Parallel execution (par())
- **Conditional** - Conditional expressions (?:)
- **FieldAccess** - Object field access
- **IndexAccess** - List/array indexing
- **LetBinding** - Variable binding
- **LLMCall** - LLM function calls
- **SQLQuery** - SQL execution
- **HTTPRequest** - HTTP requests

---

## LLM Tracing

The tracing system captures complete LLM interactions for debugging AI-powered workflows.

### What is Captured

For LLM function calls, the trace includes:

1. **Prompts**: The exact text sent to the LLM
   - Including interpolated variables
   - Including schema injection for structured output
   - After all template processing

2. **Responses**: The raw text received from the LLM
   - Before parsing/processing
   - As returned by the provider

### Works With

- **Builtin LLM functions**: `Ask()`, `ExtractAs()`
- **User-defined LLM functions**: Functions with `prompt:` attribute
- **HTTP+LLM functions**: Combined HTTP fetch + LLM processing

### Example

```javascript
// test_llm.dsl
Ask("What is 2+2?") as answer
```

```bash
$ dsl run test_llm.dsl --trace --trace-verbose
...
Detailed Trace:
Step   Node Type    Duration    Depth   Description
---------------------------------------------------------------------------
0      LLMCall         1200μs   1       Ask("What is 2+2?")

LLM Prompt:
  "What is 2+2?"

LLM Response:
  "2+2 equals 4."
```

### JSON Export Format

When exporting with `--trace-output`, LLM interactions are included:

```json
{
  "events": [
    {
      "step": 0,
      "node_type": "LLMCall",
      "description": "Ask(\"What is 2+2?\")",
      "duration_micros": 1200,
      "depth": 1,
      "llm_prompt": "What is 2+2?",
      "llm_response": "2+2 equals 4."
    }
  ]
}
```

Note: Non-LLM events omit the `llm_prompt` and `llm_response` fields to avoid bloating the JSON.

### Implementation Details

The LLM tracing follows a simple pattern:

1. **BuiltinFunctions** stores `last_prompt` and `last_response`
2. **TraceEvent** has optional `llm_prompt` and `llm_response` fields
3. **TracingInterpreter** captures these after each LLM call

This design is non-invasive and adds minimal overhead.

---

## Programmatic Usage

### Basic Setup

```rust
use dsl_interpreter::TracingInterpreter;
use dsl_core::{parse_program, compile_program_to_ir};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse and compile DSL code
    let source = "2 + 3 * 4";
    let ast = parse_program(source)?;
    let ir = compile_program_to_ir(&ast)?;

    // Create tracing interpreter
    let mut tracer = TracingInterpreter::from_ir(&ir)?;

    // Execute with tracing
    let result = tracer.eval(&ir.entry_expr).await?;

    // View trace summary
    println!("{}", tracer.trace.summary());

    Ok(())
}
```

### With Configuration

```rust
use dsl_interpreter::{TracingInterpreter, TraceConfig};

let config = TraceConfig {
    // Maximum events to keep (0 = unlimited)
    max_events: 10000,

    // Capture variable snapshots (adds overhead)
    capture_variables: false,

    // Only trace specific node types (empty = all)
    node_filter: vec!["FunctionCall".to_string(), "BinaryOp".to_string()],

    // Minimum duration to record (microseconds)
    min_duration_micros: 100,
};

let mut tracer = TracingInterpreter::with_config(config)?;
```

---

## Trace Analysis

### Summary Statistics

```rust
// Print summary report
tracer.print_summary();

// Get total execution time
let total_time = tracer.trace.total_duration();
println!("Total: {:?}", total_time);

// Get statistics by node type
let stats = tracer.trace.stats_by_node_type();
for (node_type, count, duration) in stats {
    println!("{}: {} calls in {:?}", node_type, count, duration);
}
```

### Finding Performance Issues

```rust
// Find slow operations (> 1ms)
let slow_events = tracer.trace.slow_events(1000);
for event in slow_events {
    println!("Slow: {} took {}μs", event.description, event.duration_micros);
}

// Find errors
let errors = tracer.trace.errors();
for event in errors {
    println!("Error at step {}: {}", event.step, event.error.as_ref().unwrap());
}
```

### Exporting Traces

```rust
// Export to JSON
let json = tracer.trace.to_json_pretty()?;
std::fs::write("trace.json", json)?;

// Or compact format
let json = tracer.trace.to_json()?;
std::fs::write("trace.json", json)?;
```

### JSON Format

```json
{
  "events": [
    {
      "step": 0,
      "node_type": "BinaryOp",
      "description": "Binary operation: -",
      "inputs": [],
      "output": { "Int": 65 },
      "error": null,
      "duration_micros": 300,
      "depth": 1,
      "variables": null,
      "llm_prompt": null,
      "llm_response": null
    }
  ]
}
```

---

## Configuration Options

### TraceConfig Fields

```rust
pub struct TraceConfig {
    /// Maximum number of events to keep (0 = unlimited)
    pub max_events: usize,

    /// Capture variable snapshots at each step
    /// Warning: Adds significant overhead
    pub capture_variables: bool,

    /// Only trace these node types (empty = all)
    pub node_filter: Vec<String>,

    /// Minimum duration to record (microseconds)
    /// Events faster than this are skipped
    pub min_duration_micros: u64,
}
```

### Default Configuration

```rust
TraceConfig {
    max_events: 0,              // Unlimited
    capture_variables: false,   // No variable snapshots
    node_filter: vec![],        // Trace all node types
    min_duration_micros: 0,     // Record all events
}
```

### Performance Tuning

**For production debugging:**
```rust
TraceConfig {
    max_events: 10000,           // Limit memory usage
    capture_variables: false,    // Skip variable snapshots
    node_filter: vec![],         // Trace everything
    min_duration_micros: 0,      // Catch all events
}
```

**For performance profiling:**
```rust
TraceConfig {
    max_events: 1000,            // Focus on slow operations
    capture_variables: false,    // Skip snapshots
    node_filter: vec![],         // All node types
    min_duration_micros: 1000,   // Only operations > 1ms
}
```

**For debugging specific issues:**
```rust
TraceConfig {
    max_events: 0,               // Unlimited
    capture_variables: true,     // Full variable snapshots
    node_filter: vec![           // Focus on relevant nodes
        "FunctionCall".to_string(),
        "LLMCall".to_string(),
    ],
    min_duration_micros: 0,      // All events
}
```

---

## Performance Considerations

### Overhead

Tracing adds some performance overhead:

- **Minimal overhead** (~5-10%): Basic tracing without variable snapshots
- **Moderate overhead** (~20-30%): With detailed logging
- **High overhead** (~50-100%): With variable snapshots enabled

### Best Practices

1. **Disable in production** unless debugging
2. **Use filters** to focus on specific node types
3. **Set duration thresholds** to ignore fast operations
4. **Limit max events** to prevent memory issues
5. **Avoid variable snapshots** unless necessary

### When to Use Tracing

**Good use cases:**
- Debugging unexpected behavior
- Performance profiling
- Understanding execution flow
- Analyzing LLM interactions
- Finding bottlenecks

**Not recommended for:**
- Production deployments (unless debugging)
- Performance-critical paths
- High-frequency operations
- Real-time systems

---

## Examples

### Example 1: Basic Workflow Tracing

```javascript
// workflow.dsl
"World" as name
  |> "Hello, ${_}!" as greeting
  |> Upper(greeting)
```

```bash
$ dsl run workflow.dsl --trace --trace-verbose
...
Step   Node Type       Duration     Depth    Description
--------------------------------------------------------------------------------
0      LetBinding           50μs    1        Bind: name
1      TemplateString       30μs    2        Template interpolation
2      LetBinding           20μs    1        Bind: greeting
3      FunctionCall        100μs    2        Upper(greeting)
```

### Example 2: LLM Interaction

```javascript
// ai_query.dsl
Ask("What is the capital of France?") as answer
```

```bash
$ dsl run ai_query.dsl --trace --trace-output llm_trace.json
...
✓ Trace exported to llm_trace.json
```

The JSON will contain the full prompt and response.

### Example 3: Performance Profiling

```javascript
// slow_workflow.dsl
Range(1, 1000)
  |> Map(_, x -> x * x)
  |> Filter(_, x -> x > 500)
  |> Sum(_)
```

```bash
# Find slow operations
$ dsl run slow_workflow.dsl --trace --trace-min-duration 1000
...
Slow operations (> 1000μs):
  Map: 2300μs
  Filter: 1500μs
```

### Example 4: Debugging Errors

```bash
# Enable tracing to see where error occurred
$ dsl run buggy.dsl --trace --trace-verbose
...
Step   Node Type       Duration     Depth    Description
--------------------------------------------------------------------------------
0      LetBinding          50μs     1        Bind: x
1      BinaryOp          ERROR      2        Binary operation: /

Error: Division by zero at step 1
```

---

## Related Documentation

- [Implementation Details](12-Implementation-Details.md) - Architecture internals
- [LLM Integration](../user-guide/08-LLM-Integration.md) - Using LLM functions
- [Advanced Features](11-Advanced-Features.md) - Advanced usage patterns

---

**Version:** 1.0
**Last Updated:** 2025-11-10
