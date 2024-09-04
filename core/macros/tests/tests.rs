#![allow(unused_crate_dependencies)]

use boa_engine::value::TryFromJs;
use boa_engine::{js_string, Context, JsResult, JsString, JsValue, Source};
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
fn try_from_js_derive() {
    #[derive(Debug, TryFromJs, Eq, PartialEq)]
    struct TryFromJsTest {
        a: JsString,
        #[boa(rename = "bBB")]
        b: i32,
        #[boa(from_js_with = "check_tfj_called")]
        c: i32,
    }

    fn check_tfj_called(value: &JsValue, context: &mut Context) -> JsResult<i32> {
        let v = value.to_i32(context)?;
        Ok(v / 2)
    }

    let mut context = Context::default();
    let obj = context
        .eval(Source::from_bytes(br#"({ a: "hello", bBB: 42, c: 120 })"#))
        .unwrap();

    let result = TryFromJsTest::try_from_js(&obj, &mut context).unwrap();
    assert_eq!(
        result,
        TryFromJsTest {
            a: js_string!("hello"),
            b: 42,
            c: 60
        }
    );
}
