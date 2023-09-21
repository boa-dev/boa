//! Test262 test runner
//!
//! This crate will run the full ECMAScript test suite (Test262) and report compliance of the
//! `boa` engine.
//!
#![doc = include_str!("../ABOUT.md")]
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
    clippy::too_many_lines,
    clippy::redundant_pub_crate,
    clippy::cast_precision_loss
)]

mod edition;
mod exec;
mod read;
mod results;

use self::{
    read::{read_harness, read_suite, read_test, MetaData, Negative, TestFlag},
    results::{compare_results, write_json},
};
use bitflags::bitflags;
use boa_engine::optimizer::OptimizerOptions;
use clap::{ArgAction, Parser, ValueHint};
use color_eyre::{
    eyre::{bail, eyre, WrapErr},
    Result,
};
use colored::Colorize;
use edition::SpecEdition;
use fxhash::{FxHashMap, FxHashSet};
use read::ErrorType;
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{
    ops::{Add, AddAssign},
    path::{Path, PathBuf},
    process::Command,
};

/// Structure that contains the configuration of the tester.
#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    commit: String,
    #[serde(default)]
    ignored: Ignored,
}

impl Config {
    /// Get the `Test262` repository commit.
    pub(crate) fn commit(&self) -> &str {
        &self.commit
    }

    /// Get [`Ignored`] `Test262` tests and features.
    pub(crate) const fn ignored(&self) -> &Ignored {
        &self.ignored
    }
}

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
        #[arg(
            long,
            value_hint = ValueHint::DirPath,
            conflicts_with = "test262_commit"
        )]
        test262_path: Option<PathBuf>,

        /// Override config's Test262 commit. To checkout the latest commit set this to "latest".
        #[arg(long)]
        test262_commit: Option<String>,

        /// Which specific test or test suite to run. Should be a path relative to the Test262 directory: e.g. "test/language/types/number"
        #[arg(short, long, default_value = "test", value_hint = ValueHint::AnyPath)]
        suite: PathBuf,

        /// Enable optimizations
        #[arg(long, short = 'O')]
        optimize: bool,

        /// Optional output folder for the full results information.
        #[arg(short, long, value_hint = ValueHint::DirPath)]
        output: Option<PathBuf>,

        /// Execute tests serially
        #[arg(short, long)]
        disable_parallelism: bool,

        /// Path to a TOML file containing tester config.
        #[arg(short, long, default_value = "test262_config.toml", value_hint = ValueHint::FilePath)]
        config: PathBuf,

        /// Maximum ECMAScript edition to test for.
        #[arg(long)]
        edition: Option<SpecEdition>,

        /// Displays the conformance results per ECMAScript edition.
        #[arg(long)]
        versioned: bool,
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

const DEFAULT_TEST262_DIRECTORY: &str = "test262";

/// Program entry point.
fn main() -> Result<()> {
    color_eyre::install()?;
    match Cli::parse() {
        Cli::Run {
            verbose,
            test262_path,
            test262_commit,
            suite,
            output,
            optimize,
            disable_parallelism,
            config: config_path,
            edition,
            versioned,
        } => {
            let config: Config = {
                let input = std::fs::read_to_string(config_path)?;
                toml::from_str(&input).wrap_err("could not decode tester config file")?
            };

            let test262_commit = test262_commit
                .as_deref()
                .or_else(|| Some(config.commit()))
                .filter(|s| !["", "latest"].contains(s));

            let test262_path = if let Some(path) = test262_path.as_deref() {
                path
            } else {
                clone_test262(test262_commit, verbose)?;

                Path::new(DEFAULT_TEST262_DIRECTORY)
            };

            run_test_suite(
                &config,
                verbose,
                !disable_parallelism,
                test262_path,
                suite.as_path(),
                output.as_deref(),
                edition.unwrap_or_default(),
                versioned,
                if optimize {
                    OptimizerOptions::OPTIMIZE_ALL
                } else {
                    OptimizerOptions::empty()
                },
            )
        }
        Cli::Compare {
            base,
            new,
            markdown,
        } => compare_results(base.as_path(), new.as_path(), markdown),
    }
}

/// Returns the commit hash and commit message of the provided branch name.
fn get_last_branch_commit(branch: &str) -> Result<(String, String)> {
    let result = Command::new("git")
        .arg("log")
        .args(["-n", "1"])
        .arg("--pretty=format:%H %s")
        .arg(branch)
        .current_dir(DEFAULT_TEST262_DIRECTORY)
        .output()?;

    if !result.status.success() {
        bail!(
            "test262 getting commit hash and message failed with return code {:?}",
            result.status.code()
        );
    }

    let output = std::str::from_utf8(&result.stdout)?.trim();

    let (hash, message) = output
        .split_once(' ')
        .expect("git log output to contain hash and message");

    Ok((hash.into(), message.into()))
}

