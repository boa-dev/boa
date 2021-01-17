use boa::{exec::Executable, parse, Context};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn evaluate(src: &str) -> Result<String, JsValue> {
    // Setup executor
    let context = Context::new();

    let expr = match parse(src, false) {
        Ok(res) => res,
        Err(e) => {
            return Err(format!(
                "Uncaught {}",
                context
                    .throw_syntax_error(e.to_string())
                    .expect_err("interpreter.throw_syntax_error() did not return an error")
                    .display(&context)
            )
            .into());
        }
    };
    expr.run(&context)
        .map_err(|e| JsValue::from(format!("Uncaught {}", e.display(&context))))
        .map(|v| v.display(&context).to_string())
}
