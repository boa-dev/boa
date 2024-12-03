//! Module to read the list of test suites from disk.
use crate::{error::PathToString, Error262};

use super::test_files::{Harness, HarnessFile, MetaData, Test, TestSuite};

use rustc_hash::FxHashMap;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

impl Harness {
    /// Reads the Test262 defined bindings.
    pub fn read(test262_path: &Path) -> Result<Harness, Error262> {
        fn read_harness_file(path: PathBuf) -> Result<HarnessFile, Error262> {
            let content =
                fs::read_to_string(&path).map_err(|_| Error262::HarnessFileReadError {
                    path: path.string(),
                })?;

            Ok(HarnessFile {
                content: content.into_boxed_str(),
                path: path.into_boxed_path(),
            })
        }
        let mut includes = FxHashMap::default();

        let harness_dir = test262_path.join("harness");
        for entry in fs::read_dir(&harness_dir).map_err(|_| Error262::InvalidHarnessDirecory {
            path: harness_dir.string(),
        })? {
            let entry = entry.map_err(|_| Error262::InvalidHarnessDirecory {
                path: harness_dir.string(),
            })?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if file_name == "assert.js"
                || file_name == "sta.js"
                || file_name == "doneprintHandle.js"
            {
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
}

impl TestSuite {
    /// Reads a test suite in the given path.
    pub fn read(
        path: &Path,
        ignored: &crate::structs::Ignored,
        mut ignore_suite: bool,
    ) -> Result<TestSuite, Error262> {
        let name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(Error262::InvalidPathToTestSuite)?;

        ignore_suite |= ignored.contains_test(name);

        let mut suites = Vec::new();
        let mut tests = Vec::new();

        // TODO: iterate in parallel
        for entry in path
            .read_dir()
            .map_err(|_| Error262::InvalidPathToTestSuite)?
        {
            let entry = entry.map_err(|_| Error262::InvalidPathToTestSuite)?;
            let filetype = entry
                .file_type()
                .map_err(|_| Error262::FailedToGetFileType {
                    path: entry.path().string(),
                })?;

            if filetype.is_dir() {
                suites.push(
                    TestSuite::read(entry.path().as_path(), ignored, ignore_suite).map_err(|e| {
                        Error262::SubSuiteReadError {
                            path: entry.path().string(),
                            suite: path.string(),
                            error: Box::new(e),
                        }
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

            let mut test = Test::read(&path).map_err(|e| Error262::SubTestReadError {
                path: entry.path().string(),
                suite: path.string(),
                error: Box::new(e),
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
}

impl Test {
    /// Reads information about a given test case.
    pub fn read(path: &Path) -> Result<Test, Error262> {
        let name = path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or(Error262::InvalidPathToTest)?;

        let metadata = MetaData::read(path)?;

        Test::new(name, path, metadata)
    }
}

impl MetaData {
    /// Reads the metadata from the input test code.
    pub fn read(test: &Path) -> Result<MetaData, Error262> {
        let code = fs::read_to_string(test).map_err(|_| Error262::MetadateReadError {
            path: test.string(),
        })?;

        let (_, metadata) =
            code.split_once("/*---")
                .ok_or_else(|| Error262::MetadateParseError {
                    path: test.string(),
                })?;
        let (metadata, _) =
            metadata
                .split_once("---*/")
                .ok_or_else(|| Error262::MetadateParseError {
                    path: test.string(),
                })?;
        let metadata = metadata.replace('\r', "\n");

        serde_yaml::from_str(&metadata).map_err(|_| Error262::MetadateParseError {
            path: test.string(),
        })
    }
}
