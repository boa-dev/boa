use boa_engine::object::builtins::{JsPromise, JsUint8Array};
use boa_engine::{Context, JsNativeError, JsString, JsValue};
use std::rc::Rc;

pub(super) fn bytes(body: Rc<Vec<u8>>, context: &mut Context) -> JsPromise {
    JsPromise::from_async_fn(
        async move |context| {
            JsUint8Array::from_iter(body.iter().copied(), &mut context.borrow_mut()).map(Into::into)
        },
        context,
    )
}

pub(super) fn text(body: Rc<Vec<u8>>, context: &mut Context) -> JsPromise {
    JsPromise::from_async_fn(
        async move |_| {
            let body = String::from_utf8_lossy(body.as_ref());
            Ok(JsString::from(body).into())
        },
        context,
    )
}

pub(super) fn json(body: Rc<Vec<u8>>, context: &mut Context) -> JsPromise {
    JsPromise::from_async_fn(
        async move |context| {
            let json_string = String::from_utf8_lossy(body.as_ref());
            let json = serde_json::from_str::<serde_json::Value>(&json_string)
                .map_err(|e| JsNativeError::syntax().with_message(e.to_string()))?;

            JsValue::from_json(&json, &mut context.borrow_mut())
        },
        context,
    )
}
