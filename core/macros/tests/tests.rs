#![allow(unused_crate_dependencies)]

use boa_engine::{JsStr, JsString};
use boa_macros::{js_str, utf16};

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
fn latin1_is_wrong() {
    const NON_UTF8_LATIN1: JsStr<'_> = js_str!("Hello é World!");
    assert!(NON_UTF8_LATIN1.is_latin1());

    let js_string = JsString::from(NON_UTF8_LATIN1);
    assert_eq!(
        format!("{}", js_string.to_std_string_escaped()),
        "Hello é World!"
    );
}
