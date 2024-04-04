//! Declare implementations of [`TryIntoJsResult`] trait for various types.
//! We cannot rely on a generic implementation based on `TryIntoJs` due
//! to a limitation of the Rust type system which prevents generalization
//! of traits based on an upstream crate.

use crate::TryIntoJsResult;
use boa_engine::{Context, JsBigInt, JsResult, JsString, JsSymbol, JsValue};

impl<T: TryIntoJsResult> TryIntoJsResult for Option<T> {
    fn try_into_js_result(self, context: &mut Context) -> JsResult<JsValue> {
        match self {
            Some(value) => value.try_into_js_result(context),
            None => Ok(JsValue::undefined()),
        }
    }
}

impl<T: TryIntoJsResult> TryIntoJsResult for JsResult<T> {
    fn try_into_js_result(self, context: &mut Context) -> JsResult<JsValue> {
        self.and_then(|value| value.try_into_js_result(context))
    }
}

macro_rules! impl_try_into_js_result {
    ($($t: ty),*) => {
        $(
            impl TryIntoJsResult for $t {
                fn try_into_js_result(self, _context: &mut Context) -> JsResult<JsValue> {
                    Ok(JsValue::from(self))
                }
            }
        )*
    };
}

impl_try_into_js_result!(
    bool,
    char,
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    usize,
    f32,
    f64,
    JsBigInt,
    JsString,
    JsSymbol,
    JsValue,
    ()
);
