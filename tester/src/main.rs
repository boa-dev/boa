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

    let tests = test_suites(cli.suite.as_path()).expect("could not get the list of tests to run");
}

/// Reads the Test262 defined bindings.
fn read_init(suite: &Path) -> io::Result<(String, String)> {
    let assert_js = fs::read_to_string(suite.join("harness/assert.js"))?;
    let sta_js = fs::read_to_string(suite.join("harness/sta.js"))?;

    Ok((assert_js, sta_js))
}

/// Represents a test suite.
#[derive(Debug, Clone)]
struct TestSuite {
    suites: Box<[TestSuite]>,
    cases: Box<[TestCase]>,
}

/// Represents a test case.
#[derive(Debug, Clone)]
struct TestCase {
    name: Option<Box<str>>,
    description: Box<str>,
    template: Box<str>,
    information: Option<Box<str>>,
    expected_outcome: Outcome,
    includes: Box<[Box<[str]>]>,
    flags: TestCaseFlags,
    locale: Box<[Locale]>,
    content: Box<str>,
}

/// An outcome for a test.
#[derive(Debug, Default, Clone, Copy)]
enum Outcome {
    Positive,
    Negative { phase: Phase, error_type: Box<str> },
}

/// Phase for an error.
#[derive(Debug, Clone, Copy)]
enum Phase {
    Parse,
    Early,
    Resolution,
    Runtime,
}

/// Locale information structure.
#[derive(Debug, Clone)]
struct Locale {
    locale: Box<[str]>,
}

/// Gets the list of tests to run.
fn test_suites(suite: &Path) -> io::Result<TestSuite> {
    let path = suite.join("src");
    read_suite(path.as_path());
}

/// Reads a test suite in the given path.
fn read_suite(path: &Path) -> io::Result<TestSuite> {
    let mut suites = Vec::new();
    let mut cases = Vec::new();
    for entry in path.read_dir() {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            suites.push(read_suite(entry.path().as_path())?);
        } else {
            cases.push(read_test_case(entry.path().as_path())?);
        }
    }

    Ok(TestSuite {
        suites: suites.to_boxed_slice(),
        cases: cases.to_boxed_slice(),
    })
}

/// Reads information about a given test case.
fn read_test_case(path: &Path) -> io::Result<TestCase> {
    let name = path.file_stem();
    let content = fs::read_to_string(path)?;
    todo!()
}
