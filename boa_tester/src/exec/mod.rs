//! Execution module for the test runner.

mod js262;

use crate::{
    read::ErrorType, Harness, Outcome, Phase, SpecEdition, Statistics, SuiteResult, Test,
    TestFlags, TestOutcomeResult, TestResult, TestSuite, VersionedStats,
};
use boa_engine::{
    builtins::promise::PromiseState,
    js_string,
    module::{Module, ModuleLoader, SimpleModuleLoader},
    native_function::NativeFunction,
    object::FunctionObjectBuilder,
    optimizer::OptimizerOptions,
    property::Attribute,
    script::Script,
    Context, JsArgs, JsError, JsNativeErrorKind, JsValue, Source,
};
use colored::Colorize;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use std::{cell::RefCell, eprintln, rc::Rc};

impl TestSuite {
    /// Runs the test suite.
    pub(crate) fn run(
        &self,
        harness: &Harness,
        verbose: u8,
        parallel: bool,
        max_edition: SpecEdition,
        optimizer_options: OptimizerOptions,
    ) -> SuiteResult {
        if verbose != 0 {
            println!("Suite {}:", self.path.display());
        }

        let suites: Vec<_> = if parallel {
            self.suites
                .par_iter()
                .map(|suite| suite.run(harness, verbose, parallel, max_edition, optimizer_options))
                .collect()
        } else {
            self.suites
                .iter()
                .map(|suite| suite.run(harness, verbose, parallel, max_edition, optimizer_options))
                .collect()
        };

        let tests: Vec<_> = if parallel {
            self.tests
                .par_iter()
                .filter(|test| test.edition <= max_edition)
                .flat_map(|test| test.run(harness, verbose, optimizer_options))
                .collect()
        } else {
            self.tests
                .iter()
                .filter(|test| test.edition <= max_edition)
                .flat_map(|test| test.run(harness, verbose, optimizer_options))
                .collect()
        };

        let mut features = FxHashSet::default();
        for test_iter in &*self.tests {
            features.extend(test_iter.features.iter().map(ToString::to_string));
        }

        if verbose != 0 {
            println!();
        }

        // Count passed tests and es specs
        let mut versioned_stats = VersionedStats::default();
        let mut es_next = Statistics::default();

        for test in &tests {
            match test.result {
                TestOutcomeResult::Passed => {
                    versioned_stats.apply(test.edition, |stats| {
                        stats.passed += 1;
                    });
                    es_next.passed += 1;
                }
                TestOutcomeResult::Ignored => {
                    versioned_stats.apply(test.edition, |stats| {
                        stats.ignored += 1;
                    });
                    es_next.ignored += 1;
                }
                TestOutcomeResult::Panic => {
                    versioned_stats.apply(test.edition, |stats| {
                        stats.panic += 1;
                    });
                    es_next.panic += 1;
                }
                TestOutcomeResult::Failed => {}
            }
            versioned_stats.apply(test.edition, |stats| {
                stats.total += 1;
            });
            es_next.total += 1;
        }

        // Count total tests
        for suite in &suites {
            versioned_stats += suite.versioned_stats;
            es_next += suite.stats;
            features.extend(suite.features.iter().cloned());
        }

        if verbose != 0 {
            println!(
                "Suite {} results: total: {}, passed: {}, ignored: {}, failed: {} {}, conformance: {:.2}%",
                self.path.display(),
                es_next.total,
                es_next.passed.to_string().green(),
                es_next.ignored.to_string().yellow(),
                (es_next.total - es_next.passed - es_next.ignored)
                    .to_string()
                    .red(),
                if es_next.panic == 0 {
                    String::new()
                } else {
                    format!("({})", format!("{} panics", es_next.panic).red())
                },
                (es_next.passed as f64 / es_next.total as f64) * 100.0
            );
        }
        SuiteResult {
            name: self.name.clone(),
            stats: es_next,
            versioned_stats,
            suites,
            tests,
            features,
        }
    }
}

impl Test {
    /// Runs the test.
    pub(crate) fn run(
        &self,
        harness: &Harness,
        verbose: u8,
        optimizer_options: OptimizerOptions,
    ) -> Vec<TestResult> {
        let mut results = Vec::new();
        if self.flags.contains(TestFlags::MODULE) {
            results.push(self.run_once(harness, false, verbose, optimizer_options));
        } else {
            if self.flags.contains(TestFlags::STRICT) && !self.flags.contains(TestFlags::RAW) {
                results.push(self.run_once(harness, true, verbose, optimizer_options));
            }

            if self.flags.contains(TestFlags::NO_STRICT) || self.flags.contains(TestFlags::RAW) {
                results.push(self.run_once(harness, false, verbose, optimizer_options));
            }
        }

        results
    }

