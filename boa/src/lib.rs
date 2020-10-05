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
    clippy::let_unit_value,
    missing_doc_code_examples
)]

pub mod builtins;
pub mod class;
pub mod environment;
pub mod exec;
pub mod gc;
pub mod object;
pub mod profiler;
pub mod property;
pub mod realm;
pub mod syntax;
pub mod value;

pub mod context;

use std::result::Result as StdResult;

pub(crate) use crate::{exec::Executable, profiler::BoaProfiler};

// Export things to root level
#[doc(inline)]
pub use crate::{context::Context, value::Value};

use crate::syntax::{
    ast::node::StatementList,
    parser::{ParseError, Parser},
};

/// The result of a Javascript expression is represented like this so it can succeed (`Ok`) or fail (`Err`)
#[must_use]
pub type Result<T> = StdResult<T, Value>;

/// Parses the given source code.
///
/// It will return either the statement list AST node for the code, or a parsing error if something
/// goes wrong.
#[inline]
pub fn parse(src: &str) -> StdResult<StatementList, ParseError> {
    Parser::new(src.as_bytes()).parse_all()
}

/// Execute the code using an existing Context
/// The str is consumed and the state of the Context is changed
#[cfg(test)]
pub(crate) fn forward(engine: &mut Context, src: &str) -> String {
    // Setup executor
    let expr = match parse(src) {
        Ok(res) => res,
        Err(e) => {
            return format!(
                "Uncaught {}",
                engine
                    .throw_syntax_error(e.to_string())
                    .expect_err("interpreter.throw_syntax_error() did not return an error")
                    .display()
            );
        }
    };
    expr.run(engine).map_or_else(
        |e| format!("Uncaught {}", e.display()),
        |v| v.display().to_string(),
    )
}

/// Execute the code using an existing Context.
/// The str is consumed and the state of the Context is changed
/// Similar to `forward`, except the current value is returned instad of the string
/// If the interpreter fails parsing an error value is returned instead (error object)
#[allow(clippy::unit_arg, clippy::drop_copy)]
#[cfg(test)]
pub(crate) fn forward_val(engine: &mut Context, src: &str) -> Result<Value> {
    let main_timer = BoaProfiler::global().start_event("Main", "Main");
    // Setup executor
    let result = parse(src)
        .map_err(|e| {
            engine
                .throw_syntax_error(e.to_string())
                .expect_err("interpreter.throw_syntax_error() did not return an error")
        })
        .and_then(|expr| expr.run(engine));

    // The main_timer needs to be dropped before the BoaProfiler is.
    drop(main_timer);
    BoaProfiler::global().drop();

    result
}

/// Create a clean Context and execute the code
#[cfg(test)]
pub(crate) fn exec(src: &str) -> String {
    match Context::new().eval(src) {
        Ok(value) => value.display().to_string(),
        Err(error) => error.display().to_string(),
    }
}
