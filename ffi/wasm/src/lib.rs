//! An ECMAScript Wasm implementation based on `boa_engine`.
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![allow(unused_crate_dependencies)]

use boa_engine::{Context, Source};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn main_js() {
    console_error_panic_hook::set_once();
}

/// Evaluate the given ECMAScript code.
///
/// # Errors
///
/// If the execution of the script throws, returns a `JsValue` with the error string.
#[wasm_bindgen]
pub fn evaluate(src: &str) -> Result<String, JsValue> {
    // Setup the executor
    Context::default()
        .eval(Source::from_bytes(src))
        .map_err(|e| JsValue::from(format!("Uncaught {e}")))
        .map(|v| v.display().to_string())
}
