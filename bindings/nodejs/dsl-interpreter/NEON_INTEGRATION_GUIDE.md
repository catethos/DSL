# Neon Integration Guide: Exposing DSL IR Interpreter to Node.js

## TL;DR

Expose a clean Neon-based FFI around your existing Interpreter/Runtime: accept IR as JSON strings, execute in Rust (via async task system), and return results as JSON. Start with JSON for serialization; the TypeScript layer provides ergonomic APIs.

**Scope**: M (2–4h) for complete implementation including TypeScript wrapper, types, and examples.

## Architecture Overview

This Node.js binding mirrors the Elixir/Rustler binding but adapted for Node.js conventions:

### Three-Layer Design

1. **Rust Native Layer** (`native/src/lib.rs`)
   - Neon functions that wrap the DSL interpreter
   - JSON serialization/deserialization using `serde_json`
   - Resource management via `JsBox<Arc<Mutex<Interpreter>>>`
   - Panic safety with `catch_unwind`
   - Global Tokio runtime for async operations

2. **TypeScript API Layer** (`src/index.ts`)
   - High-level classes: `Interpreter` and `TracingInterpreter`
   - Automatic JSON encoding/decoding
   - Promise-based async API
   - Typed error classes

3. **Type Definitions Layer** (`src/types.ts`)
   - Complete TypeScript interfaces for IR, Values, Errors
   - Type-safe API surface

### API Surface (8 Native Functions)

**Basic Interpreter:**
- `newInterpreter()` → returns `JsBox<InterpreterResource>`
- `interpreterFromIr(ir_json)` → creates from IR JSON
- `eval(interpreter, node_json)` → async Promise<string (JSON)>

**Tracing Interpreter:**
- `newTracingInterpreter(config_json)` → returns `JsBox<TracingInterpreterResource>`
- `tracingInterpreterFromIr(ir_json, config_json)` → creates from IR with config
- `evalWithTrace(interpreter, node_json)` → async Promise<{value, traces}>
- `clearTrace(interpreter)` → void
- `getTrace(interpreter)` → string (JSON)

### Data Flow

```
JavaScript Object → JSON.stringify → String → Neon → serde_json → IR struct
                                                         ↓
                                                      eval()
                                                         ↓
Value struct → serde_json → String → Neon → JSON.parse → JavaScript Object
```

## Key Implementation Details

### 1. Resource Management

Unlike Rustler's `ResourceArc`, Neon uses `JsBox` with Arc/Mutex for thread safety:

```rust
type InterpreterResource = Arc<Mutex<Interpreter>>;
type TracingInterpreterResource = Arc<Mutex<TracingInterpreter>>;

// Create and box a resource
fn new_interpreter(mut cx: FunctionContext) -> JsResult<JsBox<InterpreterResource>> {
    let interpreter = Interpreter::default();
    Ok(cx.boxed(Arc::new(Mutex::new(interpreter))))
}

// Use a boxed resource
fn eval(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let interpreter = cx.argument::<JsBox<InterpreterResource>>(0)?;
    let interpreter = (**interpreter).clone(); // Clone the Arc
    // ... spawn async task
}
```

### 2. Async Operations (Promises)

Neon provides a promise-based async system via channels and deferreds:

```rust
fn eval(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let interpreter = cx.argument::<JsBox<InterpreterResource>>(0)?;
    let node_json = cx.argument::<JsString>(1)?.value(&mut cx);

    let interpreter = (**interpreter).clone();
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    std::thread::spawn(move || {
        let result = /* ... do work ... */;

        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(json) => Ok(cx.string(json)),
                Err(msg) => cx.throw_error(msg),
            }
        });
    });

    Ok(promise)
}
```

**Key Points:**
- Spawn a thread to avoid blocking the event loop
- Use `cx.channel()` and `cx.promise()` for deferred resolution
- Settle the promise with `deferred.settle_with()`
- All heavy work happens off the main thread

### 3. Tokio Runtime Integration

Since the DSL interpreter is async, we need a Tokio runtime:

