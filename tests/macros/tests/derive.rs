//! Tests for the `TryFromJs` derive macro.

#![allow(unused_crate_dependencies)]

#[test]
fn try_from_js() {
    let t = trybuild::TestCases::new();
    t.pass("tests/derive/*.rs");
}
