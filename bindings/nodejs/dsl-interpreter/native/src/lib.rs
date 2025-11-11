use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use neon::prelude::*;

use lazy_static::lazy_static;
use tokio::runtime::Runtime as TokioRuntime;

use dsl_interpreter::{Interpreter, InterpreterError, TracingInterpreter};
use dsl_ir::{IR, IRNode};

// Global Tokio runtime for async operations
lazy_static! {
    static ref TOKIO_RT: TokioRuntime =
        TokioRuntime::new().expect("Failed to create Tokio runtime");
}

// Global registries for managing interpreter instances
lazy_static! {
    static ref INTERPRETERS: Mutex<HashMap<u32, Arc<Mutex<Interpreter>>>> =
        Mutex::new(HashMap::new());

    static ref TRACING_INTERPRETERS: Mutex<HashMap<u32, Arc<Mutex<TracingInterpreter>>>> =
        Mutex::new(HashMap::new());

    static ref NEXT_ID: Mutex<u32> = Mutex::new(1);
}

/// Helper: Get next unique ID
fn next_id() -> u32 {
    let mut id = NEXT_ID.lock().unwrap();
    let current = *id;
    *id += 1;
    current
}

/// Helper: Deserialize JSON string to IR
fn decode_ir(json_str: &str) -> Result<IR, String> {
    serde_json::from_str::<IR>(json_str).map_err(|e| format!("Failed to parse IR: {}", e))
}

/// Helper: Deserialize JSON to IR node
fn decode_node(json_str: &str) -> Result<IRNode, String> {
    serde_json::from_str::<IRNode>(json_str)
        .map_err(|e| format!("Failed to parse node: {}", e))
}

/// Helper: Serialize value to JSON string
fn encode_json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| format!("Failed to serialize to JSON: {}", e))
}

