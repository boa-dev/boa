#![allow(unused_crate_dependencies)]

<<<<<<< Updated upstream
use boa_macros::utf16;
=======
use boa_engine::{string::JsStrVariant, JsStr, JsString};
use boa_macros::{js_str, utf16};
>>>>>>> Stashed changes

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

#[test]
fn latin1() {
    let s = js_str!("Hello é World!");
    assert!(s.is_latin1());
    eprintln!(
        "{:?}",
        match s.variant() {
            JsStrVariant::Latin1(s) => s,
            _ => unreachable!(),
        }
    );

    let rust_str = format!("{}", JsString::from(s).to_std_string_escaped());
    assert_eq!(rust_str, "Hello é World!");

    assert!(false);
}
