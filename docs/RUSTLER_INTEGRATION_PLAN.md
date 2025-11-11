# Rustler Integration Plan: Exposing DSL IR Interpreter to Elixir

## TL;DR

Expose a small Rustler NIF surface around your existing Interpreter/Runtime: accept IR as JSON (binary/string), execute in Rust (dirty NIF), and return `{:ok, result, traces}` or `{:error, error_map}`. Start with JSON for serialization, Elixir Jason for decode; add MessagePack later if needed. 

**Scope**: M (1–3h) for minimal run/2; M-L (0.5–1d) if adding Runtime resources and tracing.

## Recommended Approach (Simple Path)

### Architecture

- **Elixir-facing API (thin)**: `DSL.Native.run/2` and `run_with_runtime/3`. Optional `DSL.Native.new_runtime/1` returns a persistent Rust Resource for reuse across calls.

- **Rust NIF crate**: Wrap your crate and expose:
  - `new_runtime(opts)` → resource
  - `run(ir_json_or_bin, opts)` → `{:ok, result_json, traces_json}` | `{:error, error_map}`
  - `run_with_runtime(runtime_res, ir_json_or_bin, opts)`

- **Serialization**: IR in as JSON (UTF-8 binary); result/traces out as JSON (UTF-8 binary). Elixir side uses `Jason.decode/1` if caller wants native terms.

- **Scheduling**: Mark `run*` NIFs as `DirtyIo` to avoid blocking the BEAM while running IO-heavy effects (HTTP/SQL/LLM).

- **Errors**: Convert your `InterpreterError` to an Elixir map with fields: kind, message, span, effect_kind, inner, etc. Return `{:error, map}`.

### Type Conversions

**Inbound:**
- Elixir: IR map or JSON string/binary.
- If map: encode with `Jason.encode!(ir)` and pass to NIF as binary for `serde_json` in Rust.

**Outbound:**
- `result`: JSON binary (`serde_json::to_vec`), Elixir optionally `Jason.decode!(result_json)`.
- `traces`: JSON binary of `Vec<TraceEvent>` if tracing enabled; else `""` or `nil`.

**Options** (Elixir map → Rust serde struct via JSON or decode Term):
- `collect_traces`: boolean (default false)
- `trace_config`: map with fields from TraceConfig (all optional)
- `format`: `:json` (default) — reserved for future `:msgpack`

### Error Handling

Map `InterpreterError` into:
- `message`: String
- `kind`: String | atom (e.g., "Runtime", "Type", "HTTP", "SQL", "LLM")
- `span`: `%{file, line, column}` | nil
- `effect_kind`: "HTTP" | "SQL" | "LLM" | "Pure" | nil
- `data`: map (optional, for extra context)

NIF returns `{:error, error_map}`. Only reserve crashing for truly unrecoverable cases (e.g., invalid binary encoding).

Use `catch_unwind` to shield BEAM from Rust panics; map to `{:error, %{kind: "panic", message: ...}}`.

### Project Structure

**Elixir app:**
```
mix.exs: {:rustler, "~> 0.30"}, {:jason, "~> 1.4"}
lib/dsl/native.ex: wrapper module
native/dsl_nif/
  Cargo.toml: depends on rustler, serde, serde_json, your Rust crate
  src/lib.rs: NIFs
```

**Rust crate (existing):**
- Add feature "nif" if you want to isolate any NIF-specific code paths.
- No changes required to IR types (they already derive Serialize/Deserialize).

## Minimal Rust NIF Skeleton

**File: `native/dsl_nif/src/lib.rs`**

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};
use rustler::{Binary, Encoder, Env, Error as NifError, NifResult, ResourceArc, Term};
use rustler::schedule::SchedulerFlags;
use serde::{Deserialize, Serialize};

// Re-export your crate
use your_crate::{Interpreter, Runtime, TraceConfig, TracingInterpreter, TraceCollector, InterpreterError};
use your_crate::ir::IR; // if re-exported as pub mod ir

rustler::atoms! { ok, error }

struct RuntimeRes(ResourceArc<Runtime>);

