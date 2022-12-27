//! A Rust API wrapper for Boa's `ArrayBuffer` Builtin ECMAScript Object
use crate::{
    builtins::array_buffer::ArrayBuffer,
    context::intrinsics::StandardConstructors,
    error::JsNativeError,
    object::{
        internal_methods::get_prototype_from_constructor, JsObject, JsObjectType, ObjectData,
    },
    Context, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsArrayBuffer` provides a wrapper for Boa's implementation of the ECMAScript `ArrayBuffer` object
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsArrayBuffer {
    inner: JsObject,
}

impl JsArrayBuffer {
    /// Create a new array buffer with byte length.
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::default();
    /// // Creates a blank array buffer of n bytes
    /// let array_buffer = JsArrayBuffer::new(4, context)?;
    ///
    /// assert_eq!(array_buffer.take()?, vec![0_u8; 4]);
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new(byte_length: usize, context: &mut Context) -> JsResult<Self> {
        let inner = ArrayBuffer::allocate(
            &context
                .intrinsics()
                .constructors()
                .array_buffer()
                .constructor()
                .into(),
            byte_length as u64,
            context,
        )?;

        Ok(Self { inner })
    }

    /// Create a new array buffer from byte block.
    ///
    /// This uses the passed byte block as the internal storage, it does not clone it!
    ///
    /// The `byte_length` will be set to `byte_block.len()`.
    ///  
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::default();
    ///
    /// // Create a buffer from a chunk of data
    /// let data_block: Vec<u8> = (0..5).collect();
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context)?;
    ///
    /// assert_eq!(array_buffer.take()?, (0..5).collect::<Vec<u8>>());
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_byte_block(byte_block: Vec<u8>, context: &mut Context) -> JsResult<Self> {
        let byte_length = byte_block.len();

        let constructor = context
            .intrinsics()
            .constructors()
            .array_buffer()
            .constructor()
            .into();

        // 1. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%ArrayBuffer.prototype%", « [[ArrayBufferData]], [[ArrayBufferByteLength]], [[ArrayBufferDetachKey]] »).
        let prototype = get_prototype_from_constructor(
            &constructor,
            StandardConstructors::array_buffer,
            context,
        )?;

        // 2. Let block be ? CreateByteDataBlock(byteLength).
        //
        // NOTE: We skip step 2. because we already have the block
        // that is passed to us as a function argument.
        let block = byte_block;

        // 3. Set obj.[[ArrayBufferData]] to block.
        // 4. Set obj.[[ArrayBufferByteLength]] to byteLength.
        let obj = JsObject::with_proto_and_data(
            prototype,
            ObjectData::array_buffer(ArrayBuffer {
                array_buffer_data: Some(block),
                array_buffer_byte_length: byte_length as u64,
                array_buffer_detach_key: JsValue::Undefined,
            }),
        );

        Ok(Self { inner: obj })
    }

    /// Create a [`JsArrayBuffer`] from a [`JsObject`], if the object is not an array buffer throw a `TypeError`.
    ///
    /// This does not clone the fields of the array buffer, it only does a shallow clone of the object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.borrow().is_array_buffer() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not an ArrayBuffer")
                .into())
        }
    }

    /// Returns the byte length of the array buffer.
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::default();
    /// // Create a buffer from a chunk of data
    /// let data_block: Vec<u8> = (0..5).collect();
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context)?;
    ///
    /// // Take the inner buffer
    /// let buffer_length = array_buffer.byte_length(context);
    ///
    /// assert_eq!(buffer_length, 5);
    /// # Ok(())
    /// # }
    ///  ```
    #[inline]
    pub fn byte_length(&self, context: &mut Context) -> usize {
        ArrayBuffer::get_byte_length(&self.inner.clone().into(), &[], context)
            .expect("it should not throw")
            .as_number()
            .expect("expected a number") as usize
    }

    /// Take the inner `ArrayBuffer`'s `array_buffer_data` field and replace it with `None`
    ///
    /// Note: This causes the pre-existing `JsArrayBuffer` to become detached.
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::default();
    /// // Create a buffer from a chunk of data
    /// let data_block: Vec<u8> = (0..5).collect();
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context)?;
    ///
    /// // Take the inner buffer
    /// let internal_buffer = array_buffer.take()?;
    ///
    /// assert_eq!(internal_buffer, (0..5).collect::<Vec<u8>>());
    ///
    /// // Anymore interaction with the buffer will return an error
    /// let detached_err = array_buffer.take();
    /// assert!(detached_err.is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn take(&self) -> JsResult<Vec<u8>> {
        self.inner
            .borrow_mut()
            .as_array_buffer_mut()
            .expect("inner must be an ArrayBuffer")
            .array_buffer_data
            .take()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("ArrayBuffer is detached")
                    .into()
            })
    }
}

impl From<JsArrayBuffer> for JsObject {
    #[inline]
    fn from(o: JsArrayBuffer) -> Self {
        o.inner.clone()
    }
}

impl From<JsArrayBuffer> for JsValue {
    #[inline]
    fn from(o: JsArrayBuffer) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsArrayBuffer {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsArrayBuffer {}
