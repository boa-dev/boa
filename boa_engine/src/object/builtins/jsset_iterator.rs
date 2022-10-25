//! This module implements a wrapper for the `SetIterator` Builtin JavaScript Object
use std::ops::Deref;

use boa_gc::{Finalize, Trace};

use crate::{
    builtins::SetIterator,
    error::JsNativeError,
    object::{JsObject, JsObjectType},
    Context, JsResult, JsValue,
};

/// `JsSetIterator` provides a wrapper for Boa's implementation of the JavaScript `SetIterator` object
#[derive(Debug, Clone, Finalize, Trace)]
pub struct JsSetIterator {
    inner: JsObject,
}

impl JsSetIterator {
    /// Create a `JsSetIterator` from a `JsObject`.
    /// If object is not a `SetIterator`, throw `TypeError`.
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.borrow().is_set_iterator() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a SetIterator")
                .into())
        }
    }
    /// Advances the `JsSetIterator` and gets the next result in the `JsSet`.
    pub fn next(&self, context: &mut Context) -> JsResult<JsValue> {
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
