use super::SuiteResult;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, BufReader, BufWriter},
    path::Path,
};

/// Structure to store full result information.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ResultInfo {
    #[serde(rename = "c")]
    commit: Box<str>,
    #[serde(rename = "u")]
    test262_commit: Box<str>,
    #[serde(rename = "r")]
    results: SuiteResult,
}

/// Structure to store full result information.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ReducedResultInfo {
    #[serde(rename = "c")]
    commit: Box<str>,
    #[serde(rename = "u")]
    test262_commit: Box<str>,
    #[serde(rename = "t")]
    total: usize,
    #[serde(rename = "o")]
    passed: usize,
    #[serde(rename = "i")]
    ignored: usize,
    #[serde(rename = "p")]
    panic: usize,
}

impl From<ResultInfo> for ReducedResultInfo {
    /// Creates a new reduced suite result from a full suite result.
    fn from(info: ResultInfo) -> Self {
        Self {
            commit: info.commit,
            test262_commit: info.test262_commit,
            total: info.results.total,
            passed: info.results.passed,
            ignored: info.results.ignored,
            panic: info.results.panic,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FeaturesInfo {
    #[serde(rename = "c")]
    commit: Box<str>,
    #[serde(rename = "u")]
    test262_commit: Box<str>,
    #[serde(rename = "n")]
    suite_name: Box<str>,
    #[serde(rename = "f")]
    features: Vec<String>,
}

fn remove_duplicates(features_vec: &[String]) -> Vec<String> {
    let mut result = features_vec.to_vec();
    result.sort();
    result.dedup();
    result
}

impl From<ResultInfo> for FeaturesInfo {
    fn from(info: ResultInfo) -> Self {
        Self {
            commit: info.commit,
            test262_commit: info.test262_commit,
            suite_name: info.results.name,
            features: remove_duplicates(&info.results.features),
        }
    }
}

/// File name of the "latest results" JSON file.
const LATEST_FILE_NAME: &str = "latest.json";

/// File name of the "all results" JSON file.
const RESULTS_FILE_NAME: &str = "results.json";

/// File name of the "features" JSON file.
const FEATURES_FILE_NAME: &str = "features.json";

/// Writes the results of running the test suite to the given JSON output file.
///
/// It will append the results to the ones already present, in an array.
pub(crate) fn write_json(
    results: SuiteResult,
    output: Option<&Path>,
    verbose: u8,
) -> io::Result<()> {
    if let Some(path) = output {
        let mut branch = env::var("GITHUB_REF").unwrap_or_default();
        if branch.starts_with("refs/pull") {
            branch = "pull".to_owned();
        }

        let path = if branch.is_empty() {
            path.to_path_buf()
        } else {
            let folder = path.join(branch);
            fs::create_dir_all(&folder)?;
            folder
        };

        // We make sure we are using the latest commit information in GitHub pages:
        update_gh_pages_repo(path.as_path(), verbose);

        if verbose != 0 {
            println!("Writing the results to {}...", path.display());
        }

        // Write the latest results.

        let latest_path = path.join(LATEST_FILE_NAME);

        let new_results = ResultInfo {
            commit: env::var("GITHUB_SHA").unwrap_or_default().into_boxed_str(),
            test262_commit: get_test262_commit(),
            results,
        };

        let latest_output = BufWriter::new(fs::File::create(latest_path)?);
        serde_json::to_writer(latest_output, &new_results)?;

        // Write the full list of results, retrieving the existing ones first.

        let all_path = path.join(RESULTS_FILE_NAME);

        let mut all_results: Vec<ReducedResultInfo> = if all_path.exists() {
            serde_json::from_reader(BufReader::new(fs::File::open(&all_path)?))?
        } else {
            Vec::new()
        };

        all_results.push(new_results.clone().into());

        let output = BufWriter::new(fs::File::create(&all_path)?);
        serde_json::to_writer(output, &all_results)?;

        if verbose != 0 {
            println!("Results written correctly");
        }

        // Write the full list of features, existing features go first.

        let features_path = path.join(FEATURES_FILE_NAME);

        let mut all_features: Vec<FeaturesInfo> = if features_path.exists() {
            serde_json::from_reader(BufReader::new(fs::File::open(&features_path)?))?
        } else {
            Vec::new()
        };

        all_features.push(new_results.into());

        let features_output = BufWriter::new(fs::File::create(&features_path)?);
        serde_json::to_writer(features_output, &all_features)?;

        if verbose != 0 {
            println!("Features written correctly");
        }
    }

    Ok(())
}

/// Gets the commit OID of the test262 submodule.
fn get_test262_commit() -> Box<str> {
    let mut commit_id = fs::read_to_string(".git/modules/test262/HEAD")
        .expect("did not find git submodule ref at '.git/modules/test262/HEAD'");
    // Remove newline.
    commit_id.pop();
    commit_id.into_boxed_str()
}

/// Updates the GitHub pages repository by pulling latest changes before writing the new things.
fn update_gh_pages_repo(path: &Path, verbose: u8) {
    if env::var("GITHUB_REF").is_ok() {
        use std::process::Command;

        // We run the command to pull the gh-pages branch: git -C ../gh-pages/ pull origin
        Command::new("git")
            .args(&["-C", "../gh-pages", "pull", "--ff-only"])
            .output()
            .expect("could not update GitHub Pages");

        // Copy the full results file
        let from = Path::new("../gh-pages/test262/refs/heads/main/").join(RESULTS_FILE_NAME);
        let to = path.join(RESULTS_FILE_NAME);

        if verbose != 0 {
            println!(
                "Copying the {} file to {} in order to add the results",
                from.display(),
                to.display()
            );
        }

        fs::copy(from, to).expect("could not copy the main results file");
    }
}

/// Compares the results of two test suite runs.
pub(crate) fn compare_results(base: &Path, new: &Path, markdown: bool) {
    let base_results: ResultInfo = serde_json::from_reader(BufReader::new(
        fs::File::open(base).expect("could not open the base results file"),
    ))
    .expect("could not read the base results");

    let new_results: ResultInfo = serde_json::from_reader(BufReader::new(
        fs::File::open(new).expect("could not open the new results file"),
    ))
    .expect("could not read the new results");

    let base_total = base_results.results.total as isize;
    let new_total = new_results.results.total as isize;
    let total_diff = new_total - base_total;

    let base_passed = base_results.results.passed as isize;
    let new_passed = new_results.results.passed as isize;
    let passed_diff = new_passed - base_passed;

    let base_ignored = base_results.results.ignored as isize;
    let new_ignored = new_results.results.ignored as isize;
    let ignored_diff = new_ignored - base_ignored;

    let base_failed = base_total - base_passed - base_ignored;
    let new_failed = new_total - new_passed - new_ignored;
    let failed_diff = new_failed - base_failed;

    let base_panics = base_results.results.panic as isize;
    let new_panics = new_results.results.panic as isize;
    let panic_diff = new_panics - base_panics;

    let base_conformance = (base_passed as f64 / base_total as f64) * 100_f64;
    let new_conformance = (new_passed as f64 / new_total as f64) * 100_f64;
    let conformance_diff = new_conformance - base_conformance;

    let test_diff = compute_result_diff(Path::new(""), &base_results.results, &new_results.results);

    if markdown {
        use num_format::{Locale, ToFormattedString};

        /// Generates a proper diff format, with some bold text if things change.
        fn diff_format(diff: isize) -> String {
            format!(
                "{}{}{}{}",
                if diff == 0 { "" } else { "**" },
                if diff > 0 { "+" } else { "" },
                diff.to_formatted_string(&Locale::en),
                if diff == 0 { "" } else { "**" }
            )
        }

        println!("#### VM implementation");

        println!("| Test result | main count | PR count | difference |");
        println!("| :---------: | :----------: | :------: | :--------: |");
        println!(
            "| Total | {} | {} | {} |",
            base_total.to_formatted_string(&Locale::en),
            new_total.to_formatted_string(&Locale::en),
            diff_format(total_diff),
        );
        println!(
            "| Passed | {} | {} | {} |",
            base_passed.to_formatted_string(&Locale::en),
            new_passed.to_formatted_string(&Locale::en),
            diff_format(passed_diff),
        );
        println!(
            "| Ignored | {} | {} | {} |",
            base_ignored.to_formatted_string(&Locale::en),
            new_ignored.to_formatted_string(&Locale::en),
            diff_format(ignored_diff),
        );
        println!(
            "| Failed | {} | {} | {} |",
            base_failed.to_formatted_string(&Locale::en),
            new_failed.to_formatted_string(&Locale::en),
            diff_format(failed_diff),
        );
        println!(
            "| Panics | {} | {} | {} |",
            base_panics.to_formatted_string(&Locale::en),
            new_panics.to_formatted_string(&Locale::en),
            diff_format(panic_diff),
        );
        println!(
            "| Conformance | {:.2}% | {:.2}% | {}{}{:.2}%{} |",
            base_conformance,
            new_conformance,
            if conformance_diff.abs() > f64::EPSILON {
                "**"
            } else {
                ""
            },
            if conformance_diff > 0_f64 { "+" } else { "" },
            conformance_diff,
            if conformance_diff.abs() > f64::EPSILON {
                "**"
            } else {
                ""
            },
        );

        if !test_diff.fixed.is_empty() {
            println!();
            println!(
                "<details><summary><b>Fixed tests ({}):</b></summary>",
                test_diff.fixed.len()
            );
            println!("\n```");
            for test in test_diff.fixed {
                println!("{test}");
            }
            println!("```");
            println!("</details>");
        }

        if !test_diff.broken.is_empty() {
            println!();
            println!(
                "<details><summary><b>Broken tests ({}):</b></summary>",
                test_diff.broken.len()
            );
            println!("\n```");
            for test in test_diff.broken {
                println!("{test}");
            }
            println!("```");
            println!("</details>");
        }

        if !test_diff.new_panics.is_empty() {
            println!();
            println!(
                "<details><summary><b>New panics ({}):</b></summary>",
                test_diff.new_panics.len()
            );
            println!("\n```");
            for test in test_diff.new_panics {
                println!("{test}");
            }
            println!("```");
            println!("</details>");
        }

        if !test_diff.panic_fixes.is_empty() {
            println!();
            println!(
                "<details><summary><b>Fixed panics ({}):</b></summary>",
                test_diff.panic_fixes.len()
            );
            println!("\n```");
            for test in test_diff.panic_fixes {
                println!("{test}");
            }
            println!("```");
            println!("</details>");
        }
    } else {
        println!("Test262 conformance changes:");
        println!("| Test result | main |    PR   | difference |");
        println!(
            "|    Passed   | {base_passed:^6} | {new_passed:^5} | {:^10} |",
            base_passed - new_passed
        );
        println!(
            "|   Ignored   | {base_ignored:^6} | {new_ignored:^5} | {:^10} |",
            base_ignored - new_ignored
        );
        println!(
            "|   Failed    | {base_failed:^6} | {new_failed:^5} | {:^10} |",
            base_failed - new_failed,
        );
        println!(
            "|   Panics    | {base_panics:^6} | {new_panics:^5} | {:^10} |",
            base_panics - new_panics
        );

        if !test_diff.fixed.is_empty() {
            println!();
            println!("Fixed tests ({}):", test_diff.fixed.len());
            for test in test_diff.fixed {
                println!("{test}");
            }
        }

        if !test_diff.broken.is_empty() {
            println!();
            println!("Broken tests ({}):", test_diff.broken.len());
            for test in test_diff.broken {
                println!("{test}");
            }
        }

        if !test_diff.new_panics.is_empty() {
            println!();
            println!("New panics ({}):", test_diff.new_panics.len());
            for test in test_diff.new_panics {
                println!("{test}");
            }
        }

        if !test_diff.panic_fixes.is_empty() {
            println!();
            println!("Fixed panics ({}):", test_diff.panic_fixes.len());
            for test in test_diff.panic_fixes {
                println!("{test}");
            }
        }
    }
}

/// Test differences.
#[derive(Debug, Clone, Default)]
struct ResultDiff {
    fixed: Vec<Box<str>>,
    broken: Vec<Box<str>>,
    new_panics: Vec<Box<str>>,
    panic_fixes: Vec<Box<str>>,
}

impl ResultDiff {
    /// Extends the diff with new results.
    fn extend(&mut self, new: Self) {
        self.fixed.extend(new.fixed);
        self.broken.extend(new.broken);
        self.new_panics.extend(new.new_panics);
        self.panic_fixes.extend(new.panic_fixes);
    }
}

/// Compares a base and a new result and returns the list of differences.
fn compute_result_diff(
    base: &Path,
    base_result: &SuiteResult,
    new_result: &SuiteResult,
) -> ResultDiff {
    use super::TestOutcomeResult;

    let mut final_diff = ResultDiff::default();

    for base_test in &base_result.tests {
        if let Some(new_test) = new_result
            .tests
            .iter()
            .find(|new_test| new_test.strict == base_test.strict && new_test.name == base_test.name)
        {
            let test_name = format!(
                "test/{}/{}.js {}(previously {:?})",
                base.display(),
                new_test.name,
                if base_test.strict {
                    "[strict mode] "
                } else {
                    ""
                },
                base_test.result
            )
            .into_boxed_str();

            #[allow(clippy::match_same_arms)]
            match (base_test.result, new_test.result) {
                (a, b) if a == b => {}
                (TestOutcomeResult::Ignored, TestOutcomeResult::Failed) => {}

                (_, TestOutcomeResult::Passed) => final_diff.fixed.push(test_name),
                (TestOutcomeResult::Panic, _) => final_diff.panic_fixes.push(test_name),
                (_, TestOutcomeResult::Failed) => final_diff.broken.push(test_name),
                (_, TestOutcomeResult::Panic) => final_diff.new_panics.push(test_name),

                _ => {}
            }
        }
    }

    for base_suite in &base_result.suites {
        if let Some(new_suite) = new_result
            .suites
            .iter()
            .find(|new_suite| new_suite.name == base_suite.name)
        {
            let new_base = base.join(new_suite.name.as_ref());
            let diff = compute_result_diff(new_base.as_path(), base_suite, new_suite);

            final_diff.extend(diff);
        }
    }

    final_diff
}
