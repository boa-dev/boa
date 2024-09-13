//! Git common

use color_eyre::{eyre::bail, Result};
use std::{path::Path, process::Command};

/// Returns the commit hash and commit message of the provided branch name.
fn get_last_branch_commit(directory: &str, branch: &str, verbose: u8) -> Result<(String, String)> {
    if verbose > 1 {
        println!("Getting last commit on '{branch}' branch");
    }
    let result = Command::new("git")
        .arg("log")
        .args(["-n", "1"])
        .arg("--pretty=format:%H %s")
        .arg(branch)
        .current_dir(directory)
        .output()?;

    if !result.status.success() {
        bail!(
            "{directory} getting commit hash and message failed with return code {:?}",
            result.status.code()
        );
    }

    let output = std::str::from_utf8(&result.stdout)?.trim();

    let (hash, message) = output
        .split_once(' ')
        .expect("git log output to contain hash and message");

    Ok((hash.into(), message.into()))
}

fn reset_commit(directory: &str, commit: &str, verbose: u8) -> Result<()> {
    if verbose != 0 {
        println!("Reset {directory} to commit: {commit}...");
    }

    let result = Command::new("git")
        .arg("reset")
        .arg("--hard")
        .arg(commit)
        .current_dir(directory)
        .status()?;

    if !result.success() {
        bail!(
            "{directory} commit {commit} checkout failed with return code: {:?}",
            result.code()
        );
    }

    Ok(())
}

/// Clone repository
pub(super) fn clone(
    directory: &str,
    repor_url: &str, // "https://github.com/tc39/test262"
    baranch: &str, // "origin/main"
    commit: Option<&str>,
    verbose: u8,
) -> Result<()> {
    let update = commit.is_none();

    if Path::new(directory).is_dir() {
        let (current_commit_hash, current_commit_message) =
            get_last_branch_commit(directory, "HEAD", verbose)?;

        if let Some(commit) = commit {
            if current_commit_hash == commit {
                return Ok(());
            }
        }

        if verbose != 0 {
            println!("Fetching latest {directory} commits...");
        }
        let result = Command::new("git")
            .arg("fetch")
            .current_dir(directory)
            .status()?;

        if !result.success() {
            bail!(
                "{directory} fetching latest failed with return code {:?}",
                result.code()
            );
        }

        if let Some(commit) = commit {
            println!("{directory} switching to commit {commit}...");
            reset_commit(directory, commit, verbose)?;
            return Ok(());
        }

        if verbose != 0 {
            println!("Checking latest {directory} with current HEAD...");
        }
        let (latest_commit_hash, latest_commit_message) =
            get_last_branch_commit(directory, baranch, verbose)?;

        if current_commit_hash != latest_commit_hash {
            if update {
                println!("Updating {directory} repository:");
            } else {
                println!("Warning {directory} repository is not in sync, use '--test262-commit latest' to automatically update it:");
            }

            println!("    Current commit: {current_commit_hash} {current_commit_message}");
            println!("    Latest commit:  {latest_commit_hash} {latest_commit_message}");

            if update {
                reset_commit(&directory, &latest_commit_hash, verbose)?;
            }
        }

        return Ok(());
    }

    println!("Cloning {repor_url} into {directory} ...");
    let result = Command::new("git")
        .arg("clone")
        .arg(repor_url)
        .arg(directory)
        .status()?;

    if !result.success() {
        bail!(
            "Cloning {repor_url} repository failed with return code {:?}",
            result.code()
        );
    }

    if let Some(commit) = commit {
        if verbose != 0 {
            println!("Reset {repor_url} to commit: {commit}...");
        }

        reset_commit(directory, commit, verbose)?;
    }

    Ok(())
}
