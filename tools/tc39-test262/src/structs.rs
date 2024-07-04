#![allow(missing_docs)]
use rustc_hash::FxHashSet;
use serde::Deserialize;
use crate::test_flags::TestFlags;

/// All possible error types
#[allow(missing_docs)]
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
    /// str representation
    pub const fn as_str(self) -> &'static str {
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
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Parse,
    Resolution,
    Runtime,
}

/// Negative test information structure.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Negative {
    pub(super) phase: Phase,
    #[serde(rename = "type")]
    pub(super) error_type: ErrorType,
}

/// An outcome for a test.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
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
            .is_some_and(|feat| self.features.contains(feat))
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
