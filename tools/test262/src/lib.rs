//! TC39 test262
mod edition;
mod error;
mod git;
mod read;
mod structs;
mod test_files;
mod test_flags;

use std::path::PathBuf;

pub use edition::SpecEdition;
pub use error::Error262;
pub use structs::{ErrorType, Ignored, Outcome, Phase};
pub use test_files::{Harness, HarnessFile, MetaData, Test, TestSuite};
pub use test_flags::TestFlags;

/// Repository Url
pub const TEST262_REPOSITORY: &str = "https://github.com/tc39/test262";
/// Git clone directory
pub const TEST262_DIRECTORY: &str = "test262";

/// Clone TC39 test262 repostiory
pub fn clone_test262(commit: Option<&str>, verbose: u8) -> color_eyre::Result<()> {
    const TEST262_REPOSITORY: &str = "https://github.com/tc39/test262";
    git::clone(
        TEST262_DIRECTORY,
        TEST262_REPOSITORY,
        &"origin/main",
        commit,
        verbose,
    )
}

/// Test Read Result
#[derive(Debug)]
pub enum ReadResult {
    /// Single Test
    Test(Harness, Test),
    /// Test Suite
    TestSuite(Harness, TestSuite)
}

#[derive(Debug)]
/// Test Reading Options
pub struct ReadOptions<'a> {
    /// Git 262 repository path:
    /// Example: "/home/test262" or "./test262"
    pub test262_path: PathBuf,
    /// Example: "https://github.com/tc39/test262"
    pub test262_repository: Option<&'a str>,
    /// Example: "origin/main"
    pub test262_branch: Option<&'a str>,
    /// Example "de3a117f02e26a53f8b7cb41f6b4a0b8473c5db4"
    pub test262_commit: Option<&'a str>,
    /// Ignored configuration
    pub ignored: Option<Ignored>,
    /// Verbose
    pub verbose: u8,
}

impl Default for ReadOptions<'_> {
    fn default() -> Self {
        Self {
            test262_path: PathBuf::from(TEST262_DIRECTORY),
            test262_branch: Default::default(),
            test262_repository: Default::default(),
            test262_commit: Default::default(),
            ignored: Default::default(),
            verbose: 0,
        }
    }
}

/// Read Test/TestSuite from tc38/test262 repository
/// Example:
/// suite: test/intl402/constructors-string-and-single-element-array.js
/// options: Default:default()
pub fn read(suite: PathBuf, options: ReadOptions<'_>) -> Result<ReadResult, Error262> {
    let test_262_path = options.test262_path;
    let ignore = options.ignored.unwrap_or_default();
    let verbose = options.verbose;

    // Clone test262 repo
    if !test_262_path.exists() {
        let repo = options.test262_repository.unwrap_or(TEST262_REPOSITORY);
        let branch = options.test262_branch.unwrap_or("origin/main");
        let commit = options.test262_commit;
        git::clone(
            &test_262_path.to_string_lossy(),
            repo,
            branch,
            commit,
            verbose,
        ).expect("Failed to clone test262") // TODO
    }

    // Repo path validation
    if !test_262_path.is_dir() {
        return Err(Error262::InvalidRepo262Path {
            path: test_262_path,
        });
    }
    let Ok(test_262_path) = test_262_path.canonicalize() else {
        return Err(Error262::InvalidRepo262Path {
            path: test_262_path,
        });
    };

    // Test/TestSuite path validation
    let absolute_or_relative = suite.canonicalize().or(test_262_path.join(&suite).canonicalize());
    let Ok(suite) = absolute_or_relative else {
        return Err(Error262::InvalidSuitePath { path: suite });
    };
    if suite.is_absolute() && !suite.starts_with(&test_262_path) {
        return Err(Error262::InvalidSuitePath { path: suite });
    }


    let harness = Harness::read(&test_262_path).expect("Failed to read Harness"); // TOOD;

    if suite.to_string_lossy().ends_with(".js") {
        let test = Test::read(&suite)?;
        return Ok(ReadResult::Test(harness, test));
    } else if suite.is_dir() {
        let test = TestSuite::read(&suite, &ignore, false)?;
        return Ok(ReadResult::TestSuite(harness, test));
    }

    Err(Error262::InvalidSuitePath { path: suite })
}

#[cfg(test)]
mod tests {
    use crate::edition::SpecEdition;
    use crate::{Ignored, MetaData, TEST262_DIRECTORY};
    use std::path::{Path, PathBuf};

