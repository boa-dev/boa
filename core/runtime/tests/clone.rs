#![allow(unused_crate_dependencies, missing_docs)]

use boa_engine::{Context, Source};
use boa_runtime::RuntimeExtension;
use rstest::rstest;
use std::path::PathBuf;

#[rstest]
fn clone(#[files("tests/clone/**/*.js")] path: PathBuf) {
    let context = &Context::default();
    boa_runtime::clone::register(None, context).expect("Could not register runtime");
    boa_runtime::extensions::ConsoleExtension::default()
        .register(None, context)
        .expect("Could not register console");

    let harness_path = PathBuf::from("./assets/harness.js");
    let harness = Source::from_filepath(&harness_path).expect("Could not load harness");
    context.eval(harness).expect("Could not eval source");

    let source = Source::from_filepath(&path).expect("Could not load source");

    if let Err(e) = context.eval(source) {
        panic!("Evaluation failed: {e}");
    }

    if let Err(e) = context.run_jobs() {
        panic!("Execution error: {e}");
    }
}
