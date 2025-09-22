//! Boa's **boa_runtime** crate contains an example runtime and basic runtime features and
//! functionality for the `boa_engine` crate for runtime implementors.
//!
//! # Example: Adding Web API's Console Object
//!
//! 1. Add **boa_runtime** as a dependency to your project along with **boa_engine**.
//!
//! ```
//! use boa_engine::{js_string, property::Attribute, Context, Source};
//! use boa_runtime::Console;
//! use boa_runtime::console::DefaultLogger;
//!
//! // Create the context.
//! let mut context = Context::default();
//!
//! // Register the Console object to the context. The DefaultLogger simply
//! // write errors to STDERR and all other logs to STDOUT.
//! Console::register_with_logger(DefaultLogger, &mut context)
//!     .expect("the console object shouldn't exist yet");
//!
//! // JavaScript source for parsing.
//! let js_code = "console.log('Hello World from a JS code string!')";
//!
//! // Parse the source code
//! match context.eval(Source::from_bytes(js_code)) {
//!     Ok(res) => {
//!         println!(
//!             "{}",
//!             res.to_string(&mut context).unwrap().to_std_string_escaped()
//!         );
//!     }
//!     Err(e) => {
//!         // Pretty print the error
//!         eprintln!("Uncaught {e}");
//!         # panic!("An error occured in boa_runtime's js_code");
//!     }
//! };
//! ```
//!
//! # Example: Add all supported Boa's Runtime Web API to your context
//!
//! ```no_run
//! use boa_engine::{js_string, property::Attribute, Context, Source};
//!
//! // Create the context.
//! let mut context = Context::default();
//!
//! // Register all objects in the context. To conditionally register extensions,
//! // call `register()` directly on the extension.
//! boa_runtime::register(
//!     (
//!         // Register the default logger.
//!         boa_runtime::extensions::ConsoleExtension::default(),
//!         // A fetcher can be added if the `fetch` feature flag is enabled.
//!         // This fetcher uses the Reqwest blocking API to allow fetching using HTTP.
//! #       #[cfg(feature = "reqwest-blocking")]
//!         boa_runtime::extensions::FetchExtension(
//!             boa_runtime::fetch::BlockingReqwestFetcher::default()
//!         ),
//!     ),
//!     None,
//!     &mut context,
//! );
//!
//! // JavaScript source for parsing.
//! let js_code = r#"
//!     fetch("https://google.com/")
//!         .then(response => response.text())
//!         .then(html => console.log(html))
//! "#;
//!
//! // Parse the source code
//! match context.eval(Source::from_bytes(js_code)) {
//!     Ok(res) => {
//!         // The result is a promise, so we need to await it.
//!         res
//!             .as_promise()
//!             .expect("Should be a promise")
//!             .await_blocking(&mut context)
//!             .expect("Should resolve()");
//!         println!(
//!             "{}",
//!             res.to_string(&mut context).unwrap().to_std_string_escaped()
//!         );
//!     }
//!     Err(e) => {
//!         // Pretty print the error
//!         eprintln!("Uncaught {e}");
//!         # panic!("An error occured in boa_runtime's js_code");
//!     }
//! };
//! ```
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg"
)]
#![cfg_attr(test, allow(clippy::needless_raw_string_hashes))] // Makes strings a bit more copy-pastable
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
// Currently throws a false positive regarding dependencies that are only used in tests.
#![allow(unused_crate_dependencies)]
#![allow(
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::let_unit_value
)]

pub mod console;

#[doc(inline)]
pub use console::{Console, ConsoleState, DefaultLogger, Logger, NullLogger};

pub mod clone;
#[cfg(feature = "fetch")]
pub mod fetch;
pub mod interval;
pub mod message;
pub mod microtask;
pub mod store;
pub mod text;
#[cfg(feature = "url")]
pub mod url;

pub mod extensions;

use crate::extensions::{
    EncodingExtension, MicrotaskExtension, StructuredCloneExtension, TimeoutExtension,
};
pub use extensions::RuntimeExtension;

/// Register all the built-in objects and functions of the `WebAPI` runtime, plus
/// any extensions defined.
///
/// # Errors
/// This will error if any of the built-in objects or functions cannot be registered.
pub fn register(
    extensions: impl RuntimeExtension,
    realm: Option<boa_engine::realm::Realm>,
    ctx: &mut boa_engine::Context,
) -> boa_engine::JsResult<()> {
    (
        TimeoutExtension,
        EncodingExtension,
        MicrotaskExtension,
        StructuredCloneExtension,
        #[cfg(feature = "url")]
        extensions::UrlExtension,
        extensions,
    )
        .register(realm, ctx)?;

    Ok(())
}

