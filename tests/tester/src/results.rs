use crate::{Statistics, TestOutcomeResult, VersionedStats};

use super::SuiteResult;
use color_eyre::{eyre::WrapErr, Result};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{BufReader, BufWriter},
    path::Path,
};

/// Structure to store full result information.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ResultInfo {
    #[serde(rename = "c")]
    commit: Box<str>,
    #[serde(rename = "u")]
    test262_commit: Box<str>,
    #[serde(rename = "a")]
    stats: Statistics,
    #[serde(rename = "v")]
    versioned_results: VersionedStats,
    #[serde(rename = "r")]
    results: FxHashMap<Box<str>, SuiteResult>,
}

/// Structure to store full result information.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ReducedResultInfo {
    #[serde(rename = "c")]
    commit: Box<str>,
    #[serde(rename = "u")]
    test262_commit: Box<str>,
    #[serde(rename = "a")]
    stats: Statistics,
}

impl From<ResultInfo> for ReducedResultInfo {
    /// Creates a new reduced suite result from a full suite result.
    fn from(info: ResultInfo) -> Self {
        Self {
            commit: info.commit,
            test262_commit: info.test262_commit,
            stats: info.stats,
        }
    }
}

/// File name of the "latest results" JSON file.
const LATEST_FILE_NAME: &str = "latest.json";

/// File name of the "all results" JSON file.
const RESULTS_FILE_NAME: &str = "results.json";

/// Writes the results of running the test suite to the given JSON output file.
///
/// It will append the results to the ones already present, in an array.
pub(crate) fn write_json(
    results: SuiteResult,
    output_dir: &Path,
    verbose: u8,
    test262_path: &Path,
) -> Result<()> {
    let mut branch = env::var("GITHUB_REF").unwrap_or_default();
    if branch.starts_with("refs/pull") {
        branch = "pull".to_owned();
    }

    let output_dir = if branch.is_empty() {
        output_dir.to_path_buf()
    } else {
        let folder = output_dir.join(&branch);
        fs::create_dir_all(&folder)?;
        folder
    };

    // We make sure we are using the latest commit information in GitHub pages:
    update_gh_pages_repo(output_dir.as_path(), &branch, verbose);

    if verbose != 0 {
        println!("Writing the results to {}...", output_dir.display());
    }

    // Write the latest results.

    let latest = output_dir.join(LATEST_FILE_NAME);

    let new_results = ResultInfo {
        commit: env::var("GITHUB_SHA").unwrap_or_default().into_boxed_str(),
        test262_commit: get_test262_commit(test262_path)
            .context("could not get the test262 commit")?,
        stats: results.stats,
        versioned_results: results.versioned_stats,
        results: results.suites,
    };

    let latest = BufWriter::new(fs::File::create(latest)?);
    serde_json::to_writer(latest, &new_results)?;

    // Write the full result history for "main"
    if branch == "main" {
        let all_path = output_dir.join(RESULTS_FILE_NAME);

        // We only keep history for the main branch
        let mut all_results: Vec<ReducedResultInfo> = if all_path.is_file() {
            serde_json::from_reader(BufReader::new(fs::File::open(&all_path)?))?
        } else {
            Vec::new()
        };

        all_results.push(new_results.into());

        let output = BufWriter::new(fs::File::create(&all_path)?);
        serde_json::to_writer(output, &all_results)?;
    }

    if verbose != 0 {
        println!("Results written correctly");
    }

    Ok(())
}

/// Gets the commit OID of the test262 submodule.
fn get_test262_commit(test262_path: &Path) -> Result<Box<str>> {
    let main_head_path = test262_path.join(".git/refs/heads/main");
    let mut commit_id = fs::read_to_string(main_head_path)?;
    // Remove newline.
    commit_id.pop();
    Ok(commit_id.into_boxed_str())
}

