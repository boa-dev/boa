//! TC39 test262
mod edition;
mod structs;
mod test_files;
mod test_flags;
mod git;
mod read;
mod error;

pub use error::Error262;
pub use edition::SpecEdition;
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

#[cfg(test)]
mod tests {
    use crate::edition::SpecEdition;
    use crate::{Ignored, MetaData, TEST262_DIRECTORY};
    use std::path::Path;

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
}
