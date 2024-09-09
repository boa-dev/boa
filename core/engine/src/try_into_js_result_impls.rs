//! Declare implementations of [`TryIntoJsResult`] trait for various types.

use crate::object::JsArray;
use crate::{Context, JsResult, JsValue, TryIntoJsResult};

impl<T> TryIntoJsResult for T
where
    T: Into<JsValue>,
{
    fn try_into_js_result(self, _cx: &mut Context) -> JsResult<JsValue> {
        Ok(self.into())
    }
}

impl<T> TryIntoJsResult for Vec<T>
where
    T: TryIntoJsResult,
{
    fn try_into_js_result(self, cx: &mut Context) -> JsResult<JsValue> {
        let array = JsArray::new(cx);
        // We have to manually enumerate because we cannot return a Result from a map monad.
        for value in self {
            array.push(value.try_into_js_result(cx)?, cx)?;
        }
        Ok(array.into())
    }
}

impl<T> TryIntoJsResult for Option<T>
where
    T: TryIntoJsResult,
{
    fn try_into_js_result(self, cx: &mut Context) -> JsResult<JsValue> {
        match self {
            Some(value) => value.try_into_js_result(cx),
            None => Ok(JsValue::undefined()),
        }
    }
}

impl<T> TryIntoJsResult for JsResult<T>
where
    T: TryIntoJsResult,
{
    fn try_into_js_result(self, cx: &mut Context) -> JsResult<JsValue> {
        self.and_then(|value| value.try_into_js_result(cx))
    }
}
