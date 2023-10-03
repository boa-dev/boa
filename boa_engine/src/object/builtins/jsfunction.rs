//! A Rust API wrapper for Boa's `Function` Builtin ECMAScript Object
use crate::{
    object::{
        internal_methods::function::{
            NATIVE_CONSTRUCTOR_INTERNAL_METHODS, NATIVE_FUNCTION_INTERNAL_METHODS,
        },
        JsObject, JsObjectType, Object,
    },
    value::TryFromJs,
    Context, JsNativeError, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// JavaScript `Function` rust object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsFunction {
    inner: JsObject,
}

impl JsFunction {
    /// Creates a new `JsFunction` from an object, without checking if the object is callable.
    pub(crate) fn from_object_unchecked(object: JsObject) -> Self {
        Self { inner: object }
    }

    /// Creates a new, empty intrinsic function object with only its function internal methods set.
    ///
    /// Mainly used to initialize objects before a [`Context`] is available to do so.
    ///
    /// [`Context`]: crate::Context
    pub(crate) fn empty_intrinsic_function(constructor: bool) -> Self {
        Self {
            inner: JsObject::from_object_and_vtable(
                Object::default(),
                if constructor {
                    &NATIVE_CONSTRUCTOR_INTERNAL_METHODS
                } else {
                    &NATIVE_FUNCTION_INTERNAL_METHODS
                },
            ),
        }
    }

    /// Creates a [`JsFunction`] from a [`JsObject`], or returns `None` if the object is not a function.
    ///
    /// This does not clone the fields of the function, it only does a shallow clone of the object.
    #[inline]
    #[must_use]
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

impl TryFromJs for JsFunction {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Self::from_object(o.clone()).ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("object is not a function")
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("value is not a Function object")
                .into()),
        }
    }
}
