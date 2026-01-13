//! Error handling for Neon bindings

use neon::prelude::*;

/// Convert an error message to a Neon throw error
/// This is used when we need to return a NeonResult from a function that returns JsResult
pub fn to_neon_error<'a, C: Context<'a>>(cx: &mut C, msg: String) -> neon::result::Throw {
    cx.throw_error::<_, ()>(msg).unwrap_err()
}
