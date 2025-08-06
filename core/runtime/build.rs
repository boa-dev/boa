#![allow(unused_crate_dependencies, missing_docs)]

pub fn main() {
    // Rerun the tests if the test files change.
    println!("cargo::rerun-if-changed=tests/");
}
