//! Module to read the list of test suites from disk.

use super::{Harness, Locale, Phase, Test, TestSuite, CLI};
use fxhash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

/// Representation of the YAML metadata in Test262 tests.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct MetaData {
    pub(super) description: Box<str>,
    pub(super) esid: Option<Box<str>>,
    pub(super) es5id: Option<Box<str>>,
    pub(super) es6id: Option<Box<str>>,
    #[serde(default)]
    pub(super) info: Box<str>,
    #[serde(default)]
    pub(super) features: Box<[Box<str>]>,
    #[serde(default)]
    pub(super) includes: Box<[Box<str>]>,
    #[serde(default)]
    pub(super) flags: Box<[TestFlag]>,
    #[serde(default)]
    pub(super) negative: Option<Negative>,
    #[serde(default)]
    pub(super) locale: Locale,
}

/// Negative test information structure.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct Negative {
    pub(super) phase: Phase,
    #[serde(rename = "type")]
    pub(super) error_type: Box<str>,
}

/// Individual test flag.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum TestFlag {
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

/// Test information structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestInfo {
    desc: Box<str>,
    info: Box<str>,
}

impl TestInfo {
    /// Creates a test information structure from the full metadata.
    fn from_metadata(metadata: &MetaData) -> Self {
        Self {
            desc: metadata.description.trim().to_owned().into_boxed_str(),
            info: metadata.info.trim().to_owned().into_boxed_str(),
        }
    }
}

/// Name of the "test information" file.
const INFO_FILE_NAME: &str = "info.json";

/// Reads the Test262 defined bindings.
pub(super) fn read_harness() -> io::Result<Harness> {
    let mut includes = FxHashMap::default();

    for entry in fs::read_dir(CLI.test262_path().join("harness"))? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if file_name == "assert.js" || file_name == "sta.js" {
            continue;
        }

        let content = fs::read_to_string(entry.path())?;

        includes.insert(
            file_name.into_owned().into_boxed_str(),
            content.into_boxed_str(),
        );
    }
    let assert = fs::read_to_string(CLI.test262_path().join("harness/assert.js"))?.into_boxed_str();
    let sta = fs::read_to_string(CLI.test262_path().join("harness/sta.js"))?.into_boxed_str();

    Ok(Harness {
        assert,
        sta,
        includes,
    })
}

/// Reads the global suite from disk.
pub(super) fn read_global_suite() -> io::Result<TestSuite> {
    let path = CLI.test262_path().join("test");

    let mut info = if let Some(path) = CLI.output() {
        let path = path.join(INFO_FILE_NAME);
        if path.exists() {
            Some(serde_json::from_reader(io::BufReader::new(
                fs::File::open(path)?,
            ))?)
        } else {
            Some(FxHashMap::default())
        }
    } else {
        None
    };

    let suite = read_suite(path.as_path(), &mut info)?;

    if let (Some(path), info) = (CLI.output(), info) {
        let path = path.join(INFO_FILE_NAME);
        if CLI.verbose() {
            println!("Writing the test information file at {}...", path.display());
        }

        let output = io::BufWriter::new(fs::File::create(path)?);
        serde_json::to_writer(output, &info)?;

        if CLI.verbose() {
            println!("Test information file written.");
        }
    }

    Ok(suite)
}

/// Reads a test suite in the given path.
fn read_suite(
    path: &Path,
    test_info: &mut Option<FxHashMap<Box<str>, TestInfo>>,
) -> io::Result<TestSuite> {
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
        st.to_string_lossy().ends_with("_FIXTURE.js")
            // TODO: see if we can fix this.
            || st.to_string_lossy() == "line-terminator-normalisation-CR.js"
    };

    // TODO: iterate in parallel
    for entry in path.read_dir()? {
        let entry = entry?;

        if entry.file_type()?.is_dir() {
            suites.push(read_suite(entry.path().as_path(), test_info)?);
        } else if filter(&entry.file_name()) {
            continue;
        } else {
            tests.push(read_test(entry.path().as_path(), test_info)?);
        }
    }

    Ok(TestSuite {
        name: name.into(),
        suites: suites.into_boxed_slice(),
        tests: tests.into_boxed_slice(),
    })
}

/// Reads information about a given test case.
fn read_test(
    path: &Path,
    test_info: &mut Option<FxHashMap<Box<str>, TestInfo>>,
) -> io::Result<Test> {
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

    if let Some(all_info) = test_info {
        let path_str = path
            .strip_prefix(CLI.test262_path())
            .expect("could not get test path string")
            .to_str()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("non-UTF-8 path found: {}", path.display()),
                )
            })?;

        let new_info = TestInfo::from_metadata(&metadata);

        let _ = all_info.insert(path_str.to_owned().into_boxed_str(), new_info);
    }

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
