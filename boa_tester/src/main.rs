//! Test262 test runner
//!
//! This crate will run the full ECMAScript test suite (Test262) and report compliance of the
//! `boa` context.
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![warn(
    clippy::perf,
    clippy::single_match_else,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::struct_excessive_bools,
    clippy::doc_markdown,
    clippy::semicolon_if_nothing_returned,
    clippy::pedantic
)]
#![deny(
    clippy::all,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::use_self,
    clippy::unnested_or_patterns,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_unwrap_or,
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
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_ptr_alignment,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions,
    clippy::let_unit_value,
    rustdoc::missing_doc_code_examples
)]

mod exec;
mod read;
mod results;

use self::{
    read::{read_harness, read_suite, read_test, MetaData, Negative, TestFlag},
    results::{compare_results, write_json},
};
use anyhow::{bail, Context};
use bitflags::bitflags;
use colored::Colorize;
use fxhash::{FxHashMap, FxHashSet};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

/// Structure to allow defining ignored tests, features and files that should
/// be ignored even when reading.
#[derive(Debug)]
struct Ignored {
    tests: FxHashSet<Box<str>>,
    features: FxHashSet<Box<str>>,
    files: FxHashSet<Box<str>>,
    flags: TestFlags,
}

impl Ignored {
    /// Checks if the ignore list contains the given test name in the list of
    /// tests to ignore.
    pub(crate) fn contains_test(&self, test: &str) -> bool {
        self.tests.contains(test)
    }

    /// Checks if the ignore list contains the given feature name in the list
    /// of features to ignore.
    pub(crate) fn contains_any_feature(&self, features: &[Box<str>]) -> bool {
        features
            .iter()
            .any(|feature| self.features.contains(feature))
    }

    /// Checks if the ignore list contains the given file name in the list to
    /// ignore from reading.
    pub(crate) fn contains_file(&self, file: &str) -> bool {
        self.files.contains(file)
    }

    pub(crate) fn contains_any_flag(&self, flags: TestFlags) -> bool {
        flags.intersects(self.flags)
    }
}

impl Default for Ignored {
    fn default() -> Self {
        Self {
            tests: FxHashSet::default(),
            features: FxHashSet::default(),
            files: FxHashSet::default(),
            flags: TestFlags::empty(),
        }
    }
}

/// List of ignored tests.
static IGNORED: Lazy<Ignored> = Lazy::new(|| {
    let path = Path::new("test_ignore.txt");
    if path.exists() {
        let filtered = fs::read_to_string(path).expect("could not read test filters");
        filtered
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .fold(Ignored::default(), |mut ign, line| {
                // let mut line = line.to_owned();
                if line.starts_with("file:") {
                    let file = line
                        .strip_prefix("file:")
                        .expect("prefix disappeared")
                        .trim()
                        .to_owned()
                        .into_boxed_str();
                    let test = if file.ends_with(".js") {
                        file.strip_suffix(".js")
                            .expect("suffix disappeared")
                            .to_owned()
                            .into_boxed_str()
                    } else {
                        file.clone()
                    };
                    ign.files.insert(file);
                    ign.tests.insert(test);
                } else if line.starts_with("feature:") {
                    ign.features.insert(
                        line.strip_prefix("feature:")
                            .expect("prefix disappeared")
                            .trim()
                            .to_owned()
                            .into_boxed_str(),
                    );
                } else if line.starts_with("flag:") {
                    let flag = line
                        .strip_prefix("flag:")
                        .expect("prefix disappeared")
                        .trim()
                        .parse::<TestFlag>()
                        .expect("invalid flag found");
                    ign.flags.insert(flag.into());
                } else {
                    let mut test = line.trim();
                    if test
                        .rsplit('.')
                        .next()
                        .map(|ext| ext.eq_ignore_ascii_case("js"))
                        == Some(true)
                    {
                        test = test.strip_suffix(".js").expect("suffix disappeared");
                    }
                    ign.tests.insert(test.to_owned().into_boxed_str());
                }
                ign
            })
    } else {
        Ignored::default()
    }
});

