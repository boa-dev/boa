//! Execution module for the test runner.

mod js262;

use super::{
    Harness, Outcome, Phase, SuiteResult, Test, TestFlags, TestOutcomeResult, TestResult,
    TestSuite, IGNORED,
};
use boa::{syntax::Parser, Context, Interner, JsValue};
use colored::Colorize;
use rayon::prelude::*;
use std::panic;

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
        }
    }
}

impl Test {
    /// Runs the test.
    pub(crate) fn run(&self, harness: &Harness, verbose: u8) -> Vec<TestResult> {
        let mut results = Vec::new();
        if self.flags.contains(TestFlags::STRICT) {
            results.push(self.run_once(harness, true, verbose));
        }

        if self.flags.contains(TestFlags::NO_STRICT) || self.flags.contains(TestFlags::RAW) {
            results.push(self.run_once(harness, false, verbose));
        }

        results
    }

    /// Runs the test once, in strict or non-strict mode
    fn run_once(&self, harness: &Harness, strict: bool, verbose: u8) -> TestResult {
        if verbose > 1 {
            println!(
                "`{}`{}: starting",
                self.name,
                if strict { " (strict mode)" } else { "" }
            );
        }

        let (result, result_text) = if !IGNORED.contains_any_flag(self.flags)
            && !IGNORED.contains_test(&self.name)
            && !IGNORED.contains_any_feature(&self.features)
            && (matches!(self.expected_outcome, Outcome::Positive)
                || matches!(
                    self.expected_outcome,
                    Outcome::Negative {
                        phase: Phase::Parse,
                        error_type: _,
                    }
                )
                || matches!(
                    self.expected_outcome,
                    Outcome::Negative {
                        phase: Phase::Early,
                        error_type: _,
                    }
                )
                || matches!(
                    self.expected_outcome,
                    Outcome::Negative {
                        phase: Phase::Runtime,
                        error_type: _,
                    }
                )) {
            let res = panic::catch_unwind(|| match self.expected_outcome {
                Outcome::Positive => {
                    // TODO: implement async and add `harness/doneprintHandle.js` to the includes.

                    match self.set_up_env(harness, strict) {
                        Ok(mut context) => {
                            context.set_strict_mode(strict);
                            let res = context.eval(&self.content.as_ref());

                            let passed = res.is_ok();
                            let text = match res {
                                Ok(val) => val.display().to_string(),
                                Err(e) => format!("Uncaught {}", e.display()),
                            };

                            (passed, text)
                        }
                        Err(e) => (false, e),
                    }
                }
                Outcome::Negative {
                    phase: Phase::Parse | Phase::Early,
                    ref error_type,
                } => {
                    assert_eq!(
                        error_type.as_ref(),
                        "SyntaxError",
                        "non-SyntaxError parsing/early error found in {}",
                        self.name
                    );

                    let mut interner = Interner::default();
                    match Parser::new(self.content.as_bytes(), strict).parse_all(&mut interner) {
                        Ok(n) => (false, format!("{n:?}")),
                        Err(e) => (true, format!("Uncaught {e}")),
                    }
                }
                Outcome::Negative {
                    phase: Phase::Resolution,
                    error_type: _,
                } => todo!("check module resolution errors"),
                Outcome::Negative {
                    phase: Phase::Runtime,
                    ref error_type,
                } => {
                    let mut interner = Interner::default();
                    if let Err(e) =
                        Parser::new(self.content.as_bytes(), strict).parse_all(&mut interner)
                    {
                        (false, format!("Uncaught {e}"))
                    } else {
                        match self.set_up_env(harness, strict) {
                            Ok(mut context) => {
                                context.set_strict_mode(strict);
                                match context.eval(&self.content.as_ref()) {
                                    Ok(res) => (false, res.display().to_string()),
                                    Err(e) => {
                                        let passed =
                                            e.display().to_string().contains(error_type.as_ref());

                                        (passed, format!("Uncaught {}", e.display()))
                                    }
                                }
                            }
                            Err(e) => (false, e),
                        }
                    }
                }
            });

            let result = res.map_or_else(
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
                    if matches!(result, (TestOutcomeResult::Passed, _)) {
                        "Passed".green()
                    } else if matches!(result, (TestOutcomeResult::Failed, _)) {
                        "Failed".red()
                    } else {
                        "⚠ Panic ⚠".red()
                    }
                );
            } else {
                print!(
                    "{}",
                    if matches!(result, (TestOutcomeResult::Passed, _)) {
                        ".".green()
                    } else {
                        ".".red()
                    }
                );
            }

            result
        } else {
            if verbose > 1 {
                println!(
                    "`{}`{}: {}",
                    self.name,
                    if strict { " (strict mode)" } else { "" },
                    "Ignored".yellow()
                );
            } else {
                print!("{}", ".".yellow());
            }
            (TestOutcomeResult::Ignored, String::new())
        };

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
    fn set_up_env(&self, harness: &Harness, strict: bool) -> Result<Context, String> {
        // Create new Realm
        let mut context = Context::default();

        // Register the print() function.
        context.register_global_function("print", 1, test262_print);

        // add the $262 object.
        let _js262 = js262::init(&mut context);

        if strict {
            context
                .eval(r#""use strict";"#)
                .map_err(|e| format!("could not set strict mode:\n{}", e.display()))?;
        }

        context
            .eval(&harness.assert.as_ref())
            .map_err(|e| format!("could not run assert.js:\n{}", e.display()))?;
        context
            .eval(&harness.sta.as_ref())
            .map_err(|e| format!("could not run sta.js:\n{}", e.display()))?;

        for include in self.includes.iter() {
            context
                .eval(
                    &harness
                        .includes
                        .get(include)
                        .ok_or_else(|| format!("could not find the {include} include file."))?
                        .as_ref(),
                )
                .map_err(|e| {
                    format!(
                        "could not run the {include} include file:\nUncaught {}",
                        e.display()
                    )
                })?;
        }

        Ok(context)
    }
}

/// `print()` function required by the test262 suite.
fn test262_print(_this: &JsValue, _: &[JsValue], _context: &mut Context) -> boa::JsResult<JsValue> {
    todo!("print() function");
}