#[derive(Default, Deserialize)]
struct RunOpts {
    #[serde(default)]
    collect_traces: bool,
    #[serde(default)]
    trace_config: Option<TraceConfig>,
}

fn decode_ir<'a>(bin: Binary<'a>) -> Result<IR, String> {
    serde_json::from_slice::<IR>(bin.as_slice()).map_err(|e| e.to_string())
}

fn encode_json<'a, T: Serialize>(env: Env<'a>, val: &T) -> Result<Binary<'a>, NifError> {
    let bytes = serde_json::to_vec(val).map_err(|_| NifError::RaiseAtom("json_encode_failed"))?;
    Ok(Binary::from_owned(bytes, env))
}

fn map_error<'a>(env: Env<'a>, err: InterpreterError) -> Term<'a> {
    // Adjust based on your InterpreterError shape
    let map = rustler::types::map::map_new(env)
        .map_put("message", err.to_string())
        .unwrap_or_default();
    map.encode(env)
}

#[rustler::nif(schedule = "DirtyIo")]
fn new_runtime() -> NifResult<ResourceArc<Runtime>> {
    // Customize runtime initialization as needed
    Ok(ResourceArc::new(Runtime::default()))
}

#[rustler::nif(schedule = "DirtyIo")]
fn run<'a>(env: Env<'a>, ir_bin: Binary<'a>, opts_term: Term<'a>) -> NifResult<Term<'a>> {
    let opts: RunOpts = opts_term.decode().unwrap_or_default();

    let exec = || -> Result<Term<'a>, Term<'a>> {
        let ir = decode_ir(ir_bin).map_err(|m| (error(), m).encode(env))?;
        let mut runtime = Runtime::default();

        if opts.collect_traces {
            let trace_cfg = opts.trace_config.unwrap_or_default();
            let mut tracer = TracingInterpreter::new(Interpreter::default(), TraceCollector::new(trace_cfg));
            match tracer.execute(&ir, &mut runtime) {
                Ok((value, traces)) => {
                    let value_bin = encode_json(env, &value).map_err(|_| (error(), "json_encode_failed").encode(env))?;
                    let traces_bin = encode_json(env, &traces).map_err(|_| (error(), "json_encode_failed").encode(env))?;
                    Ok((ok(), value_bin, traces_bin).encode(env))
                }
                Err(e) => Err((error(), map_error(env, e)).encode(env)),
            }
        } else {
            let mut interp = Interpreter::default();
            match interp.execute(&ir, &mut runtime) {
                Ok(value) => {
                    let value_bin = encode_json(env, &value).map_err(|_| (error(), "json_encode_failed").encode(env))?;
                    Ok((ok(), value_bin, rustler::types::atom::nil().to_term(env)).encode(env))
                }
                Err(e) => Err((error(), map_error(env, e)).encode(env)),
            }
        }
    };

    match catch_unwind(AssertUnwindSafe(exec)) {
        Ok(Ok(term)) => Ok(term),
        Ok(Err(err_term)) => Ok(err_term),
        Err(_) => Ok((error(), "panic").encode(env)),
    }
}

