use crate::{
    exec::{Executor, Interpreter},
    realm::Realm,
    syntax::{ast::expr::Expr, lexer::Lexer, parser::Parser},
};
use wasm_bindgen::prelude::*;

// WASM
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn evaluate(src: &str) -> String {
    let mut lexer = Lexer::new(&src);
    match lexer.lex() {
        Ok(_v) => (),
        Err(v) => log(&v.to_string()),
    }

    let tokens = lexer.tokens;

    // Setup executor
    let expr: Expr;

    match Parser::new(tokens).parse_all() {
        Ok(v) => {
            expr = v;
        }
        Err(_v) => {
            log("parsing fail");
            return String::from("parsing failed");
        }
    }
    // Create new Realm
    let realm = Realm::create();
    let mut engine: Interpreter = Executor::new(realm);
    let result = engine.run(&expr);
    match result {
        Ok(v) => v.to_string(),
        Err(v) => format!("{}: {}", "error", v.to_string()),
    }
}
