//! Tests for the macros in this crate.

#![allow(unused_crate_dependencies)]

use boa_engine::value::{TryFromJs, TryIntoJs};
use boa_engine::{Context, JsResult, JsValue, Source, js_string};
use boa_string::JsString;

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

/// Regression test for #4360: TryIntoJs-derived structs must have Object.prototype in their
/// prototype chain, ensuring toString() returns "[object Object]" rather than throwing.
#[test]
fn try_into_js_has_object_prototype() {
    #[derive(TryIntoJs)]
    struct MyStruct {
        name: String,
        value: i32,
    }

    let mut context = Context::default();

    let s = MyStruct {
        name: "test".to_string(),
        value: 42,
    };

    let js_val = s.try_into_js(&mut context).unwrap();
    let obj = js_val.as_object().expect("should be an object");

    // toString should return "[object Object]" (proves Object.prototype is in the chain)
    let to_string_result = obj
        .get(js_string!("toString"), &mut context)
        .unwrap();
    assert!(!to_string_result.is_undefined(), "toString should be inherited from Object.prototype");

    let result = context
        .eval(Source::from_bytes(b"(function(o) { return o.toString(); })"))
        .unwrap();
    let func = result.as_callable().unwrap();
    let string_result = func
        .call(&JsValue::undefined(), &[js_val.clone()], &mut context)
        .unwrap();
    assert_eq!(string_result, JsValue::from(js_string!("[object Object]")));

    // Properties should be accessible
    let name_val = obj.get(js_string!("name"), &mut context).unwrap();
    assert_eq!(name_val, JsValue::from(js_string!("test")));

    let value_val = obj.get(js_string!("value"), &mut context).unwrap();
    assert_eq!(value_val, JsValue::from(42));

    // hasOwnProperty should work (proves Object.prototype is in the chain)
    let has_own = context
        .eval(Source::from_bytes(
            b"(function(o) { return o.hasOwnProperty('name'); })",
        ))
        .unwrap();
    let func = has_own.as_callable().unwrap();
    let result = func
        .call(&JsValue::undefined(), &[js_val], &mut context)
        .unwrap();
    assert_eq!(result, JsValue::from(true));
}
