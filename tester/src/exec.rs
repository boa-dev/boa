//! Execution module for the test runner.

use super::{Harness, Outcome, Phase, SuiteResult, Test, TestFlags, TestResult, TestSuite, CLI};
use boa::{parse, Context};
use colored::Colorize;
use fxhash::FxHashSet;
use once_cell::sync::Lazy;
use std::{fs, panic, path::Path};

/// List of ignored tests.
static IGNORED: Lazy<FxHashSet<Box<str>>> = Lazy::new(|| {
    let path = Path::new("test_ignore.txt");
    if path.exists() {
        let filtered = fs::read_to_string(path).expect("could not read test filters");
        filtered
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .map(|line| line.to_owned().into_boxed_str())
            .collect::<FxHashSet<_>>()
    } else {
        FxHashSet::default()
    }
});

impl TestSuite {
    /// Runs the test suite.
    pub(crate) fn run(&self, harness: &Harness) -> SuiteResult {
        if CLI.verbose() {
            println!("Suite {}:", self.name);
        }

        // TODO: in parallel
        let suites: Vec<_> = self.suites.iter().map(|suite| suite.run(harness)).collect();

        // TODO: in parallel
        let tests: Vec<_> = self.tests.iter().map(|test| test.run(harness)).collect();

        if CLI.verbose() {
            println!();
        }

        // Count passed tests
        let mut passed = 0;
        let mut ignored = 0;
        for test in &tests {
            if let Some(true) = test.passed {
                passed += 1;
            } else if test.passed.is_none() {
                ignored += 1;
            }
        }

        // Count total tests
        let mut total = tests.len();
        for suite in &suites {
            total += suite.total;
            passed += suite.passed;
            ignored += suite.ignored;
        }

        if CLI.verbose() {
            println!(
                "Results: total: {}, passed: {}, ignored: {}, conformance: {:.2}%",
                total,
                passed,
                ignored,
                (passed as f64 / total as f64) * 100.0
            );
        }

        SuiteResult {
            name: self.name.clone(),
            total,
            passed,
            ignored,
            suites,
            tests: tests.into_boxed_slice(),
        }
    }
}

impl Test {
    /// Runs the test.
    pub(crate) fn run(&self, harness: &Harness) -> TestResult {
        // println!("Starting `{}`", self.name);

        let passed = if !self.flags.intersects(TestFlags::ASYNC | TestFlags::MODULE)
            && !IGNORED.contains(&self.name)
        {
            let res = panic::catch_unwind(|| {
                match self.expected_outcome {
                    Outcome::Positive => {
                        let mut passed = true;

                        if self.flags.contains(TestFlags::RAW) {
                            let mut engine = self.set_up_env(&harness, false);
                            let res = engine.eval(&self.content);

                            passed = res.is_ok()
                        } else {
                            if self.flags.contains(TestFlags::STRICT) {
                                let mut engine = self.set_up_env(&harness, true);
                                let res = engine.eval(&self.content);

                                passed = res.is_ok()
                            }

                            if passed && self.flags.contains(TestFlags::NO_STRICT) {
                                let mut engine = self.set_up_env(&harness, false);
                                let res = engine.eval(&self.content);

                                passed = res.is_ok()
                            }
                        }

                        passed
                    }
                    Outcome::Negative {
                        phase: Phase::Parse,
                        ref error_type,
                    } => {
                        assert_eq!(
                            error_type.as_ref(),
                            "SyntaxError",
                            "non-SyntaxError parsing error found in {}",
                            self.name
                        );

                        parse(&self.content).is_err()
                    }
                    Outcome::Negative {
                        phase: _,
                        error_type: _,
                    } => {
                        // TODO: check the phase
                        false
                    }
                }
            });

            let passed = res.unwrap_or_else(|_| {
                eprintln!("last panic was on test \"{}\"", self.name);
                false
            });

            print!("{}", if passed { ".".green() } else { ".".red() });

            Some(passed)
        } else {
            // Ignoring async tests for now.
            // TODO: implement async and add `harness/doneprintHandle.js` to the includes.
            print!("{}", ".".yellow());
            None
        };

        TestResult {
            name: self.name.clone(),
            passed,
        }
    }

    /// Sets the environment up to run the test.
    fn set_up_env(&self, harness: &Harness, strict: bool) -> Context {
        // Create new Realm
        // TODO: in parallel.
        let mut engine = Context::new();

        // TODO: set up the environment.

        if strict {
            engine
                .eval(r#""use strict";"#)
                .expect("could not set strict mode");
        }

        engine
            .eval(&harness.assert)
            .expect("could not run assert.js");
        engine.eval(&harness.sta).expect("could not run sta.js");

        self.includes.iter().for_each(|include| {
            let res = engine.eval(
                &harness
                    .includes
                    .get(include)
                    .expect("could not find include file"),
            );
            if let Err(e) = res {
                eprintln!("could not run the {} include file.", include);
                panic!("Uncaught {}", e.display());
            }
        });

        engine
    }
}
