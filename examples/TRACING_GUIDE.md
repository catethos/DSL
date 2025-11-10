# DSL Tracing Guide

## Quick Reference

The DSL provides built-in **recursive execution tracing** for debugging, profiling, and optimization. All child node evaluations are automatically traced, giving you complete visibility into your program's execution.

### Basic Usage

```bash
# Enable tracing
dsl run myfile.dsl --trace

# Verbose output with detailed trace events
dsl run myfile.dsl --trace --trace-verbose

# Export trace to JSON
dsl run myfile.dsl --trace --trace-output trace.json

# Filter specific node types
dsl run myfile.dsl --trace --trace-filter BinaryOp,FunctionCall

# Only show slow operations (> 1000μs)
dsl run myfile.dsl --trace --trace-min-duration 1000
```

## Command-Line Options

| Option | Description |
|--------|-------------|
| `--trace` | Enable execution tracing |
| `--trace-verbose` | Show detailed trace table |
| `--trace-output <file>` | Export trace to JSON file |
| `--trace-filter <types>` | Filter by node types (comma-separated) |
| `--trace-min-duration <μs>` | Only show events slower than N microseconds |

## Example Output

### Basic Trace

```bash
$ dsl run examples/greet.dsl --trace
Running examples/greet.dsl...
Compiling to IR...
Executing...

Result:
"HELLO, WORLD!"

=== Trace Summary ===
Total events: 4
Total duration: 359µs

Breakdown by node type:
  Sequential      1 calls  200µs
  FunctionCall    1 calls   84µs
  TemplateString  2 calls   75µs
```

Notice how **all nested evaluations** are captured - the template strings, function call, and sequential operation.

### Verbose Trace

```bash
$ dsl run test.dsl --trace --trace-verbose
...
Detailed Trace:
Step   Node Type       Duration     Depth    Description
--------------------------------------------------------------------------------
0      BinaryOp               300μs 1        Binary operation: -
```

### JSON Export

```bash
$ dsl run test.dsl --trace --trace-output trace.json
...
✓ Trace exported to trace.json
```

JSON format:
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
      "variables": null
    }
  ]
}
```

## Node Types You Can Filter

Common node types for filtering:

- `BinaryOp` - Binary operations (+, -, *, /, etc.)
- `FunctionCall` - Function calls
- `Conditional` - If/else expressions
- `Sequential` - Pipeline operations (|>)
- `Parallel` - Parallel operations (||)
- `Match` - Pattern matching
- `Block` - Block expressions
- `List` - List construction
- `Map` - Map/object construction
- `Variable` - Variable references
- `FieldAccess` - Field access (.)
- `IndexAccess` - Index access ([])

## Use Cases

### 1. Performance Profiling

Find slow operations:
```bash
dsl run myfile.dsl --trace --trace-min-duration 1000
```

### 2. Debugging

See detailed execution flow:
```bash
dsl run myfile.dsl --trace --trace-verbose
```

### 3. Function Call Analysis

Track only function calls:
```bash
dsl run myfile.dsl --trace --trace-filter FunctionCall --trace-verbose
```

### 4. Export for Analysis

Save trace for later review:
```bash
dsl run myfile.dsl --trace --trace-output analysis.json
```

### 5. Combined Analysis

```bash
dsl run myfile.dsl \
  --trace \
  --trace-verbose \
  --trace-filter BinaryOp,FunctionCall \
  --trace-min-duration 100 \
  --trace-output detailed_trace.json
```

## Programmatic Usage

You can also use tracing from Rust code:

```rust
use dsl_interpreter::{TracingInterpreter, TraceConfig};
use dsl_core::compile_to_ir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "2 + 3 * 4";
    let ir = compile_to_ir(source)?;

    let config = TraceConfig {
        max_events: 10000,
        capture_variables: false,
        node_filter: vec!["BinaryOp".to_string()],
        min_duration_micros: 0,
    };

    let mut tracer = TracingInterpreter::from_ir_with_config(&ir, config)?;
    let result = tracer.eval(&ir.entry_expr).await?;

    println!("{}", tracer.trace.summary());
    tracer.export_trace("trace.json")?;

    Ok(())
}
```

## Performance Impact

- **Minimal overhead**: ~1-5% for most operations
- **Memory usage**: ~100-200 bytes per event
- **Recommendations**:
  - Use `--trace-filter` to reduce noise
  - Use `--trace-min-duration` to focus on slow operations
  - Set reasonable limits for long-running programs

## See Also

- [Full Tracing Documentation](docs/09-Tracing-Interpreter.md)
- [Example Programs](examples/tracing_demo.rs)
- [API Reference](crates/dsl-interpreter/src/tracing.rs)