/// Register only the extensions provided. An application can use this to register
/// extensions that it previously hadn't registered.
///
/// # Errors
/// This will error if any of the built-in objects or functions cannot be registered.
pub fn register_extensions(
    extensions: impl RuntimeExtension,
    realm: Option<boa_engine::realm::Realm>,
    ctx: &mut boa_engine::Context,
) -> boa_engine::JsResult<()> {
    extensions.register(realm, ctx)?;

    Ok(())
}

#[cfg(test)]
pub(crate) mod test {
    use crate::extensions::ConsoleExtension;
    use crate::register;
    use boa_engine::{Context, JsError, JsResult, JsValue, Source, builtins};
    use std::borrow::Cow;
    use std::path::{Path, PathBuf};
    use std::pin::Pin;

    /// A test action executed in a test function.
    #[allow(missing_debug_implementations)]
    pub(crate) struct TestAction(Inner);

    #[allow(dead_code)]
    #[allow(clippy::type_complexity)]
    enum Inner {
        RunHarness,
        Run {
            source: Cow<'static, str>,
        },
        RunFile {
            path: PathBuf,
        },
        RunJobs,
        InspectContext {
            op: Box<dyn FnOnce(&mut Context)>,
        },
        InspectContextAsync {
            op: Box<dyn for<'a> FnOnce(&'a mut Context) -> Pin<Box<dyn Future<Output = ()> + 'a>>>,
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
            op: fn(JsValue, &mut Context) -> bool,
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
            op: fn(&mut Context) -> bool,
        },
    }

    impl TestAction {
        #[allow(unused)]
        pub(crate) fn harness() -> Self {
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
        pub(crate) fn inspect_context(op: impl FnOnce(&mut Context) + 'static) -> Self {
            Self(Inner::InspectContext { op: Box::new(op) })
        }

        /// Executes `op` with the currently active context in an async environment.
        pub(crate) fn inspect_context_async(op: impl AsyncFnOnce(&mut Context) + 'static) -> Self {
            Self(Inner::InspectContextAsync {
                op: Box::new(move |ctx| Box::pin(op(ctx))),
            })
        }
    }

    /// Executes a list of test actions on a new, default context.
    #[track_caller]
    pub(crate) fn run_test_actions(actions: impl IntoIterator<Item = TestAction>) {
        let context = &mut Context::default();
        register(ConsoleExtension::default(), None, context)
            .expect("failed to register WebAPI objects");
        run_test_actions_with(actions, context);
    }

    /// Executes a list of test actions on the provided context.
    #[track_caller]
    #[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
    pub(crate) fn run_test_actions_with(
        actions: impl IntoIterator<Item = TestAction>,
        context: &mut Context,
    ) {
        #[track_caller]
        fn forward_val(context: &mut Context, source: &str) -> JsResult<JsValue> {
            context.eval(Source::from_bytes(source))
        }

        #[track_caller]
        fn forward_file(context: &mut Context, path: impl AsRef<Path>) -> JsResult<JsValue> {
            let p = path.as_ref();
            context.eval(Source::from_filepath(p).map_err(JsError::from_rust)?)
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
                    if let Err(e) = forward_file(context, "./assets/harness.js") {
                        panic!("Uncaught {e} in the test harness");
                    }
                }
                Inner::Run { source } => {
                    if let Err(e) = forward_val(context, &source) {
                        panic!("{}\nUncaught {e}", fmt_test(&source, i));
                    }
                }
                Inner::RunFile { path } => {
                    if let Err(e) = forward_file(context, &path) {
                        panic!("Uncaught {e} in file {path:?}");
                    }
                    forward_file(context, &path).expect("failed to run file");
                }
                Inner::RunJobs => {
                    if let Err(e) = context.run_jobs() {
                        panic!("Uncaught {e} in a job");
                    }
                }
                Inner::InspectContext { op } => {
                    op(context);
                }
                Inner::InspectContextAsync { op } => futures_lite::future::block_on(op(context)),
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
                        panic!(
                            "{}\nExpected opaque error, got native error `{}`",
                            fmt_test(&source, i),
                            err
                        )
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
