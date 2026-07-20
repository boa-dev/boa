//! Boa's **`boa_wintertc`** crate implements the [WinterTC (TC55) Minimum Common Web API](https://min-common-api.proposal.wintertc.org/)
//! for the `boa_engine` crate.
//!
//! `WinterTC` (TC55) is an Ecma International Technical Committee working towards a baseline set
//! of Web Platform APIs that all server-side JavaScript runtimes (Deno, Bun, Cloudflare Workers,
//! Node.js, etc.) agree to implement, enabling portable server-side JavaScript.
//!
//! # Relationship to `boa_runtime`
//!
//! `boa_wintertc` is a standalone crate that depends only on `boa_engine`.
//! `boa_runtime` depends on `boa_wintertc` and re-exports its APIs, so users of `boa_runtime`
//! automatically get TC55 compliance without any extra setup.
//!
//! If you only want the TC55-mandated APIs and nothing else, depend on `boa_wintertc` directly.
//!
//! # Example: Registering all TC55 APIs
//!
//! ```no_run
//! use boa_engine::Context;
//!
//! let mut context = Context::default();
//!
//! boa_wintertc::register(None, &mut context)
//!     .expect("failed to register TC55 APIs");
//! ```
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![allow(
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::let_unit_value
)]

pub mod abort;
pub mod base64;
pub mod clone;
pub mod console;
pub mod encoding;
pub mod events;
#[cfg(feature = "fetch")]
pub mod fetch;
pub mod microtask;
pub mod store;
pub mod timers;
#[cfg(feature = "url")]
pub mod url;

/// Register all TC55-mandated Web APIs into the given [`boa_engine::Context`].
///
/// This registers the Minimum Common Web API as specified by `WinterTC` (TC55):
/// <https://min-common-api.proposal.wintertc.org/>
///
/// # Errors
///
/// Returns a [`boa_engine::JsError`] if any API fails to register (e.g. a global
/// object already exists with a conflicting name).
#[allow(clippy::needless_pass_by_value)]
pub fn register(
    realm: Option<boa_engine::realm::Realm>,
    ctx: &mut boa_engine::Context,
) -> boa_engine::JsResult<()> {
    console::register(realm.clone(), ctx)?;
    timers::register(realm.clone(), ctx)?;
    encoding::register(realm.clone(), ctx)?;
    microtask::register(realm.clone(), ctx)?;
    clone::register(realm.clone(), ctx)?;
    base64::register(realm.clone(), ctx)?;
    abort::register(realm.clone(), ctx)?;
    #[cfg(feature = "url")]
    url::register(realm.clone(), ctx)?;
    #[cfg(feature = "fetch")]
    fetch::register(realm, ctx)?;

    Ok(())
}

#[cfg(test)]
pub(crate) mod test {
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
        #[allow(unused)]
        pub(crate) fn inspect_context(op: impl FnOnce(&mut Context) + 'static) -> Self {
            Self(Inner::InspectContext { op: Box::new(op) })
        }

        /// Executes `op` with the currently active context in an async environment.
        #[allow(unused)]
        pub(crate) fn inspect_context_async(op: impl AsyncFnOnce(&mut Context) + 'static) -> Self {
            Self(Inner::InspectContextAsync {
                op: Box::new(move |ctx| Box::pin(op(ctx))),
            })
        }
    }

    /// Executes a list of test actions on a new, default context with all TC55 APIs registered.
    #[track_caller]
    #[allow(unused)]
    pub(crate) fn run_test_actions(actions: impl IntoIterator<Item = TestAction>) {
        let context = &mut Context::default();
        register(None, context).expect("failed to register WinterTC APIs");
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

                    assert_eq!(native.kind(), &kind, "{}", fmt_test(&source, i));
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
