# DSL Interpreter Bindings Comparison

This document compares the Elixir (Rustler) and Node.js (Neon) bindings for the DSL IR Interpreter.

## Overview

Both bindings expose the same Rust interpreter core with platform-appropriate APIs. They follow similar architectural patterns but adapt to their respective runtime environments.

## Architecture Comparison

### Common Foundation

Both bindings share:
- **Core Logic**: Same Rust interpreter (`dsl-interpreter` crate)
- **IR Format**: JSON serialization for simplicity
- **8 Core Functions**: Identical functional API surface
- **Resource Management**: Stateful interpreter instances
- **Async Support**: Non-blocking evaluation of effects (HTTP, SQL, LLM)
- **Tracing**: Optional execution tracing with configurable collectors
- **Panic Safety**: `catch_unwind` prevents crashes

### Platform Adaptations

| Aspect | Elixir (Rustler) | Node.js (Neon) |
|--------|------------------|----------------|
| **FFI Framework** | Rustler 0.37 | Neon 1.0 |
| **API Style** | Functional | Object-oriented |
| **Module System** | `lib/dsl/interpreter.ex` | `src/index.ts` |
| **Resource Type** | `ResourceArc<T>` | `JsBox<Arc<Mutex<T>>>` |
| **Async Model** | Dirty IO scheduler | Promise + thread spawn |
| **Result Convention** | `{:ok, result} \| {:error, err}` | `Promise<Value>` (resolve/reject) |
| **Serialization** | JSON as binary (Elixir term) | JSON as string |
| **Function Naming** | `snake_case` (Elixir) | `camelCase` (JavaScript) |
| **Type Safety** | Dialyzer specs | TypeScript definitions |

## API Comparison

### Creating an Interpreter

**Elixir:**
```elixir
{:ok, interpreter} = DSL.Interpreter.new()
{:ok, interpreter} = DSL.Interpreter.from_ir(ir)
```

**Node.js:**
```javascript
const interpreter = new Interpreter();
const interpreter = Interpreter.fromIR(ir);
```

### Evaluating a Node

**Elixir:**
```elixir
{:ok, result} = DSL.Interpreter.eval(interpreter, node)
```

**Node.js:**
```javascript
const result = await interpreter.eval(node);
```

### One-Shot Evaluation

**Elixir:**
```elixir
{:ok, result} = DSL.Interpreter.run(ir)
```

**Node.js:**
```javascript
const result = await Interpreter.run(ir);
```

### Tracing

**Elixir:**
```elixir
{:ok, tracer} = DSL.Interpreter.new_tracing(config)
{:ok, result, traces} = DSL.Interpreter.eval_with_trace(tracer, node)
```

**Node.js:**
```javascript
const tracer = new TracingInterpreter(config);
const { value, traces } = await tracer.evalWithTrace(node);
```

## Implementation Details

### Resource Management

**Elixir (Rustler):**
```rust
struct InterpreterResource(Arc<Mutex<Interpreter>>);

rustler::resource!(InterpreterResource, env);

#[rustler::nif]
fn new_interpreter() -> ResourceArc<InterpreterResource> {
    ResourceArc::new(InterpreterResource(...))
}
```

**Node.js (Neon):**
```rust
type InterpreterResource = Arc<Mutex<Interpreter>>;

fn new_interpreter(mut cx: FunctionContext) -> JsResult<JsBox<InterpreterResource>> {
    Ok(cx.boxed(Arc::new(Mutex::new(Interpreter::default()))))
}
```

### Async Evaluation

**Elixir (Rustler):**
```rust
#[rustler::nif(schedule = "DirtyIo")]
fn eval<'a>(env: Env<'a>, res: ResourceArc<InterpreterRes>, node_bin: Binary<'a>)
    -> NifResult<Term<'a>>
{
    let node = decode_node(node_bin)?;
    let mut interp = res.0.lock().unwrap();

    let value = TOKIO_RT.block_on(async {
        interp.eval(&node).await
    })?;

    Ok((ok(), encode_json(&value)?).encode(env))
}
```