fn reset_test262_commit(commit: &str, verbose: u8) -> Result<()> {
    if verbose != 0 {
        println!("Reset test262 to commit: {commit}...");
    }

    let result = Command::new("git")
        .arg("reset")
        .arg("--hard")
        .arg(commit)
        .current_dir(DEFAULT_TEST262_DIRECTORY)
        .status()?;

    if !result.success() {
        bail!(
            "test262 commit {commit} checkout failed with return code: {:?}",
            result.code()
        );
    }

    Ok(())
}

fn clone_test262(commit: Option<&str>, verbose: u8) -> Result<()> {
    const TEST262_REPOSITORY: &str = "https://github.com/tc39/test262";

    let update = commit.is_none();

    if Path::new(DEFAULT_TEST262_DIRECTORY).is_dir() {
        if verbose != 0 {
            println!("Fetching latest test262 commits...");
        }
        let result = Command::new("git")
            .arg("fetch")
            .current_dir(DEFAULT_TEST262_DIRECTORY)
            .status()?;

        if !result.success() {
            bail!(
                "Test262 fetching latest failed with return code {:?}",
                result.code()
            );
        }

        let (current_commit_hash, current_commit_message) = get_last_branch_commit("HEAD")?;

        if let Some(commit) = commit {
            if current_commit_hash != commit {
                println!("Test262 switching to commit {commit}...");
                reset_test262_commit(commit, verbose)?;
            }
            return Ok(());
        }

        if verbose != 0 {
            println!("Checking latest Test262 with current HEAD...");
        }
        let (latest_commit_hash, latest_commit_message) = get_last_branch_commit("origin/main")?;

        if current_commit_hash != latest_commit_hash {
            if update {
                println!("Updating Test262 repository:");
            } else {
                println!("Warning Test262 repository is not in sync, use '--test262-commit latest' to automatically update it:");
            }

            println!("    Current commit: {current_commit_hash} {current_commit_message}");
            println!("    Latest commit:  {latest_commit_hash} {latest_commit_message}");

            if update {
                reset_test262_commit(&latest_commit_hash, verbose)?;
            }
        }

        return Ok(());
    }

    println!("Cloning test262...");
    let result = Command::new("git")
        .arg("clone")
        .arg(TEST262_REPOSITORY)
        .arg(DEFAULT_TEST262_DIRECTORY)
        .status()?;

    if !result.success() {
        bail!(
            "Cloning Test262 repository failed with return code {:?}",
            result.code()
        );
    }

    if let Some(commit) = commit {
        if verbose != 0 {
            println!("Reset Test262 to commit: {commit}...");
        }

        reset_test262_commit(commit, verbose)?;
    }

    Ok(())
}

/// Runs the full test suite.
#[allow(clippy::too_many_arguments)]
fn run_test_suite(
    config: &Config,
    verbose: u8,
    parallel: bool,
    test262_path: &Path,
    suite: &Path,
    output: Option<&Path>,
    edition: SpecEdition,
    versioned: bool,
    optimizer_options: OptimizerOptions,
) -> Result<()> {
    if let Some(path) = output {
        if path.exists() {
            if !path.is_dir() {
                bail!("the output path must be a directory.");
            }
        } else {
            std::fs::create_dir_all(path).wrap_err("could not create the output directory")?;
        }
    }

    if verbose != 0 {
        println!("Loading the test suite...");
    }
    let harness = read_harness(test262_path).wrap_err("could not read harness")?;

    if suite.to_string_lossy().ends_with(".js") {
        let test = read_test(&test262_path.join(suite)).wrap_err_with(|| {
            let suite = suite.display();
            format!("could not read the test {suite}")
        })?;

        if test.edition <= edition {
            if verbose != 0 {
                println!("Test loaded, starting...");
            }
            test.run(&harness, verbose, optimizer_options);
        } else {
            println!(
                "Minimum spec edition of test is bigger than the specified edition. Skipping."
            );
        }

        println!();
    } else {
        let suite =
            read_suite(&test262_path.join(suite), config.ignored(), false).wrap_err_with(|| {
                let suite = suite.display();
                format!("could not read the suite {suite}")
            })?;

        if verbose != 0 {
            println!("Test suite loaded, starting tests...");
        }
        let results = suite.run(&harness, verbose, parallel, edition, optimizer_options);

        if versioned {
            let mut table = comfy_table::Table::new();
            table.load_preset(comfy_table::presets::UTF8_HORIZONTAL_ONLY);
            table.set_header(vec![
                "Edition", "Total", "Passed", "Ignored", "Failed", "Panics", "%",
            ]);
            for column in table.column_iter_mut().skip(1) {
                column.set_cell_alignment(comfy_table::CellAlignment::Right);
            }
            for (v, stats) in SpecEdition::all_editions()
                .filter(|v| *v <= edition)
                .map(|v| {
                    let stats = results.versioned_stats.get(v).unwrap_or(results.stats);
                    (v, stats)
                })
            {
                let Statistics {
                    total,
                    passed,
                    ignored,
                    panic,
                } = stats;
                let failed = total - passed - ignored;
                let conformance = (passed as f64 / total as f64) * 100.0;
                let conformance = format!("{conformance:.2}");
                table.add_row(vec![
                    v.to_string(),
                    total.to_string(),
                    passed.to_string(),
                    ignored.to_string(),
                    failed.to_string(),
                    panic.to_string(),
                    conformance,
                ]);
            }
            println!("\n\nResults\n");
            println!("{table}");
        } else {
            let Statistics {
                total,
                passed,
                ignored,
                panic,
            } = results.stats;
            println!("\n\nResults ({edition}):");
            println!("Total tests: {total}");
            println!("Passed tests: {}", passed.to_string().green());
            println!("Ignored tests: {}", ignored.to_string().yellow());
            println!(
                "Failed tests: {} ({})",
                (total - passed - ignored).to_string().red(),
                format!("{panic} panics").red()
            );
            println!(
                "Conformance: {:.2}%",
                (passed as f64 / total as f64) * 100.0
            );
        }

        if let Some(output) = output {
            write_json(results, output, verbose, test262_path)
                .wrap_err("could not write the results to the output JSON file")?;
        }
    }

    Ok(())
}