#[rustler::nif(schedule = "DirtyIo")]
fn run_with_runtime<'a>(env: Env<'a>, runtime_res: ResourceArc<Runtime>, ir_bin: Binary<'a>, opts_term: Term<'a>) -> NifResult<Term<'a>> {
    let opts: RunOpts = opts_term.decode().unwrap_or_default();

    let exec = || -> Result<Term<'a>, Term<'a>> {
        let ir = decode_ir(ir_bin).map_err(|m| (error(), m).encode(env))?;
        // Borrow the runtime resource (clone ResourceArc, mut borrow inside if needed)
        let mut runtime = ResourceArc::clone(&runtime_res);

        if opts.collect_traces {
            let trace_cfg = opts.trace_config.unwrap_or_default();
            let mut tracer = TracingInterpreter::new(Interpreter::default(), TraceCollector::new(trace_cfg));
            match tracer.execute(&ir, &mut runtime) {
                Ok((value, traces)) => {
                    let value_bin = encode_json(env, &value).map_err(|_| (error(), "json_encode_failed").encode(env))?;
                    let traces_bin = encode_json(env, &traces).map_err(|_| (error(), "json_encode_failed").encode(env))?;
                    Ok((ok(), value_bin, traces_bin).encode(env))
                }
                Err(e) => Err((error(), map_error(env, e)).encode(env)),
            }
        } else {
            let mut interp = Interpreter::default();
            match interp.execute(&ir, &mut runtime) {
                Ok(value) => {
                    let value_bin = encode_json(env, &value).map_err(|_| (error(), "json_encode_failed").encode(env))?;
                    Ok((ok(), value_bin, rustler::types::atom::nil().to_term(env)).encode(env))
                }
                Err(e) => Err((error(), map_error(env, e)).encode(env)),
            }
        }
    };

    match catch_unwind(AssertUnwindSafe(exec)) {
        Ok(Ok(term)) => Ok(term),
        Ok(Err(err_term)) => Ok(err_term),
        Err(_) => Ok((error(), "panic").encode(env)),
    }
}

rustler::init!(
    "Elixir.DSL.Native",
    [new_runtime, run, run_with_runtime],
    load = on_load
);

fn on_load<'a>(_env: Env<'a>, _info: Term<'a>) -> bool {
    // Initialize logging or globals if needed
    true
}
```

## Elixir Wrapper

**File: `lib/dsl/native.ex`**

```elixir
defmodule DSL.Native do
  @moduledoc false
  use Rustler, otp_app: :dsl_native, crate: "dsl_nif"

  @on_load :load_nif
  def load_nif, do: :ok

  @spec new_runtime() :: reference()
  def new_runtime, do: :erlang.nif_error(:nif_not_loaded)

  @spec run(map() | binary(), keyword()) ::
          {:ok, binary(), binary() | nil} | {:error, map() | binary()}
  def run(ir, opts \\ []) do
    ir_bin =
      case ir do
        %{} -> Jason.encode!(ir)
        bin when is_binary(bin) -> bin
      end

    run_nif(ir_bin, opts)
    |> maybe_decode()
  end

  @spec run_with_runtime(reference(), map() | binary(), keyword()) ::
          {:ok, binary(), binary() | nil} | {:error, map() | binary()}
  def run_with_runtime(runtime, ir, opts \\ []) do
    ir_bin =
      case ir do
        %{} -> Jason.encode!(ir)
        bin when is_binary(bin) -> bin
      end

    run_with_runtime_nif(runtime, ir_bin, opts)
    |> maybe_decode()
  end

  defp run_nif(_ir_bin, _opts), do: :erlang.nif_error(:nif_not_loaded)
  defp run_with_runtime_nif(_rt, _ir_bin, _opts), do: :erlang.nif_error(:nif_not_loaded)

  defp maybe_decode({:ok, value_json, traces_json}) do
    {:ok, Jason.decode!(value_json), decode_opt(traces_json)}
  rescue
    _ -> {:ok, value_json, traces_json}
  end

  defp maybe_decode({:error, err}) when is_binary(err) do
    with {:ok, map} <- Jason.decode(err), do: {:error, map}, else: (_ -> {:error, err})
  end

  defp maybe_decode(other), do: other

  defp decode_opt(nil), do: nil
  defp decode_opt(""), do: nil
  defp decode_opt(json) when is_binary(json) do
    Jason.decode!(json)
  rescue
    _ -> json
  end
end
```

## Cargo Configuration

**File: `native/dsl_nif/Cargo.toml`**

```toml
[package]
name = "dsl_nif"
version = "0.1.0"
edition = "2021"

[lib]
name = "dsl_nif"
crate-type = ["cdylib"]

