//! Runtime wrapper for Neon bindings

use neon::prelude::*;
use neon::types::Finalize;
use std::sync::Mutex;

use lattice::runtime::{LatticeRuntime, LatticeValue, RuntimeBuilder};

use crate::convert::{js_to_lattice_value, lattice_value_to_js};
use crate::error::to_neon_error;
use crate::schema::{function_signature_to_js, type_schema_to_js};

/// Boxed runtime wrapper - Neon's pattern for storing Rust data in JS objects
pub struct RuntimeBox(Mutex<LatticeRuntime>);

impl Finalize for RuntimeBox {}

impl RuntimeBox {
    fn with_runtime<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut LatticeRuntime) -> Result<R, String>,
    {
        let mut guard = self.0.lock().map_err(|_| "Runtime lock poisoned".to_string())?;
        f(&mut guard)
    }
}

/// Create a new Lattice runtime
/// JS: createRuntime(options?: { llm?: boolean, sql?: boolean }): RuntimeHandle
pub fn create_runtime(mut cx: FunctionContext) -> JsResult<JsBox<RuntimeBox>> {
    // Parse options object if provided
    let (llm, sql) = if let Some(arg) = cx.argument_opt(0) {
        if let Ok(options) = arg.downcast::<JsObject, _>(&mut cx) {
            let llm = options
                .get_opt::<JsBoolean, _, _>(&mut cx, "llm")?
                .map(|v| v.value(&mut cx))
                .unwrap_or(false);
            let sql = options
                .get_opt::<JsBoolean, _, _>(&mut cx, "sql")?
                .map(|v| v.value(&mut cx))
                .unwrap_or(false);
            (llm, sql)
        } else {
            (false, false)
        }
    } else {
        (false, false)
    };

    // Build runtime (matching PyO3 pattern)
    let mut builder = RuntimeBuilder::new();

    if llm {
        builder = builder
            .with_default_llm_provider()
            .map_err(|e| to_neon_error(&mut cx, e.to_string()))?;
    } else {
        builder = builder.without_llm();
    }

    #[cfg(feature = "sql")]
    if sql {
        builder = builder
            .with_default_sql_provider()
            .map_err(|e| to_neon_error(&mut cx, e.to_string()))?;
    } else {
        builder = builder.without_sql();
    }

    #[cfg(not(feature = "sql"))]
    if sql {
        return cx.throw_error("SQL support not compiled in. Rebuild with 'sql' feature.");
    } else {
        builder = builder.without_sql();
    }

    let built = builder
        .build()
        .map_err(|e| to_neon_error(&mut cx, e.to_string()))?;
    let runtime = LatticeRuntime::from_built(built);

    Ok(cx.boxed(RuntimeBox(Mutex::new(runtime))))
}

/// Evaluate Lattice source code
/// JS: eval(runtime: RuntimeHandle, source: string, bindings?: object): any
pub fn eval(mut cx: FunctionContext) -> JsResult<JsValue> {
    let runtime = cx.argument::<JsBox<RuntimeBox>>(0)?;
    let source = cx.argument::<JsString>(1)?.value(&mut cx);

    // Check for optional bindings
    let bindings: Option<Vec<(String, LatticeValue)>> = if let Some(arg) = cx.argument_opt(2) {
        if let Ok(obj) = arg.downcast::<JsObject, _>(&mut cx) {
            let names = obj.get_own_property_names(&mut cx)?;
            let len = names.len(&mut cx);
            let mut result = Vec::with_capacity(len as usize);

            for i in 0..len {
                let key: Handle<JsString> = names.get(&mut cx, i)?;
                let key_str = key.value(&mut cx);
                let value_js: Handle<JsValue> = obj.get(&mut cx, key_str.as_str())?;
                let value = js_to_lattice_value(&mut cx, value_js)?;
                result.push((key_str, value));
            }
            Some(result)
        } else {
            None
        }
    } else {
        None
    };

    let result = runtime.with_runtime(|rt| {
        if let Some(bindings) = bindings {
            rt.eval_with_bindings(&source, bindings)
        } else {
            rt.eval(&source)
        }
        .map_err(|e| e.to_string())
    });

    match result {
        Ok(value) => lattice_value_to_js(&mut cx, &value),
        Err(e) => cx.throw_error(e),
    }
}

/// Evaluate a Lattice file
/// JS: evalFile(runtime: RuntimeHandle, path: string): any
pub fn eval_file(mut cx: FunctionContext) -> JsResult<JsValue> {
    let runtime = cx.argument::<JsBox<RuntimeBox>>(0)?;
    let path = cx.argument::<JsString>(1)?.value(&mut cx);

    let result = runtime.with_runtime(|rt| {
        rt.eval_file(std::path::Path::new(&path))
            .map_err(|e| e.to_string())
    });

    match result {
        Ok(value) => lattice_value_to_js(&mut cx, &value),
        Err(e) => cx.throw_error(e),
    }
}