/// Helper: Map InterpreterError to JavaScript Error object
#[allow(dead_code)]
fn map_error<'a>(cx: &mut impl Context<'a>, err: InterpreterError) -> JsResult<'a, JsObject> {
    let obj = cx.empty_object();

    match err {
        InterpreterError::LLMError {
            message,
            function_name,
            source_span,
            prompt,
            response,
        } => {
            let kind = cx.string("LLMError");
            obj.set(cx, "kind", kind)?;

            let msg = cx.string(&message);
            obj.set(cx, "message", msg)?;

            if let Some(fname) = function_name {
                let func_name_str = cx.string(&fname);
                obj.set(cx, "functionName", func_name_str)?;
            }

            if let Some(span) = source_span {
                let span_obj = cx.empty_object();
                let file = cx.string(&span.file);
                span_obj.set(cx, "file", file)?;
                let line = cx.number(span.line as f64);
                span_obj.set(cx, "line", line)?;
                let column = cx.number(span.column as f64);
                span_obj.set(cx, "column", column)?;
                obj.set(cx, "span", span_obj)?;
            }

            if let Some(p) = prompt {
                let prompt_str = cx.string(&p);
                obj.set(cx, "prompt", prompt_str)?;
            }

            if let Some(r) = response {
                let response_str = cx.string(&r);
                obj.set(cx, "response", response_str)?;
            }
        }

        InterpreterError::HTTPError {
            message,
            function_name,
            source_span,
            method,
            url,
        } => {
            let kind = cx.string("HTTPError");
            obj.set(cx, "kind", kind)?;

            let msg = cx.string(&message);
            obj.set(cx, "message", msg)?;

            if let Some(fname) = function_name {
                let func_name_str = cx.string(&fname);
                obj.set(cx, "functionName", func_name_str)?;
            }

            if let Some(span) = source_span {
                let span_obj = cx.empty_object();
                let file = cx.string(&span.file);
                span_obj.set(cx, "file", file)?;
                let line = cx.number(span.line as f64);
                span_obj.set(cx, "line", line)?;
                let column = cx.number(span.column as f64);
                span_obj.set(cx, "column", column)?;
                obj.set(cx, "span", span_obj)?;
            }

            if let Some(m) = method {
                let method_str = cx.string(&m);
                obj.set(cx, "method", method_str)?;
            }

            if let Some(u) = url {
                let url_str = cx.string(&u);
                obj.set(cx, "url", url_str)?;
            }
        }

        InterpreterError::SQLError {
            message,
            function_name,
            source_span,
            query,
        } => {
            let kind = cx.string("SQLError");
            obj.set(cx, "kind", kind)?;

            let msg = cx.string(&message);
            obj.set(cx, "message", msg)?;

            if let Some(fname) = function_name {
                let func_name_str = cx.string(&fname);
                obj.set(cx, "functionName", func_name_str)?;
            }

            if let Some(span) = source_span {
                let span_obj = cx.empty_object();
                let file = cx.string(&span.file);
                span_obj.set(cx, "file", file)?;
                let line = cx.number(span.line as f64);
                span_obj.set(cx, "line", line)?;
                let column = cx.number(span.column as f64);
                span_obj.set(cx, "column", column)?;
                obj.set(cx, "span", span_obj)?;
            }

            if let Some(q) = query {
                let query_str = cx.string(&q);
                obj.set(cx, "query", query_str)?;
            }
        }

        InterpreterError::TypeError {
            message,
            expected,
            got,
            source_span,
        } => {
            let kind = cx.string("TypeError");
            obj.set(cx, "kind", kind)?;

            let msg = cx.string(&message);
            obj.set(cx, "message", msg)?;

            let expected_str = cx.string(&expected);
            obj.set(cx, "expected", expected_str)?;

            let got_str = cx.string(&got);
            obj.set(cx, "got", got_str)?;

            if let Some(span) = source_span {
                let span_obj = cx.empty_object();
                let file = cx.string(&span.file);
                span_obj.set(cx, "file", file)?;
                let line = cx.number(span.line as f64);
                span_obj.set(cx, "line", line)?;
                let column = cx.number(span.column as f64);
                span_obj.set(cx, "column", column)?;
                obj.set(cx, "span", span_obj)?;
            }
        }

        InterpreterError::RuntimeError {
            message,
            source_span,
        } => {
            let kind = cx.string("RuntimeError");
            obj.set(cx, "kind", kind)?;

            let msg = cx.string(&message);
            obj.set(cx, "message", msg)?;

            if let Some(span) = source_span {
                let span_obj = cx.empty_object();
                let file = cx.string(&span.file);
                span_obj.set(cx, "file", file)?;
                let line = cx.number(span.line as f64);
                span_obj.set(cx, "line", line)?;
                let column = cx.number(span.column as f64);
                span_obj.set(cx, "column", column)?;
                obj.set(cx, "span", span_obj)?;
            }
        }

        InterpreterError::UnknownVariable { name, source_span } => {
            let kind = cx.string("UnknownVariable");
            obj.set(cx, "kind", kind)?;

            let msg = cx.string(&format!("Unknown variable: {}", name));
            obj.set(cx, "message", msg)?;

            let var_name = cx.string(&name);
            obj.set(cx, "variableName", var_name)?;

            if let Some(span) = source_span {
                let span_obj = cx.empty_object();
                let file = cx.string(&span.file);
                span_obj.set(cx, "file", file)?;
                let line = cx.number(span.line as f64);
                span_obj.set(cx, "line", line)?;
                let column = cx.number(span.column as f64);
                span_obj.set(cx, "column", column)?;
                obj.set(cx, "span", span_obj)?;
            }
        }

        InterpreterError::UnknownFunction { name, source_span } => {
            let kind = cx.string("UnknownFunction");
            obj.set(cx, "kind", kind)?;

            let msg = cx.string(&format!("Unknown function: {}", name));
            obj.set(cx, "message", msg)?;

            let func_name = cx.string(&name);
            obj.set(cx, "functionName", func_name)?;

            if let Some(span) = source_span {
                let span_obj = cx.empty_object();
                let file = cx.string(&span.file);
                span_obj.set(cx, "file", file)?;
                let line = cx.number(span.line as f64);
                span_obj.set(cx, "line", line)?;
                let column = cx.number(span.column as f64);
                span_obj.set(cx, "column", column)?;
                obj.set(cx, "span", span_obj)?;
            }
        }

        InterpreterError::UnknownIntrinsic { name, source_span } => {
            let kind = cx.string("UnknownIntrinsic");
            obj.set(cx, "kind", kind)?;

            let msg = cx.string(&format!("Unknown intrinsic: {}", name));
            obj.set(cx, "message", msg)?;

            let intr_name = cx.string(&name);
            obj.set(cx, "intrinsicName", intr_name)?;

            if let Some(span) = source_span {
                let span_obj = cx.empty_object();
                let file = cx.string(&span.file);
                span_obj.set(cx, "file", file)?;
                let line = cx.number(span.line as f64);
                span_obj.set(cx, "line", line)?;
                let column = cx.number(span.column as f64);
                span_obj.set(cx, "column", column)?;
                obj.set(cx, "span", span_obj)?;
            }
        }

        InterpreterError::InvalidArguments {
            message,
            source_span,
        } => {
            let kind = cx.string("InvalidArguments");
            obj.set(cx, "kind", kind)?;

            let msg = cx.string(&message);
            obj.set(cx, "message", msg)?;

            if let Some(span) = source_span {
                let span_obj = cx.empty_object();
                let file = cx.string(&span.file);
                span_obj.set(cx, "file", file)?;
                let line = cx.number(span.line as f64);
                span_obj.set(cx, "line", line)?;
                let column = cx.number(span.column as f64);
                span_obj.set(cx, "column", column)?;
                obj.set(cx, "span", span_obj)?;
            }
        }
    }

    Ok(obj)
}

