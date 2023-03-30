use boa_macros::utf16;

#[test]
fn literal() {
    let utf16 = utf16!("hello!");
    let manual = "hello!".encode_utf16().collect::<Vec<_>>();
    assert_eq!(manual, utf16);
}

#[test]
fn utf16() {
    let utf16 = utf16!("hello!😁😁😁");
    let manual = "hello!😁😁😁".encode_utf16().collect::<Vec<_>>();
    assert_eq!(manual, utf16);
}

#[test]
fn try_from_js() {
    let t = trybuild::TestCases::new();
    t.pass("tests/derive/simple_struct.rs");
    t.pass("tests/derive/from_js_with.rs");
}
