# DSL IR Interpreter - Node.js Bindings

Node.js bindings for the DSL IR Interpreter using [Neon](https://neon-bindings.com/). This package provides high-performance evaluation of DSL Intermediate Representation (IR) with support for tracing, effects (HTTP, SQL, LLM), and comprehensive error handling.

## Features

- **High Performance**: Native Rust implementation via Neon
- **Async/Await Support**: Promise-based API for non-blocking evaluation
- **Tracing**: Built-in execution tracing for debugging and observability
- **Effect Handlers**: Support for HTTP requests, SQL queries, and LLM calls
- **Type Safety**: Full TypeScript type definitions
- **Error Handling**: Comprehensive error types with source location information

## Installation

```bash
npm install @dsl/interpreter
```

### Prerequisites

- Node.js >= 16.0.0
- Rust toolchain (for building from source)

## Quick Start

### Basic Usage

```javascript
const { Interpreter } = require('@dsl/interpreter');

async function main() {
  // Create an interpreter
  const interpreter = new Interpreter();

  // Define a simple IR node (adding two numbers)
  const node = {
    type: "BinaryOp",
    op: "Add",
    left: { type: "Literal", value: { type: "int", value: 10 } },
    right: { type: "Literal", value: { type: "int", value: 32 } }
  };

  // Evaluate the node
  const result = await interpreter.eval(node);
  console.log(result); // { type: 'int', value: 42 }
}

main();
```

### Using IR Definitions

```javascript
const { Interpreter } = require('@dsl/interpreter');

async function main() {
  // Define an IR with functions and an entry expression
  const ir = {
    version: "1.0",
    functions: {
      add: {
        params: ["a", "b"],
        body: {
          type: "BinaryOp",
          op: "Add",
          left: { type: "Variable", name: "a" },
          right: { type: "Variable", name: "b" }
        }
      }
    },
    entry_expr: {
      type: "FunctionCall",
      function: { type: "Variable", name: "add" },
      arguments: [
        { type: "Literal", value: { type: "int", value: 5 } },
        { type: "Literal", value: { type: "int", value: 7 } }
      ]
    }
  };

  // One-shot evaluation
  const result = await Interpreter.run(ir);
  console.log(result); // { type: 'int', value: 12 }
}

main();
```

### Tracing Execution

```javascript
const { TracingInterpreter } = require('@dsl/interpreter');

async function main() {
  // Create a tracing interpreter with configuration
  const config = {
    max_depth: 10,
    capture_values: true,
    capture_spans: true
  };

  const interpreter = new TracingInterpreter(config);

  const node = {
    type: "BinaryOp",
    op: "Multiply",
    left: { type: "Literal", value: { type: "int", value: 6 } },
    right: { type: "Literal", value: { type: "int", value: 7 } }
  };

  // Evaluate with traces
  const { value, traces } = await interpreter.evalWithTrace(node);

  console.log('Result:', value);
  console.log('Trace events:', traces.length);

  traces.forEach(trace => {
    console.log(`${trace.event_type} at depth ${trace.depth}:`, trace.node_type);
  });
}

main();
```

## API Reference

### Interpreter

The main interpreter class for evaluating DSL IR nodes.

#### Constructor

```typescript
new Interpreter()
```

Creates a new interpreter instance.

#### Static Methods

##### `fromIR(ir: IR | string): Interpreter`

Creates an interpreter from an IR definition.

- **ir**: IR object or JSON string
- **Returns**: New Interpreter instance

##### `run(ir: IR | string, node?: Node | string): Promise<Value>`

Convenience method that creates an interpreter and evaluates a node in one call.

- **ir**: IR object or JSON string
- **node**: Optional node to evaluate (defaults to `entry_expr` from IR)
- **Returns**: Promise resolving to the evaluated value

#### Instance Methods

##### `eval(node: Node | string): Promise<Value>`

Evaluates an IR node asynchronously.

- **node**: IR node object or JSON string
- **Returns**: Promise resolving to the evaluated value
- **Throws**: `DSLInterpreterError` on evaluation failure

### TracingInterpreter

Interpreter with built-in execution tracing capabilities.

#### Constructor

```typescript
new TracingInterpreter(config?: TraceConfig)
```

Creates a new tracing interpreter with optional configuration.

- **config**: Optional trace configuration

#### Static Methods

##### `fromIR(ir: IR | string, config?: TraceConfig): TracingInterpreter`

Creates a tracing interpreter from an IR definition.

- **ir**: IR object or JSON string
- **config**: Optional trace configuration
- **Returns**: New TracingInterpreter instance

##### `run(ir: IR | string, node?: Node | string, config?: TraceConfig): Promise<{value: Value, traces: TraceEvent[]}>`

Convenience method for one-shot evaluation with tracing.

- **ir**: IR object or JSON string
- **node**: Optional node to evaluate (defaults to `entry_expr` from IR)
- **config**: Optional trace configuration
- **Returns**: Promise resolving to value and traces

#### Instance Methods

##### `eval(node: Node | string): Promise<Value>`

Evaluates a node (traces are collected but not returned).

- **node**: IR node object or JSON string
- **Returns**: Promise resolving to the evaluated value

##### `evalWithTrace(node: Node | string): Promise<{value: Value, traces: TraceEvent[]}>`

Evaluates a node and returns both the result and execution traces.

- **node**: IR node object or JSON string
- **Returns**: Promise resolving to object with `value` and `traces`

##### `clearTrace(): void`

Clears the internal trace buffer.

##### `getTrace(): TraceEvent[]`

Retrieves the current trace buffer.

- **Returns**: Array of trace events

## Type Definitions

### Value

Runtime values returned by the interpreter:

```typescript
type Value =
  | { type: "null" }
  | { type: "bool"; value: boolean }
  | { type: "int"; value: number }
  | { type: "float"; value: number }
  | { type: "string"; value: string }
  | { type: "array"; value: Value[] }
  | { type: "object"; value: Record<string, Value> }
  | { type: "function"; name: string; params: string[]; body: Node };
```

### TraceConfig

Configuration options for tracing:

```typescript
interface TraceConfig {
  max_depth?: number;           // Maximum trace depth (default: unlimited)
  capture_values?: boolean;     // Capture intermediate values (default: false)
  capture_spans?: boolean;      // Capture source locations (default: false)
  filter_effect_kinds?: EffectKind[];  // Filter by effect type
}
```

### TraceEvent

Individual trace event:

```typescript
interface TraceEvent {
  timestamp: number;
  depth: number;
  event_type: "Enter" | "Exit" | "Error" | "EffectStart" | "EffectEnd";
  node_type?: string;
  value?: Value;
  span?: Span;
  effect_kind?: "Pure" | "HTTP" | "SQL" | "LLM";
  metadata?: Record<string, unknown>;
}
```

## Error Handling

All errors thrown by the interpreter are instances of `DSLInterpreterError`, which extends the standard `Error` class and includes additional context.

### Error Types

- **LLMError**: Error during LLM API call
- **HTTPError**: Error during HTTP request
- **SQLError**: Error during SQL query
- **TypeError**: Type mismatch or validation error
- **RuntimeError**: General runtime error
- **UnknownVariable**: Reference to undefined variable
- **UnknownFunction**: Call to undefined function
- **UnknownIntrinsic**: Call to undefined intrinsic
- **InvalidArguments**: Invalid function arguments

### Error Properties

```typescript
class DSLInterpreterError extends Error {
  kind: string;              // Error kind
  details: InterpreterError; // Full error details with context
}
```

### Example Error Handling

```javascript
try {
  const result = await interpreter.eval(node);
} catch (error) {
  if (error instanceof DSLInterpreterError) {
    console.error(`${error.kind}: ${error.message}`);
    if (error.details.span) {
      const { file, line, column } = error.details.span;
      console.error(`  at ${file}:${line}:${column}`);
    }
  }
}
```

## Effect Handlers

The interpreter supports various side effects with proper error handling and tracing:

### HTTP Requests

```javascript
const node = {
  type: "Intrinsic",
  name: "http_get",
  arguments: [
    { type: "Literal", value: { type: "string", value: "https://api.example.com/data" } }
  ]
};
```

### SQL Queries

```javascript
const node = {
  type: "Intrinsic",
  name: "sql_query",
  arguments: [
    { type: "Literal", value: { type: "string", value: "SELECT * FROM users" } }
  ]
};
```

### LLM Calls

```javascript
const node = {
  type: "Intrinsic",
  name: "llm_complete",
  arguments: [
    { type: "Literal", value: { type: "string", value: "What is 2+2?" } }
  ]
};
```

## Building from Source

```bash
# Install dependencies
npm install

# Build the native module and TypeScript
npm run build

# Run tests
npm test
```

## Architecture

The binding follows a two-layer architecture:

1. **Rust Native Layer** (`native/src/lib.rs`): Neon-based native module that wraps the Rust interpreter
2. **TypeScript API Layer** (`src/index.ts`): High-level, ergonomic TypeScript API

### Design Decisions

- **JSON Serialization**: IR and values are serialized as JSON for simplicity and maintainability
- **Async Operations**: All evaluations use Node.js promises via Neon's async task system
- **Resource Management**: Interpreter instances are managed using `JsBox` with `Arc<Mutex<>>` for thread safety
- **Panic Safety**: All native functions use `catch_unwind` to prevent Rust panics from crashing Node.js
- **Tokio Runtime**: Global Tokio runtime handles async Rust operations (HTTP, SQL, LLM)

## Comparison with Elixir Binding

This Node.js binding mirrors the design of the Elixir/Rustler binding with platform-appropriate adaptations:

| Feature | Elixir (Rustler) | Node.js (Neon) |
|---------|------------------|----------------|
| API Style | Functional | Object-oriented |
| Async | Task/GenServer | Promise/async-await |
| Resources | ResourceArc | JsBox |
| Serialization | JSON (binary) | JSON (string) |
| Error Handling | {:ok, result} / {:error, reason} | Promise resolve/reject |
| Scheduling | DirtyIo scheduler | Thread pool + event loop |

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass (`npm test`)
2. TypeScript compiles without errors (`npm run build`)
3. Code follows existing style conventions

## License

MIT

## See Also

- [DSL IR Specification](../../../docs/)
- [Elixir Binding](../elixir/dsl_interpreter/)
- [Neon Documentation](https://neon-bindings.com/)