    /// Runs the test once, in strict or non-strict mode
    fn run_once(
        &self,
        harness: &Harness,
        strict: bool,
        verbose: u8,
        optimizer_options: OptimizerOptions,
    ) -> TestResult {
        let Ok(source) = Source::from_filepath(&self.path) else {
            if verbose > 1 {
                println!(
                    "`{}`{}: {}",
                    self.path.display(),
                    if strict { " (strict mode)" } else { "" },
                    "Invalid file".red()
                );
            } else {
                print!("{}", "F".red());
            }
            return TestResult {
                name: self.name.clone(),
                edition: self.edition,
                strict,
                result: TestOutcomeResult::Failed,
                result_text: Box::from("Could not read test file."),
            };
        };
        if self.ignored {
            if verbose > 1 {
                println!(
                    "`{}`{}: {}",
                    self.path.display(),
                    if strict { " (strict mode)" } else { "" },
                    "Ignored".yellow()
                );
            } else {
                print!("{}", "-".yellow());
            }
            return TestResult {
                name: self.name.clone(),
                edition: self.edition,
                strict,
                result: TestOutcomeResult::Ignored,
                result_text: Box::default(),
            };
        }
        if verbose > 1 {
            println!(
                "`{}`{}: starting",
                self.path.display(),
                if strict { " (strict mode)" } else { "" }
            );
        }

        let result = std::panic::catch_unwind(|| match self.expected_outcome {
            Outcome::Positive => {
                let async_result = AsyncResult::default();
                let loader = &SimpleModuleLoader::new(
                    self.path.parent().expect("test should have a parent dir"),
                )
                .expect("test path should be canonicalizable");
                let dyn_loader: &dyn ModuleLoader = loader;
                let context = &mut Context::builder()
                    .module_loader(dyn_loader)
                    .build()
                    .expect("cannot fail with default global object");

                if let Err(e) = self.set_up_env(harness, context, async_result.clone()) {
                    return (false, e);
                }

                context.set_optimizer_options(optimizer_options);

                // TODO: timeout
                let value = if self.is_module() {
                    let module = match Module::parse(source, None, context) {
                        Ok(module) => module,
                        Err(err) => return (false, format!("Uncaught {err}")),
                    };

                    loader.insert(
                        self.path
                            .canonicalize()
                            .expect("test path should be canonicalizable"),
                        module.clone(),
                    );

                    let promise = match module.load_link_evaluate(context) {
                        Ok(promise) => promise,
                        Err(err) => return (false, format!("Uncaught {err}")),
                    };

                    context.run_jobs();

                    match promise
                        .state()
                        .expect("tester can only use builtin promises")
                    {
                        PromiseState::Pending => {
                            return (false, "module should have been executed".to_string())
                        }
                        PromiseState::Fulfilled(v) => v,
                        PromiseState::Rejected(err) => {
                            let output = JsError::from_opaque(err.clone())
                                .try_native(context)
                                .map_or_else(
                                    |_| format!("Uncaught {}", err.display()),
                                    |err| {
                                        format!(
                                            "Uncaught {err}{}",
                                            err.cause().map_or_else(String::new, |cause| format!(
                                                "\n  caused by {cause}"
                                            ))
                                        )
                                    },
                                );

                            return (false, output);
                        }
                    }
                } else {
                    context.strict(strict);
                    match context.eval(source) {
                        Ok(v) => v,
                        Err(err) => return (false, format!("Uncaught {err}")),
                    }
                };

                context.run_jobs();

                match *async_result.inner.borrow() {
                    UninitResult::Err(ref e) => return (false, format!("Uncaught {e}")),
                    UninitResult::Uninit if self.flags.contains(TestFlags::ASYNC) => {
                        return (
                            false,
                            "async test did not print \"Test262:AsyncTestComplete\"".to_string(),
                        )
                    }
                    _ => {}
                }

                (true, value.display().to_string())
            }
            Outcome::Negative {
                phase: Phase::Parse,
                error_type,
            } => {
                assert_eq!(
                    error_type,
                    ErrorType::SyntaxError,
                    "non-SyntaxError parsing/early error found in {}",
                    self.path.display()
                );

                let context = &mut Context::default();

                context.set_optimizer_options(OptimizerOptions::OPTIMIZE_ALL);

                if self.is_module() {
                    match Module::parse(source, None, context) {
                        Ok(_) => (false, "ModuleItemList parsing should fail".to_owned()),
                        Err(e) => (true, format!("Uncaught {e}")),
                    }
                } else {
                    context.strict(strict);
                    match Script::parse(source, None, context) {
                        Ok(_) => (false, "StatementList parsing should fail".to_owned()),
                        Err(e) => (true, format!("Uncaught {e}")),
                    }
                }
            }
            Outcome::Negative {
                phase: Phase::Resolution,
                error_type,
            } => {
                let loader = &SimpleModuleLoader::new(
                    self.path.parent().expect("test should have a parent dir"),
                )
                .expect("test path should be canonicalizable");
                let dyn_loader: &dyn ModuleLoader = loader;
                let context = &mut Context::builder()
                    .module_loader(dyn_loader)
                    .build()
                    .expect("cannot fail with default global object");

                let module = match Module::parse(source, None, context) {
                    Ok(module) => module,
                    Err(err) => return (false, format!("Uncaught {err}")),
                };

                loader.insert(
                    self.path
                        .canonicalize()
                        .expect("test path should be canonicalizable"),
                    module.clone(),
                );

                let promise = module.load(context);

                context.run_jobs();

                match promise
                    .state()
                    .expect("tester can only use builtin promises")
                {
                    PromiseState::Pending => {
                        return (false, "module didn't try to load".to_string())
                    }
                    PromiseState::Fulfilled(_) => {
                        // Try to link to see if the resolution error shows there.
                    }
                    PromiseState::Rejected(err) => {
                        let err = JsError::from_opaque(err);
                        return (
                            is_error_type(&err, error_type, context),
                            format!("Uncaught {err}"),
                        );
                    }
                }

                if let Err(err) = module.link(context) {
                    (
                        is_error_type(&err, error_type, context),
                        format!("Uncaught {err}"),
                    )
                } else {
                    (false, "module resolution didn't fail".to_string())
                }
            }
            Outcome::Negative {
                phase: Phase::Runtime,
                error_type,
            } => {
                let loader = &SimpleModuleLoader::new(
                    self.path.parent().expect("test should have a parent dir"),
                )
                .expect("test path should be canonicalizable");
                let dyn_loader: &dyn ModuleLoader = loader;
                let context = &mut Context::builder()
                    .module_loader(dyn_loader)
                    .build()
                    .expect("cannot fail with default global object");
                context.strict(strict);
                context.set_optimizer_options(optimizer_options);

                if let Err(e) = self.set_up_env(harness, context, AsyncResult::default()) {
                    return (false, e);
                }
                let error = if self.is_module() {
                    let module = match Module::parse(source, None, context) {
                        Ok(module) => module,
                        Err(e) => return (false, format!("Uncaught {e}")),
                    };

                    loader.insert(
                        self.path
                            .canonicalize()
                            .expect("test path should be canonicalizable"),
                        module.clone(),
                    );

                    let promise = module.load(context);

                    context.run_jobs();

                    match promise
                        .state()
                        .expect("tester can only use builtin promises")
                    {
                        PromiseState::Pending => {
                            return (false, "module didn't try to load".to_string())
                        }
                        PromiseState::Fulfilled(_) => {}
                        PromiseState::Rejected(err) => {
                            return (false, format!("Uncaught {}", err.display()))
                        }
                    }

                    if let Err(err) = module.link(context) {
                        return (false, format!("Uncaught {err}"));
                    }

                    let promise = module.evaluate(context);

                    context.run_jobs();

                    match promise
                        .state()
                        .expect("tester can only use builtin promises")
                    {
                        PromiseState::Pending => {
                            return (false, "module didn't try to evaluate".to_string())
                        }
                        PromiseState::Fulfilled(val) => return (false, val.display().to_string()),
                        PromiseState::Rejected(err) => JsError::from_opaque(err),
                    }
                } else {
                    context.strict(strict);
                    let script = match Script::parse(source, None, context) {
                        Ok(code) => code,
                        Err(e) => return (false, format!("Uncaught {e}")),
                    };

                    match script.evaluate(context) {
                        Ok(_) => return (false, "Script execution should fail".to_owned()),
                        Err(e) => e,
                    }
                };

                (
                    is_error_type(&error, error_type, context),
                    format!("Uncaught {error}"),
                )
            }
        });

        let (result, result_text) = result.map_or_else(
            |_| {
                eprintln!("last panic was on test \"{}\"", self.path.display());
                (TestOutcomeResult::Panic, String::new())
            },
            |(res, text)| {
                if res {
                    (TestOutcomeResult::Passed, text)
                } else {
                    (TestOutcomeResult::Failed, text)
                }
            },
        );

        if verbose > 1 {
            println!(
                "`{}`{}: {}",
                self.path.display(),
                if strict { " (strict mode)" } else { "" },
                if result == TestOutcomeResult::Passed {
                    "Passed".green()
                } else if result == TestOutcomeResult::Failed {
                    "Failed".red()
                } else {
                    "⚠ Panic ⚠".red()
                }
            );
        } else {
            print!(
                "{}",
                if result == TestOutcomeResult::Passed {
                    ".".green()
                } else {
                    "F".red()
                }
            );
        }

        if verbose > 2 {
            println!(
                "`{}`{}: result text",
                self.path.display(),
                if strict { " (strict mode)" } else { "" },
            );
            println!("{result_text}");
            println!();
        }

        TestResult {
            name: self.name.clone(),
            edition: self.edition,
            strict,
            result,
            result_text: result_text.into_boxed_str(),
        }
    }

