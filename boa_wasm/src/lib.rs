use boa::{Executable, Interpreter, Parser, Realm};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn evaluate(src: &str) -> Result<String, JsValue> {
    let expr = Parser::new(src.as_bytes())
        .parse_all()
        .map_err(|e| JsValue::from(format!("Parsing Error: {}", e)))?;

    // Setup executor
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Setup executor
    expr.run(&mut engine)
        .map_err(|e| JsValue::from(format!("Error: {}", e.display())))
        .map(|v| v.display().to_string())
}
