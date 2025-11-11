use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use rustler::{Binary, Encoder, Env, Error as NifError, NifResult, ResourceArc, Term};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use tokio::runtime::Runtime as TokioRuntime;

// Re-export DSL interpreter types
use dsl_interpreter::{Interpreter, TracingInterpreter, InterpreterError};
use dsl_ir::{IR, IRNode, Value};

// Initialize Tokio runtime for async operations
lazy_static! {
    static ref TOKIO_RT: TokioRuntime =
        TokioRuntime::new().expect("Failed to create Tokio runtime");
}

rustler::atoms! {
    ok,
    error,
    // Error kinds
    llm_error,
    http_error,
    sql_error,
    type_error,
    runtime_error,
    unknown_variable,
    unknown_function,
    unknown_intrinsic,
    invalid_arguments,
    panic
}

// Resource types for holding interpreter instances
struct InterpreterResource(Arc<Mutex<Interpreter>>);
struct TracingInterpreterResource(Arc<Mutex<TracingInterpreter>>);

// Helper functions for JSON encoding/decoding
fn decode_json<'a, T: for<'de> Deserialize<'de>>(bin: Binary<'a>) -> Result<T, String> {
    serde_json::from_slice::<T>(bin.as_slice())
        .map_err(|e| format!("JSON decode error: {}", e))
}

fn encode_json<'a, T: Serialize>(env: Env<'a>, val: &T) -> Result<Binary<'a>, NifError> {
    let bytes = serde_json::to_vec(val)
        .map_err(|e| NifError::Term(Box::new(format!("JSON encode error: {}", e))))?;
    let mut owned = rustler::OwnedBinary::new(bytes.len()).unwrap();
    owned.as_mut_slice().copy_from_slice(&bytes);
    Ok(Binary::from_owned(owned, env))
}

