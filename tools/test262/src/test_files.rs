use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserialize;
use std::path::Path;

use super::structs::*;
use crate::{SpecEdition, TestFlags};

/// All the harness include files.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Harness {
    pub assert: HarnessFile,
    pub sta: HarnessFile,
    pub doneprint_handle: HarnessFile,
    pub includes: FxHashMap<Box<str>, HarnessFile>,
}

/// Represents harness file.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct HarnessFile {
    pub content: Box<str>,
    pub path: Box<Path>,
}

/// Represents a test.
#[allow(dead_code)]
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Test {
    pub name: Box<str>,
    pub path: Box<Path>,
    pub description: Box<str>,
    pub esid: Option<Box<str>>,
    pub edition: SpecEdition,
    pub flags: TestFlags,
    pub information: Box<str>,
    pub expected_outcome: Outcome,
    pub features: FxHashSet<Box<str>>,
    pub includes: FxHashSet<Box<str>>,
    pub locale: Locale,
    pub ignored: bool,
}

impl Test {
    /// Creates a new test.
    pub fn new<N, C>(name: N, path: C, metadata: MetaData) -> Result<Self, crate::Error262>
    where
        N: Into<Box<str>>,
        C: Into<Box<Path>>,
    {
        let edition = SpecEdition::from_test_metadata(&metadata).map_err(|feats| {
            crate::Error262::MetadataUnknownFeatures(feats.into_iter().map(String::from).collect())
        })?;

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
    pub fn set_ignored(&mut self) {
        self.ignored = true;
    }

    /// Checks if this is a module test.
    #[inline]
    pub const fn is_module(&self) -> bool {
        self.flags.contains(TestFlags::MODULE)
    }
}

/// Represents a test suite.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: Box<str>,
    pub path: Box<Path>,
    pub suites: Box<[TestSuite]>,
    pub tests: Box<[Test]>,
}

/// Representation of the YAML metadata in Test262 tests.
#[allow(missing_docs)]
#[derive(Debug, Clone, Deserialize)]
pub struct MetaData {
    pub description: Box<str>,
    pub esid: Option<Box<str>>,
    #[allow(dead_code)]
    pub es5id: Option<Box<str>>,
    pub es6id: Option<Box<str>>,
    #[serde(default)]
    pub info: Box<str>,
    #[serde(default)]
    pub features: Box<[Box<str>]>,
    #[serde(default)]
    pub includes: Box<[Box<str>]>,
    #[serde(default)]
    pub flags: Box<[crate::test_flags::TestFlag]>,
    #[serde(default)]
    pub negative: Option<Negative>,
    #[serde(default)]
    pub locale: Locale,
}

/// Locale information structure.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(transparent)]
#[allow(dead_code)]
pub struct Locale {
    locale: Box<[Box<str>]>,
}
