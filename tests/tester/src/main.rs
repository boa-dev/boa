//! Test262 test runner
//!
//! This crate will run the full ECMAScript test suite (Test262) and report compliance of the
//! `boa` engine.
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![allow(
    clippy::too_many_lines,
    clippy::redundant_pub_crate,
    clippy::cast_precision_loss
)]

mod exec;
mod results;

use exec::{RunTestSuite, RunTest};
use tc39_test262::{read, Ignored, SpecEdition, TestFlags};

use self::results::{compare_results, write_json};

use boa_engine::optimizer::OptimizerOptions;
use clap::{ArgAction, Parser, ValueHint};
use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use colored::Colorize;
use once_cell::sync::Lazy;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    ops::{Add, AddAssign},
    path::{Path, PathBuf},
    time::Instant,
};

static START: Lazy<Instant> = Lazy::new(Instant::now);

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

        /// Injects the `Console` object into every context created.
        #[arg(long)]
        console: bool,
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
    // Safety: This is needed because we run tests in multiple threads.
    // It is safe because tests do not modify the environment.
    unsafe {
        time::util::local_offset::set_soundness(time::util::local_offset::Soundness::Unsound);
    }

    // initializes the monotonic clock.
    Lazy::force(&START);
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
            console,
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
                tc39_test262::clone_test262(test262_commit, verbose)?;
                Path::new(tc39_test262::TEST262_DIRECTORY)
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
                console,
            )
        }
        Cli::Compare {
            base,
            new,
            markdown,
        } => compare_results(base.as_path(), new.as_path(), markdown),
    }
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
    console: bool,
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
    let harness = read::read_harness(test262_path).wrap_err("could not read harness")?;

    if suite.to_string_lossy().ends_with(".js") {
        let test = read::read_test(&test262_path.join(suite)).wrap_err_with(|| {
            let suite = suite.display();
            format!("could not read the test {suite}")
        })?;

        if test.edition <= edition {
            if verbose != 0 {
                println!("Test loaded, starting...");
            }
            test.run(&harness, verbose, optimizer_options, console);
        } else {
            println!(
                "Minimum spec edition of test is bigger than the specified edition. Skipping."
            );
        }

        println!();
    } else {
        let suite = read::read_suite(&test262_path.join(suite), config.ignored(), false)
            .wrap_err_with(|| {
                let suite = suite.display();
                format!("could not read the suite {suite}")
            })?;

        if verbose != 0 {
            println!("Test suite loaded, starting tests...");
        }
        let results = suite.run(
            &harness,
            verbose,
            parallel,
            edition,
            optimizer_options,
            console,
        );

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
