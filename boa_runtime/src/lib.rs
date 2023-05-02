//! Boa's **boa_runtime** crate contains an example runtime and basic runtime features and functionality for the `boa_engine` crate for
//! runtime implementors.
//!
//! # About Boa
//!
//! Boa is an open-source, experimental ECMAScript Engine written in Rust for lexing, parsing and
//! executing ECMAScript/JavaScript. Currently, Boa supports some of the [language][boa-conformance].
//! More information can be viewed at [Boa's website][boa-web].
//!
//! Try out the most recent release with Boa's live demo [playground][boa-playground].
//!
//! # Example: Adding Web API's Console Object
//!
//! 1. Add **boa_runtime** as a dependency to your project along with **boa_engine**.
//!
//! ```
//! use boa-engine::{ Context, Source };
//! use boa_runtime::Console;
//!
//! // Create the context.
//! let mut context = Context::default();
//!
//! // Initialize the Console object.
//! let console = Console::init(context);
//!
//! // Register the console as a global property to the context.
//! context
//!     .register_global_property(Console::NAME, console, Attribute::all)
//!     .expect("the console object shouldn't exist yet")
//!
//! // Parse the source code
//! match context.eval_script(Source::from_bytes(js_code)) {
//!    Ok(res) => {
//!        println!(
//!            "{}",
//!            res.to_string(&mut context).unwrap().to_std_string_escaped()
//!        );
//!    }
//!    Err(e) => {
//!        // Pretty print the error
//!        eprintln!("Uncaught {e}");
//!    }
//! };
//!
//! ```
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
//! [ecma-402]: https://tc39.es/ecma402
//! [boa-conformance]: https://boajs.dev/boa/test262/
//! [boa-web]: https://boajs.dev/
//! [boa-playground]: https://boajs.dev/boa/playground/

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
    clippy::undocumented_unsafe_blocks
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::let_unit_value
)]

mod console;

#[doc(inline)]
pub use console::Console;

#[cfg(test)]
pub(crate) mod test {
    use boa_engine::{builtins, Context, JsResult, JsValue, Source};
    use std::borrow::Cow;

    /// A test action executed in a test function.
    #[allow(missing_debug_implementations)]
    #[derive(Clone)]
    pub(crate) struct TestAction(Inner);

    #[derive(Clone)]
    #[allow(dead_code)]
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

    impl TestAction {
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
    }

    /// Executes a list of test actions on a new, default context.
    #[track_caller]
    pub(crate) fn run_test_actions(actions: impl IntoIterator<Item = TestAction>) {
        let context = &mut Context::default();
        run_test_actions_with(actions, context);
    }

    /// Executes a list of test actions on the provided context.
    #[track_caller]
    #[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
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
}