// ============================================================================
// NIF 1: new_interpreter() -> returns ID
// ============================================================================

fn new_interpreter(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<u32, String> {
        let interpreter = Interpreter::new()
            .map_err(|e| format!("Failed to create interpreter: {}", e))?;

        let id = next_id();
        INTERPRETERS.lock().unwrap().insert(id, Arc::new(Mutex::new(interpreter)));

        Ok(id)
    }));

    match result {
        Ok(Ok(id)) => Ok(cx.number(id as f64)),
        Ok(Err(msg)) => cx.throw_error(msg),
        Err(_) => cx.throw_error("Panic in new_interpreter"),
    }
}

// ============================================================================
// NIF 2: interpreter_from_ir(ir_json) -> returns ID
// ============================================================================

fn interpreter_from_ir(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let ir_json = cx.argument::<JsString>(0)?.value(&mut cx);

    let result = catch_unwind(AssertUnwindSafe(|| -> Result<u32, String> {
        let ir = decode_ir(&ir_json)?;
        let interpreter = Interpreter::from_ir(&ir)
            .map_err(|e| format!("Failed to create interpreter from IR: {}", e))?;

        let id = next_id();
        INTERPRETERS.lock().unwrap().insert(id, Arc::new(Mutex::new(interpreter)));

        Ok(id)
    }));

    match result {
        Ok(Ok(id)) => Ok(cx.number(id as f64)),
        Ok(Err(msg)) => cx.throw_error(msg),
        Err(_) => cx.throw_error("Panic in interpreter_from_ir"),
    }
}

// ============================================================================
// NIF 3: eval(interpreter_id, node_json) - Async
// ============================================================================

