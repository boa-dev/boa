//! A Rust API wrapper for Boa's `DataView` Builtin ECMAScript Object
use crate::{
    builtins::{array_buffer::BufferObject, DataView},
    object::{JsArrayBuffer, JsObject},
    value::TryFromJs,
    Context, JsNativeError, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsDataView` provides a wrapper for Boa's implementation of the ECMAScript `DataView` object
///
/// # Examples
/// ```
/// # use boa_engine::{
/// #     object::builtins::{JsArrayBuffer, JsDataView},
/// #     Context, JsValue, JsResult,
/// # };
/// # fn main() -> JsResult<()> {
/// // Create a new context and ArrayBuffer
/// let context = &mut Context::default();
/// let array_buffer = JsArrayBuffer::new(4, context)?;
///
/// // Create a new Dataview from pre-existing ArrayBuffer
/// let data_view = JsDataView::from_js_array_buffer(array_buffer, None, None, context)?;
///
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Trace, Finalize)]
#[boa_gc(unsafe_no_drop)]
pub struct JsDataView {
    inner: JsObject<DataView>,
}

impl From<JsDataView> for JsObject<DataView> {
    #[inline]
    fn from(value: JsDataView) -> Self {
        value.inner
    }
}

impl From<JsObject<DataView>> for JsDataView {
    #[inline]
    fn from(value: JsObject<DataView>) -> Self {
        Self { inner: value }
    }
}

impl JsDataView {
    /// Create a new `JsDataView` object from an existing `JsArrayBuffer`.
    pub fn from_js_array_buffer(
        buffer: JsArrayBuffer,
        offset: Option<usize>,
        byte_len: Option<usize>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let offset = offset.unwrap_or_default();

        let (buf_byte_len, is_fixed_len) = {
            let buffer = buffer.borrow();
            let buffer = &buffer.data;

            // 4. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
            let Some(slice) = buffer.bytes() else {
                return Err(JsNativeError::typ()
                    .with_message("ArrayBuffer is detached")
                    .into());
            };

            // 5. Let bufferByteLength be ArrayBufferByteLength(buffer, seq-cst).
            let buf_len = slice.len();

            // 6. If offset > bufferByteLength, throw a RangeError exception.
            if offset > buf_len {
                return Err(JsNativeError::range()
                    .with_message("Start offset is outside the bounds of the buffer")
                    .into());
            }

            // 7. Let bufferIsFixedLength be IsFixedLengthArrayBuffer(buffer).
            (buf_len, buffer.is_fixed_len())
        };

        // 8. If byteLength is undefined, then
        let view_byte_len = if let Some(byte_len) = byte_len {
            // 9. Else,
            //     a. Let viewByteLength be ? ToIndex(byteLength).
            //     b. If offset + viewByteLength > bufferByteLength, throw a RangeError exception.
            if offset + byte_len > buf_byte_len {
                return Err(JsNativeError::range()
                    .with_message("Invalid data view length")
                    .into());
            }

            Some(byte_len)
        } else {
            // a. If bufferIsFixedLength is true, then
            //     i. Let viewByteLength be bufferByteLength - offset.
            // b. Else,
            //     i. Let viewByteLength be auto.
            is_fixed_len.then_some(buf_byte_len - offset)
        };

        // 10. Let O be ? OrdinaryCreateFromConstructor(NewTarget, "%DataView.prototype%",
        //     « [[DataView]], [[ViewedArrayBuffer]], [[ByteLength]], [[ByteOffset]] »).
        let prototype = context.intrinsics().constructors().data_view().prototype();

        // 11. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        // 12. Set bufferByteLength to ArrayBufferByteLength(buffer, seq-cst).
        let Some(buf_byte_len) = buffer.borrow().data.bytes().map(<[u8]>::len) else {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer is detached")
                .into());
        };

        // 13. If offset > bufferByteLength, throw a RangeError exception.
        if offset > buf_byte_len {
            return Err(JsNativeError::range()
                .with_message("DataView offset outside of buffer array bounds")
                .into());
        }

        // 14. If byteLength is not undefined, then
        if let Some(view_byte_len) = view_byte_len.filter(|_| byte_len.is_some()) {
            // a. If offset + viewByteLength > bufferByteLength, throw a RangeError exception.
            if offset + view_byte_len > buf_byte_len {
                return Err(JsNativeError::range()
                    .with_message("DataView offset outside of buffer array bounds")
                    .into());
            }
        }

        let obj = JsObject::new(
            context.root_shape(),
            prototype,
            DataView {
                // 15. Set O.[[ViewedArrayBuffer]] to buffer.
                viewed_array_buffer: BufferObject::Buffer(buffer.into()),
                // 16. Set O.[[ByteLength]] to viewByteLength.
                byte_length: view_byte_len,
                // 17. Set O.[[ByteOffset]] to offset.
                byte_offset: offset,
            },
        );

