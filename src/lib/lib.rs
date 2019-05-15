extern crate chrono;
extern crate gc;
extern crate rand;
extern crate serde_json;

#[macro_use]
extern crate gc_derive;

pub mod exec;
pub mod js;
pub mod syntax;

use crate::exec::{Executor, Interpreter};
use crate::syntax::lexer::Lexer;
use crate::syntax::parser::Parser;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub fn exec(src: String) {
    let mut lexer = Lexer::new(&src);
    lexer.lex().unwrap();
    let tokens = lexer.tokens;

    // Setup executor
    let expr = Parser::new(tokens).parse_all().unwrap();
    // print!("{:#?}", expr);

    let mut engine: Interpreter = Executor::new();
    let result = engine.run(&expr);
    match result {
        Ok(v) => print!("{}", v),
        Err(v) => print!("Error: {}", v),
    }
}

#[wasm_bindgen]
pub fn evaluate(src: &str) {
    let mut lexer = Lexer::new(&src);
    match lexer.lex() {
        Ok(_v) => (),
        Err(v) => log(&v.to_string()),
    }

    let tokens = lexer.tokens;

    // Setup executor
    let expr: syntax::ast::expr::Expr;

    match Parser::new(tokens).parse_all() {
        Ok(v) => {
            expr = v;
        }
        Err(v) => {
            log("parsing fail");
            return;
        }
    }

    let mut engine: Interpreter = Executor::new();
    let result = engine.run(&expr);
    match result {
        Ok(v) => log(&v.to_string()),
        Err(v) => log("Error"),
    }
}