/// Boa test262 tester
#[derive(StructOpt, Debug)]
#[structopt(name = "Boa test262 tester")]
enum Cli {
    /// Run the test suite.
    Run {
        /// Whether to show verbose output.
        #[structopt(short, long, parse(from_occurrences))]
        verbose: u8,

        /// Path to the Test262 suite.
        #[structopt(long, parse(from_os_str), default_value = "./test262")]
        test262_path: PathBuf,

        /// Which specific test or test suite to run. Should be a path relative to the Test262 directory: e.g. "test/language/types/number"
        #[structopt(short, long, parse(from_os_str), default_value = "test")]
        suite: PathBuf,

        /// Optional output folder for the full results information.
        #[structopt(short, long, parse(from_os_str))]
        output: Option<PathBuf>,

        /// Execute tests serially
        #[structopt(short, long)]
        disable_parallelism: bool,
    },
    Compare {
        /// Base results of the suite.
        #[structopt(parse(from_os_str))]
        base: PathBuf,

        /// New results to compare.
        #[structopt(parse(from_os_str))]
        new: PathBuf,

        /// Whether to use markdown output
        #[structopt(short, long)]
        markdown: bool,
    },
}

/// Program entry point.
fn main() {
    match Cli::from_args() {
        Cli::Run {
            verbose,
            test262_path,
            suite,
            output,
            disable_parallelism,
        } => {
            if let Err(e) = run_test_suite(
                verbose,
                !disable_parallelism,
                test262_path.as_path(),
                suite.as_path(),
                output.as_deref(),
            ) {
                eprintln!("Error: {e}");
                let mut src = e.source();
                while let Some(e) = src {
                    eprintln!("    caused by: {e}");
                    src = e.source();
                }
                std::process::exit(1);
            }
        }
        Cli::Compare {
            base,
            new,
            markdown,
        } => compare_results(base.as_path(), new.as_path(), markdown),
    }
}

/// Runs the full test suite.
fn run_test_suite(
    verbose: u8,
    parallel: bool,
    test262_path: &Path,
    suite: &Path,
    output: Option<&Path>,
) -> anyhow::Result<()> {
    if let Some(path) = output {
        if path.exists() {
            if !path.is_dir() {
                bail!("the output path must be a directory.");
            }
        } else {
            fs::create_dir_all(path).context("could not create the output directory")?;
        }
    }

    if verbose != 0 {
        println!("Loading the test suite...");
    }
    let harness = read_harness(test262_path).context("could not read harness")?;

    if suite.to_string_lossy().ends_with(".js") {
        let test = read_test(&test262_path.join(suite)).with_context(|| {
            let suite = suite.display();
            format!("could not read the test {suite}")
        })?;

        if verbose != 0 {
            println!("Test loaded, starting...");
        }
        test.run(&harness, verbose);

        println!();
    } else {
        let suite = read_suite(&test262_path.join(suite)).with_context(|| {
            let suite = suite.display();
            format!("could not read the suite {suite}")
        })?;

        if verbose != 0 {
            println!("Test suite loaded, starting tests...");
        }
        let results = suite.run(&harness, verbose, parallel);

        println!();
        println!("Results:");
        println!("Total tests: {}", results.total);
        println!("Passed tests: {}", results.passed.to_string().green());
        println!("Ignored tests: {}", results.ignored.to_string().yellow());
        println!(
            "Failed tests: {} (panics: {})",
            (results.total - results.passed - results.ignored)
                .to_string()
                .red(),
            results.panic.to_string().red()
        );
        println!(
            "Conformance: {:.2}%",
            (results.passed as f64 / results.total as f64) * 100.0
        );

        write_json(results, output, verbose)
            .context("could not write the results to the output JSON file")?;
    }

    Ok(())
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
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SuiteResult {
    #[serde(rename = "n")]
    name: Box<str>,
    #[serde(rename = "c")]
    total: usize,
    #[serde(rename = "o")]
    passed: usize,
    #[serde(rename = "i")]
    ignored: usize,
    #[serde(rename = "p")]
    panic: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(rename = "s")]
    suites: Vec<SuiteResult>,
    #[serde(rename = "t")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    tests: Vec<TestResult>,
    #[serde(rename = "f")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    features: Vec<String>,
}

/// Outcome of a test.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct TestResult {
    #[serde(rename = "n")]
    name: Box<str>,
    #[serde(rename = "s", default)]
    strict: bool,
    #[serde(skip)]
    result_text: Box<str>,
    #[serde(rename = "r")]
    result: TestOutcomeResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum TestOutcomeResult {
    #[serde(rename = "O")]
    Passed,
    #[serde(rename = "I")]
    Ignored,
    #[serde(rename = "F")]
    Failed,
    #[serde(rename = "P")]
    Panic,
}

/// Represents a test.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
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

    /// Sets the name of the test.
    fn set_name<N>(&mut self, name: N)
    where
        N: Into<Box<str>>,
    {
        self.name = name.into();
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
                result |= Self::default();
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
#[allow(dead_code)]
struct Locale {
    locale: Box<[Box<str>]>,
}