// Convert InterpreterError to Elixir error map
fn map_error<'a>(env: Env<'a>, err: InterpreterError) -> Term<'a> {
    use rustler::types::map::map_new;

    let base_map = map_new(env);

    match err {
        InterpreterError::LLMError { message, function_name, source_span, prompt, response } => {
            let mut map = base_map
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), llm_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), message.encode(env))
                .unwrap();

            if let Some(fname) = function_name {
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "function_name").unwrap(), fname.encode(env)).unwrap();
            }
            if let Some(span) = source_span {
                let span_map = map_new(env)
                    .map_put(rustler::types::atom::Atom::from_str(env, "file").unwrap(), span.file.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "line").unwrap(), span.line.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "column").unwrap(), span.column.encode(env)).unwrap();
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "span").unwrap(), span_map).unwrap();
            }
            if let Some(p) = prompt {
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "prompt").unwrap(), p.encode(env)).unwrap();
            }
            if let Some(r) = response {
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "response").unwrap(), r.encode(env)).unwrap();
            }
            map.encode(env)
        }
        InterpreterError::HTTPError { message, function_name, source_span, method, url } => {
            let mut map = base_map
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), http_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), message.encode(env))
                .unwrap();

            if let Some(fname) = function_name {
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "function_name").unwrap(), fname.encode(env)).unwrap();
            }
            if let Some(span) = source_span {
                let span_map = map_new(env)
                    .map_put(rustler::types::atom::Atom::from_str(env, "file").unwrap(), span.file.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "line").unwrap(), span.line.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "column").unwrap(), span.column.encode(env)).unwrap();
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "span").unwrap(), span_map).unwrap();
            }
            if let Some(m) = method {
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "method").unwrap(), m.encode(env)).unwrap();
            }
            if let Some(u) = url {
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "url").unwrap(), u.encode(env)).unwrap();
            }
            map.encode(env)
        }
        InterpreterError::SQLError { message, function_name, source_span, query } => {
            let mut map = base_map
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), sql_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), message.encode(env))
                .unwrap();

            if let Some(fname) = function_name {
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "function_name").unwrap(), fname.encode(env)).unwrap();
            }
            if let Some(span) = source_span {
                let span_map = map_new(env)
                    .map_put(rustler::types::atom::Atom::from_str(env, "file").unwrap(), span.file.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "line").unwrap(), span.line.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "column").unwrap(), span.column.encode(env)).unwrap();
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "span").unwrap(), span_map).unwrap();
            }
            if let Some(q) = query {
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "query").unwrap(), q.encode(env)).unwrap();
            }
            map.encode(env)
        }
        InterpreterError::TypeError { message, expected, got, source_span } => {
            let mut map = base_map
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), type_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), message.encode(env))
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "expected").unwrap(), expected.encode(env))
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "got").unwrap(), got.encode(env))
                .unwrap();

            if let Some(span) = source_span {
                let span_map = map_new(env)
                    .map_put(rustler::types::atom::Atom::from_str(env, "file").unwrap(), span.file.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "line").unwrap(), span.line.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "column").unwrap(), span.column.encode(env)).unwrap();
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "span").unwrap(), span_map).unwrap();
            }
            map.encode(env)
        }
        InterpreterError::RuntimeError { message, source_span } => {
            let mut map = base_map
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), runtime_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), message.encode(env))
                .unwrap();

            if let Some(span) = source_span {
                let span_map = map_new(env)
                    .map_put(rustler::types::atom::Atom::from_str(env, "file").unwrap(), span.file.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "line").unwrap(), span.line.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "column").unwrap(), span.column.encode(env)).unwrap();
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "span").unwrap(), span_map).unwrap();
            }
            map.encode(env)
        }
        InterpreterError::UnknownVariable { name, source_span } => {
            let mut map = base_map
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), unknown_variable())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), format!("Unknown variable: {}", name).encode(env))
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "name").unwrap(), name.encode(env))
                .unwrap();

            if let Some(span) = source_span {
                let span_map = map_new(env)
                    .map_put(rustler::types::atom::Atom::from_str(env, "file").unwrap(), span.file.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "line").unwrap(), span.line.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "column").unwrap(), span.column.encode(env)).unwrap();
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "span").unwrap(), span_map).unwrap();
            }
            map.encode(env)
        }
        InterpreterError::UnknownFunction { name, source_span } => {
            let mut map = base_map
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), unknown_function())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), format!("Unknown function: {}", name).encode(env))
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "name").unwrap(), name.encode(env))
                .unwrap();

            if let Some(span) = source_span {
                let span_map = map_new(env)
                    .map_put(rustler::types::atom::Atom::from_str(env, "file").unwrap(), span.file.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "line").unwrap(), span.line.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "column").unwrap(), span.column.encode(env)).unwrap();
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "span").unwrap(), span_map).unwrap();
            }
            map.encode(env)
        }
        InterpreterError::UnknownIntrinsic { name, source_span } => {
            let mut map = base_map
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), unknown_intrinsic())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), format!("Unknown intrinsic: {}", name).encode(env))
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "name").unwrap(), name.encode(env))
                .unwrap();

            if let Some(span) = source_span {
                let span_map = map_new(env)
                    .map_put(rustler::types::atom::Atom::from_str(env, "file").unwrap(), span.file.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "line").unwrap(), span.line.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "column").unwrap(), span.column.encode(env)).unwrap();
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "span").unwrap(), span_map).unwrap();
            }
            map.encode(env)
        }
        InterpreterError::InvalidArguments { message, source_span } => {
            let mut map = base_map
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), invalid_arguments())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), message.encode(env))
                .unwrap();

            if let Some(span) = source_span {
                let span_map = map_new(env)
                    .map_put(rustler::types::atom::Atom::from_str(env, "file").unwrap(), span.file.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "line").unwrap(), span.line.encode(env)).unwrap()
                    .map_put(rustler::types::atom::Atom::from_str(env, "column").unwrap(), span.column.encode(env)).unwrap();
                map = map.map_put(rustler::types::atom::Atom::from_str(env, "span").unwrap(), span_map).unwrap();
            }
            map.encode(env)
        }
    }
}

