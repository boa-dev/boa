//! Module to read the list of test suites from disk.

use crate::{HarnessFile, Ignored};

use super::{Harness, Locale, Phase, Test, TestSuite};
use color_eyre::{
    eyre::{eyre, WrapErr},
    Result,
};
use fxhash::FxHashMap;
use serde::Deserialize;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

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
    pub(super) error_type: ErrorType,
}

/// All possible error types
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)] // Better than appending `rename` to all variants
pub(super) enum ErrorType {
    Test262Error,
    SyntaxError,
    ReferenceError,
    RangeError,
    TypeError,
}

impl ErrorType {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Test262Error => "Test262Error",
            Self::SyntaxError => "SyntaxError",
            Self::ReferenceError => "ReferenceError",
            Self::RangeError => "RangeError",
            Self::TypeError => "TypeError",
        }
    }
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

/// Reads the Test262 defined bindings.
pub(super) fn read_harness(test262_path: &Path) -> Result<Harness> {
    fn read_harness_file(path: PathBuf) -> Result<HarnessFile> {
        let content = fs::read_to_string(path.as_path())
            .wrap_err_with(|| format!("error reading the harness file `{}`", path.display()))?;

        Ok(HarnessFile {
            content: content.into_boxed_str(),
            path: path.into_boxed_path(),
        })
    }
    let mut includes = FxHashMap::default();

    for entry in fs::read_dir(test262_path.join("harness"))
        .wrap_err("error reading the harness directory")?
    {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if file_name == "assert.js" || file_name == "sta.js" || file_name == "doneprintHandle.js" {
            continue;
        }

        includes.insert(
            file_name.into_owned().into_boxed_str(),
            read_harness_file(entry.path())?,
        );
    }
    let assert = read_harness_file(test262_path.join("harness/assert.js"))?;
    let sta = read_harness_file(test262_path.join("harness/sta.js"))?;
    let doneprint_handle = read_harness_file(test262_path.join("harness/doneprintHandle.js"))?;

    Ok(Harness {
        assert,
        sta,
        doneprint_handle,
        includes,
    })
}

/// Reads a test suite in the given path.
pub(super) fn read_suite(
    path: &Path,
    ignored: &Ignored,
    mut ignore_suite: bool,
) -> Result<TestSuite> {
    let name = path
        .file_name()
        .ok_or_else(|| eyre!(format!("test suite with no name found: {}", path.display())))?
        .to_str()
        .ok_or_else(|| eyre!(format!("non-UTF-8 suite name found: {}", path.display())))?;

    ignore_suite |= ignored.contains_test(name);

    let mut suites = Vec::new();
    let mut tests = Vec::new();

    // TODO: iterate in parallel
    for entry in path.read_dir().wrap_err("retrieving entry")? {
        let entry = entry?;

        if entry.file_type().wrap_err("retrieving file type")?.is_dir() {
            suites.push(
                read_suite(entry.path().as_path(), ignored, ignore_suite).wrap_err_with(|| {
                    let path = entry.path();
                    let suite = path.display();
                    format!("error reading sub-suite {suite}")
                })?,
            );
        } else if entry.file_name().to_string_lossy().contains("_FIXTURE") {
            continue;
        } else {
            let mut test = read_test(entry.path().as_path()).wrap_err_with(|| {
                let path = entry.path();
                let suite = path.display();
                format!("error reading test {suite}")
            })?;

            if ignore_suite
                || ignored.contains_any_flag(test.flags)
                || ignored.contains_test(&test.name)
                || test
                    .features
                    .iter()
                    .any(|feat| ignored.contains_feature(feat))
            {
                test.set_ignored();
            }
            tests.push(test);
        }
    }

    Ok(TestSuite {
        name: name.into(),
        path: Box::from(path),
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

    let metadata = read_metadata(path)?;

    Ok(Test::new(name, path, metadata))
}

/// Reads the metadata from the input test code.
fn read_metadata(test: &Path) -> io::Result<MetaData> {
    use once_cell::sync::Lazy;
    use regex::bytes::Regex;

    /// Regular expression to retrieve the metadata of a test.
    static META_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"/\*\-{3}((?:.|\n)*)\-{3}\*/"#)
            .expect("could not compile metadata regular expression")
    });

    let code = fs::read(test)?;

    let yaml = META_REGEX
        .captures(&code)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("no metadata found for test {}", test.display()),
            )
        })?
        .get(1)
        .map(|m| String::from_utf8_lossy(m.as_bytes()))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("no metadata found for test {}", test.display()),
            )
        })?
        .replace('\r', "\n");

    serde_yaml::from_str(&yaml).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
