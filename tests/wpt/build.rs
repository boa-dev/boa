#![allow(dead_code)]

use serde::Deserialize;
use std::path::Path;

/// Structure that contains the configuration of the tester.
#[derive(Debug, Deserialize)]
struct Config {
    rev: Option<String>,
}

fn prep_repository(rev: &str, root: impl AsRef<Path>) {
    let root = root.as_ref();
    // See if the repo already exists.
    let repo = if !std::fs::exists(root).unwrap() {
        // This repo is quite large, so do a shallow clone then perform the update.
        let mut options = git2::FetchOptions::new();
        options.depth(1);

        let mut repo_builder = git2::build::RepoBuilder::new();
        repo_builder.fetch_options(options);

        repo_builder
            .clone("https://github.com/web-platform-tests/wpt", root)
            .expect("Could not clone")
    } else {
        git2::Repository::open(root).unwrap()
    };

    let head_sha = repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id()
        .to_string();
    if head_sha == rev {
        // There's nothing to do.
        return;
    }

    // Fetch the sha instead.
    let mut options = git2::FetchOptions::new();
    options.depth(1);
    repo.find_remote("origin")
        .expect("Could not find remote (origin)")
        .fetch(&[rev], Some(&mut options), None)
        .expect("Could not fetch repo");

    // Then checkout to it and we're done.
    repo.reset(
        repo.find_commit(git2::Oid::from_str(rev).unwrap())
            .unwrap()
            .as_object(),
        git2::ResetType::Hard,
        None,
    )
    .expect("Could not reset the repository");
}

fn main() {
    println!("cargo:rerun-if-changed=tests_wpt");

    let config: Config = {
        let input = std::fs::read_to_string("../../test_wpt_config.toml")
            .expect("Could not read config file");
        toml::from_str(&input).expect("Config file is invalid TOML")
    };

    let root = "../../tests_wpt";

    // Clone the WPT repository.
    if let Some(rev) = config.rev {
        prep_repository(&rev, root);
    }

    // If user already declared WPT_ROOT, keep it.
    if std::env::var("WPT_ROOT").is_err() {
        println!("cargo:rerun-if-changed={}", root);
        println!("cargo:rustc-env=WPT_ROOT={}", root);
    }
}
