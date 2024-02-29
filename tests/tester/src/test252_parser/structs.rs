use bitflags::bitflags;
use color_eyre::eyre::eyre;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::path::Path;

use super::edition::SpecEdition;

/// All the harness include files.
#[derive(Debug, Clone)]
pub struct Harness {
    pub assert: HarnessFile,
    pub sta: HarnessFile,
    pub doneprint_handle: HarnessFile,
    pub includes: FxHashMap<Box<str>, HarnessFile>,
}

#[derive(Debug, Clone)]
pub struct HarnessFile {
    pub content: Box<str>,
    pub path: Box<Path>,
}

/// Represents a test.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    pub fn new<N, C>(name: N, path: C, metadata: MetaData) -> color_eyre::Result<Self>
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
    pub fn set_ignored(&mut self) {
        self.ignored = true;
    }

    /// Checks if this is a module test.
    #[inline]
    pub const fn is_module(&self) -> bool {
        self.flags.contains(TestFlags::MODULE)
    }
}

/// All possible error types
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)] // Better than appending `rename` to all variants
pub enum ErrorType {
    Test262Error,
    SyntaxError,
    ReferenceError,
    RangeError,
    TypeError,
}

impl ErrorType {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Test262Error => "Test262Error",
            Self::SyntaxError => "SyntaxError",
            Self::ReferenceError => "ReferenceError",
            Self::RangeError => "RangeError",
            Self::TypeError => "TypeError",
        }
    }
}

/// Phase for an error.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Parse,
    Resolution,
    Runtime,
}

/// Negative test information structure.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Negative {
    pub(super) phase: Phase,
    #[serde(rename = "type")]
    pub(super) error_type: ErrorType,
}

/// An outcome for a test.
#[derive(Debug, Clone)]
pub enum Outcome {
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

/// Locale information structure.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(transparent)]
#[allow(dead_code)]
pub struct Locale {
    locale: Box<[Box<str>]>,
}

/// Represents a test suite.
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub name: Box<str>,
    pub path: Box<Path>,
    pub suites: Box<[TestSuite]>,
    pub tests: Box<[Test]>,
}

/// Representation of the YAML metadata in Test262 tests.
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
    pub flags: Box<[TestFlag]>,
    #[serde(default)]
    pub negative: Option<Negative>,
    #[serde(default)]
    pub locale: Locale,
}

// tests flags

/// Individual test flag.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TestFlag {
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

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub  struct TestFlags: u16 {
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

/// Structure to allow defining ignored tests, features and files that should
/// be ignored even when reading.
#[derive(Debug, Deserialize)]
pub struct Ignored {
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