/// Call a Lattice function by name
/// JS: call(runtime: RuntimeHandle, name: string, args: any[]): any
pub fn call(mut cx: FunctionContext) -> JsResult<JsValue> {
    let runtime = cx.argument::<JsBox<RuntimeBox>>(0)?;
    let name = cx.argument::<JsString>(1)?.value(&mut cx);
    let args_array = cx.argument::<JsArray>(2)?;

    // Convert JS args to LatticeValues BEFORE acquiring lock
    let len = args_array.len(&mut cx);
    let mut lattice_args = Vec::with_capacity(len as usize);
    for i in 0..len {
        let arg: Handle<JsValue> = args_array.get(&mut cx, i)?;
        lattice_args.push(js_to_lattice_value(&mut cx, arg)?);
    }

    let result = runtime.with_runtime(|rt| rt.call(&name, lattice_args).map_err(|e| e.to_string()));

    match result {
        Ok(value) => lattice_value_to_js(&mut cx, &value),
        Err(e) => cx.throw_error(e),
    }
}

/// Get a global variable's value
/// JS: getGlobal(runtime: RuntimeHandle, name: string): any
pub fn get_global(mut cx: FunctionContext) -> JsResult<JsValue> {
    let runtime = cx.argument::<JsBox<RuntimeBox>>(0)?;
    let name = cx.argument::<JsString>(1)?.value(&mut cx);

    let result = runtime.with_runtime(|rt| Ok(rt.get_global(&name)));

    match result {
        Ok(Some(value)) => lattice_value_to_js(&mut cx, &value),
        Ok(None) => Ok(cx.null().upcast()),
        Err(e) => cx.throw_error(e),
    }
}

/// Set a global variable
/// JS: setGlobal(runtime: RuntimeHandle, name: string, value: any): void
pub fn set_global(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let runtime = cx.argument::<JsBox<RuntimeBox>>(0)?;
    let name = cx.argument::<JsString>(1)?.value(&mut cx);
    let value = cx.argument::<JsValue>(2)?;

    let lattice_value = js_to_lattice_value(&mut cx, value)?;

    let result = runtime.with_runtime(|rt| {
        rt.set_global(&name, lattice_value);
        Ok(())
    });

    match result {
        Ok(()) => Ok(cx.undefined()),
        Err(e) => cx.throw_error(e),
    }
}

/// Check if a function exists
/// JS: hasFunction(runtime: RuntimeHandle, name: string): boolean
pub fn has_function(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let runtime = cx.argument::<JsBox<RuntimeBox>>(0)?;
    let name = cx.argument::<JsString>(1)?.value(&mut cx);

    let result = runtime.with_runtime(|rt| Ok(rt.has_function(&name)));

    match result {
        Ok(exists) => Ok(cx.boolean(exists)),
        Err(e) => cx.throw_error(e),
    }
}

/// Reset the runtime, clearing all state
/// JS: reset(runtime: RuntimeHandle): void
pub fn reset(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let runtime = cx.argument::<JsBox<RuntimeBox>>(0)?;

    let result = runtime.with_runtime(|rt| {
        rt.reset();
        Ok(())
    });

    match result {
        Ok(()) => Ok(cx.undefined()),
        Err(e) => cx.throw_error(e),
    }
}

/// Get all registered type schemas
/// JS: getTypes(runtime: RuntimeHandle): object[]
pub fn get_types(mut cx: FunctionContext) -> JsResult<JsArray> {
    let runtime = cx.argument::<JsBox<RuntimeBox>>(0)?;

    let result = runtime.with_runtime(|rt| {
        let types = rt.get_types();
        Ok(types.into_iter().collect::<Vec<_>>())
    });

    match result {
        Ok(types) => {
            let arr = cx.empty_array();
            for (i, schema) in types.iter().enumerate() {
                let js_schema = type_schema_to_js(&mut cx, schema)?;
                arr.set(&mut cx, i as u32, js_schema)?;
            }
            Ok(arr)
        }
        Err(e) => cx.throw_error(e),
    }
}

/// Get all function signatures
/// JS: getFunctionSignatures(runtime: RuntimeHandle): object[]
pub fn get_function_signatures(mut cx: FunctionContext) -> JsResult<JsArray> {
    let runtime = cx.argument::<JsBox<RuntimeBox>>(0)?;

    let result = runtime.with_runtime(|rt| {
        let sigs = rt.get_function_signatures();
        Ok(sigs.into_iter().collect::<Vec<_>>())
    });

    match result {
        Ok(sigs) => {
            let arr = cx.empty_array();
            for (i, sig) in sigs.iter().enumerate() {
                let js_sig = function_signature_to_js(&mut cx, sig)?;
                arr.set(&mut cx, i as u32, js_sig)?;
            }
            Ok(arr)
        }
        Err(e) => cx.throw_error(e),
    }
}

/// Get debug info from the last LLM call
/// JS: takeLlmDebug(runtime: RuntimeHandle): object | null
pub fn take_llm_debug(mut cx: FunctionContext) -> JsResult<JsValue> {
    let runtime = cx.argument::<JsBox<RuntimeBox>>(0)?;

    let result = runtime.with_runtime(|rt| Ok(rt.take_llm_debug()));

    match result {
        Ok(Some(debug)) => {
            let obj = cx.empty_object();
            let prompt = cx.string(&debug.prompt);
            obj.set(&mut cx, "prompt", prompt)?;
            let raw_response = cx.string(&debug.raw_response);
            obj.set(&mut cx, "raw_response", raw_response)?;
            let function_name = cx.string(&debug.function_name);
            obj.set(&mut cx, "function_name", function_name)?;
            let return_type = cx.string(&debug.return_type);
            obj.set(&mut cx, "return_type", return_type)?;
            Ok(obj.upcast())
        }
        Ok(None) => Ok(cx.null().upcast()),
        Err(e) => cx.throw_error(e),
    }
}
