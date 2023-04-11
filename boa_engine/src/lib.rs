//! Boa's **`boa_engine`** crate implements ECMAScript's standard library of builtin objects
//! and an ECMAScript context, bytecompiler, and virtual machine for code execution.
//!
//! # About Boa
//!
//! Boa is an open-source, experimental ECMAScript Engine written in Rust for lexing, parsing and
//! executing ECMAScript/JavaScript. Currently, Boa supports some of the [language][boa-conformance].
//! More information can be viewed at [Boa's website][boa-web].
//!
//! Try out the most recent release with Boa's live demo [playground][boa-playground].
//!
//! # Example usage
//!
//! You can find multiple examples of the usage of Boa in the [`boa_examples`][examples] crate. In
//! order to use Boa in your project, you will need to add the `boa_engine` crate to your
//! `Cargo.toml` file. You will need to use a [`Source`] structure to handle the JavaScript code
//! to execute, and a [`Context`] structure to execute the code:
//!
//! ```
//! use boa_engine::{Context, Source};
//!
//! let js_code = "console.log('Hello World from a JS code string!')";
//!
//! // Instantiate the execution context
//! let mut context = Context::default();
//!
//! // Parse the source code
//! match context.eval_script(Source::from_bytes(js_code)) {
//!     Ok(res) => {
//!         println!(
//!             "{}",
//!             res.to_string(&mut context).unwrap().to_std_string_escaped()
//!         );
//!     }
//!     Err(e) => {
//!         // Pretty print the error
//!         eprintln!("Uncaught {e}");
//!     }
//! };
//! ```
//!
//! # Crate Features
//!
//!  - **serde** - Enables serialization and deserialization of the AST (Abstract Syntax Tree).
//!  - **console** - Enables `boa`'s [WHATWG `console`][whatwg] object implementation.
//!  - **profiler** - Enables profiling with measureme (this is mostly internal).
//!  - **intl** - Enables `boa`'s [ECMA-402 Internationalization API][ecma-402] (`Intl` object)
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
//! [boa-conformance]: https://boajs.dev/boa/test262/
//! [boa-web]: https://boajs.dev/
//! [boa-playground]: https://boajs.dev/boa/playground/
//! [examples]: https://github.com/boa-dev/boa/tree/main/boa_examples

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
pub mod optimizer;
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
        native_function::NativeFunction,
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
    native_function::NativeFunction,
    object::JsObject,
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

#[cfg(test)]
use std::borrow::Cow;

/// A test action executed in a test function.
#[cfg(test)]
#[derive(Clone)]
pub(crate) struct TestAction(Inner);

