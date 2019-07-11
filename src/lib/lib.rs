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
#![warn(clippy::pedantic)]
#![allow(
    unsafe_code,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::excessive_precision,
    clippy::module_name_repetitions,
    clippy::pub_enum_variant_names,
    clippy::cognitive_complexity
)]

#[macro_use]
extern crate gc_derive;

pub mod environment;
pub mod exec;
pub mod js;
pub mod syntax;

use crate::exec::{Executor, Interpreter};
use crate::syntax::ast::expr::Expr;
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

fn parser_expr(src: &str) -> Expr {
    let mut lexer = Lexer::new(src);
    lexer.lex().unwrap();
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
