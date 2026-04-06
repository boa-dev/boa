//! This module implements the `HeadersIterator` object.
//!
//! More information:
//!  - [Fetch specification][spec]
//!
//! [spec]: https://fetch.spec.whatwg.org/#headers-class

use boa_engine::{
    Context, JsData, JsResult, JsString, JsValue, boa_class,
    builtins::iterable::create_iter_result_object, error::JsNativeError, interop::JsClass,
    object::JsObject, object::builtins::JsArray,
};
use boa_gc::{Finalize, Trace};

use super::headers::JsHeaders;

/// The `IterationKind` enum represents the different kinds of iteration that can be performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IterationKind {
    /// Iterates over the keys.
    Key,
    /// Iterates over the values.
    Value,
    /// Iterates over the keys and values.
    KeyAndValue,
}

/// The Headers Iterator object represents an iteration over a Headers object.
/// It implements the iterator protocol.
///
/// More information:
///  - [Fetch specification][spec]
///
/// [spec]: https://fetch.spec.whatwg.org/#headers-class
#[derive(Debug, Finalize, Trace, JsData)]
pub(crate) struct HeadersIterator {
    iterated_headers: JsObject<JsHeaders>,
    next_index: usize,
    #[unsafe_ignore_trace]
    iteration_kind: IterationKind,
}

#[boa_class(rename = "Headers Iterator")]
impl HeadersIterator {
    /// Prevent direct construction — `HeadersIterator` instances are only
    /// created internally via [`HeadersIterator::create_headers_iterator`].
    #[boa(constructor)]
    fn constructor() -> JsResult<Self> {
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }

    /// `%HeadersIteratorPrototype%.next()`
    ///
    /// Advances the iterator and returns the next `{ value, done }` result.
    #[boa(method)]
    fn next(&mut self, context: &mut Context) -> JsValue {
        let item_kind = self.iteration_kind;

        let element = self
            .iterated_headers
            .borrow()
            .data()
            .headers
            .borrow()
            .iter()
            .nth(self.next_index)
            .map(|(k, v)| {
                (
                    JsValue::from(JsString::from(k.as_str())),
                    JsValue::from(JsString::from(v.to_str().unwrap_or(""))),
                )
            });

        if let Some((key, value)) = element {
            self.next_index += 1;

            return match item_kind {
                IterationKind::Key => create_iter_result_object(key, false, context),
                IterationKind::Value => create_iter_result_object(value, false, context),
                IterationKind::KeyAndValue => {
                    let result = JsArray::from_iter([key, value], context);
                    create_iter_result_object(result.into(), false, context)
                }
            };
        }

        create_iter_result_object(JsValue::undefined(), true, context)
    }

    /// Returns `this`, making the iterator itself iterable (`for...of` support).
    #[boa(method)]
    #[boa(symbol = "iterator")]
    fn symbol_iterator(this: JsClass<Self>) -> JsValue {
        this.inner().into()
    }
}

impl HeadersIterator {
    /// Creates a new iterator over the given `Headers` object.
    pub(crate) fn create_headers_iterator(
        headers: JsObject<JsHeaders>,
        kind: IterationKind,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let iter = Self {
            iterated_headers: headers,
            next_index: 0,
            iteration_kind: kind,
        };

        let proto = context
            .realm()
            .get_class::<Self>()
            .ok_or_else(|| boa_engine::js_error!(Error: "Headers Iterator not registered"))?
            .prototype();

        let headers_iterator = JsObject::from_proto_and_data(proto, iter);
        Ok(headers_iterator.into())
    }
}
