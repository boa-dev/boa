//! Execution module for the test runner.

mod js262;

use crate::{
    read::ErrorType, Harness, Outcome, Phase, SuiteResult, Test, TestFlags, TestOutcomeResult,
    TestResult, TestSuite,
};
use boa_engine::{
    context::ContextBuilder, job::SimpleJobQueue, native_function::NativeFunction,
    object::FunctionObjectBuilder, property::Attribute, Context, JsArgs, JsNativeErrorKind,
    JsValue, Source,
};
use colored::Colorize;
use rayon::prelude::*;
use std::{cell::RefCell, rc::Rc};

impl TestSuite {
    /// Runs the test suite.
    pub(crate) fn run(&self, harness: &Harness, verbose: u8, parallel: bool) -> SuiteResult {
        if verbose != 0 {
            println!("Suite {}:", self.path.display());
        }

        let suites: Vec<_> = if parallel {
            self.suites
                .par_iter()
                .map(|suite| suite.run(harness, verbose, parallel))
                .collect()
        } else {
            self.suites
                .iter()
                .map(|suite| suite.run(harness, verbose, parallel))
                .collect()
        };

        let tests: Vec<_> = if parallel {
            self.tests
                .par_iter()
                .flat_map(|test| test.run(harness, verbose))
                .collect()
        } else {
            self.tests
                .iter()
                .flat_map(|test| test.run(harness, verbose))
                .collect()
        };

        let mut features = Vec::new();
        for test_iter in self.tests.iter() {
            for feature_iter in test_iter.features.iter() {
                features.push(feature_iter.to_string());
            }
        }

        if verbose != 0 {
            println!();
        }

        // Count passed tests
        let mut passed = 0;
        let mut ignored = 0;
        let mut panic = 0;
        for test in &tests {
            match test.result {
                TestOutcomeResult::Passed => passed += 1,
                TestOutcomeResult::Ignored => ignored += 1,
                TestOutcomeResult::Panic => panic += 1,
                TestOutcomeResult::Failed => {}
            }
        }

        // Count total tests
        let mut total = tests.len();
        for suite in &suites {
            total += suite.total;
            passed += suite.passed;
            ignored += suite.ignored;
            panic += suite.panic;
            features.append(&mut suite.features.clone());
        }

        if verbose != 0 {
            println!(
                "Suite {} results: total: {total}, passed: {}, ignored: {}, failed: {} (panics: \
                    {}{}), conformance: {:.2}%",
                self.path.display(),
                passed.to_string().green(),
                ignored.to_string().yellow(),
                (total - passed - ignored).to_string().red(),
                if panic == 0 {
                    "0".normal()
                } else {
                    panic.to_string().red()
                },
                if panic == 0 { "" } else { " ⚠" }.red(),
                (passed as f64 / total as f64) * 100.0
            );
        }

        SuiteResult {
            name: self.name.clone(),
            total,
            passed,
            ignored,
            panic,
            suites,
            tests,
            features,
        }
    }
}

impl Test {
    /// Runs the test.
    pub(crate) fn run(&self, harness: &Harness, verbose: u8) -> Vec<TestResult> {
        let mut results = Vec::new();
        if self.flags.contains(TestFlags::STRICT) && !self.flags.contains(TestFlags::RAW) {
            results.push(self.run_once(harness, true, verbose));
        }

        if self.flags.contains(TestFlags::NO_STRICT) || self.flags.contains(TestFlags::RAW) {
            results.push(self.run_once(harness, false, verbose));
        }

        results
    }

    /// Runs the test once, in strict or non-strict mode
    fn run_once(&self, harness: &Harness, strict: bool, verbose: u8) -> TestResult {
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
                strict,
                result: TestOutcomeResult::Failed,
                result_text: Box::from("Could not read test file.")
            }
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
                let queue = SimpleJobQueue::new();
                let context = &mut ContextBuilder::new()
                    .job_queue(&queue)
                    .build()
                    .expect("cannot fail with default global");

                if let Err(e) = self.set_up_env(harness, context, async_result.clone()) {
                    return (false, e);
                }
                context.strict(strict);

                // TODO: timeout
                let value = match if self.is_module() {
                    context.eval_module(source)
                } else {
                    context.eval_script(source)
                } {
                    Ok(v) => v,
                    Err(e) => return (false, format!("Uncaught {e}")),
                };

                context.run_jobs();

                if let Err(e) = async_result.inner.borrow().as_ref() {
                    return (false, format!("Uncaught {e}"));
                }

