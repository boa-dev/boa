use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors
#[allow(missing_docs)]
pub enum Error262 {
    InvalidHarnessDirecory {
        path: String,
    },
    HarnessFileReadError {
        path: String,
    },
    FailedToGetFileType {
        path: String,
    },
    // test
    InvalidPathToTest,
    SubTestReadError {
        path: String,
        suite: String,
        error: Box<Error262>,
    },
    // test suite
    InvalidPathToTestSuite,
    SubSuiteReadError {
        path: String,
        suite: String,
        error: Box<Error262>,
    },
    // test metadata
    MetadataUnknownFeatures(Vec<String>),
    MetadateReadError {
        path: String,
    },
    MetadateParseError {
        path: String,
    },
}
impl std::error::Error for Error262 {}

impl std::fmt::Display for Error262 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub(super) trait PathToString {
    fn string(&self) -> String;
}

impl PathToString for PathBuf {
    fn string(&self) -> String {
        self.display().to_string()
    }
}

impl PathToString for Path {
    fn string(&self) -> String {
        self.display().to_string()
    }
}
