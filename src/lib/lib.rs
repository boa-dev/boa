#![forbid(
    //missing_docs,
    //warnings,
    anonymous_parameters,
    unused_extern_crates,
    unused_import_braces,
    missing_copy_implementations,
    //trivial_casts,
    variant_size_differences,
    missing_debug_implementations,
    trivial_numeric_casts
)]
// Debug trait derivation will show an error if forbidden.
#![deny(unused_qualifications)]
#![deny(clippy::all)]
#![warn(
    clippy::pedantic,
    clippy::restriction,
    clippy::cognitive_complexity,
    //missing_docs
)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::missing_inline_in_public_items,
    clippy::implicit_return,
    clippy::wildcard_enum_match_arm
)]

pub mod environment;
pub mod exec;
pub mod js;
pub mod syntax;

use crate::{
    exec::{Executor, Interpreter},
    syntax::{ast::expr::Expr, lexer::Lexer, parser::Parser},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn parser_expr(src: &str) -> Expr {
    let mut lexer = Lexer::new(src);
    lexer.lex().expect("Failed lexing");
    let tokens = lexer.tokens;
    Parser::new(tokens).parse_all().unwrap()
}

/// Execute the code using an existing Interpreter
/// The str is consumed and the state of the Interpreter is changed
pub fn forward(engine: &mut Interpreter, src: &str) -> String {
    // Setup executor
    let expr = parser_expr(src);
    let result = engine.run(&expr);
    match result {
        Ok(v) => v.to_string(),
        Err(v) => format!("{}: {}", "Error", v.to_string()),
    }
}

/// Create a clean Interpreter and execute the code
pub fn exec(src: &str) -> String {
    let mut engine: Interpreter = Executor::new();
    forward(&mut engine, src)
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

    let mut engine: Interpreter = Executor::new();
    let result = engine.run(&expr);
    match result {
        Ok(v) => v.to_string(),
        Err(v) => {
            log(&format!("{} {}", "asudihsiu", v.to_string()));
            format!("{}: {}", "error", v.to_string())
        }
    }
}