```rust
use lazy_static::lazy_static;
use tokio::runtime::Runtime as TokioRuntime;

lazy_static! {
    static ref TOKIO_RT: TokioRuntime =
        TokioRuntime::new().expect("Failed to create Tokio runtime");
}

// In eval function:
let value = TOKIO_RT
    .block_on(async { interp.eval(&node).await })
    .map_err(|e| format!("Evaluation failed: {:?}", e))?;
```

### 4. Panic Safety

Wrap all operations with `catch_unwind` to prevent Rust panics from crashing Node.js:

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};

fn eval(mut cx: FunctionContext) -> JsResult<JsPromise> {
    // ... setup ...

    std::thread::spawn(move || {
        let result = catch_unwind(AssertUnwindSafe(|| -> Result<String, String> {
            // ... actual work ...
        }));

        deferred.settle_with(&channel, move |mut cx| {
            match result {
                Ok(Ok(json)) => Ok(cx.string(json)),
                Ok(Err(msg)) => cx.throw_error(msg),
                Err(_) => cx.throw_error("Panic in eval"),
            }
        });
    });

    Ok(promise)
}
```

### 5. Error Mapping

Convert Rust `InterpreterError` to JavaScript objects with full context:

```rust
fn map_error<'a>(cx: &mut impl Context<'a>, err: InterpreterError) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    match err {
        InterpreterError::LLMError { message, prompt, response, span } => {
            let kind = cx.string("LLMError");
            obj.set(cx, "kind", kind)?;

            let msg = cx.string(&message);
            obj.set(cx, "message", msg)?;

            // ... set optional fields ...
        }
        // ... other variants ...
    }

    Ok(obj)
}
```

On the TypeScript side:

```typescript
export class DSLInterpreterError extends Error {
  public readonly kind: string;
  public readonly details: InterpreterError;

  constructor(details: InterpreterError) {
    super(details.message);
    this.name = "DSLInterpreterError";
    this.kind = details.kind;
    this.details = details;
  }
}
```

### 6. TypeScript Wrapper Design

Provide ergonomic, idiomatic JavaScript/TypeScript API:

```typescript
export class Interpreter {
  private readonly handle: unknown;

  constructor() {
    this.handle = native.newInterpreter();
  }

  static fromIR(ir: IR | string): Interpreter {
    const irJSON = typeof ir === "string" ? ir : JSON.stringify(ir);
    const handle = native.interpreterFromIr(irJSON);
    const interpreter = Object.create(Interpreter.prototype);
    interpreter.handle = handle;
    return interpreter;
  }

  async eval(node: Node | string): Promise<Value> {
    const nodeJSON = typeof node === "string" ? node : JSON.stringify(node);
    const resultJSON = await native.eval(this.handle, nodeJSON);
    return JSON.parse(resultJSON);
  }

  static async run(ir: IR | string, node?: Node | string): Promise<Value> {
    const interpreter = Interpreter.fromIR(ir);
    if (!node) {
      const irObj = typeof ir === "string" ? JSON.parse(ir) : ir;
      node = irObj.entry_expr;
    }
    return interpreter.eval(node);
  }
}
```

**Design Principles:**
- Accept both objects and JSON strings for flexibility
- Provide static convenience methods (`run()`, `fromIR()`)
- Hide native handle from public API
- Return native JavaScript types (promises, objects, arrays)

## Build System Integration

### Cargo.toml Configuration

```toml
[package]
name = "dsl-interpreter-neon"
version = "0.1.0"
edition = "2021"

[lib]
name = "index"  # Must match the .node filename
crate-type = ["cdylib"]

