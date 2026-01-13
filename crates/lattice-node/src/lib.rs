//! Neon bindings for embedding Lattice in Node.js
//!
//! This crate provides Node.js bindings that allow JavaScript/TypeScript
//! applications to use the Lattice runtime.
//!
//! # Usage in Node.js
//!
//! ```javascript
//! const { Runtime } = require('lattice-lang');
//!
//! const rt = new Runtime();
//! const result = rt.eval("1 + 2 + 3");
//! console.log(result); // 6
//! ```

use neon::prelude::*;

mod convert;
mod error;
mod runtime;
mod schema;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createRuntime", runtime::create_runtime)?;
    cx.export_function("eval", runtime::eval)?;
    cx.export_function("evalFile", runtime::eval_file)?;
    cx.export_function("call", runtime::call)?;
    cx.export_function("getGlobal", runtime::get_global)?;
    cx.export_function("setGlobal", runtime::set_global)?;
    cx.export_function("hasFunction", runtime::has_function)?;
    cx.export_function("reset", runtime::reset)?;
    cx.export_function("getTypes", runtime::get_types)?;
    cx.export_function("getFunctionSignatures", runtime::get_function_signatures)?;
    cx.export_function("takeLlmDebug", runtime::take_llm_debug)?;
    Ok(())
}