                (true, value.display().to_string())
            }
            Outcome::Negative {
                phase: Phase::Parse | Phase::Early,
                error_type,
            } => {
                assert_eq!(
                    error_type,
                    ErrorType::SyntaxError,
                    "non-SyntaxError parsing/early error found in {}",
                    self.path.display()
                );

                let context = &mut Context::default();
                context.strict(strict);
                if self.is_module() {
                    match context.parse_module(source) {
                        Ok(module_item_list) => match context.compile_module(&module_item_list) {
                            Ok(_) => (false, "ModuleItemList compilation should fail".to_owned()),
                            Err(e) => (true, format!("Uncaught {e:?}")),
                        },
                        Err(e) => (true, format!("Uncaught {e}")),
                    }
                } else {
                    match context.parse_script(source) {
                        Ok(statement_list) => match context.compile_script(&statement_list) {
                            Ok(_) => (false, "StatementList compilation should fail".to_owned()),
                            Err(e) => (true, format!("Uncaught {e:?}")),
                        },
                        Err(e) => (true, format!("Uncaught {e}")),
                    }
                }
            }
            Outcome::Negative {
                phase: Phase::Resolution,
                error_type: _,
            } => todo!("check module resolution errors"),
            Outcome::Negative {
                phase: Phase::Runtime,
                error_type,
            } => {
                let context = &mut Context::default();
                context.strict(strict);
                if let Err(e) = self.set_up_env(harness, context, AsyncResult::default()) {
                    return (false, e);
                }
                let code = if self.is_module() {
                    match context
                        .parse_module(source)
                        .map_err(Into::into)
                        .and_then(|stmts| context.compile_module(&stmts))
                    {
                        Ok(code) => code,
                        Err(e) => return (false, format!("Uncaught {e}")),
                    }
                } else {
                    match context
                        .parse_script(source)
                        .map_err(Into::into)
                        .and_then(|stmts| context.compile_script(&stmts))
                    {
                        Ok(code) => code,
                        Err(e) => return (false, format!("Uncaught {e}")),
                    }
                };

                let e = match context.execute(code) {
                    Ok(res) => return (false, res.display().to_string()),
                    Err(e) => e,
                };
                if let Ok(e) = e.try_native(context) {
                    match &e.kind {
                        JsNativeErrorKind::Syntax if error_type == ErrorType::SyntaxError => {}
                        JsNativeErrorKind::Reference if error_type == ErrorType::ReferenceError => {
                        }
                        JsNativeErrorKind::Range if error_type == ErrorType::RangeError => {}
                        JsNativeErrorKind::Type if error_type == ErrorType::TypeError => {}
                        _ => return (false, format!("Uncaught {e}")),
                    }
                    (true, format!("Uncaught {e}"))
                } else {
                    let passed = e
                        .as_opaque()
                        .expect("try_native cannot fail if e is not opaque")
                        .as_object()
                        .and_then(|o| o.get("constructor", context).ok())
                        .as_ref()
                        .and_then(JsValue::as_object)
                        .and_then(|o| o.get("name", context).ok())
                        .as_ref()
                        .and_then(JsValue::as_string)
                        .map(|s| s == error_type.as_str())
                        .unwrap_or_default();
                    (passed, format!("Uncaught {e}"))
                }
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
            .eval_script(assert)
            .map_err(|e| format!("could not run assert.js:\n{e}"))?;
        context
            .eval_script(sta)
            .map_err(|e| format!("could not run sta.js:\n{e}"))?;

        if self.flags.contains(TestFlags::ASYNC) {
            let dph = Source::from_reader(
                harness.doneprint_handle.content.as_bytes(),
                Some(&harness.doneprint_handle.path),
            );
            context
                .eval_script(dph)
                .map_err(|e| format!("could not run doneprintHandle.js:\n{e}"))?;
        }

        for include_name in self.includes.iter() {
            let include = harness
                .includes
                .get(include_name)
                .ok_or_else(|| format!("could not find the {include_name} include file."))?;
            let source = Source::from_reader(include.content.as_bytes(), Some(&include.path));
            context.eval_script(source).map_err(|e| {
                format!("could not run the harness `{include_name}`:\nUncaught {e}",)
            })?;
        }

        Ok(())
    }
}

/// Registers the print function in the context.
fn register_print_fn(context: &mut Context<'_>, async_result: AsyncResult) {
    // We use `FunctionBuilder` to define a closure with additional captures.
    let js_function = FunctionObjectBuilder::new(
        context,
        // SAFETY: `AsyncResult` has only non-traceable captures, making this safe.
        unsafe {
            NativeFunction::from_closure(move |_, args, context| {
                let message = args
                    .get_or_undefined(0)
                    .to_string(context)?
                    .to_std_string_escaped();
                if message != "Test262:AsyncTestComplete" {
                    *async_result.inner.borrow_mut() = Err(message);
                }
                Ok(JsValue::undefined())
            })
        },
    )
    .name("print")
    .length(1)
    .build();

    context.register_global_property(
        "print",
        js_function,
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
    );
}
/// Object which includes the result of the async operation.
#[derive(Debug, Clone)]
struct AsyncResult {
    inner: Rc<RefCell<Result<(), String>>>,
}

impl Default for AsyncResult {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Ok(()))),
        }
    }
}
