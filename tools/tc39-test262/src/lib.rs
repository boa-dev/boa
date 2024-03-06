//! TC39 test262
mod edition;
mod structs;
mod test_files;
mod test_flags;

pub mod git;
pub mod read;
pub use structs::{ErrorType, Outcome, Phase, Ignored};
pub use test_files::{Harness, HarnessFile, MetaData, Test, TestSuite};
pub use test_flags::TestFlags;
pub use edition::SpecEdition;

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

    #[test]
    #[ignore = "manual testing"]
    fn should_clone_test262() {
        super::clone_test262(None, 0).unwrap();
    }
}