[dependencies]
rustler = { version = "0.30", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
your-crate-name = { path = "../../", features = [] }
```

## Mix Configuration

**File: `mix.exs` (excerpt)**

```elixir
defp deps do
  [
    {:rustler, "~> 0.30"},
    {:jason, "~> 1.4"}
  ]
end
```

## Implementation Steps

1. **Create Elixir project** (or add to existing)
   - `mix new dsl_native`
   - Add deps (rustler, jason)

2. **Initialize Rustler**
   - `mix rustler.new`
   - Choose crate name `dsl_nif`; set path to `native/dsl_nif`

3. **Wire Rust crate**
   - Add your-crate-name dependency in `native/dsl_nif/Cargo.toml` (path to your Rust library)
   - Implement `src/lib.rs` per skeleton, substituting actual Interpreter API calls (execute signature and tracing)
   - Ensure IR is publicly accessible (pub use or module path)

4. **Map errors**
   - Implement `map_error` to project your `InterpreterError` into a user-friendly map
   - Include span (file, line, column), effect_kind, and any domain-specific fields

5. **Scheduling**
   - Keep run functions annotated with `schedule = "DirtyIo"`

6. **Build/test**
   - `mix deps.get && mix compile`
   - Write ExUnit tests calling `DSL.Native.run/2` with a small IR JSON having only pure nodes (e.g., literal addition) and with effect_kind fields to validate trace collection off/on

7. **Add guardrails**
   - Input size limit: reject IR binaries > e.g. 16 MB with `{:error, %{kind: "input_too_large"}}`
   - Wrap NIF bodies with `catch_unwind` (as shown)

8. **Document API**
   - Document that `run/2` returns `{:ok, result_term, traces_term?}` if decoding succeeds; otherwise JSON binaries

## Rationale and Trade-offs

- **JSON as the FFI**: Your IR is already serde-serializable; using JSON minimizes code and moving parts. No need to maintain NIF type mirrors for the many enums/structs.

- **Dirty NIFs** keeps BEAM responsive while Rust performs IO/computation.

- **ResourceArc runtime** allows reuse and future config/stateful connectors (e.g., SQL pools).

- **Trade-offs**: JSON adds serialization overhead and copies. Acceptable for simplicity; switch to MessagePack or term encoding only if profiling shows need.

## Risks and Guardrails

- **Long/blocking operations in NIF**: Mitigate with DirtyIo and (if necessary) introduce timeouts in interpreter for HTTP/SQL calls.

- **Large IR/result payloads**: Enforce size limits, consider compression (gzip) later if needed.

- **Panics in Rust**: `catch_unwind` wrap and return `{:error, %{kind: "panic"}}`.

- **Non-deterministic side effects**: Surface effect_kind and spans in errors/traces; let callers opt into tracing for observability.

- **Versioning mismatches**: Include IR.version in request; validate it in Rust and return `{:error, %{kind: "version_mismatch"}}` if unsupported.

## When to Consider the Advanced Path

- High-throughput or latency-sensitive workloads where JSON serialization becomes a bottleneck
- Need for streaming traces/events back to BEAM during execution
- Reusing heavy shared state (e.g., HTTP client pools, SQL pools, LLM clients) across calls
- Typed Elixir experience without JSON decode/encode steps

## Optional Advanced Path (Brief)

- **Typed NIF mapping**: Use `serde_rustler` or custom NifMap/NifTaggedEnum derivations on thin wrapper types around IR to pass as native BEAM terms, avoiding JSON overhead.

- **MessagePack/binary**: Add `format: :msgpack`; Elixir passes raw binary and uses msgpack libs; Rust uses rmp-serde for zero-alloc-ish decode.

- **Streaming traces**: Take caller pid in opts; in Rust, send trace events via `env.send(&pid, {:trace, event})` as they occur; still return summary at end.

- **Async job model**: Move execution into a Rust threadpool; NIF returns a reference; progress and result delivered to BEAM with messages. Useful if you need cancellation/timeouts handled from Elixir.

## Notes for Implementation

- Replace the placeholder `Interpreter::default()/execute` signatures with your actual API (you export Interpreter, Runtime, TraceCollector, TraceConfig, TracingInterpreter).

- If Interpreter returns a domain Value type, ensure it derives Serialize. If not, convert into `serde_json::Value` before encoding.

- Ensure TraceEvent derives Serialize (lib.rs re-exports it).
