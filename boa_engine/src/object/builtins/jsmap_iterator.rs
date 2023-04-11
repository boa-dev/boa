//! A Rust API wrapper for Boa's `MapIterator` Builtin ECMAScript Object
use crate::{
    builtins::map::MapIterator,
    error::JsNativeError,
    object::{JsObject, JsObjectType},
    value::TryFromJs,
    Context, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsMapIterator` provides a wrapper for Boa's implementation of the ECMAScript `MapIterator` object.
#[derive(Debug, Clone, Finalize, Trace)]
pub struct JsMapIterator {
    inner: JsObject,
}

impl JsMapIterator {
    /// Create a [`JsMapIterator`] from a [`JsObject`]. If object is not a `MapIterator`, throw `TypeError`
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.borrow().is_map_iterator() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a MapIterator")
                .into())
        }
    }

    /// Advances the `JsMapIterator` and gets the next result in the `JsMap`
    pub fn next(&self, context: &mut Context<'_>) -> JsResult<JsValue> {
        MapIterator::next(&self.inner.clone().into(), &[], context)
    }
}

impl From<JsMapIterator> for JsObject {
    #[inline]
    fn from(o: JsMapIterator) -> Self {
        o.inner.clone()
    }
}

impl From<JsMapIterator> for JsValue {
    #[inline]
    fn from(o: JsMapIterator) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsMapIterator {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsMapIterator {}

impl TryFromJs for JsMapIterator {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Self::from_object(o.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("value is not a MapIterator object")
                .into()),
        }
    }
}
