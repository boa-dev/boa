//! A Rust API wrapper for Boa's `SetIterator` Builtin ECMAScript Object
use std::ops::Deref;

use boa_gc::{Finalize, Trace};

use crate::{
    builtins::set::SetIterator,
    error::JsNativeError,
    object::{JsObject, JsObjectType},
    value::TryFromJs,
    Context, JsResult, JsValue,
};

/// `JsSetIterator` provides a wrapper for Boa's implementation of the ECMAScript `SetIterator` object
#[derive(Debug, Clone, Finalize, Trace)]
pub struct JsSetIterator {
    inner: JsObject,
}

impl JsSetIterator {
    /// Create a `JsSetIterator` from a `JsObject`.
    /// If object is not a `SetIterator`, throw `TypeError`.
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is_set_iterator() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a SetIterator")
                .into())
        }
    }
    /// Advances the `JsSetIterator` and gets the next result in the `JsSet`.
    pub fn next(&self, context: &mut dyn Context<'_>) -> JsResult<JsValue> {
        SetIterator::next(&self.inner.clone().into(), &[JsValue::Null], context)
    }
}

impl From<JsSetIterator> for JsObject {
    #[inline]
    fn from(o: JsSetIterator) -> Self {
        o.inner.clone()
    }
}

impl From<JsSetIterator> for JsValue {
    #[inline]
    fn from(o: JsSetIterator) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsSetIterator {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsSetIterator {}

impl TryFromJs for JsSetIterator {
    fn try_from_js(value: &JsValue, _context: &mut dyn Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Self::from_object(o.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("value is not a SetIterator object")
                .into()),
        }
    }
}