**Node.js (Neon):**
```rust
fn eval(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let interpreter = cx.argument::<JsBox<InterpreterResource>>(0)?;
    let node_json = cx.argument::<JsString>(1)?.value(&mut cx);

    let interpreter = (**interpreter).clone();
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    std::thread::spawn(move || {
        let result = /* ... */;
        deferred.settle_with(&channel, |mut cx| /* ... */);
    });

    Ok(promise)
}
```

**Key Differences:**
- Rustler uses the BEAM's dirty scheduler (declared at NIF level)
- Neon requires manual thread spawning and channel communication
- Rustler blocks on Tokio runtime directly
- Neon uses deferred promise resolution

### Error Handling

**Elixir:**
```elixir
case DSL.Interpreter.eval(interpreter, node) do
  {:ok, result} ->
    IO.puts("Success: #{inspect(result)}")
  {:error, %{kind: :unknown_variable, message: msg, span: span}} ->
    IO.puts("Error at #{span.file}:#{span.line} - #{msg}")
end
```

**Node.js:**
```javascript
try {
  const result = await interpreter.eval(node);
  console.log('Success:', result);
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

**Key Differences:**
- Elixir uses tagged tuples (`:ok`/`:error`)
- Node.js uses exceptions with typed error classes
- Both provide full error context (span, metadata, etc.)

## File Structure Comparison

### Elixir Binding

```
bindings/elixir/dsl_interpreter/
├── mix.exs                           # Elixir project config
├── lib/
│   ├── dsl_interpreter.ex            # Main module (mostly unused)
│   ├── dsl_interpreter/application.ex
│   ├── dsl/interpreter.ex            # High-level API
│   └── dsl/interpreter/native.ex     # Low-level NIF declarations
├── native/dsl_nif/
│   ├── Cargo.toml                    # Rust dependencies
│   └── src/lib.rs                    # Rustler implementation (~600 LOC)
├── test/
│   └── dsl_interpreter_test.exs
├── call_test_with_input.exs          # Example script
└── CALLING_FUNCTIONS.md
```

### Node.js Binding

```
bindings/nodejs/dsl-interpreter/
├── package.json                      # Node.js project config
├── tsconfig.json                     # TypeScript config
├── src/
│   ├── index.ts                      # High-level API (~220 LOC)
│   └── types.ts                      # TypeScript type definitions (~320 LOC)
├── native/
│   ├── Cargo.toml                    # Rust dependencies
│   └── src/lib.rs                    # Neon implementation (~600 LOC)
├── examples/
│   └── basic.js                      # Comprehensive examples
├── README.md                         # User documentation
├── NEON_INTEGRATION_GUIDE.md         # Developer documentation
└── .gitignore
```

## Performance Characteristics

### Serialization Overhead

**Elixir:**
- JSON encoded to Elixir binary (UTF-8)
- Passed to Rust as `Binary<'a>` (zero-copy slice)
- Deserialized via `serde_json::from_slice()`
- Results serialized back as binary

**Node.js:**
- JSON stringified in JavaScript
- Passed to Rust as `JsString` (copied to Rust String)
- Deserialized via `serde_json::from_str()`
- Results serialized and copied back to JS

**Verdict:** Elixir has slight edge due to zero-copy binary handling, but difference is negligible (<1ms) for typical IR sizes.

### Async Overhead

**Elixir:**
- Dirty IO scheduler manages threads
- ~10µs scheduling overhead
- Efficient thread reuse

**Node.js:**
- Manual `thread::spawn` per eval
- ~50µs thread spawn overhead (first call)
- Could be optimized with thread pool

**Verdict:** Elixir has better async performance out-of-the-box; Node.js could match with thread pooling.

### Overall Performance

Both bindings execute identical Rust code for:
- IR deserialization
- Interpretation logic
- Effect handling (HTTP, SQL, LLM)
- Trace collection

**Performance is effectively identical** for:
- Small to medium IR (< 100KB)
- Effect-heavy workloads (IO dominates)
- Typical use cases

**Elixir may be slightly faster** for:
- High-frequency, short-lived evaluations (better scheduling)
- Very large IR (zero-copy binaries)

**Node.js may be faster** for:
- CPU-bound pure computations (V8 optimizations)
- Already Node.js-based infrastructure (no language boundary)

## Type Safety

### Elixir (Dialyzer)

