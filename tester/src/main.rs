//! Test262 test runner
//!
//! This crate will run the full ECMAScript test suite (Test262) and report compliance of the
//! `boa` engine.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/jasonwilliams/boa/master/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/jasonwilliams/boa/master/assets/logo.svg"
)]
#![deny(
    unused_qualifications,
    clippy::all,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    // rustdoc,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
)]
#![warn(clippy::perf, clippy::single_match_else, clippy::dbg_macro)]
#![allow(
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions,
    clippy::let_unit_value,
    missing_doc_code_examples
)]

mod exec;
mod read;

use self::read::{read_global_suite, read_harness, MetaData, Negative, TestFlag};
use bitflags::bitflags;
use fxhash::FxHashMap;
use serde::Deserialize;
use std::path::PathBuf;
use structopt::StructOpt;

/// Boa test262 tester
#[derive(StructOpt, Debug)]
#[structopt(name = "Boa test262 tester")]
struct Cli {
    // Whether to show verbose output.
    #[structopt(short, long)]
    verbose: bool,

    /// Path to the Test262 suite.
    #[structopt(short, long, parse(from_os_str), default_value = "./test262")]
    suite: PathBuf,

    // Run the tests only in parsing mode.
    #[structopt(short = "p", long)]
    only_parse: bool,
}

/// Program entry point.
fn main() {
    let cli = Cli::from_args();

    let harness =
        read_harness(cli.suite.as_path()).expect("could not read initialization bindings");

    let global_suite =
        read_global_suite(cli.suite.as_path()).expect("could not get the list of tests to run");

    let results = global_suite.run(&harness);

    println!("Results:");
    println!("Total tests: {}", results.total_tests);
    println!("Passed tests: {}", results.passed_tests);
    println!(
        "Conformance: {:.2}%",
        (results.passed_tests as f64 / results.total_tests as f64) * 100.0
    )
}

/// All the harness include files.
#[derive(Debug, Clone)]
struct Harness {
    assert: Box<str>,
    sta: Box<str>,
    includes: FxHashMap<Box<str>, Box<str>>,
}

/// Represents a test suite.
#[derive(Debug, Clone)]
struct TestSuite {
    name: Box<str>,
    suites: Box<[TestSuite]>,
    tests: Box<[Test]>,
}

/// Outcome of a test suite.
#[derive(Debug, Clone)]
struct SuiteResult {
    name: Box<str>,
    passed: bool,
    total_tests: usize,
    passed_tests: usize,
    ignored_tests: usize,
    suites: Box<[SuiteResult]>,
    tests: Box<[TestResult]>,
}

/// Outcome of a test.
#[derive(Debug, Clone)]
struct TestResult {
    name: Box<str>,
    passed: Option<bool>,
}

/// Represents a test.
#[derive(Debug, Clone)]
struct Test {
    name: Box<str>,
    description: Box<str>,
    esid: Option<Box<str>>,
    flags: TestFlags,
    information: Box<str>,
    features: Box<[Box<str>]>,
    expected_outcome: Outcome,
    includes: Box<[Box<str>]>,
    locale: Locale,
    content: Box<str>,
}

impl Test {
    /// Creates a new test.
    #[inline]
    fn new<N, C>(name: N, content: C, metadata: MetaData) -> Self
    where
        N: Into<Box<str>>,
        C: Into<Box<str>>,
    {
        Self {
            name: name.into(),
            description: metadata.description,
            esid: metadata.esid,
            flags: metadata.flags.into(),
            information: metadata.info,
            features: metadata.features,
            expected_outcome: Outcome::from(metadata.negative),
            includes: metadata.includes,
            locale: metadata.locale,
            content: content.into(),
        }
    }
}

/// An outcome for a test.
#[derive(Debug, Clone)]
enum Outcome {
    Positive,
    Negative { phase: Phase, error_type: Box<str> },
}

impl Default for Outcome {
    fn default() -> Self {
        Self::Positive
    }
}

impl From<Option<Negative>> for Outcome {
    fn from(neg: Option<Negative>) -> Self {
        neg.map(|neg| Self::Negative {
            phase: neg.phase,
            error_type: neg.error_type,
        })
        .unwrap_or_default()
    }
}

bitflags! {
    struct TestFlags: u16 {
        const STRICT = 0b000000001;
        const NO_STRICT = 0b000000010;
        const MODULE = 0b000000100;
        const RAW = 0b000001000;
        const ASYNC = 0b000010000;
        const GENERATED = 0b000100000;
        const CAN_BLOCK_IS_FALSE = 0b001000000;
        const CAN_BLOCK_IS_TRUE = 0b010000000;
        const NON_DETERMINISTIC = 0b100000000;
    }
}

impl Default for TestFlags {
    fn default() -> Self {
        Self::STRICT | Self::NO_STRICT
    }
}

impl From<TestFlag> for TestFlags {
    fn from(flag: TestFlag) -> Self {
        match flag {
            TestFlag::OnlyStrict => Self::STRICT,
            TestFlag::NoStrict => Self::NO_STRICT,
            TestFlag::Module => Self::MODULE,
            TestFlag::Raw => Self::RAW,
            TestFlag::Async => Self::ASYNC,
            TestFlag::Generated => Self::GENERATED,
            TestFlag::CanBlockIsFalse => Self::CAN_BLOCK_IS_FALSE,
            TestFlag::CanBlockIsTrue => Self::CAN_BLOCK_IS_TRUE,
            TestFlag::NonDeterministic => Self::NON_DETERMINISTIC,
        }
    }
}

impl<T> From<T> for TestFlags
where
    T: AsRef<[TestFlag]>,
{
    fn from(flags: T) -> Self {
        let flags = flags.as_ref();
        if flags.is_empty() {
            Self::default()
        } else {
            let mut result = Self::empty();
            for flag in flags {
                result |= Self::from(*flag);
            }

            if !result.intersects(Self::default()) {
                result |= Self::default()
            }

            result
        }
    }
}

/// Phase for an error.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Phase {
    Parse,
    Early,
    Resolution,
    Runtime,
}

/// Locale information structure.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(transparent)]
struct Locale {
    locale: Box<[Box<str>]>,
}
