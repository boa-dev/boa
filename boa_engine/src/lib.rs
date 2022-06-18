//! This is an experimental Javascript lexer, parser and compiler written in Rust. Currently, it
//! has support for some of the language.
//!
//! # Crate Features
//!  - **serde** - Enables serialization and deserialization of the AST (Abstract Syntax Tree).
//!  - **console** - Enables `boa`'s [WHATWG `console`][whatwg] object implementation.
//!  - **profiler** - Enables profiling with measureme (this is mostly internal).
//!  - **intl** - Enables `boa`'s [ECMA-402 Internationalization API][ecma-402] (`Intl` object)
//!
//! [whatwg]: https://console.spec.whatwg.org
//! [ecma-402]: https://tc39.es/ecma402

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(
    clippy::perf,
    clippy::single_match_else,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::struct_excessive_bools,
    clippy::doc_markdown,
    clippy::semicolon_if_nothing_returned,
    clippy::pedantic
)]
#![deny(
    clippy::all,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::unnested_or_patterns,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_unwrap_or,
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
#![allow(
    clippy::use_self, // TODO: deny once false positives are fixed
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_ptr_alignment,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions,
    clippy::let_unit_value,
    rustdoc::missing_doc_code_examples
)]

pub mod bigint;
pub mod builtins;
pub mod bytecompiler;
pub mod class;
pub mod context;
pub mod environments;
pub mod job;
pub mod object;
pub mod property;
pub mod realm;
pub mod string;
pub mod symbol;
pub mod syntax;
pub mod value;
pub mod vm;

#[cfg(test)]
mod tests;

/// A convenience module that re-exports the most commonly-used Boa APIs
pub mod prelude {
    pub use crate::{object::JsObject, Context, JsBigInt, JsResult, JsString, JsValue};
}

use std::result::Result as StdResult;

// Export things to root level
#[doc(inline)]
pub use crate::{
    bigint::JsBigInt, context::Context, string::JsString, symbol::JsSymbol, value::JsValue,
};

/// The result of a Javascript expression is represented like this so it can succeed (`Ok`) or fail (`Err`)
pub type JsResult<T> = StdResult<T, JsValue>;

/// Execute the code using an existing `Context`.
///
/// The state of the `Context` is changed, and a string representation of the result is returned.
#[cfg(test)]
pub(crate) fn forward<S>(context: &mut Context, src: S) -> String
where
    S: AsRef<[u8]>,
{
    context.eval(src.as_ref()).map_or_else(
        |e| format!("Uncaught {}", e.display()),
        |v| v.display().to_string(),
    )
}

/// Execute the code using an existing Context.
/// The str is consumed and the state of the Context is changed
/// Similar to `forward`, except the current value is returned instead of the string
/// If the interpreter fails parsing an error value is returned instead (error object)
#[allow(clippy::unit_arg, clippy::drop_copy)]
#[cfg(test)]
pub(crate) fn forward_val<T: AsRef<[u8]>>(context: &mut Context, src: T) -> JsResult<JsValue> {
    use boa_profiler::Profiler;

    let main_timer = Profiler::global().start_event("Main", "Main");

    let src_bytes: &[u8] = src.as_ref();
    let result = context.eval(src_bytes);

    // The main_timer needs to be dropped before the Profiler is.
    drop(main_timer);
    Profiler::global().drop();

    result
}

/// Create a clean Context and execute the code
#[cfg(test)]
pub(crate) fn exec<T: AsRef<[u8]>>(src: T) -> String {
    let src_bytes: &[u8] = src.as_ref();

    match Context::default().eval(src_bytes) {
        Ok(value) => value.display().to_string(),
        Err(error) => error.display().to_string(),
    }
}

#[cfg(test)]
pub(crate) enum TestAction {
    Execute(&'static str),
    TestEq(&'static str, &'static str),
    TestStartsWith(&'static str, &'static str),
}

/// Create a clean Context, call "forward" for each action, and optionally
/// assert equality of the returned value or if returned value starts with
/// expected string.
#[cfg(test)]
#[track_caller]
pub(crate) fn check_output(actions: &[TestAction]) {
    let mut context = Context::default();

    let mut i = 1;
    for action in actions {
        match action {
            TestAction::Execute(src) => {
                forward(&mut context, src);
            }
            TestAction::TestEq(case, expected) => {
                assert_eq!(
                    &forward(&mut context, case),
                    expected,
                    "Test case {} ('{}')",
                    i,
                    case
                );
                i += 1;
            }
            TestAction::TestStartsWith(case, expected) => {
                assert!(
                    &forward(&mut context, case).starts_with(expected),
                    "Test case {} ('{}')",
                    i,
                    case
                );
                i += 1;
            }
        }
    }
}