    /// Sets the environment up to run the test.
    fn set_up_env(
        &self,
        harness: &Harness,
        context: &mut Context<'_>,
        async_result: AsyncResult,
    ) -> Result<(), String> {
        // Register the print() function.
        register_print_fn(context, async_result);

        // add the $262 object.
        let _js262 = js262::register_js262(context);

        if self.flags.contains(TestFlags::RAW) {
            return Ok(());
        }

        let assert = Source::from_reader(
            harness.assert.content.as_bytes(),
            Some(&harness.assert.path),
        );
        let sta = Source::from_reader(harness.sta.content.as_bytes(), Some(&harness.sta.path));

        context
            .eval(assert)
            .map_err(|e| format!("could not run assert.js:\n{e}"))?;
        context
            .eval(sta)
            .map_err(|e| format!("could not run sta.js:\n{e}"))?;

        if self.flags.contains(TestFlags::ASYNC) {
            let dph = Source::from_reader(
                harness.doneprint_handle.content.as_bytes(),
                Some(&harness.doneprint_handle.path),
            );
            context
                .eval(dph)
                .map_err(|e| format!("could not run doneprintHandle.js:\n{e}"))?;
        }

        for include_name in &self.includes {
            let include = harness
                .includes
                .get(include_name)
                .ok_or_else(|| format!("could not find the {include_name} include file."))?;
            let source = Source::from_reader(include.content.as_bytes(), Some(&include.path));
            context.eval(source).map_err(|e| {
                format!("could not run the harness `{include_name}`:\nUncaught {e}",)
            })?;
        }

        Ok(())
    }
}

