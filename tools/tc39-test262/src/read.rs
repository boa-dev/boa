//! Module to read the list of test suites from disk.
use super::test_files::{Harness, HarnessFile, MetaData, Test, TestSuite};

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{
    eyre::{OptionExt, WrapErr},
    Result,
};
use rustc_hash::FxHashMap;

/// Reads the Test262 defined bindings.
pub fn read_harness(test262_path: &Path) -> Result<Harness> {
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
pub fn read_suite(
    path: &Path,
    ignored: &crate::structs::Ignored,
    mut ignore_suite: bool,
) -> Result<TestSuite> {
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_eyre("invalid path for test suite")?;

    ignore_suite |= ignored.contains_test(name);

    let mut suites = Vec::new();
    let mut tests = Vec::new();

    // TODO: iterate in parallel
    for entry in path.read_dir().wrap_err("could not retrieve entry")? {
        let entry = entry?;
        let filetype = entry.file_type().wrap_err("could not retrieve file type")?;

        if filetype.is_dir() {
            suites.push(
                read_suite(entry.path().as_path(), ignored, ignore_suite).wrap_err_with(|| {
                    let path = entry.path();
                    let suite = path.display();
                    format!("error reading sub-suite {suite}")
                })?,
            );
            continue;
        }

        let path = entry.path();

        if path.extension() != Some(OsStr::new("js")) {
            // Ignore files that aren't executable.
            continue;
        }

        if path
            .file_stem()
            .is_some_and(|stem| stem.as_encoded_bytes().ends_with(b"FIXTURE"))
        {
            // Ignore files that are fixtures.
            continue;
        }

        let mut test = read_test(&path).wrap_err_with(|| {
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

    Ok(TestSuite {
        name: name.into(),
        path: Box::from(path),
        suites: suites.into_boxed_slice(),
        tests: tests.into_boxed_slice(),
    })
}

/// Reads information about a given test case.
pub fn read_test(path: &Path) -> Result<Test> {
    let name = path
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_eyre("invalid path for test")?;

    let metadata = read_metadata(path)?;

    Test::new(name, path, metadata)
}

/// Reads the metadata from the input test code.
pub fn read_metadata(test: &Path) -> Result<MetaData> {
    let code = fs::read_to_string(test)?;

    let (_, metadata) = code
        .split_once("/*---")
        .ok_or_eyre("invalid test metadata")?;
    let (metadata, _) = metadata
        .split_once("---*/")
        .ok_or_eyre("invalid test metadata")?;
    let metadata = metadata.replace('\r', "\n");

    serde_yaml::from_str(&metadata).map_err(Into::into)
}
