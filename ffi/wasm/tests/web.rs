#![allow(unused_crate_dependencies)]
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