#[cfg(test)]
#[derive(Clone)]
enum Inner {
    RunHarness,
    Run {
        source: Cow<'static, str>,
    },
    InspectContext {
        op: fn(&mut Context<'_>),
    },
    Assert {
        source: Cow<'static, str>,
    },
    AssertEq {
        source: Cow<'static, str>,
        expected: JsValue,
    },
    AssertWithOp {
        source: Cow<'static, str>,
        op: fn(JsValue, &mut Context<'_>) -> bool,
    },
    AssertOpaqueError {
        source: Cow<'static, str>,
        expected: JsValue,
    },
    AssertNativeError {
        source: Cow<'static, str>,
        kind: builtins::error::ErrorKind,
        message: &'static str,
    },
    AssertContext {
        op: fn(&mut Context<'_>) -> bool,
    },
}

#[cfg(test)]
impl TestAction {
    /// Evaluates some utility functions used in tests.
    pub(crate) const fn run_harness() -> Self {
        Self(Inner::RunHarness)
    }

    /// Runs `source`, panicking if the execution throws.
    pub(crate) fn run(source: impl Into<Cow<'static, str>>) -> Self {
        Self(Inner::Run {
            source: source.into(),
        })
    }

    /// Executes `op` with the currently active context.
    ///
    /// Useful to make custom assertions that must be done from Rust code.
    pub(crate) fn inspect_context(op: fn(&mut Context<'_>)) -> Self {
        Self(Inner::InspectContext { op })
    }

    /// Asserts that evaluating `source` returns the `true` value.
    pub(crate) fn assert(source: impl Into<Cow<'static, str>>) -> Self {
        Self(Inner::Assert {
            source: source.into(),
        })
    }

    /// Asserts that the script returns `expected` when evaluating `source`.
    pub(crate) fn assert_eq(
        source: impl Into<Cow<'static, str>>,
        expected: impl Into<JsValue>,
    ) -> Self {
        Self(Inner::AssertEq {
            source: source.into(),
            expected: expected.into(),
        })
    }

    /// Asserts that calling `op` with the value obtained from evaluating `source` returns `true`.
    ///
    /// Useful to check properties of the obtained value that cannot be checked from JS code.
    pub(crate) fn assert_with_op(
        source: impl Into<Cow<'static, str>>,
        op: fn(JsValue, &mut Context<'_>) -> bool,
    ) -> Self {
        Self(Inner::AssertWithOp {
            source: source.into(),
            op,
        })
    }

    /// Asserts that evaluating `source` throws the opaque error `value`.
    pub(crate) fn assert_opaque_error(
        source: impl Into<Cow<'static, str>>,
        value: impl Into<JsValue>,
    ) -> Self {
        Self(Inner::AssertOpaqueError {
            source: source.into(),
            expected: value.into(),
        })
    }

    /// Asserts that evaluating `source` throws a native error of `kind` and `message`.
    pub(crate) fn assert_native_error(
        source: impl Into<Cow<'static, str>>,
        kind: builtins::error::ErrorKind,
        message: &'static str,
    ) -> Self {
        Self(Inner::AssertNativeError {
            source: source.into(),
            kind,
            message,
        })
    }

    /// Asserts that calling `op` with the currently executing context returns `true`.
    pub(crate) fn assert_context(op: fn(&mut Context<'_>) -> bool) -> Self {
        Self(Inner::AssertContext { op })
    }
}

/// Executes a list of test actions on a new, default context.
#[cfg(test)]
#[track_caller]
pub(crate) fn run_test_actions(actions: impl IntoIterator<Item = TestAction>) {
    let context = &mut Context::default();
    run_test_actions_with(actions, context);
}

/// Executes a list of test actions on the provided context.
#[cfg(test)]
#[track_caller]
pub(crate) fn run_test_actions_with(
    actions: impl IntoIterator<Item = TestAction>,
    context: &mut Context<'_>,
) {
    #[track_caller]
    fn forward_val(context: &mut Context<'_>, source: &str) -> JsResult<JsValue> {
        context.eval_script(Source::from_bytes(source))
    }

    #[track_caller]
    fn fmt_test(source: &str, test: usize) -> String {
        format!(
            "\n\nTest case {test}: \n```\n{}\n```",
            textwrap::indent(source, "    ")
        )
    }

    // Some unwrapping patterns look weird because they're replaceable
    // by simpler patterns like `unwrap_or_else` or `unwrap_err
    let mut i = 1;
    for action in actions.into_iter().map(|a| a.0) {
        match action {
            Inner::RunHarness => {
                // add utility functions for testing
                // TODO: extract to a file
                forward_val(
                    context,
                    r#"
                        function equals(a, b) {
                            if (Array.isArray(a) && Array.isArray(b)) {
                                return arrayEquals(a, b);
                            }
                            return a === b;
                        }
                        function arrayEquals(a, b) {
                            return Array.isArray(a) &&
                                Array.isArray(b) &&
                                a.length === b.length &&
                                a.every((val, index) => equals(val, b[index]));
                        }
                    "#,
                )
                .expect("failed to evaluate test harness");
            }
            Inner::Run { source } => {
                if let Err(e) = forward_val(context, &source) {
                    panic!("{}\nUncaught {e}", fmt_test(&source, i));
                }
            }
            Inner::InspectContext { op } => {
                op(context);
            }
            Inner::Assert { source } => {
                let val = match forward_val(context, &source) {
                    Err(e) => panic!("{}\nUncaught {e}", fmt_test(&source, i)),
                    Ok(v) => v,
                };
                let Some(val) = val.as_boolean() else {
                    panic!(
                        "{}\nTried to assert with the non-boolean value `{}`",
                        fmt_test(&source, i),
                        val.display()
                    )
                };
                assert!(val, "{}", fmt_test(&source, i));
                i += 1;
            }
            Inner::AssertEq { source, expected } => {
                let val = match forward_val(context, &source) {
                    Err(e) => panic!("{}\nUncaught {e}", fmt_test(&source, i)),
                    Ok(v) => v,
                };
                assert_eq!(val, expected, "{}", fmt_test(&source, i));
                i += 1;
            }
            Inner::AssertWithOp { source, op } => {
                let val = match forward_val(context, &source) {
                    Err(e) => panic!("{}\nUncaught {e}", fmt_test(&source, i)),
                    Ok(v) => v,
                };
                assert!(op(val, context), "{}", fmt_test(&source, i));
                i += 1;
            }
            Inner::AssertOpaqueError { source, expected } => {
                let err = match forward_val(context, &source) {
                    Ok(v) => panic!(
                        "{}\nExpected error, got value `{}`",
                        fmt_test(&source, i),
                        v.display()
                    ),
                    Err(e) => e,
                };
                let Some(err) = err.as_opaque() else {
                    panic!("{}\nExpected opaque error, got native error `{}`", fmt_test(&source, i), err)
                };

                assert_eq!(err, &expected, "{}", fmt_test(&source, i));
                i += 1;
            }
            Inner::AssertNativeError {
                source,
                kind,
                message,
            } => {
                let err = match forward_val(context, &source) {
                    Ok(v) => panic!(
                        "{}\nExpected error, got value `{}`",
                        fmt_test(&source, i),
                        v.display()
                    ),
                    Err(e) => e,
                };
                let native = match err.try_native(context) {
                    Ok(err) => err,
                    Err(e) => panic!(
                        "{}\nCouldn't obtain a native error: {e}",
                        fmt_test(&source, i)
                    ),
                };

                assert_eq!(&native.kind, &kind, "{}", fmt_test(&source, i));
                assert_eq!(native.message(), message, "{}", fmt_test(&source, i));
                i += 1;
            }
            Inner::AssertContext { op } => {
                assert!(op(context), "Test case {i}");
                i += 1;
            }
        }
    }
}