[dependencies]
neon = { version = "1.0", default-features = false, features = ["napi-6"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
lazy_static = "1.4"
tokio = { version = "1.0", features = ["rt", "rt-multi-thread", "macros"] }

[dependencies.dsl-interpreter]
path = "../../../../crates/dsl-interpreter"

[profile.release]
lto = true
opt-level = 3
```

### package.json Configuration

```json
{
  "name": "@dsl/interpreter",
  "version": "0.1.0",
  "main": "index.js",
  "types": "index.d.ts",
  "scripts": {
    "build": "cargo-cp-artifact -nc index.node -- cargo build --message-format=json-render-diagnostics --release && tsc",
    "install": "cargo-cp-artifact -nc index.node -- cargo build --message-format=json-render-diagnostics --release"
  },
  "dependencies": {
    "cargo-cp-artifact": "^0.1"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "typescript": "^5.0.0"
  }
}
```

### Build Process

1. `npm install` → Builds native module via `cargo build --release`
2. `cargo-cp-artifact` → Copies `target/release/libindex.{so,dylib,dll}` to `index.node`
3. `tsc` → Compiles TypeScript to JavaScript with type declarations

## Comparison: Neon vs Rustler

| Aspect | Neon (Node.js) | Rustler (Elixir) |
|--------|----------------|------------------|
| **Resource Type** | `JsBox<Arc<Mutex<T>>>` | `ResourceArc<T>` |
| **Async Model** | Promises via channels/deferreds | Dirty schedulers |
| **Threading** | Manual `thread::spawn` + event loop | Dirty IO scheduler |
| **Serialization** | JSON strings | JSON binaries |
| **Error Handling** | Throw JS errors or reject promises | Return `{:error, reason}` tuples |
| **Function Export** | `cx.export_function(name, fn)` | `rustler::init!(...)` |
| **Module Init** | `#[neon::main]` | `rustler::init!` |
| **Panic Safety** | `catch_unwind` required | `catch_unwind` + DirtyCpuBound |

## Rationale & Trade-offs

### Why JSON?

**Pros:**
- IR already derives `Serialize`/`Deserialize`
- Simple, maintainable, language-agnostic
- No need to mirror complex Rust types in Neon
- Performance adequate for typical workloads

**Cons:**
- Serialization overhead (typically <1ms for small IR)
- Copy overhead (can be optimized later with binary formats)

**Alternative:** Binary formats (MessagePack, CBOR) can be added later if profiling shows need.

### Why Promises?

**Pros:**
- Idiomatic JavaScript async pattern
- Non-blocking evaluation
- Works with async/await
- Composable with Promise APIs

**Cons:**
- More complex implementation than sync
- Thread spawning overhead (mitigated by thread pool)

### Why JsBox + Arc + Mutex?

**Pros:**
- Safe sharing across threads
- Prevents data races
- Automatic memory management
- Familiar pattern from Rust ecosystem

**Cons:**
- Mutex overhead (typically negligible)
- No concurrent reads (could use RwLock if needed)

## Testing Strategy

### Unit Tests (Rust)

```bash
cd native
cargo test
```

### Integration Tests (JavaScript)

```javascript
// test/basic.test.js
const { Interpreter } = require('../');
const assert = require('assert');

describe('Interpreter', () => {
  it('evaluates simple arithmetic', async () => {
    const interpreter = new Interpreter();
    const node = {
      type: "BinaryOp",
      op: "Add",
      left: { type: "Literal", value: { type: "int", value: 2 } },
      right: { type: "Literal", value: { type: "int", value: 3 } }
    };
    const result = await interpreter.eval(node);
    assert.strictEqual(result.value, 5);
  });
});
```

### Example Scripts

Run comprehensive examples:

```bash
node examples/basic.js
```

## Performance Considerations

### Serialization Overhead

- JSON encoding/decoding: typically <1ms for IR nodes <100KB
- Acceptable for most use cases
- If profiling shows bottleneck, consider MessagePack or binary formats

### Thread Spawning

- Each `eval()` spawns a thread (via `std::thread::spawn`)
- Thread pool can be added if high-frequency calls are needed
- Trade-off: simplicity vs. performance

### Mutex Contention

- Single Mutex per interpreter instance
- No contention unless same instance used concurrently
- Consider `RwLock` if read-heavy workload

## Deployment & Distribution

### Pre-built Binaries

Consider using `neon-cli`'s cross-compilation or CI to build binaries for:
- Linux (x64, ARM64)
- macOS (Intel, Apple Silicon)
- Windows (x64)

### Source Distribution

Include Rust source in npm package; users build during `npm install`.

**Pros:**
- Works on any platform with Rust toolchain
- No binary hosting needed

**Cons:**
- Requires Rust on user machines
- Slower installation

## Migration from Elixir Patterns

If you're familiar with the Elixir/Rustler binding, here are key differences:

### Resource Creation

**Elixir/Rustler:**
```rust
fn new_interpreter(_env: Env, _args: &[Term]) -> NifResult<ResourceArc<InterpreterRes>> {
    Ok(ResourceArc::new(InterpreterRes(...)))
}
```

**Node.js/Neon:**
```rust
fn new_interpreter(mut cx: FunctionContext) -> JsResult<JsBox<InterpreterResource>> {
    Ok(cx.boxed(Arc::new(Mutex::new(Interpreter::default()))))
}
```

### Async Evaluation

**Elixir/Rustler:**
```rust
#[rustler::nif(schedule = "DirtyIo")]
fn eval(res: ResourceArc<InterpreterRes>, node_json: Binary) -> Result<Binary, String> {
    // Direct synchronous-looking code
    // Runtime handles scheduling
}
```

**Node.js/Neon:**
```rust
fn eval(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let (deferred, promise) = cx.promise();
    std::thread::spawn(move || {
        // Work happens here
        deferred.settle_with(/* result */);
    });
    Ok(promise)
}
```

### Error Handling

**Elixir/Rustler:**
```rust
Ok((ok(), value).encode(env))
Err((error(), error_map).encode(env))
```

**Node.js/Neon:**
```rust
Ok(cx.string(json))  // Success
cx.throw_error(msg)  // Error
```

## Future Enhancements

### 1. Binary Serialization

Replace JSON with MessagePack or CBOR for performance:

```rust
use rmp_serde;

fn decode_ir(bytes: &[u8]) -> Result<IR, String> {
    rmp_serde::from_slice(bytes).map_err(|e| e.to_string())
}
```

### 2. Streaming Traces

Send trace events incrementally via Node.js EventEmitter:

```typescript
class TracingInterpreter extends EventEmitter {
  async eval(node: Node): Promise<Value> {
    // Native code emits events: this.emit('trace', event)
  }
}
```

### 3. Worker Thread Pool

Reuse threads for better performance:

```rust
use rayon::ThreadPoolBuilder;

lazy_static! {
    static ref THREAD_POOL: rayon::ThreadPool =
        ThreadPoolBuilder::new().num_threads(4).build().unwrap();
}
```

### 4. Cancellation Support

Allow canceling long-running evaluations:

```typescript
class Interpreter {
  async eval(node: Node, signal?: AbortSignal): Promise<Value> {
    // Native code checks abort signal periodically
  }
}
```

## Common Issues & Solutions

### Issue: "Cannot find module 'index.node'"

**Solution:** Run `npm run build` to compile native module.

### Issue: Rust panic crashes Node.js

**Solution:** Ensure all native functions use `catch_unwind`.

### Issue: "Failed to lock interpreter"

**Solution:** Avoid concurrent `eval()` calls on same interpreter instance, or use `RwLock` for read-heavy workloads.

### Issue: Slow JSON parsing

**Solution:** Profile with real workload; if bottleneck, switch to MessagePack.

### Issue: TypeScript types don't match runtime

**Solution:** Regenerate types after changing IR structures; consider codegen.

## Resources

- [Neon Documentation](https://neon-bindings.com/)
- [Neon Examples](https://github.com/neon-bindings/examples)
- [N-API Documentation](https://nodejs.org/api/n-api.html)
- [Tokio Documentation](https://tokio.rs/)

## Summary

This Neon binding provides a clean, idiomatic Node.js interface to the DSL interpreter:

✅ **8 native functions** exposing interpreter and tracing
✅ **JSON serialization** for simplicity and maintainability
✅ **Promise-based async** for non-blocking evaluation
✅ **Comprehensive error handling** with panic safety
✅ **TypeScript types** for IDE support and type safety
✅ **Resource management** via JsBox + Arc + Mutex
✅ **Tokio runtime** for async Rust operations

The implementation closely mirrors the Elixir/Rustler binding while following Node.js and JavaScript conventions.
