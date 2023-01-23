//! Boa's **`boa_engine`** crate implements ECMAScript's standard library of builtin objects
//! and an ECMAScript context, bytecompiler, and virtual machine for code execution.
//!
//! # Crate Features
//!  - **serde** - Enables serialization and deserialization of the AST (Abstract Syntax Tree).
//!  - **console** - Enables `boa`'s [WHATWG `console`][whatwg] object implementation.
//!  - **profiler** - Enables profiling with measureme (this is mostly internal).
//!  - **intl** - Enables `boa`'s [ECMA-402 Internationalization API][ecma-402] (`Intl` object)
//!
//! # About Boa
//! Boa is an open-source, experimental ECMAScript Engine written in Rust for lexing, parsing and executing ECMAScript/JavaScript. Currently, Boa
//! supports some of the [language][boa-conformance]. More information can be viewed at [Boa's website][boa-web].
//!
//! Try out the most recent release with Boa's live demo [playground][boa-playground].  
//!
//! # Boa Crates
//!  - **`boa_ast`** - Boa's ECMAScript Abstract Syntax Tree.
//!  - **`boa_engine`** - Boa's implementation of ECMAScript builtin objects and execution.
//!  - **`boa_gc`** - Boa's garbage collector.
//!  - **`boa_interner`** - Boa's string interner.
//!  - **`boa_parser`** - Boa's lexer and parser.
//!  - **`boa_profiler`** - Boa's code profiler.
//!  - **`boa_unicode`** - Boa's Unicode identifier.
//!  - **`boa_icu_provider`** - Boa's ICU4X data provider.
//!
//! [whatwg]: https://console.spec.whatwg.org
//! [ecma-402]: https://tc39.es/ecma402
//! [boa-conformance]: https://boa-dev.github.io/boa/test262/
//! [boa-web]: https://boa-dev.github.io/
//! [boa-playground]: https://boa-dev.github.io/boa/playground/

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,

    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,

    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,

    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    // Currently throws a false positive regarding dependencies that are only used in benchmarks.
    unused_crate_dependencies,
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::missing_errors_doc,
    clippy::let_unit_value,
    clippy::option_if_let_else,
    // Currently lints in places where `Self` would have a type parameter.
    clippy::use_self,

    // It may be worth to look if we can fix the issues highlighted by these lints.
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]

extern crate static_assertions as sa;

pub mod bigint;
pub mod builtins;
pub mod bytecompiler;
pub mod class;
pub mod context;
pub mod environments;
pub mod error;
pub mod job;
pub mod native_function;
pub mod object;
pub mod property;
pub mod realm;
pub mod string;
pub mod symbol;
pub mod value;
pub mod vm;

#[cfg(feature = "console")]
pub mod console;

pub(crate) mod tagged;
#[cfg(test)]
mod tests;

/// A convenience module that re-exports the most commonly-used Boa APIs
pub mod prelude {
    pub use crate::{
        error::{JsError, JsNativeError, JsNativeErrorKind},
        object::JsObject,
        Context, JsBigInt, JsResult, JsString, JsValue,
    };
    pub use boa_parser::Source;
}

use std::result::Result as StdResult;

// Export things to root level
#[doc(inline)]
pub use crate::{
    bigint::JsBigInt,
    context::Context,
    error::{JsError, JsNativeError, JsNativeErrorKind},
    string::JsString,
    symbol::JsSymbol,
    value::JsValue,
};
#[doc(inline)]
pub use boa_parser::Source;

/// The result of a Javascript expression is represented like this so it can succeed (`Ok`) or fail (`Err`)
pub type JsResult<T> = StdResult<T, JsError>;

/// A utility trait to make working with function arguments easier.
pub trait JsArgs {
    /// Utility function to `get` a parameter from a `[JsValue]` or default to `JsValue::Undefined`
    /// if `get` returns `None`.
    ///
    /// Call this if you are thinking of calling something similar to
    /// `args.get(n).cloned().unwrap_or_default()` or
    /// `args.get(n).unwrap_or(&undefined)`.
    ///
    /// This returns a reference for efficiency, in case you only need to call methods of `JsValue`,
    /// so try to minimize calling `clone`.
    fn get_or_undefined(&self, index: usize) -> &JsValue;
}

impl JsArgs for [JsValue] {
    fn get_or_undefined(&self, index: usize) -> &JsValue {
        const UNDEFINED: &JsValue = &JsValue::Undefined;
        self.get(index).unwrap_or(UNDEFINED)
    }
}

/// Execute the code using an existing `Context`.
///
/// The state of the `Context` is changed, and a string representation of the result is returned.
#[cfg(test)]
pub(crate) fn forward<S>(context: &mut Context<'_>, src: &S) -> String
where
    S: AsRef<[u8]> + ?Sized,
{
    context
        .eval(Source::from_bytes(src))
        .map_or_else(|e| format!("Uncaught {e}"), |v| v.display().to_string())
}

/// Execute the code using an existing Context.
/// The str is consumed and the state of the Context is changed
/// Similar to `forward`, except the current value is returned instead of the string
/// If the interpreter fails parsing an error value is returned instead (error object)
#[allow(clippy::unit_arg, clippy::drop_copy)]
#[cfg(test)]
pub(crate) fn forward_val<T: AsRef<[u8]> + ?Sized>(
    context: &mut Context<'_>,
    src: &T,
) -> JsResult<JsValue> {
    use boa_profiler::Profiler;

    let main_timer = Profiler::global().start_event("Main", "Main");

    let result = context.eval(Source::from_bytes(src));

    // The main_timer needs to be dropped before the Profiler is.
    drop(main_timer);
    Profiler::global().drop();

    result
}

/// Create a clean Context and execute the code
#[cfg(test)]
pub(crate) fn exec<T: AsRef<[u8]> + ?Sized>(src: &T) -> String {
    match Context::default().eval(Source::from_bytes(src)) {
        Ok(value) => value.display().to_string(),
        Err(error) => error.to_string(),
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
                    "Test case {i} ('{case}')"
                );
                i += 1;
            }
            TestAction::TestStartsWith(case, expected) => {
                assert!(
                    &forward(&mut context, case).starts_with(expected),
                    "Test case {i} ('{case}')",
                );
                i += 1;
            }
        }
    }
}
