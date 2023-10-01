//! An ECMAScript WASM implementation based on boa_engine.
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]

use boa_engine::{Context, Source};
use chrono as _;
use getrandom as _;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn main() {
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
