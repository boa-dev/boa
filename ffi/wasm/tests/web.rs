//! Tests for the wasm module.

#![expect(
    unused_crate_dependencies,
    reason = "https://github.com/rust-lang/rust/issues/95513"
)]
#![cfg(all(
    any(target_arch = "wasm32", target_arch = "wasm64"),
    target_os = "unknown"
))]
#![allow(missing_docs)]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn simple() {
    const CODE: &str = r"
    function greet(targetName) {
      return 'Hello, ' + targetName + '!';
    }

    greet('World')
    ";

    let result = boa_wasm::evaluate(CODE).unwrap();

    assert_eq!(result, "\"Hello, World!\"");
}