    #[test]
    #[ignore = "manual"]
    fn should_clone_test262() {
        super::clone_test262(None, 0).unwrap();
    }

    #[test]
    #[ignore = "manual"]
    fn should_read_harness() {
        let harness = super::Harness::read(Path::new(TEST262_DIRECTORY)).unwrap();
        assert!(harness.assert.path.is_file());
        assert!(harness.sta.path.is_file());
        assert!(harness.doneprint_handle.path.is_file());
    }

    #[test]
    #[ignore = "manual"]
    fn should_read_test_suite_and_test() {
        let path = Path::new(TEST262_DIRECTORY)
            .join("test")
            .join("language")
            .join("import");
        let test_suite = super::TestSuite::read(&path, &Ignored::default(), false).unwrap();
        assert!(!test_suite.name.is_empty());
        assert!(!test_suite.tests.is_empty());

        let test_path = &test_suite.tests[0].path;
        let test = super::Test::read(test_path);
        assert!(test.is_ok());
    }

    #[test]
    fn should_ignore_unknown_features() {
        let metadata = MetaData {
            description: String::into_boxed_str("test_example description".to_string()),
            esid: None,
            es5id: None,
            es6id: None,
            info: String::into_boxed_str("test_example".to_string()),
            features: Box::new([String::into_boxed_str("unknown_feature_abc".to_string())]),
            includes: Box::new([]),
            flags: Box::new([]),
            negative: None,
            locale: Default::default(),
        };
        assert_eq!(
            Ok(SpecEdition::ESNext),
            SpecEdition::from_test_metadata(&metadata)
        );
    }

    #[test]
    fn should_get_minimal_required_edition_from_test_metadata() {
        let metadata = MetaData {
            description: String::into_boxed_str("test_example description".to_string()),
            esid: None,
            es5id: None,
            es6id: None,
            info: String::into_boxed_str("test_example".to_string()),
            features: Box::new(
                [
                    String::into_boxed_str("TypedArray".to_string()), // ES6
                    String::into_boxed_str("well-formed-json-stringify".to_string()),
                ], // ES10
            ),
            includes: Box::new([]),
            flags: Box::new([]),
            negative: None,
            locale: Default::default(),
        };
        assert_eq!(
            Ok(SpecEdition::ES10),
            SpecEdition::from_test_metadata(&metadata)
        );
    }

    #[test]
    #[ignore = "manual"]
    fn should_read_test_with_path_relative_to_repository() {
        let suite = PathBuf::from("test/intl402/constructors-string-and-single-element-array.js");
        let result = super::read(suite, Default::default()).expect("Test");
        match result {
            crate::ReadResult::Test(_, test) =>  {
                assert_eq!(test.name.as_ref(), "constructors-string-and-single-element-array");
            },
            _ => unreachable!("ReadResult::Test was expected"),
        }
    }

    #[test]
    #[ignore = "manual"]
    fn should_read_test_with_path_relative_to_current_directory() {
        let suite = PathBuf::from("test262/test/intl402/constructors-string-and-single-element-array.js");
        let result = super::read(suite, Default::default()).expect("Test");
        match result {
            crate::ReadResult::Test(_, test) =>  {
                assert_eq!(test.name.as_ref(), "constructors-string-and-single-element-array");
            },
            _ => unreachable!("ReadResult::Test was expected"),
        }
    }

    #[test]
    #[ignore = "manual"]
    fn should_read_test_suite() {
        let suite = PathBuf::from("test262/test/intl402/");
        let result = super::read(suite, Default::default()).expect("Test");
        match result {
            crate::ReadResult::TestSuite(_, test) =>  {
                assert_eq!(test.name.as_ref(), "intl402");
            },
            _ => unreachable!("ReadResult::TestSuite was expected"),
        }
    }

    #[test]
    #[ignore = "manual"]
    fn should_read_test_suite_custom_options() {
        let suite = PathBuf::from("test262_2/test/intl402/");
        let result = super::read(suite, crate::ReadOptions {
            test262_path: PathBuf::from("./test262_2"),
            ..Default::default()
        }).expect("Test");

        match result {
            crate::ReadResult::TestSuite(_, test) =>  {
                assert_eq!(test.name.as_ref(), "intl402");
            },
            _ => unreachable!("ReadResult::TestSuite was expected"),
        }
    }
}
