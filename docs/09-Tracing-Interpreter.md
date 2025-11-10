# Tracing Interpreter

The DSL provides a **Tracing Interpreter** that records detailed execution information for debugging, profiling, and optimization purposes.

## Quick Start (CLI)

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

## Overview

The `TracingInterpreter` wraps the standard `Interpreter` and captures trace events at each execution step, including:

- Node type being executed
- Input and output values
- Execution duration (microseconds)
- Call stack depth
- Optional variable snapshots
- Error information

## Basic Usage

```rust
use dsl_interpreter::{TracingInterpreter};
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

## Configuration

You can configure tracing behavior with `TraceConfig`:

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

### Finding Issues

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

## Exporting Traces

### JSON Export

```rust
// Export to JSON file
tracer.export_trace("/tmp/trace.json")?;

// Or get JSON string
let json = tracer.trace.to_json()?;
println!("{}", json);
```

### JSON Format

```json
{
  "events": [
    {
      "step": 0,
      "node_type": "BinaryOp",
      "description": "Binary operation: +",
      "inputs": [],
      "output": {
        "Int": 15
      },
      "error": null,
      "duration_micros": 42,
      "depth": 1,
      "variables": null
    }
  ]
}
```

### Loading Traces

```rust
// Load trace from file for analysis
let trace = TraceCollector::from_json_file("/tmp/trace.json")?;
println!("{}", trace.summary());
```

## Use Cases

### 1. Performance Profiling

Identify bottlenecks in your DSL code:

```rust
let mut tracer = TracingInterpreter::from_ir(&ir)?;
tracer.eval(&ir.entry_expr).await?;

// Find operations that took > 100μs
let slow_ops = tracer.trace.slow_events(100);
for event in slow_ops {
    println!("{}: {}μs", event.description, event.duration_micros);
}
```

### 2. Debugging

Trace execution flow step-by-step:

```rust
for (i, event) in tracer.trace.events.iter().enumerate() {
    println!("Step {}: {} - {:?}", i, event.description, event.output);
}
```

### 3. Optimization Analysis

Compare different implementations:

```rust
// Version 1
let mut tracer1 = TracingInterpreter::from_ir(&ir1)?;
tracer1.eval(&ir1.entry_expr).await?;
let time1 = tracer1.trace.total_duration();

// Version 2
let mut tracer2 = TracingInterpreter::from_ir(&ir2)?;
tracer2.eval(&ir2.entry_expr).await?;
let time2 = tracer2.trace.total_duration();

println!("Version 1: {:?}, Version 2: {:?}", time1, time2);
```

### 4. Call Graph Analysis

Analyze call stack depth and recursion:

```rust
let max_depth = tracer.trace.events.iter()
    .map(|e| e.depth)
    .max()
    .unwrap_or(0);
println!("Maximum call depth: {}", max_depth);
```

## API Reference

### TracingInterpreter

- `new()` - Create with default config
- `with_config(config)` - Create with custom config
- `from_ir(ir)` - Create from IR with default config
- `from_ir_with_config(ir, config)` - Create from IR with custom config
- `eval(&mut self, node)` - Evaluate IR node with tracing
- `clear_trace(&mut self)` - Clear trace history
- `export_trace(&self, path)` - Export to JSON file
- `print_summary(&self)` - Print summary to stdout

### TraceCollector

- `new(config)` - Create with config
- `with_default_config()` - Create with defaults
- `record(&mut self, event)` - Add trace event
- `clear(&mut self)` - Clear all events
- `total_duration(&self)` - Get total execution time
- `stats_by_node_type(&self)` - Get statistics by node type
- `slow_events(&self, threshold)` - Get events slower than threshold
- `errors(&self)` - Get all error events
- `to_json(&self)` - Export as JSON string
- `to_json_file(&self, path)` - Export to JSON file
- `from_json_file(path)` - Load from JSON file
- `summary(&self)` - Get summary report string

### TraceEvent

Fields:
- `step: usize` - Sequential step number
- `node_type: String` - Type of IR node
- `description: String` - Human-readable description
- `inputs: Vec<Value>` - Input values
- `output: Option<Value>` - Output value
- `error: Option<String>` - Error message if failed
- `duration_micros: u128` - Execution duration in microseconds
- `depth: usize` - Call stack depth
- `variables: Option<Vec<(String, Value)>>` - Variable snapshots (if enabled)

### TraceConfig

Fields:
- `max_events: usize` - Max events to keep (0 = unlimited)
- `capture_variables: bool` - Include variable snapshots
- `node_filter: Vec<String>` - Only trace these node types (empty = all)
- `min_duration_micros: u128` - Minimum duration to record (0 = all)

## Examples

See `crates/dsl-interpreter/examples/tracing_demo.rs` for a comprehensive demonstration of all tracing features.

Run with:
```bash
cargo run --example tracing_demo --release
```

## Performance Considerations

- **Overhead**: Tracing adds minimal overhead (~1-5%) for most operations
- **Memory**: Each event uses ~100-200 bytes. Use `max_events` to limit memory usage
- **Variable Capture**: Enabling `capture_variables` significantly increases overhead
- **Filtering**: Use `node_filter` to reduce noise and improve performance

## Future Enhancements

Planned features:
- Flame graph generation
- Interactive trace viewer
- Trace comparison tools
- Hot path detection
- Memory profiling
- Trace replay functionality
