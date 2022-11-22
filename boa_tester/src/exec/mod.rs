//! Execution module for the test runner.

mod js262;

use super::{
    Harness, Outcome, Phase, SuiteResult, Test, TestFlags, TestOutcomeResult, TestResult, TestSuite,
};
use crate::read::ErrorType;
use boa_engine::{
    builtins::JsArgs, object::FunctionBuilder, property::Attribute, Context, JsNativeErrorKind,
    JsResult, JsValue,
};
use boa_gc::{Finalize, Gc, GcCell, Trace};
use boa_parser::Parser;
use colored::Colorize;
use rayon::prelude::*;
use std::borrow::Cow;

impl TestSuite {
    /// Runs the test suite.
    pub(crate) fn run(&self, harness: &Harness, verbose: u8, parallel: bool) -> SuiteResult {
        if verbose != 0 {
            println!("Suite {}:", self.name);
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
                self.name,
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
        if self.ignored {
            if verbose > 1 {
                println!(
                    "`{}`{}: {}",
                    self.name,
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
                self.name,
                if strict { " (strict mode)" } else { "" }
            );
        }

        let test_content = if strict {
            Cow::Owned(format!("\"use strict\";\n{}", self.content))
        } else {
            Cow::Borrowed(&*self.content)
        };

        let result = std::panic::catch_unwind(|| match self.expected_outcome {
            Outcome::Positive => {
                let mut context = Context::default();
                let async_result = AsyncResult::default();

                if let Err(e) = self.set_up_env(harness, &mut context, async_result.clone()) {
                    return (false, e);
                }

                // TODO: timeout
                let value = match context.eval(&*test_content) {
                    Ok(v) => v,
                    Err(e) => return (false, format!("Uncaught {e}")),
                };

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
                    self.name
                );

                let mut context = Context::default();
                match context.parse(&*test_content) {
                    Ok(statement_list) => match context.compile(&statement_list) {
                        Ok(_) => (false, "StatementList compilation should fail".to_owned()),
                        Err(e) => (true, format!("Uncaught {e:?}")),
                    },
                    Err(e) => (true, format!("Uncaught {e}")),
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
                let mut context = Context::default();
                if let Err(e) = self.set_up_env(harness, &mut context, AsyncResult::default()) {
                    return (false, e);
                }
                let code = match Parser::new(test_content.as_bytes())
                    .parse_all(context.interner_mut())
                    .map_err(Into::into)
                    .and_then(|stmts| context.compile(&stmts))
                {
                    Ok(code) => code,
                    Err(e) => return (false, format!("Uncaught {e}")),
                };

                // TODO: timeout
                let e = match context.execute(code) {
                    Ok(res) => return (false, res.display().to_string()),
                    Err(e) => e,
                };
                if let Ok(e) = e.try_native(&mut context) {
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
                        .and_then(|o| o.get("constructor", &mut context).ok())
                        .as_ref()
                        .and_then(JsValue::as_object)
                        .and_then(|o| o.get("name", &mut context).ok())
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
                eprintln!("last panic was on test \"{}\"", self.name);
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
                self.name,
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
                self.name,
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
        context: &mut Context,
        async_result: AsyncResult,
    ) -> Result<(), String> {
        // Register the print() function.
        Self::register_print_fn(context, async_result);

        // add the $262 object.
        let _js262 = js262::init(context);

        if self.flags.contains(TestFlags::RAW) {
            return Ok(());
        }

        context
            .eval(harness.assert.as_ref())
            .map_err(|e| format!("could not run assert.js:\n{e}"))?;
        context
            .eval(harness.sta.as_ref())
            .map_err(|e| format!("could not run sta.js:\n{e}"))?;

        if self.flags.contains(TestFlags::ASYNC) {
            context
                .eval(harness.doneprint_handle.as_ref())
                .map_err(|e| format!("could not run doneprintHandle.js:\n{e}"))?;
        }

        for include in self.includes.iter() {
            context
                .eval(
                    harness
                        .includes
                        .get(include)
                        .ok_or_else(|| format!("could not find the {include} include file."))?
                        .as_ref(),
                )
                .map_err(|e| format!("could not run the {include} include file:\nUncaught {e}"))?;
        }

        Ok(())
    }

    /// Registers the print function in the context.
    fn register_print_fn(context: &mut Context, async_result: AsyncResult) {
        // We use `FunctionBuilder` to define a closure with additional captures.
        let js_function =
            FunctionBuilder::closure_with_captures(context, test262_print, async_result)
                .name("print")
                .length(1)
                .build();

        context.register_global_property(
            "print",
            js_function,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );
    }
}

/// Object which includes the result of the async operation.
#[derive(Debug, Clone, Trace, Finalize)]
struct AsyncResult {
    inner: Gc<GcCell<Result<(), String>>>,
}

impl Default for AsyncResult {
    fn default() -> Self {
        Self {
            inner: Gc::new(GcCell::new(Ok(()))),
        }
    }
}

/// `print()` function required by the test262 suite.
#[allow(clippy::unnecessary_wraps)]
fn test262_print(
    _this: &JsValue,
    args: &[JsValue],
    async_result: &mut AsyncResult,
    context: &mut Context,
) -> JsResult<JsValue> {
    let message = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    if message != "Test262:AsyncTestComplete" {
        *async_result.inner.borrow_mut() = Err(message);
    }
    Ok(JsValue::undefined())
}
