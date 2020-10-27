use boa::{exec::Executable, parse, Context};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn evaluate(src: &str) -> Result<String, JsValue> {
    // Setup executor
    let mut context = Context::new();

    let expr = match parse(src, false) {
        Ok(res) => res,
        Err(e) => {
            return Err(format!(
                "Uncaught {}",
                context
                    .throw_syntax_error(e.to_string())
                    .expect_err("interpreter.throw_syntax_error() did not return an error")
                    .display()
            )
            .into());
        }
    };
    expr.run(&mut context)
        .map_err(|e| JsValue::from(format!("Uncaught {}", e.display())))
        .map(|v| v.display().to_string())
}
