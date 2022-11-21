//! Test262 test runner
//!
//! This crate will run the full ECMAScript test suite (Test262) and report compliance of the
//! `boa` engine.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
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
)]
#![allow(
    clippy::use_self,
    clippy::too_many_lines,
    clippy::redundant_pub_crate,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]

mod exec;
mod read;
mod results;

use self::{
    read::{read_harness, read_suite, read_test, MetaData, Negative, TestFlag},
    results::{compare_results, write_json},
};
use bitflags::bitflags;
use clap::{ArgAction, Parser, ValueHint};
use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use colored::Colorize;
use fxhash::{FxHashMap, FxHashSet};
use read::ErrorType;
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

/// Structure to allow defining ignored tests, features and files that should
/// be ignored even when reading.
#[derive(Debug, Deserialize)]
struct Ignored {
    #[serde(default)]
    tests: FxHashSet<Box<str>>,
    #[serde(default)]
    features: FxHashSet<Box<str>>,
    #[serde(default = "TestFlags::empty")]
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
    pub(crate) fn contains_feature(&self, feature: &str) -> bool {
        if self.features.contains(feature) {
            return true;
        }
        // Some features are an accessor instead of a simple feature name e.g. `Intl.DurationFormat`.
        // This ensures those are also ignored.
        feature
            .split('.')
            .next()
            .map(|feat| self.features.contains(feat))
            .unwrap_or_default()
    }

    pub(crate) const fn contains_any_flag(&self, flags: TestFlags) -> bool {
        flags.intersects(self.flags)
    }
}

impl Default for Ignored {
    fn default() -> Self {
        Self {
            tests: FxHashSet::default(),
            features: FxHashSet::default(),
            flags: TestFlags::empty(),
        }
    }
}

/// Boa test262 tester
#[derive(Debug, Parser)]
#[command(author, version, about, name = "Boa test262 tester")]
enum Cli {
    /// Run the test suite.
    Run {
        /// Whether to show verbose output.
        #[arg(short, long, action = ArgAction::Count)]
        verbose: u8,

        /// Path to the Test262 suite.
        #[arg(long, default_value = "./test262", value_hint = ValueHint::DirPath)]
        test262_path: PathBuf,

        /// Which specific test or test suite to run. Should be a path relative to the Test262 directory: e.g. "test/language/types/number"
        #[arg(short, long, default_value = "test", value_hint = ValueHint::AnyPath)]
        suite: PathBuf,

        /// Optional output folder for the full results information.
        #[arg(short, long, value_hint = ValueHint::DirPath)]
        output: Option<PathBuf>,

        /// Execute tests serially
        #[arg(short, long)]
        disable_parallelism: bool,

        /// Path to a TOML file with the ignored tests, features, flags and/or files.
        #[arg(short, long, default_value = "test_ignore.toml", value_hint = ValueHint::FilePath)]
        ignored: PathBuf,
    },
    /// Compare two test suite results.
    Compare {
        /// Base results of the suite.
        #[arg(value_hint = ValueHint::FilePath)]
        base: PathBuf,

        /// New results to compare.
        #[arg(value_hint = ValueHint::FilePath)]
        new: PathBuf,

        /// Whether to use markdown output
        #[arg(short, long)]
        markdown: bool,
    },
}

/// Program entry point.
fn main() -> Result<()> {
    color_eyre::install()?;
    match Cli::parse() {
        Cli::Run {
            verbose,
            test262_path,
            suite,
            output,
            disable_parallelism,
            ignored: ignore,
        } => run_test_suite(
            verbose,
            !disable_parallelism,
            test262_path.as_path(),
            suite.as_path(),
            output.as_deref(),
            ignore.as_path(),
        ),
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
    ignored: &Path,
) -> Result<()> {
    if let Some(path) = output {
        if path.exists() {
            if !path.is_dir() {
                bail!("the output path must be a directory.");
            }
        } else {
            fs::create_dir_all(path).wrap_err("could not create the output directory")?;
        }
    }

    let ignored = {
        let mut input = String::new();
        let mut f = File::open(ignored).wrap_err("could not open ignored tests file")?;
        f.read_to_string(&mut input)
            .wrap_err("could not read ignored tests file")?;
        toml::from_str(&input).wrap_err("could not decode ignored tests file")?
    };

    if verbose != 0 {
        println!("Loading the test suite...");
    }
    let harness = read_harness(test262_path).wrap_err("could not read harness")?;

    if suite.to_string_lossy().ends_with(".js") {
        let test = read_test(&test262_path.join(suite)).wrap_err_with(|| {
            let suite = suite.display();
            format!("could not read the test {suite}")
        })?;

        if verbose != 0 {
            println!("Test loaded, starting...");
        }
        test.run(&harness, verbose);

        println!();
    } else {
        let suite = read_suite(&test262_path.join(suite), &ignored, false).wrap_err_with(|| {
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
            .wrap_err("could not write the results to the output JSON file")?;
    }

    Ok(())
}

/// All the harness include files.
#[derive(Debug, Clone)]
struct Harness {
    assert: Box<str>,
    sta: Box<str>,
    doneprint_handle: Box<str>,
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
    ignored: bool,
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
            ignored: false,
        }
    }

    fn set_ignored(&mut self) {
        self.ignored = true;
    }
}

/// An outcome for a test.
#[derive(Debug, Clone)]
enum Outcome {
    Positive,
    Negative { phase: Phase, error_type: ErrorType },
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
        const STRICT = 0b0_0000_0001;
        const NO_STRICT = 0b0_0000_0010;
        const MODULE = 0b0_0000_0100;
        const RAW = 0b0_0000_1000;
        const ASYNC = 0b0_0001_0000;
        const GENERATED = 0b0_0010_0000;
        const CAN_BLOCK_IS_FALSE = 0b0_0100_0000;
        const CAN_BLOCK_IS_TRUE = 0b0_1000_0000;
        const NON_DETERMINISTIC = 0b1_0000_0000;
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

impl<'de> Deserialize<'de> for TestFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FlagsVisitor;

        impl<'de> Visitor<'de> for FlagsVisitor {
            type Value = TestFlags;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a sequence of flags")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut flags = TestFlags::empty();
                while let Some(elem) = seq.next_element::<TestFlag>()? {
                    flags |= elem.into();
                }
                Ok(flags)
            }
        }

        struct RawFlagsVisitor;

        impl Visitor<'_> for RawFlagsVisitor {
            type Value = TestFlags;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a flags number")
            }

            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                TestFlags::from_bits(v).ok_or_else(|| {
                    E::invalid_value(Unexpected::Unsigned(v.into()), &"a valid flag number")
                })
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_seq(FlagsVisitor)
        } else {
            deserializer.deserialize_u16(RawFlagsVisitor)
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