/// Updates the GitHub pages repository by pulling latest changes before writing the new things.
fn update_gh_pages_repo(path: &Path, branch: &str, verbose: u8) {
    use std::process::Command;

    // We run the command to pull the gh-pages branch: git -C ../gh-pages/ pull --ff-only
    if verbose != 0 {
        println!("Cloning the Github Pages branch in ../gh-pages/...");
    }
    Command::new("git")
        .args(["-C", "../gh-pages", "pull", "--ff-only"])
        .output()
        .expect("could not update GitHub Pages");

    if branch == "main" {
        // Copy the full results file if in the main branch
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
#[allow(clippy::cast_possible_wrap)]
pub(crate) fn compare_results(base: &Path, new: &Path, markdown: bool) -> Result<()> {
    let base_results: ResultInfo = serde_json::from_reader(BufReader::new(
        fs::File::open(base).wrap_err("could not open the base results file")?,
    ))
    .wrap_err("could not read the base results")?;

    let new_results: ResultInfo = serde_json::from_reader(BufReader::new(
        fs::File::open(new).wrap_err("could not open the new results file")?,
    ))
    .wrap_err("could not read the new results")?;

    let base_total = base_results.stats.total as isize;
    let new_total = new_results.stats.total as isize;
    let total_diff = new_total - base_total;

    let base_passed = base_results.stats.passed as isize;
    let new_passed = new_results.stats.passed as isize;
    let passed_diff = new_passed - base_passed;

    let base_ignored = base_results.stats.ignored as isize;
    let new_ignored = new_results.stats.ignored as isize;
    let ignored_diff = new_ignored - base_ignored;

    let base_failed = base_total - base_passed - base_ignored;
    let new_failed = new_total - new_passed - new_ignored;
    let failed_diff = new_failed - base_failed;

    let base_panics = base_results.stats.panic as isize;
    let new_panics = new_results.stats.panic as isize;
    let panic_diff = new_panics - base_panics;

    let base_conformance = (base_passed as f64 / base_total as f64) * 100_f64;
    let new_conformance = (new_passed as f64 / new_total as f64) * 100_f64;
    let conformance_diff = new_conformance - base_conformance;

    let test_diff = std::iter::zip(base_results.results, new_results.results).fold(
        ResultDiff::default(),
        |mut diff, ((base_name, base_suite), (_new_name, new_suite))| {
            diff.extend(compute_result_diff(
                Path::new(base_name.as_ref()),
                &base_suite,
                &new_suite,
            ));
            diff
        },
    );

    if markdown {
        /// Simple function to add commas as thousands separator for integers.
        fn pretty_int(i: isize) -> String {
            let mut res = String::new();

            for (idx, val) in i.abs().to_string().chars().rev().enumerate() {
                if idx != 0 && idx % 3 == 0 {
                    res.insert(0, ',');
                }
                res.insert(0, val);
            }
            res
        }

        /// Generates a proper diff format, with some bold text if things change.
        fn diff_format(diff: isize) -> String {
            format!(
                "{}{}{}{}",
                if diff == 0 { "" } else { "**" },
                if diff.is_positive() {
                    "+"
                } else if diff.is_negative() {
                    "-"
                } else {
                    ""
                },
                pretty_int(diff),
                if diff == 0 { "" } else { "**" }
            )
        }

        println!("| Test result | main count | PR count | difference |");
        println!("| :---------: | :----------: | :------: | :--------: |");
        println!(
            "| Total | {} | {} | {} |",
            pretty_int(base_total),
            pretty_int(new_total),
            diff_format(total_diff),
        );
        println!(
            "| Passed | {} | {} | {} |",
            pretty_int(base_passed),
            pretty_int(new_passed),
            diff_format(passed_diff),
        );
        println!(
            "| Ignored | {} | {} | {} |",
            pretty_int(base_ignored),
            pretty_int(new_ignored),
            diff_format(ignored_diff),
        );
        println!(
            "| Failed | {} | {} | {} |",
            pretty_int(base_failed),
            pretty_int(new_failed),
            diff_format(failed_diff),
        );
        println!(
            "| Panics | {} | {} | {} |",
            pretty_int(base_panics),
            pretty_int(new_panics),
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

    Ok(())
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
    let mut final_diff = ResultDiff::default();

    for (base_name, base_test) in &base_result.tests {
        if let Some((new_name, new_test)) = new_result
            .tests
            .iter()
            .find(|(new_name, _new_test)| new_name == &base_name)
        {
            let test_name = format!("test/{}/{}.js", base.display(), new_name);

            if let (Some(base_result), Some(new_result)) = (base_test.strict, new_test.strict) {
                let test_name = format!("{test_name} [strict mode] (previously {base_result:?})");

                add_results(&mut final_diff, test_name, base_result, new_result);
            }

            if let (Some(base_result), Some(new_result)) = (base_test.strict, new_test.strict) {
                let test_name = format!("{test_name} (previously {base_result:?})");

                add_results(&mut final_diff, test_name, base_result, new_result);
            }
        }
    }

    for (base_name, base_suite) in &base_result.suites {
        if let Some((new_name, new_suite)) = new_result
            .suites
            .iter()
            .find(|(new_name, _new_suite)| new_name == &base_name)
        {
            let new_base = base.join(new_name.as_ref());
            let diff = compute_result_diff(new_base.as_path(), base_suite, new_suite);

            final_diff.extend(diff);
        }
    }

    final_diff
}

/// Adds the results to the final diff.
fn add_results<N>(
    final_diff: &mut ResultDiff,
    test_name: N,
    base_result: TestOutcomeResult,
    new_result: TestOutcomeResult,
) where
    N: Into<Box<str>>,
{
    let test_name = test_name.into();

    match (base_result, new_result) {
        (a, b) if a == b => {}
        (TestOutcomeResult::Ignored, TestOutcomeResult::Failed) => {}

        (_, TestOutcomeResult::Passed) => final_diff.fixed.push(test_name),
        (TestOutcomeResult::Panic, _) => final_diff.panic_fixes.push(test_name),
        (_, TestOutcomeResult::Failed) => final_diff.broken.push(test_name),
        (_, TestOutcomeResult::Panic) => final_diff.new_panics.push(test_name),

        _ => {}
    }
}