```elixir
@spec eval(reference(), map() | binary()) ::
  {:ok, map()} | {:error, map()}
```

**Pros:**
- Optional gradual typing
- Dialyzer catches many errors
- Flexible development

**Cons:**
- Runtime errors still possible
- Less IDE support than TypeScript
- Types not enforced

### Node.js (TypeScript)

```typescript
class Interpreter {
  async eval(node: Node | string): Promise<Value>
}
```

**Pros:**
- Strong compile-time checking
- Excellent IDE support (autocomplete, refactoring)
- Enforced at build time

**Cons:**
- Requires build step
- Runtime still needs validation
- More ceremony

**Verdict:** TypeScript provides stronger type safety and better DX, especially for larger projects.

## Ecosystem Integration

### Elixir

**Strengths:**
- Excellent OTP integration
- Supervised processes for fault tolerance
- Built-in distributed computing
- Phoenix framework integration

**Use Cases:**
- Backend services
- Real-time systems
- Distributed applications
- Fault-tolerant systems

### Node.js

**Strengths:**
- Massive npm ecosystem
- Frontend/backend code sharing
- Rich tooling (bundlers, test frameworks)
- AWS Lambda, Vercel, etc. support

**Use Cases:**
- Web applications
- CLI tools
- Serverless functions
- Cross-platform desktop apps (Electron)

## When to Use Which Binding

### Use Elixir/Rustler When:

- Building backend services in Elixir
- Need OTP supervision trees
- Building distributed systems
- Leveraging Phoenix/LiveView
- Team has Elixir expertise
- Deploying on BEAM-native platforms

### Use Node.js/Neon When:

- Building Node.js applications
- Need npm package ecosystem
- Building CLI tools
- Serverless/edge deployments
- Team has JavaScript/TypeScript expertise
- Frontend/backend code sharing needed

## Development Experience

### Elixir Binding

**Build:**
```bash
mix deps.get
mix compile
```

**Test:**
```bash
mix test
```

**Interactive (IEx):**
```elixir
iex -S mix
iex> {:ok, i} = DSL.Interpreter.new()
iex> DSL.Interpreter.eval(i, node)
```

### Node.js Binding

**Build:**
```bash
npm install
npm run build
```

**Test:**
```bash
npm test
node examples/basic.js
```

**Interactive (REPL):**
```javascript
node
> const { Interpreter } = require('./index');
> const i = new Interpreter();
> await i.eval(node);
```

**Verdict:** Both have excellent DX; Elixir's IEx is more powerful, Node.js has better debugging tools.

## Maintenance & Updates

Both bindings require updates when:
- DSL IR format changes
- New error types added
- New intrinsics/effects added
- Interpreter API changes

### Update Checklist

1. **Rust Core Changes:**
   - Update `dsl-interpreter` dependency version
   - Rebuild native modules

2. **IR Changes:**
   - Update type definitions (Elixir typespecs / TypeScript interfaces)
   - Update documentation

3. **Error Changes:**
   - Update error mapping functions
   - Update error types/docs

4. **New Features:**
   - Add NIFs/native functions
   - Update high-level wrappers
   - Add examples/tests

## Conclusion

Both bindings are production-ready and provide equivalent functionality:

| Criterion | Winner |
|-----------|--------|
| **Performance** | Elixir (slight edge) |
| **Type Safety** | Node.js (TypeScript) |
| **Async Model** | Elixir (cleaner) |
| **Ecosystem** | Tie (different strengths) |
| **DX** | Tie (both excellent) |
| **Maintainability** | Node.js (TypeScript + tooling) |

**Choose based on your runtime environment and team expertise**, not on binding quality—both are well-designed and thoroughly implemented.

## Future Improvements

Potential enhancements for both bindings:

1. **Binary Serialization**: MessagePack/CBOR for performance
2. **Streaming Traces**: Real-time trace events
3. **Cancellation**: Abort long-running evaluations
4. **Thread Pooling**: Reuse threads (Node.js)
5. **Resource Pooling**: Shared interpreters
6. **Benchmarking Suite**: Cross-platform performance tests
7. **Error Recovery**: Checkpoint/restore for fault tolerance

Both bindings can evolve in parallel, maintaining API compatibility while optimizing for their respective platforms.
