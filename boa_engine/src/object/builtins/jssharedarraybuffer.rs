//! A Rust API wrapper for Boa's `SharedArrayBuffer` Builtin ECMAScript Object
use crate::{
    builtins::array_buffer::SharedArrayBuffer,
    error::JsNativeError,
    object::{JsObject, JsObjectType, ObjectData},
    value::TryFromJs,
    Context, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsSharedArrayBuffer` provides a wrapper for Boa's implementation of the ECMAScript `ArrayBuffer` object
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsSharedArrayBuffer {
    inner: JsObject,
}

impl JsSharedArrayBuffer {
    /// Creates a new [`JsSharedArrayBuffer`] with `byte_length` bytes of allocated space.
    #[inline]
    pub fn new(byte_length: usize, context: &mut Context<'_>) -> JsResult<Self> {
        let inner = SharedArrayBuffer::allocate(
            &context
                .intrinsics()
                .constructors()
                .shared_array_buffer()
                .constructor()
                .into(),
            byte_length as u64,
            context,
        )?;

        Ok(Self { inner })
    }

    /// Creates a [`JsSharedArrayBuffer`] from a shared raw buffer.
    #[inline]
    pub fn from_buffer(buffer: SharedArrayBuffer, context: &mut Context<'_>) -> Self {
        let proto = context
            .intrinsics()
            .constructors()
            .shared_array_buffer()
            .prototype();

        let inner = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ObjectData::shared_array_buffer(buffer),
        );

        Self { inner }
    }

    /// Creates a [`JsSharedArrayBuffer`] from a [`JsObject`], throwing a `TypeError` if the object
    /// is not a shared array buffer.
    ///
    /// This does not clone the fields of the shared array buffer, it only does a shallow clone of
    /// the object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is_shared_array_buffer() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not an ArrayBuffer")
                .into())
        }
    }

    /// Returns the byte length of the array buffer.
    #[inline]
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.borrow()
            .as_shared_array_buffer()
            .expect("should be an array buffer")
            .len()
    }

    /// Gets the raw buffer of this `JsSharedArrayBuffer`.
    #[inline]
    #[must_use]
    pub fn inner(&self) -> SharedArrayBuffer {
        self.borrow()
            .as_shared_array_buffer()
            .expect("should be an array buffer")
            .clone()
    }
}

impl From<JsSharedArrayBuffer> for JsObject {
    #[inline]
    fn from(o: JsSharedArrayBuffer) -> Self {
        o.inner.clone()
    }
}

impl From<JsSharedArrayBuffer> for JsValue {
    #[inline]
    fn from(o: JsSharedArrayBuffer) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsSharedArrayBuffer {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsSharedArrayBuffer {}

impl TryFromJs for JsSharedArrayBuffer {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Self::from_object(o.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("value is not a SharedArrayBuffer object")
                .into()),
        }
    }
}
