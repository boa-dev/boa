// Debug trait derivation will show an error if forbidden.
#![deny(unused_qualifications, clippy::correctness, clippy::style)]
#![warn(clippy::perf)]
#![allow(clippy::cognitive_complexity)]

pub mod builtins;
pub mod environment;
pub mod exec;
pub mod realm;
pub mod syntax;
#[cfg(feature = "wasm-bindgen")]
mod wasm;

use crate::{
    builtins::value::ResultValue,
    exec::{Executor, Interpreter},
    realm::Realm,
    syntax::{ast::expr::Expr, lexer::Lexer, parser::Parser},
};
#[cfg(feature = "wasm-bindgen")]
pub use wasm::*;

fn parser_expr(src: &str) -> Expr {
    let mut lexer = Lexer::new(src);
    lexer.lex().expect("lexing failed");
    let tokens = lexer.tokens;
    Parser::new(tokens).parse_all().expect("parsing failed")
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

/// Execute the code using an existing Interpreter.
/// The str is consumed and the state of the Interpreter is changed
/// Similar to `forward`, except the current value is returned instad of the string
/// If the interpreter fails parsing an error value is returned instead (error object)
pub fn forward_val(engine: &mut Interpreter, src: &str) -> ResultValue {
    // Setup executor
    let expr = parser_expr(src);
    engine.run(&expr)
}

/// Create a clean Interpreter and execute the code
pub fn exec(src: &str) -> String {
    // Create new Realm
    let realm = Realm::create();
    let mut engine: Interpreter = Executor::new(realm);
    forward(&mut engine, src)
}
