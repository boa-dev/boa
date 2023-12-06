#![allow(unused_crate_dependencies, unused_tuple_struct_fields)]

#[test]
fn try_from_js() {
    let t = trybuild::TestCases::new();
    t.pass("tests/derive/simple_struct.rs");
    t.pass("tests/derive/from_js_with.rs");
}