        // 18. Return O.
        Ok(Self { inner: obj })
    }

    /// Create a new `JsDataView` object from an existing object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        object
            .downcast::<DataView>()
            .map(|inner| Self { inner })
            .map_err(|_| {
                JsNativeError::typ()
                    .with_message("object is not a DataView")
                    .into()
            })
    }

    /// Returns the `viewed_array_buffer` field for [`JsDataView`]
    #[inline]
    pub fn buffer(&self, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_buffer(&self.inner.clone().upcast().into(), &[], context)
    }

    /// Returns the `byte_length` property of [`JsDataView`] as a usize integer
    #[inline]
    pub fn byte_length(&self, context: &mut Context) -> JsResult<usize> {
        DataView::get_byte_length(&self.inner.clone().upcast().into(), &[], context)
            .map(|v| v.as_number().expect("value should be a number") as usize)
    }

    /// Returns the `byte_offset` field property of [`JsDataView`] as a usize integer
    #[inline]
    pub fn byte_offset(&self, context: &mut Context) -> JsResult<usize> {
        DataView::get_byte_offset(&self.inner.clone().upcast().into(), &[], context)
            .map(|v| v.as_number().expect("byte_offset value must be a number") as usize)
    }

    /// Returns a signed 64-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn get_big_int64(
        &self,
        byte_offset: usize,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<i64> {
        DataView::get_big_int64(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), is_little_endian.into()],
            context,
        )
        .map(|v| v.as_number().expect("value must be a number") as i64)
    }

    /// Returns an unsigned 64-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn get_big_uint64(
        &self,
        byte_offset: usize,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<u64> {
        DataView::get_big_uint64(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), is_little_endian.into()],
            context,
        )
        .map(|v| v.as_number().expect("value must be a number") as u64)
    }

    /// Returns a signed 32-bit float integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn get_float32(
        &self,
        byte_offset: usize,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<f32> {
        DataView::get_float32(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), is_little_endian.into()],
            context,
        )
        .map(|v| v.as_number().expect("value must be a number") as f32)
    }

    /// Returns a signed 64-bit float integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn get_float64(
        &self,
        byte_offset: usize,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<f64> {
        DataView::get_float64(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), is_little_endian.into()],
            context,
        )
        .map(|v| v.as_number().expect("value must be a number"))
    }

    /// Returns a signed 8-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn get_int8(
        &self,
        byte_offset: usize,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<i8> {
        DataView::get_int8(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), is_little_endian.into()],
            context,
        )
        .map(|v| v.as_number().expect("value must be a number") as i8)
    }

    /// Returns a signed 16-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn get_int16(
        &self,
        byte_offset: usize,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<i16> {
        DataView::get_int16(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), is_little_endian.into()],
            context,
        )
        .map(|v| v.as_number().expect("value must be a number") as i16)
    }

    /// Returns a signed 32-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn get_int32(
        &self,
        byte_offset: usize,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<i32> {
        DataView::get_int32(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), is_little_endian.into()],
            context,
        )
        .map(|v| v.as_number().expect("value must be a number") as i32)
    }

    /// Returns an unsigned 8-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn get_uint8(
        &self,
        byte_offset: usize,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<u8> {
        DataView::get_uint8(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), is_little_endian.into()],
            context,
        )
        .map(|v| v.as_number().expect("value must be a number") as u8)
    }

    /// Returns an unsigned 16-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn get_unit16(
        &self,
        byte_offset: usize,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<u16> {
        DataView::get_uint16(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), is_little_endian.into()],
            context,
        )
        .map(|v| v.as_number().expect("value must be a number") as u16)
    }

    /// Returns an unsigned 32-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn get_uint32(
        &self,
        byte_offset: usize,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<u32> {
        DataView::get_uint32(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), is_little_endian.into()],
            context,
        )
        .map(|v| v.as_number().expect("value must be a number") as u32)
    }

    /// Sets a signed 64-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn set_big_int64(
        &self,
        byte_offset: usize,
        value: i64,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        DataView::set_big_int64(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }

    /// Sets an unsigned 64-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn set_big_uint64(
        &self,
        byte_offset: usize,
        value: u64,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        DataView::set_big_uint64(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }

    /// Sets a signed 32-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn set_float32(
        &self,
        byte_offset: usize,
        value: f32,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        DataView::set_float32(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }

    /// Sets a signed 64-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn set_float64(
        &self,
        byte_offset: usize,
        value: f64,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        DataView::set_float64(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }

    /// Sets a signed 8-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn set_int8(
        &self,
        byte_offset: usize,
        value: i8,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        DataView::set_int8(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }

    /// Sets a signed 16-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn set_int16(
        &self,
        byte_offset: usize,
        value: i16,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        DataView::set_int16(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }

    /// Sets a signed 32-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn set_int32(
        &self,
        byte_offset: usize,
        value: i32,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        DataView::set_int32(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }

    /// Sets an unsigned 8-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn set_uint8(
        &self,
        byte_offset: usize,
        value: u8,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        DataView::set_uint8(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }

    /// Sets an unsigned 16-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn set_unit16(
        &self,
        byte_offset: usize,
        value: u16,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        DataView::set_uint16(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }

    /// Sets an unsigned 32-bit integer at the specified offset from the start of the [`JsDataView`]
    #[inline]
    pub fn set_unit32(
        &self,
        byte_offset: usize,
        value: u32,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        DataView::set_uint32(
            &self.inner.clone().upcast().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }
}

impl From<JsDataView> for JsObject {
    #[inline]
    fn from(o: JsDataView) -> Self {
        o.inner.upcast()
    }
}

impl From<JsDataView> for JsValue {
    #[inline]
    fn from(o: JsDataView) -> Self {
        o.inner.upcast().into()
    }
}

impl Deref for JsDataView {
    type Target = JsObject<DataView>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsDataView {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Self::from_object(o.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("value is not an DataView object")
                .into()),
        }
    }
}