// NIF: Create a new interpreter
#[rustler::nif]
fn new_interpreter<'a>(env: Env<'a>) -> NifResult<Term<'a>> {
    match catch_unwind(AssertUnwindSafe(|| {
        Interpreter::new()
    })) {
        Ok(Ok(interpreter)) => {
            let resource = ResourceArc::new(InterpreterResource(Arc::new(Mutex::new(interpreter))));
            Ok((ok(), resource).encode(env))
        }
        Ok(Err(e)) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), runtime_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), format!("{:?}", e).encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
        Err(_) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), panic())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), "Rust panic in new_interpreter".encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
    }
}

// NIF: Create interpreter from IR
#[rustler::nif]
fn interpreter_from_ir<'a>(env: Env<'a>, ir_bin: Binary<'a>) -> NifResult<Term<'a>> {
    match catch_unwind(AssertUnwindSafe(|| {
        let ir: IR = decode_json(ir_bin)?;
        Interpreter::from_ir(&ir)
            .map_err(|e| format!("{:?}", e))
    })) {
        Ok(Ok(interpreter)) => {
            let resource = ResourceArc::new(InterpreterResource(Arc::new(Mutex::new(interpreter))));
            Ok((ok(), resource).encode(env))
        }
        Ok(Err(msg)) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), runtime_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), msg.encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
        Err(_) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), panic())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), "Rust panic in interpreter_from_ir".encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
    }
}

// NIF: Evaluate an IR node
#[rustler::nif(schedule = "DirtyIo")]
fn eval<'a>(env: Env<'a>, interpreter_res: ResourceArc<InterpreterResource>, node_bin: Binary<'a>) -> NifResult<Term<'a>> {
    match catch_unwind(AssertUnwindSafe(|| {
        let node: IRNode = decode_json(node_bin)?;
        let mut interpreter = interpreter_res.0.lock().unwrap();

        TOKIO_RT.block_on(async {
            interpreter.eval(&node).await
        })
        .map_err(|e| format!("{:?}", e))
    })) {
        Ok(Ok(value)) => {
            match encode_json(env, &value) {
                Ok(value_json) => Ok((ok(), value_json).encode(env)),
                Err(_) => {
                    let err_map = rustler::types::map::map_new(env)
                        .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), runtime_error())
                        .unwrap()
                        .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), "Failed to encode result".encode(env))
                        .unwrap();
                    Ok((error(), err_map).encode(env))
                }
            }
        }
        Ok(Err(msg)) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), runtime_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), msg.encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
        Err(_) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), panic())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), "Rust panic in eval".encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
    }
}

// NIF: Create a new tracing interpreter
#[rustler::nif]
fn new_tracing_interpreter<'a>(env: Env<'a>, _config_bin: Binary<'a>) -> NifResult<Term<'a>> {
    match catch_unwind(AssertUnwindSafe(|| {
        TracingInterpreter::new()
    })) {
        Ok(Ok(interpreter)) => {
            let resource = ResourceArc::new(TracingInterpreterResource(Arc::new(Mutex::new(interpreter))));
            Ok((ok(), resource).encode(env))
        }
        Ok(Err(e)) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), runtime_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), format!("{:?}", e).encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
        Err(_) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), panic())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), "Rust panic in new_tracing_interpreter".encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
    }
}

// NIF: Create tracing interpreter from IR
#[rustler::nif]
fn tracing_interpreter_from_ir<'a>(env: Env<'a>, ir_bin: Binary<'a>, _config_bin: Binary<'a>) -> NifResult<Term<'a>> {
    match catch_unwind(AssertUnwindSafe(|| {
        let ir: IR = decode_json(ir_bin)?;
        TracingInterpreter::from_ir(&ir).map_err(|e| format!("{:?}", e))
    })) {
        Ok(Ok(interpreter)) => {
            let resource = ResourceArc::new(TracingInterpreterResource(Arc::new(Mutex::new(interpreter))));
            Ok((ok(), resource).encode(env))
        }
        Ok(Err(msg)) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), runtime_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), msg.encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
        Err(_) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), panic())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), "Rust panic in tracing_interpreter_from_ir".encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
    }
}

