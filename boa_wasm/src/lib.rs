use boa::Context;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn evaluate(src: &str) -> Result<String, JsValue> {
    // Setup executor
    Context::new()
        .eval(src)
        .map_err(|e| JsValue::from(format!("Uncaught {}", e.display())))
        .map(|v| v.display().to_string())
}
