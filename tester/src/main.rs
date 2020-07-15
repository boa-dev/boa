use bitflags::bitflags;
use rayon::prelude::*;
use serde::Deserialize;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
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

    let (assert_js, sta_js) =
        read_init(cli.suite.as_path()).expect("could not read initialization bindings");

    let global_suite =
        test_suites(cli.suite.as_path()).expect("could not get the list of tests to run");

    let results = global_suite.run(&assert_js, &sta_js);

    println!("Results:");
    println!("Total tests: {}", results.total_tests);
    println!("Passed tests: {}", results.passed_tests);
    println!(
        "Conformance: {:.2}%",
        (results.passed_tests as f64 / results.total_tests as f64) * 100.0
    )
}

/// Reads the Test262 defined bindings.
fn read_init(suite: &Path) -> io::Result<(String, String)> {
    let assert_js = fs::read_to_string(suite.join("harness/assert.js"))?;
    let sta_js = fs::read_to_string(suite.join("harness/sta.js"))?;

    Ok((assert_js, sta_js))
}

/// Gets the list of tests to run.
fn test_suites(suite: &Path) -> io::Result<TestSuite> {
    let path = suite.join("test");

    read_suite(path.as_path())
}

/// Reads a test suite in the given path.
fn read_suite(path: &Path) -> io::Result<TestSuite> {
    use std::ffi::OsStr;

    let name = path
        .file_stem()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("test suite with no name found: {}", path.display()),
            )
        })?
        .to_str()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("non-UTF-8 suite name found: {}", path.display()),
            )
        })?;

    let mut suites = Vec::new();
    let mut tests = Vec::new();

    let filter = |st: &OsStr| {
        st.to_string_lossy().ends_with("_FIXTURE")
            // TODO: see if we can fix this.
            || st.to_string_lossy() == "line-terminator-normalisation-CR"
    };

    // TODO: iterate in parallel
    for entry in path.read_dir()? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            suites.push(read_suite(entry.path().as_path())?);
        } else if entry.path().file_stem().map(filter).unwrap_or(false) {
            continue;
        } else {
            tests.push(read_test(entry.path().as_path())?);
        }
    }

    Ok(TestSuite {
        name: name.into(),
        suites: suites.into_boxed_slice(),
        tests: tests.into_boxed_slice(),
    })
}

/// Reads information about a given test case.
fn read_test(path: &Path) -> io::Result<Test> {
    let name = path
        .file_stem()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("test with no file name found: {}", path.display()),
            )
        })?
        .to_str()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("non-UTF-8 file name found: {}", path.display()),
            )
        })?;

    let content = fs::read_to_string(path)?;

    let metadata = read_metadata(&content)?;

    Ok(Test::new(name, content, metadata))
}

/// Reads the metadata from the input test code.
fn read_metadata(code: &str) -> io::Result<MetaData> {
    use once_cell::sync::Lazy;
    use regex::Regex;

    /// Regular expression to retrieve the metadata of a test.
    static META_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"/\*\-{3}((?:.|\n)*)\-{3}\*/"#)
            .expect("could not compile metadata regular expression")
    });

    let yaml = META_REGEX
        .captures(code)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no metadata found"))?
        .get(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no metadata found"))?
        .as_str();

    serde_yaml::from_str(yaml).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Represents a test suite.
#[derive(Debug, Clone)]
struct TestSuite {
    name: Box<str>,
    suites: Box<[TestSuite]>,
    tests: Box<[Test]>,
}

impl TestSuite {
    /// Runs the test suite.
    fn run(&self, assert_js: &str, sta_js: &str) -> SuiteOutcome {
        // TODO: in parallel
        let suites: Vec<_> = self
            .suites
            .into_iter()
            .map(|suite| suite.run(assert_js, sta_js))
            .collect();

        // TODO: in parallel
        let tests: Vec<_> = self
            .tests
            .into_iter()
            .map(|test| test.run(assert_js, sta_js))
            .collect();

        // Count passed tests
        let mut passed_tests = 0;
        for test in &tests {
            if test.passed {
                passed_tests += 1;
            }
        }

        // Count total tests
        let mut total_tests = tests.len();
        for suite in &suites {
            total_tests += suite.total_tests;
        }

        let passed = passed_tests == total_tests;

        SuiteOutcome {
            name: self.name.clone(),
            passed,
            total_tests,
            passed_tests,
            suites: suites.into_boxed_slice(),
            tests: tests.into_boxed_slice(),
        }
    }
}

/// Outcome of a test suite.
#[derive(Debug, Clone)]
struct SuiteOutcome {
    name: Box<str>,
    passed: bool,
    total_tests: usize,
    passed_tests: usize,
    suites: Box<[SuiteOutcome]>,
    tests: Box<[TestOutcome]>,
}

/// Outcome of a test.
#[derive(Debug, Clone)]
struct TestOutcome {
    name: Box<str>,
    passed: bool,
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
    includes: Box<[Box<Path>]>,
    locale: Locale,
    content: Box<str>,
}

impl Test {
    /// Creates a new test
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

    /// Runs the test.
    fn run(&self, assert_js: &str, sta_js: &str) -> TestOutcome {
        use boa::*;
        use std::panic;

        let res = panic::catch_unwind(|| {
            // Create new Realm
            // TODO: in parallel.
            let realm = Realm::create();
            let mut engine = Interpreter::new(realm);

            forward_val(&mut engine, assert_js).expect("could not run assert.js");
            forward_val(&mut engine, sta_js).expect("could not run sta.js");

            // TODO: set up the environment.

            let res = forward_val(&mut engine, &self.content);

            match self.expected_outcome {
                Outcome::Positive if res.is_err() => false,
                Outcome::Negative {
                    phase: _,
                    error_type: _,
                } if res.is_ok() => false,
                Outcome::Positive => true,
                Outcome::Negative {
                    phase,
                    ref error_type,
                } => {
                    // TODO: check the phase
                    true
                }
            }
        });

        let passed = res.unwrap_or(false);

        // if passed {
        //     println!("{} passed!!", self.name);
        // } else {
        //     eprintln!("{} failed :(", self.name);
        // }

        TestOutcome {
            name: self.name.clone(),
            passed,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct MetaData {
    description: Box<str>,
    esid: Option<Box<str>>,
    es5id: Option<Box<str>>,
    es6id: Option<Box<str>>,
    #[serde(default)]
    info: Box<str>,
    #[serde(default)]
    features: Box<[Box<str>]>,
    #[serde(default)]
    includes: Box<[Box<Path>]>,
    #[serde(default)]
    flags: Box<[TestFlag]>,
    #[serde(default)]
    negative: Option<Negative>,
    #[serde(default)]
    locale: Locale,
}

/// Negative test information structure.
#[derive(Debug, Clone, Deserialize)]
struct Negative {
    phase: Phase,
    #[serde(rename = "type")]
    error_type: Box<str>,
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

/// Individual test flag.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TestFlag {
    OnlyStrict,
    NoStrict,
    Module,
    Raw,
    Async,
    Generated,
    #[serde(rename = "CanBlockIsFalse")]
    CanBlockIsFalse,
    #[serde(rename = "CanBlockIsTrue")]
    CanBlockIsTrue,
    #[serde(rename = "non-deterministic")]
    NonDeterministic,
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
