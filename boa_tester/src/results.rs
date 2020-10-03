use super::{SuiteResult, CLI};
use git2::Repository;
use hex::ToHex;
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
    #[serde(rename = "p")]
    passed: usize,
    #[serde(rename = "i")]
    ignored: usize,
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
pub(crate) fn write_json(results: SuiteResult) -> io::Result<()> {
    if let Some(path) = CLI.output() {
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

        if CLI.verbose() {
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

        all_results.push(new_results.into());

        let output = BufWriter::new(fs::File::create(&all_path)?);
        serde_json::to_writer(output, &all_results)?;

        if CLI.verbose() {
            println!("Results written correctly");
        }
    }

    Ok(())
}

/// Gets the commit OID of the test262 submodule.
fn get_test262_commit() -> Box<str> {
    let repo = Repository::open(".").expect("could not open git repository in current directory");

    let submodule = repo
        .submodules()
        .expect("could not get the list of submodules of the repo")
        .into_iter()
        .find(|sub| dbg!(sub.path()) == Path::new("test262"))
        .expect("test262 submodule not found");

    submodule
        .index_id()
        .expect("could not get the commit OID")
        .encode_hex::<String>()
        .into_boxed_str()
}
