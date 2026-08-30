#![allow(unused)]

use boa_engine::{
    value::{TryFromJs, TryIntoJs},
    Context, JsResult, JsValue,
};

#[derive(TryFromJs, TryIntoJs)]
struct Blah {
    #[boa(into_js_with = "my_custom_converter_to_js")]
    #[boa(from_js_with = "my_custom_converter_from_js")]
    x: i32,
}

fn my_custom_converter_to_js(value: &i32, _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::new(*value))
}

fn my_custom_converter_from_js(value: &JsValue, _context: &mut Context) -> JsResult<i32> {
    value.try_js_into(_context)
}

fn main() {}