// NIF: Evaluate with tracing
#[rustler::nif(schedule = "DirtyIo")]
fn eval_with_trace<'a>(env: Env<'a>, interpreter_res: ResourceArc<TracingInterpreterResource>, node_bin: Binary<'a>) -> NifResult<Term<'a>> {
    match catch_unwind(AssertUnwindSafe(|| {
        let node: IRNode = decode_json(node_bin)?;
        let mut interpreter = interpreter_res.0.lock().unwrap();

        let result = TOKIO_RT.block_on(async {
            interpreter.eval(&node).await
        });

        match result {
            Ok(value) => {
                let trace_json = interpreter.trace.to_json()
                    .map_err(|e| format!("Failed to serialize trace: {}", e))?;
                Ok((value, trace_json))
            }
            Err(e) => Err(format!("{:?}", e))
        }
    })) {
        Ok(Ok((value, trace_json))) => {
            match encode_json(env, &value) {
                Ok(value_json) => {
                    let trace_bytes = trace_json.into_bytes();
                    let mut trace_bin = rustler::OwnedBinary::new(trace_bytes.len()).unwrap();
                    trace_bin.as_mut_slice().copy_from_slice(&trace_bytes);
                    let trace_binary = Binary::from_owned(trace_bin, env);
                    Ok((ok(), value_json, trace_binary).encode(env))
                }
                Err(_) => {
                    let err_map = rustler::types::map::map_new(env)
                        .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), runtime_error())
                        .unwrap()
                        .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), "Failed to encode result".encode(env))
                        .unwrap();
                    Ok((error(), err_map).encode(env))
                }
            }
        }
        Ok(Err(msg)) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), runtime_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), msg.encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
        Err(_) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), panic())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), "Rust panic in eval_with_trace".encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
    }
}

// NIF: Clear trace
#[rustler::nif]
fn clear_trace<'a>(env: Env<'a>, interpreter_res: ResourceArc<TracingInterpreterResource>) -> NifResult<Term<'a>> {
    match catch_unwind(AssertUnwindSafe(|| {
        let mut interpreter = interpreter_res.0.lock().unwrap();
        interpreter.clear_trace();
    })) {
        Ok(()) => Ok(ok().encode(env)),
        Err(_) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), panic())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), "Rust panic in clear_trace".encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
    }
}

// NIF: Get trace
#[rustler::nif]
fn get_trace<'a>(env: Env<'a>, interpreter_res: ResourceArc<TracingInterpreterResource>) -> NifResult<Term<'a>> {
    match catch_unwind(AssertUnwindSafe(|| {
        let interpreter = interpreter_res.0.lock().unwrap();
        interpreter.trace.to_json()
            .map_err(|e| format!("Failed to serialize trace: {}", e))
    })) {
        Ok(Ok(trace_json)) => {
            let trace_bytes = trace_json.into_bytes();
            let mut trace_bin = rustler::OwnedBinary::new(trace_bytes.len()).unwrap();
            trace_bin.as_mut_slice().copy_from_slice(&trace_bytes);
            let trace_binary = Binary::from_owned(trace_bin, env);
            Ok((ok(), trace_binary).encode(env))
        }
        Ok(Err(msg)) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), runtime_error())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), msg.encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
        Err(_) => {
            let err_map = rustler::types::map::map_new(env)
                .map_put(rustler::types::atom::Atom::from_str(env, "kind").unwrap(), panic())
                .unwrap()
                .map_put(rustler::types::atom::Atom::from_str(env, "message").unwrap(), "Rust panic in get_trace".encode(env))
                .unwrap();
            Ok((error(), err_map).encode(env))
        }
    }
}

rustler::init!(
    "Elixir.DSL.Interpreter.Native",
    [
        new_interpreter,
        interpreter_from_ir,
        eval,
        new_tracing_interpreter,
        tracing_interpreter_from_ir,
        eval_with_trace,
        clear_trace,
        get_trace
    ],
    load = on_load
);

fn on_load<'a>(env: Env<'a>, _info: Term<'a>) -> bool {
    rustler::resource!(InterpreterResource, env);
    rustler::resource!(TracingInterpreterResource, env);
    true
}