fn eval(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let id = cx.argument::<JsNumber>(0)?.value(&mut cx) as u32;
    let node_json = cx.argument::<JsString>(1)?.value(&mut cx);

    // Get interpreter from registry
    let interpreter = {
        let registry = INTERPRETERS.lock().unwrap();
        registry.get(&id)
            .ok_or_else(|| format!("Invalid interpreter ID: {}", id))
            .map(|interp| interp.clone())
    };

    let interpreter = match interpreter {
        Ok(interp) => interp,
        Err(msg) => return cx.throw_error(msg),
    };

    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    std::thread::spawn(move || {
        let result = catch_unwind(AssertUnwindSafe(|| -> Result<String, String> {
            let node = decode_node(&node_json)?;

            let mut interp = interpreter
                .lock()
                .map_err(|e| format!("Failed to lock interpreter: {}", e))?;

            let value = TOKIO_RT
                .block_on(async { interp.eval(&node).await })
                .map_err(|e| format!("Evaluation failed: {:?}", e))?;

            encode_json(&value)
        }));

        deferred.settle_with(&channel, move |mut cx| match result {
            Ok(Ok(json)) => {
                let result_str = cx.string(json);
                Ok(result_str)
            }
            Ok(Err(msg)) => cx.throw_error(msg),
            Err(_) => cx.throw_error("Panic in eval"),
        });
    });

    Ok(promise)
}

// ============================================================================
// NIF 4: new_tracing_interpreter(config_json) -> returns ID
// ============================================================================

fn new_tracing_interpreter(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // For now, ignore config and use default - TraceConfig doesn't implement Deserialize
    // You can add a custom config parser later if needed

    let result = catch_unwind(AssertUnwindSafe(|| -> Result<u32, String> {
        let tracing_interp = TracingInterpreter::new()
            .map_err(|e| format!("Failed to create tracing interpreter: {}", e))?;

        let id = next_id();
        TRACING_INTERPRETERS.lock().unwrap().insert(id, Arc::new(Mutex::new(tracing_interp)));

        Ok(id)
    }));

    match result {
        Ok(Ok(id)) => Ok(cx.number(id as f64)),
        Ok(Err(msg)) => cx.throw_error(msg),
        Err(_) => cx.throw_error("Panic in new_tracing_interpreter"),
    }
}

// ============================================================================
// NIF 5: tracing_interpreter_from_ir(ir_json, config_json) -> returns ID
// ============================================================================

fn tracing_interpreter_from_ir(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let ir_json = cx.argument::<JsString>(0)?.value(&mut cx);
    // Ignore config_json for now

    let result = catch_unwind(AssertUnwindSafe(|| -> Result<u32, String> {
        let ir = decode_ir(&ir_json)?;
        let tracing_interp = TracingInterpreter::from_ir(&ir)
            .map_err(|e| format!("Failed to create tracing interpreter from IR: {}", e))?;

        let id = next_id();
        TRACING_INTERPRETERS.lock().unwrap().insert(id, Arc::new(Mutex::new(tracing_interp)));

        Ok(id)
    }));

    match result {
        Ok(Ok(id)) => Ok(cx.number(id as f64)),
        Ok(Err(msg)) => cx.throw_error(msg),
        Err(_) => cx.throw_error("Panic in tracing_interpreter_from_ir"),
    }
}

// ============================================================================
// NIF 6: eval_with_trace(interpreter_id, node_json) - Async
// ============================================================================

fn eval_with_trace(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let id = cx.argument::<JsNumber>(0)?.value(&mut cx) as u32;
    let node_json = cx.argument::<JsString>(1)?.value(&mut cx);

    // Get interpreter from registry
    let interpreter = {
        let registry = TRACING_INTERPRETERS.lock().unwrap();
        registry.get(&id)
            .ok_or_else(|| format!("Invalid tracing interpreter ID: {}", id))
            .map(|interp| interp.clone())
    };

    let interpreter = match interpreter {
        Ok(interp) => interp,
        Err(msg) => return cx.throw_error(msg),
    };

    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    std::thread::spawn(move || {
        let result = catch_unwind(AssertUnwindSafe(|| -> Result<(String, String), String> {
            let node = decode_node(&node_json)?;

            let mut interp = interpreter
                .lock()
                .map_err(|e| format!("Failed to lock interpreter: {}", e))?;

            let value = TOKIO_RT
                .block_on(async { interp.eval(&node).await })
                .map_err(|e| format!("Evaluation failed: {:?}", e))?;

            // Get the traces from the collector
            let trace_collector = interp.get_trace();
            let traces = &trace_collector.events;

            let value_json = encode_json(&value)?;
            let traces_json = encode_json(&traces)?;

            Ok((value_json, traces_json))
        }));

        deferred.settle_with(&channel, move |mut cx| match result {
            Ok(Ok((value_json, traces_json))) => {
                let obj = cx.empty_object();

                let value_str = cx.string(value_json);
                obj.set(&mut cx, "value", value_str)?;

                let traces_str = cx.string(traces_json);
                obj.set(&mut cx, "traces", traces_str)?;

                Ok(obj)
            }
            Ok(Err(msg)) => cx.throw_error(msg),
            Err(_) => cx.throw_error("Panic in eval_with_trace"),
        });
    });

    Ok(promise)
}