/// Returns `true` if `error` is a `target_type` error.
fn is_error_type(error: &JsError, target_type: ErrorType, context: &mut Context<'_>) -> bool {
    if let Ok(error) = error.try_native(context) {
        match &error.kind {
            JsNativeErrorKind::Syntax if target_type == ErrorType::SyntaxError => {}
            JsNativeErrorKind::Reference if target_type == ErrorType::ReferenceError => {}
            JsNativeErrorKind::Range if target_type == ErrorType::RangeError => {}
            JsNativeErrorKind::Type if target_type == ErrorType::TypeError => {}
            _ => return false,
        }
        true
    } else {
        let passed = error
            .as_opaque()
            .expect("try_native cannot fail if e is not opaque")
            .as_object()
            .and_then(|o| o.get(js_string!("constructor"), context).ok())
            .as_ref()
            .and_then(JsValue::as_object)
            .and_then(|o| o.get(js_string!("name"), context).ok())
            .as_ref()
            .and_then(JsValue::as_string)
            .map(|s| s == target_type.as_str())
            .unwrap_or_default();
        passed
    }
}

/// Registers the print function in the context.
fn register_print_fn(context: &mut Context<'_>, async_result: AsyncResult) {
    // We use `FunctionBuilder` to define a closure with additional captures.
    let js_function = FunctionObjectBuilder::new(
        context.realm(),
        // SAFETY: `AsyncResult` has only non-traceable captures, making this safe.
        unsafe {
            NativeFunction::from_closure(move |_, args, context| {
                let message = args
                    .get_or_undefined(0)
                    .to_string(context)?
                    .to_std_string_escaped();
                let mut result = async_result.inner.borrow_mut();

                match *result {
                    UninitResult::Uninit | UninitResult::Ok(_) => {
                        if message == "Test262:AsyncTestComplete" {
                            *result = UninitResult::Ok(());
                        } else {
                            *result = UninitResult::Err(message);
                        }
                    }
                    UninitResult::Err(_) => {}
                }

                Ok(JsValue::undefined())
            })
        },
    )
    .name("print")
    .length(1)
    .build();

    context
        .register_global_property(
            js_string!("print"),
            js_function,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .expect("shouldn't fail with the default global");
}

/// A `Result` value that is possibly uninitialized.
///
/// This is mainly used to check if an async test did call `print` to signal the termination of
/// a test. Otherwise, all async tests that result in `UninitResult::Uninit` are considered
/// as failed.
///
/// The Test262 [interpreting guide][guide] contains more information about how to run async tests.
///
/// [guide]: https://github.com/tc39/test262/blob/main/INTERPRETING.md#flags
#[derive(Debug, Clone, Copy, Default)]
enum UninitResult<T, E> {
    #[default]
    Uninit,
    Ok(T),
    Err(E),
}

/// Object which includes the result of the async operation.
#[derive(Debug, Clone)]
struct AsyncResult {
    inner: Rc<RefCell<UninitResult<(), String>>>,
}

impl Default for AsyncResult {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(UninitResult::default())),
        }
    }
}
