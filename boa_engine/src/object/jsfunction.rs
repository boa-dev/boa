use crate::{
    object::{JsObject, JsObjectType},
    JsValue,
};
use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// JavaScript `Function` rust object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsFunction {
    inner: JsObject,
}

impl JsFunction {
    #[inline]
    pub(crate) fn from_object_unchecked(object: JsObject) -> Self {
        Self { inner: object }
    }

    /// Create a [`JsFunction`] from a [`JsObject`], or return `None` if the object is not a function.
    ///
    /// This does not clone the fields of the function, it only does a shallow clone of the object.
    #[inline]
    pub fn from_object(object: JsObject) -> Option<Self> {
        object
            .is_callable()
            .then(|| Self::from_object_unchecked(object))
    }
}

impl From<JsFunction> for JsObject {
    #[inline]
    fn from(o: JsFunction) -> Self {
        o.inner.clone()
    }
}

impl From<JsFunction> for JsValue {
    #[inline]
    fn from(o: JsFunction) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsFunction {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsFunction {}