/// All the harness include files.
#[derive(Debug, Clone)]
struct Harness {
    assert: HarnessFile,
    sta: HarnessFile,
    doneprint_handle: HarnessFile,
    includes: FxHashMap<Box<str>, HarnessFile>,
}

#[derive(Debug, Clone)]
struct HarnessFile {
    content: Box<str>,
    path: Box<Path>,
}

/// Represents a test suite.
#[derive(Debug, Clone)]
struct TestSuite {
    name: Box<str>,
    path: Box<Path>,
    suites: Box<[TestSuite]>,
    tests: Box<[Test]>,
}

/// Represents a tests statistic
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
struct Statistics {
    #[serde(rename = "t")]
    total: usize,
    #[serde(rename = "o")]
    passed: usize,
    #[serde(rename = "i")]
    ignored: usize,
    #[serde(rename = "p")]
    panic: usize,
}

impl Add for Statistics {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            total: self.total + rhs.total,
            passed: self.passed + rhs.passed,
            ignored: self.ignored + rhs.ignored,
            panic: self.panic + rhs.panic,
        }
    }
}

impl AddAssign for Statistics {
    fn add_assign(&mut self, rhs: Self) {
        self.total += rhs.total;
        self.passed += rhs.passed;
        self.ignored += rhs.ignored;
        self.panic += rhs.panic;
    }
}

/// Represents tests statistics separated by ECMAScript edition
#[derive(Default, Debug, Copy, Clone, Serialize)]
struct VersionedStats {
    es5: Statistics,
    es6: Statistics,
    es7: Statistics,
    es8: Statistics,
    es9: Statistics,
    es10: Statistics,
    es11: Statistics,
    es12: Statistics,
    es13: Statistics,
    es14: Statistics,
}

impl<'de> Deserialize<'de> for VersionedStats {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner {
            es5: Statistics,
            es6: Statistics,
            es7: Statistics,
            es8: Statistics,
            es9: Statistics,
            es10: Statistics,
            es11: Statistics,
            es12: Statistics,
            es13: Statistics,
            #[serde(default)]
            es14: Option<Statistics>,
        }

        let inner = Inner::deserialize(deserializer)?;

        let Inner {
            es5,
            es6,
            es7,
            es8,
            es9,
            es10,
            es11,
            es12,
            es13,
            es14,
        } = inner;
        let es14 = es14.unwrap_or(es13);

        Ok(Self {
            es5,
            es6,
            es7,
            es8,
            es9,
            es10,
            es11,
            es12,
            es13,
            es14,
        })
    }
}

impl VersionedStats {
    /// Applies `f` to all the statistics for which its edition is bigger or equal
    /// than `min_edition`.
    fn apply(&mut self, min_edition: SpecEdition, f: fn(&mut Statistics)) {
        for edition in SpecEdition::all_editions().filter(|&edition| min_edition <= edition) {
            if let Some(stats) = self.get_mut(edition) {
                f(stats);
            }
        }
    }