// ============================================================================
// NIF 7: clear_trace(interpreter_id)
// ============================================================================

fn clear_trace(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = cx.argument::<JsNumber>(0)?.value(&mut cx) as u32;

    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let registry = TRACING_INTERPRETERS.lock().unwrap();
        let interpreter = registry.get(&id)
            .ok_or_else(|| format!("Invalid tracing interpreter ID: {}", id))?;

        let mut interp = interpreter
            .lock()
            .map_err(|e| format!("Failed to lock interpreter: {}", e))?;

        interp.clear_trace();
        Ok(())
    }));

    match result {
        Ok(Ok(())) => Ok(cx.undefined()),
        Ok(Err(msg)) => cx.throw_error(msg),
        Err(_) => cx.throw_error("Panic in clear_trace"),
    }
}

// ============================================================================
// NIF 8: get_trace(interpreter_id)
// ============================================================================

fn get_trace(mut cx: FunctionContext) -> JsResult<JsString> {
    let id = cx.argument::<JsNumber>(0)?.value(&mut cx) as u32;

    let result = catch_unwind(AssertUnwindSafe(|| -> Result<String, String> {
        let registry = TRACING_INTERPRETERS.lock().unwrap();
        let interpreter = registry.get(&id)
            .ok_or_else(|| format!("Invalid tracing interpreter ID: {}", id))?;

        let interp = interpreter
            .lock()
            .map_err(|e| format!("Failed to lock interpreter: {}", e))?;

        let trace_collector = interp.get_trace();
        let traces = &trace_collector.events;
        encode_json(&traces)
    }));

    match result {
        Ok(Ok(json)) => Ok(cx.string(json)),
        Ok(Err(msg)) => cx.throw_error(msg),
        Err(_) => cx.throw_error("Panic in get_trace"),
    }
}

// ============================================================================
// NIF 9: destroy_interpreter(interpreter_id) - Clean up resource
// ============================================================================

fn destroy_interpreter(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = cx.argument::<JsNumber>(0)?.value(&mut cx) as u32;

    INTERPRETERS.lock().unwrap().remove(&id);

    Ok(cx.undefined())
}

// ============================================================================
// NIF 10: destroy_tracing_interpreter(interpreter_id) - Clean up resource
// ============================================================================

fn destroy_tracing_interpreter(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let id = cx.argument::<JsNumber>(0)?.value(&mut cx) as u32;

    TRACING_INTERPRETERS.lock().unwrap().remove(&id);

    Ok(cx.undefined())
}

// ============================================================================
// Module initialization
// ============================================================================

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("newInterpreter", new_interpreter)?;
    cx.export_function("interpreterFromIr", interpreter_from_ir)?;
    cx.export_function("eval", eval)?;
    cx.export_function("newTracingInterpreter", new_tracing_interpreter)?;
    cx.export_function("tracingInterpreterFromIr", tracing_interpreter_from_ir)?;
    cx.export_function("evalWithTrace", eval_with_trace)?;
    cx.export_function("clearTrace", clear_trace)?;
    cx.export_function("getTrace", get_trace)?;
    cx.export_function("destroyInterpreter", destroy_interpreter)?;
    cx.export_function("destroyTracingInterpreter", destroy_tracing_interpreter)?;
    Ok(())
}
