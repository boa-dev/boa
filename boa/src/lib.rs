//! This is an experimental Javascript lexer, parser and compiler written in Rust. Currently, it has support for some of the language.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/jasonwilliams/boa/master/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/jasonwilliams/boa/master/assets/logo.svg"
)]
#![deny(
    unused_qualifications,
    clippy::all,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    // rustdoc,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
)]
#![warn(clippy::perf, clippy::single_match_else, clippy::dbg_macro)]
#![allow(
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions,
    missing_doc_code_examples
)]

pub mod builtins;
pub mod environment;
pub mod exec;
pub mod realm;
pub mod syntax;

use crate::{builtins::value::ResultValue, syntax::ast::node::StatementList};
pub use crate::{
    exec::{Executable, Interpreter},
    realm::Realm,
    syntax::{lexer::Lexer, parser::Parser},
};

fn parser_expr(src: &str) -> Result<StatementList, String> {
    let mut lexer = Lexer::new(src);
    lexer.lex().map_err(|e| format!("Syntax Error: {}", e))?;
    let tokens = lexer.tokens;
    Parser::new(&tokens)
        .parse_all()
        .map_err(|e| format!("Parsing Error: {}", e))
}

/// Execute the code using an existing Interpreter
/// The str is consumed and the state of the Interpreter is changed
pub fn forward(engine: &mut Interpreter, src: &str) -> String {
    // Setup executor
    let expr = match parser_expr(src) {
        Ok(res) => res,
        Err(e) => return e,
    };
    expr.run(engine)
        .map_or_else(|e| format!("Error: {}", e), |v| v.to_string())
}

/// Execute the code using an existing Interpreter.
/// The str is consumed and the state of the Interpreter is changed
/// Similar to `forward`, except the current value is returned instad of the string
/// If the interpreter fails parsing an error value is returned instead (error object)
pub fn forward_val(engine: &mut Interpreter, src: &str) -> ResultValue {
    // Setup executor
    match parser_expr(src) {
        Ok(expr) => expr.run(engine),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

/// Create a clean Interpreter and execute the code
pub fn exec(src: &str) -> String {
    // Create new Realm
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    forward(&mut engine, src)
}