    /// Gets the statistics corresponding to `edition`, returning `None` if `edition`
    /// is `SpecEdition::ESNext`.
    const fn get(&self, edition: SpecEdition) -> Option<Statistics> {
        let stats = match edition {
            SpecEdition::ES5 => self.es5,
            SpecEdition::ES6 => self.es6,
            SpecEdition::ES7 => self.es7,
            SpecEdition::ES8 => self.es8,
            SpecEdition::ES9 => self.es9,
            SpecEdition::ES10 => self.es10,
            SpecEdition::ES11 => self.es11,
            SpecEdition::ES12 => self.es12,
            SpecEdition::ES13 => self.es13,
            SpecEdition::ES14 => self.es14,
            SpecEdition::ESNext => return None,
        };
        Some(stats)
    }

    /// Gets a mutable reference to the statistics corresponding to `edition`, returning `None` if
    /// `edition` is `SpecEdition::ESNext`.
    fn get_mut(&mut self, edition: SpecEdition) -> Option<&mut Statistics> {
        let stats = match edition {
            SpecEdition::ES5 => &mut self.es5,
            SpecEdition::ES6 => &mut self.es6,
            SpecEdition::ES7 => &mut self.es7,
            SpecEdition::ES8 => &mut self.es8,
            SpecEdition::ES9 => &mut self.es9,
            SpecEdition::ES10 => &mut self.es10,
            SpecEdition::ES11 => &mut self.es11,
            SpecEdition::ES12 => &mut self.es12,
            SpecEdition::ES13 => &mut self.es13,
            SpecEdition::ES14 => &mut self.es14,
            SpecEdition::ESNext => return None,
        };
        Some(stats)
    }
}

impl Add for VersionedStats {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            es5: self.es5 + rhs.es5,
            es6: self.es6 + rhs.es6,
            es7: self.es7 + rhs.es7,
            es8: self.es8 + rhs.es8,
            es9: self.es9 + rhs.es9,
            es10: self.es10 + rhs.es10,
            es11: self.es11 + rhs.es11,
            es12: self.es12 + rhs.es12,
            es13: self.es13 + rhs.es13,
            es14: self.es14 + rhs.es14,
        }
    }
}

impl AddAssign for VersionedStats {
    fn add_assign(&mut self, rhs: Self) {
        self.es5 += rhs.es5;
        self.es6 += rhs.es6;
        self.es7 += rhs.es7;
        self.es8 += rhs.es8;
        self.es9 += rhs.es9;
        self.es10 += rhs.es10;
        self.es11 += rhs.es11;
        self.es12 += rhs.es12;
        self.es13 += rhs.es13;
        self.es14 += rhs.es14;
    }
}

/// Outcome of a test suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SuiteResult {
    #[serde(rename = "n")]
    name: Box<str>,
    #[serde(rename = "a")]
    stats: Statistics,
    #[serde(rename = "av", default)]
    versioned_stats: VersionedStats,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(rename = "s")]
    suites: Vec<SuiteResult>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(rename = "t")]
    tests: Vec<TestResult>,
    #[serde(skip_serializing_if = "FxHashSet::is_empty", default)]
    #[serde(rename = "f")]
    features: FxHashSet<String>,
}

/// Outcome of a test.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct TestResult {
    #[serde(rename = "n")]
    name: Box<str>,
    #[serde(rename = "v", default)]
    edition: SpecEdition,
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
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Test {
    name: Box<str>,
    path: Box<Path>,
    description: Box<str>,
    esid: Option<Box<str>>,
    edition: SpecEdition,
    flags: TestFlags,
    information: Box<str>,
    expected_outcome: Outcome,
    features: FxHashSet<Box<str>>,
    includes: FxHashSet<Box<str>>,
    locale: Locale,
    ignored: bool,
}

impl Test {
    /// Creates a new test.
    fn new<N, C>(name: N, path: C, metadata: MetaData) -> Result<Self>
    where
        N: Into<Box<str>>,
        C: Into<Box<Path>>,
    {
        let edition = SpecEdition::from_test_metadata(&metadata)
            .map_err(|feats| eyre!("test metadata contained unknown features: {feats:?}"))?;

        Ok(Self {
            edition,
            name: name.into(),
            description: metadata.description,
            esid: metadata.esid,
            flags: metadata.flags.into(),
            information: metadata.info,
            features: metadata.features.into_vec().into_iter().collect(),
            expected_outcome: Outcome::from(metadata.negative),
            includes: metadata.includes.into_vec().into_iter().collect(),
            locale: metadata.locale,
            path: path.into(),
            ignored: false,
        })
    }

    /// Sets the test as ignored.
    #[inline]
    fn set_ignored(&mut self) {
        self.ignored = true;
    }

    /// Checks if this is a module test.
    #[inline]
    const fn is_module(&self) -> bool {
        self.flags.contains(TestFlags::MODULE)
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
    #[derive(Debug, Clone, Copy)]
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
