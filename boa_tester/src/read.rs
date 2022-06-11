//! Module to read the list of test suites from disk.

use super::{Harness, Locale, Phase, Test, TestSuite, IGNORED};
use anyhow::Context;
use fxhash::FxHashMap;
use serde::Deserialize;
use std::{fs, io, path::Path, str::FromStr};

/// Representation of the YAML metadata in Test262 tests.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
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

impl FromStr for TestFlag {
    type Err = String; // TODO: improve error type.

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "onlyStrict" => Ok(Self::OnlyStrict),
            "noStrict" => Ok(Self::NoStrict),
            "module" => Ok(Self::Module),
            "raw" => Ok(Self::Raw),
            "async" => Ok(Self::Async),
            "generated" => Ok(Self::Generated),
            "CanBlockIsFalse" => Ok(Self::CanBlockIsFalse),
            "CanBlockIsTrue" => Ok(Self::CanBlockIsTrue),
            "non-deterministic" => Ok(Self::NonDeterministic),
            _ => Err(format!("unknown test flag: {s}")),
        }
    }
}

/// Reads the Test262 defined bindings.
pub(super) fn read_harness(test262_path: &Path) -> anyhow::Result<Harness> {
    let mut includes = FxHashMap::default();

    for entry in
        fs::read_dir(test262_path.join("harness")).context("error reading the harness directory")?
    {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if file_name == "assert.js" || file_name == "sta.js" || file_name == "doneprintHandle.js" {
            continue;
        }

        let content = fs::read_to_string(entry.path())
            .with_context(|| format!("error reading the harnes/{file_name}"))?;

        includes.insert(
            file_name.into_owned().into_boxed_str(),
            content.into_boxed_str(),
        );
    }
    let assert = fs::read_to_string(test262_path.join("harness/assert.js"))
        .context("error reading harnes/assert.js")?
        .into_boxed_str();
    let sta = fs::read_to_string(test262_path.join("harness/sta.js"))
        .context("error reading harnes/sta.js")?
        .into_boxed_str();
    let doneprint_handle = fs::read_to_string(test262_path.join("harness/doneprintHandle.js"))
        .context("error reading harnes/doneprintHandle.js")?
        .into_boxed_str();

    Ok(Harness {
        assert,
        sta,
        doneprint_handle,
        includes,
    })
}

/// Reads a test suite in the given path.
pub(super) fn read_suite(path: &Path) -> anyhow::Result<TestSuite> {
    let name = path
        .file_name()
        .with_context(|| format!("test suite with no name found: {}", path.display()))?
        .to_str()
        .with_context(|| format!("non-UTF-8 suite name found: {}", path.display()))?;

    let mut suites = Vec::new();
    let mut tests = Vec::new();

    // TODO: iterate in parallel
    for entry in path.read_dir().context("retrieving entry")? {
        let entry = entry?;

        if entry.file_type().context("retrieving file type")?.is_dir() {
            suites.push(read_suite(entry.path().as_path()).with_context(|| {
                let path = entry.path();
                let suite = path.display();
                format!("error reading sub-suite {suite}")
            })?);
        } else if entry.file_name().to_string_lossy().contains("_FIXTURE") {
            continue;
        } else if IGNORED.contains_file(&entry.file_name().to_string_lossy()) {
            let mut test = Test::default();
            test.set_name(entry.file_name().to_string_lossy());
            tests.push(test);
        } else {
            tests.push(read_test(entry.path().as_path()).with_context(|| {
                let path = entry.path();
                let suite = path.display();
                format!("error reading test {suite}")
            })?);
        }
    }

    Ok(TestSuite {
        name: name.into(),
        suites: suites.into_boxed_slice(),
        tests: tests.into_boxed_slice(),
    })
}

/// Reads information about a given test case.
pub(super) fn read_test(path: &Path) -> io::Result<Test> {
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
    let metadata = read_metadata(&content, path)?;

    Ok(Test::new(name, content, metadata))
}

/// Reads the metadata from the input test code.
fn read_metadata(code: &str, test: &Path) -> io::Result<MetaData> {
    use once_cell::sync::Lazy;
    use regex::Regex;

    /// Regular expression to retrieve the metadata of a test.
    static META_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"/\*\-{3}((?:.|\n)*)\-{3}\*/"#)
            .expect("could not compile metadata regular expression")
    });

    let yaml = META_REGEX
        .captures(code)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("no metadata found for test {}", test.display()),
            )
        })?
        .get(1)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("no metadata found for test {}", test.display()),
            )
        })?
        .as_str()
        .replace('\r', "\n");

    serde_yaml::from_str(&yaml).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
