//! This module implements the `HeadersIterator` object.
//!
//! More information:
//!  - [Fetch specification][spec]
//!
//! [spec]: https://fetch.spec.whatwg.org/#headers-class

use boa_engine::{
    Context, JsData, JsResult, JsString, JsValue,
    builtins::iterable::create_iter_result_object,
    class::{Class, ClassBuilder},
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    object::JsObject,
    object::builtins::JsArray,
    property::{Attribute, PropertyNameKind},
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace};

use super::headers::JsHeaders;

/// The Headers Iterator object represents an iteration over a Headers object.
/// It implements the iterator protocol.
///
/// More information:
///  - [Fetch specification][spec]
///
/// [spec]: https://fetch.spec.whatwg.org/#headers-class
#[derive(Debug, Finalize, Trace, JsData)]
pub(crate) struct HeadersIterator {
    iterated_headers: JsObject,
    next_index: usize,
    #[unsafe_ignore_trace]
    iteration_kind: PropertyNameKind,
}

impl Class for HeadersIterator {
    const NAME: &'static str = "Headers Iterator";
    const LENGTH: usize = 0;

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class.method(
            js_string!("next"),
            0,
            NativeFunction::from_fn_ptr(Self::next),
        );
        class.static_property(
            JsSymbol::to_string_tag(),
            JsString::from("Headers Iterator"),
            Attribute::CONFIGURABLE,
        );
        class.method(
            JsSymbol::iterator(),
            0,
            NativeFunction::from_fn_ptr(|this, _args, _context| Ok(this.clone())),
        );
        Ok(())
    }

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }
}

impl HeadersIterator {
    /// Creates a new iterator over the given Headers.
    pub(crate) fn create_headers_iterator(
        headers: JsObject,
        kind: PropertyNameKind,
        context: &mut Context,
    ) -> JsValue {
        let iter = Self {
            iterated_headers: headers,
            next_index: 0,
            iteration_kind: kind,
        };
        let headers_iterator = JsObject::from_proto_and_data(
            context
                .get_global_class::<Self>()
                .expect("Headers Iterator not registered")
                .prototype(),
            iter,
        );
        headers_iterator.into()
    }

    /// %HeadersIteratorPrototype%.next( )
    ///
    /// Advances the iterator and gets the next result in the headers.
    pub(crate) fn next(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object();
        let mut headers_iterator = object
            .as_ref()
            .and_then(JsObject::downcast_mut::<Self>)
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a Headers Iterator"))?;

        let item_kind = headers_iterator.iteration_kind;
        let element = {
            let headers = headers_iterator
                .iterated_headers
                .downcast_ref::<JsHeaders>()
                .ok_or_else(|| {
                    JsNativeError::typ().with_message("Object is not a Headers object")
                })?;

            headers
                .headers_map()
                .iter()
                .nth(headers_iterator.next_index)
                .map(|(k, v)| {
                    (
                        JsValue::from(JsString::from(k.as_str())),
                        JsValue::from(JsString::from(v.to_str().unwrap_or(""))),
                    )
                })
        };

        if let Some((key, value)) = element {
            headers_iterator.next_index += 1;

            let item = match item_kind {
                PropertyNameKind::Key => Ok(create_iter_result_object(key, false, context)),
                PropertyNameKind::Value => Ok(create_iter_result_object(value, false, context)),
                PropertyNameKind::KeyAndValue => {
                    let result = JsArray::from_iter([key, value], context);
                    Ok(create_iter_result_object(result.into(), false, context))
                }
            };
            return item;
        }

        Ok(create_iter_result_object(
            JsValue::undefined(),
            true,
            context,
        ))
    }
}
