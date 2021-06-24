/*!
This is an experimental Javascript lexer, parser and compiler written in Rust. Currently, it has support for some of the language.

# Crate Features
 - **serde** - Enables serialization and deserialization of the AST (Abstract Syntax Tree).
 - **console** - Enables `boa`s WHATWG `console` object implementation.
 - **profiler** - Enables profiling with measureme (this is mostly internal).

**/

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

// builtins module has a lot of built-in functions that need unnecessary_wraps
#[allow(clippy::unnecessary_wraps)]
pub mod builtins;
pub mod class;
pub mod environment;
pub mod exec;
pub mod gc;
pub mod object;
pub mod profiler;
pub mod property;
pub mod realm;
pub mod symbol;
// syntax module has a lot of acronyms
#[allow(clippy::upper_case_acronyms)]
pub mod syntax;
pub mod value;
#[cfg(feature = "vm")]
pub mod vm;

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
pub fn parse<T: AsRef<[u8]>>(src: T, strict_mode: bool) -> StdResult<StatementList, ParseError> {
    let src_bytes: &[u8] = src.as_ref();
    Parser::new(src_bytes, strict_mode).parse_all()
}

/// Execute the code using an existing Context
/// The str is consumed and the state of the Context is changed
#[cfg(test)]
pub(crate) fn forward<T: AsRef<[u8]>>(context: &mut Context, src: T) -> String {
    let src_bytes: &[u8] = src.as_ref();

    // Setup executor
    let expr = match parse(src_bytes, false) {
        Ok(res) => res,
        Err(e) => {
            return format!(
                "Uncaught {}",
                context
                    .throw_syntax_error(e.to_string())
                    .expect_err("interpreter.throw_syntax_error() did not return an error")
                    .display()
            );
        }
    };
    expr.run(context).map_or_else(
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
pub(crate) fn forward_val<T: AsRef<[u8]>>(context: &mut Context, src: T) -> Result<Value> {
    let main_timer = BoaProfiler::global().start_event("Main", "Main");

    let src_bytes: &[u8] = src.as_ref();
    // Setup executor
    let result = parse(src_bytes, false)
        .map_err(|e| {
            context
                .throw_syntax_error(e.to_string())
                .expect_err("interpreter.throw_syntax_error() did not return an error")
        })
        .and_then(|expr| expr.run(context));

    // The main_timer needs to be dropped before the BoaProfiler is.
    drop(main_timer);
    BoaProfiler::global().drop();

    result
}

/// Create a clean Context and execute the code
#[cfg(test)]
pub(crate) fn exec<T: AsRef<[u8]>>(src: T) -> String {
    let src_bytes: &[u8] = src.as_ref();

    match Context::new().eval(src_bytes) {
        Ok(value) => value.display().to_string(),
        Err(error) => error.display().to_string(),
    }
}
