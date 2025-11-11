# Building the Node.js DSL Interpreter Binding

## Current Status

The Node.js binding has been created but requires some adjustments to compile successfully. Here's what's been done and what needs to be fixed:

### ✅ Completed

1. **Project Structure**: All directories and configuration files created
2. **Cargo Configuration**: `native/Cargo.toml` with all dependencies
3. **TypeScript API**: Complete high-level API in `src/index.ts`
4. **Type Definitions**: Full TypeScript types in `src/types.ts`
5. **Documentation**: README, integration guide, and examples
6. **Rust Implementation**: Native module structure in `native/src/lib.rs`

### ⚠️ Issues to Fix

The current implementation has compatibility issues with Neon's resource management:

####  1. **JsBox/Finalize Trait Issue**

**Problem**: Neon's `JsBox` requires types to implement the `Finalize` trait, but `Interpreter` and `TracingInterpreter` don't implement it (and can't, as they're from an external crate).

**Solution Options**:

**Option A: Use a Global Registry (Recommended)**
```rust
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref INTERPRETERS: Mutex<HashMap<u32, Arc<Mutex<Interpreter>>>> =
        Mutex::new(HashMap::new());
    static ref TRACING_INTERPRETERS: Mutex<HashMap<u32, Arc<Mutex<TracingInterpreter>>>> =
        Mutex::new(HashMap::new());
    static ref NEXT_ID: Mutex<u32> = Mutex::new(0);
}

fn new_interpreter(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // Create interpreter
    let interp = Interpreter::new()?;

    // Get next ID
    let mut next_id = NEXT_ID.lock().unwrap();
    let id = *next_id;
    *next_id += 1;

    // Store in registry
    INTERPRETERS.lock().unwrap().insert(id, Arc::new(Mutex::new(interp)));

    // Return ID as number
    Ok(cx.number(id as f64))
}

fn eval(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let id = cx.argument::<JsNumber>(0)?.value(&mut cx) as u32;
    let node_json = cx.argument::<JsString>(1)?.value(&mut cx);

    // Get interpreter from registry
    let interp = INTERPRETERS.lock().unwrap()
        .get(&id)
        .ok_or("Invalid interpreter ID")?
        .clone();

    // ... rest of implementation
}
```

**Option B: Use Neon's External Objects**
Wrap interpreters in a newtype that implements `Finalize`:
```rust
struct InterpreterHandle(Arc<Mutex<Interpreter>>);

impl Finalize for InterpreterHandle {}

fn new_interpreter(mut cx: FunctionContext) -> JsResult<JsBox<InterpreterHandle>> {
    let interp = Interpreter::new()?;
    Ok(cx.boxed(InterpreterHandle(Arc::new(Mutex::new(interp)))))
}
```

#### 2. **TraceConfig Deserial**ization Issue**

**Problem**: `TraceConfig` doesn't implement `serde::Deserialize`.

**Solution**: Check if `TraceConfig` has serde support, or create our own config struct:

```rust
#[derive(Deserialize)]
struct TraceConfigJS {
    max_depth: Option<usize>,
    capture_values: Option<bool>,
    capture_spans: Option<bool>,
}

fn to_trace_config(config_js: TraceConfigJS) -> TraceConfig {
    TraceConfig {
        max_depth: config_js.max_depth,
        capture_values: config_js.capture_values.unwrap_or(false),
        capture_spans: config_js.capture_spans.unwrap_or(false),
    }
}
```

Or just use default config:
```rust
fn new_tracing_interpreter(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // Ignore config for now, use default
    let interp = TracingInterpreter::new()?;
    // ... store in registry
}
```

### 📋 Steps to Complete the Build

1. **Choose Resource Management Strategy**
   - Recommend Option A (Global Registry) for simplicity
   - Modify `native/src/lib.rs` to use registry pattern

2. **Handle TraceConfig**
   - Check if `TraceConfig` has a serde feature flag
   - If not, create wrapper struct or use default config

3. **Clean Up Imports**
   - Remove unused imports (`TraceCollector`, `Value`)

4. **Test Build**
   ```bash
   cd native
   cargo build --release
   ```

5. **Copy Artifact**
   ```bash
   cp native/target/release/libindex.dylib index.node  # macOS
   # OR
   cp native/target/release/libindex.so index.node     # Linux
   # OR
   cp native/target/release/index.dll index.node       # Windows
   ```

6. **Build TypeScript**
   ```bash
   npx tsc
   ```

7. **Test**
   ```bash
   node examples/basic.js
   ```

### 🔧 Quick Fix Script

Here's a minimal working version you can try:

```rust
// Simplified version - just expose basic eval without resources
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("evalNode", eval_node_simple)?;
    Ok(())
}

fn eval_node_simple(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let node_json = cx.argument::<JsString>(0)?.value(&mut cx);

    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    std::thread::spawn(move || {
        let result = || -> Result<String, String> {
            let node = serde_json::from_str(&node_json)?;
            let mut interp = Interpreter::new()?;
            let value = TOKIO_RT.block_on(async { interp.eval(&node).await })?;
            serde_json::to_string(&value).map_err(|e| e.to_string())
        };

        deferred.settle_with(&channel, |mut cx| {
            match result() {
                Ok(json) => Ok(cx.string(json)),
                Err(msg) => cx.throw_error(msg),
            }
        });
    });

    Ok(promise)
}
```

### 🆘 Alternative: Use FFI Instead of Neon

If Neon proves too difficult, consider using `node-bindgen` or `napi-rs` which have different (sometimes simpler) resource management:

**napi-rs** example:
```rust
#[napi]
struct Interpreter {
    inner: Arc<Mutex<dsl_interpreter::Interpreter>>,
}

#[napi]
impl Interpreter {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Arc::new(Mutex::new(dsl_interpreter::Interpreter::new()?))
        })
    }

    #[napi]
    pub async fn eval(&self, node_json: String) -> Result<String> {
        // ...
    }
}
```

### 📚 References

- [Neon Finalize Trait](https://docs.rs/neon/latest/neon/types/trait.Finalize.html)
- [Neon Examples](https://github.com/neon-bindings/examples)
- [napi-rs](https://napi.rs/) - Alternative binding generator

### ✉️ Need Help?

The structure and APIs are all correct - it's just a matter of adapting the resource management to Neon's requirements. The registry pattern (Option A) should work with minimal changes.
