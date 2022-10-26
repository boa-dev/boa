//! This module implements a wrapper for the `DataView` Builtin JavaScript Object
use crate::{
    builtins::DataView,
    context::intrinsics::StandardConstructors,
    object::{
        internal_methods::get_prototype_from_constructor, JsArrayBuffer, JsObject, JsObjectType,
        ObjectData,
    },
    Context, JsNativeError, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsDataView` Provides a wrapper for Boa's implementation of the JavaScript `DataView` object
///
/// # Examples
/// ```
/// # use boa_engine::{
/// #     object::builtins::{JsArrayBuffer, JsDataView},
/// #     Context, JsValue
/// # };
///
/// // Create a new context and ArrayBuffer
/// let context = &mut Context::default();
/// let array_buffer = JsArrayBuffer::new(4, context).unwrap();
///
/// // Create a new Dataview from pre-existing ArrayBuffer
/// let data_view = JsDataView::from_js_array_buffer(&array_buffer, None, None, context).unwrap();
/// ```
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsDataView {
    inner: JsObject,
}

impl JsDataView {
    #[inline]
    pub fn from_js_array_buffer(
        array_buffer: &JsArrayBuffer,
        offset: Option<u64>,
        byte_length: Option<u64>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let (byte_offset, byte_length) = {
            let borrowed_buffer = array_buffer.borrow();
            let buffer = borrowed_buffer.as_array_buffer().ok_or_else(|| {
                JsNativeError::typ().with_message("buffer must be an ArrayBuffer")
            })?;

            let provided_offset = offset.unwrap_or(0_u64);

            // Check if buffer is detached.
            if buffer.is_detached_buffer() {
                return Err(JsNativeError::typ()
                    .with_message("ArrayBuffer is detached")
                    .into());
            };

            let array_buffer_length = buffer.array_buffer_byte_length();

            if provided_offset > array_buffer_length {
                return Err(JsNativeError::range()
                    .with_message("Provided offset is outside the bounds of the buffer")
                    .into());
            }

            let view_byte_length = if let Some(..) = byte_length {
                // Get the provided length
                let provided_length = byte_length.expect("byte_length must be a u64");

                // Check that the provided length and offset does not exceed the bounds of the ArrayBuffer
                if provided_offset + provided_length > array_buffer_length {
                    return Err(JsNativeError::range()
                        .with_message("Invalid data view length")
                        .into());
                }

                provided_length
            } else {
                array_buffer_length - provided_offset
            };

            (provided_offset, view_byte_length)
        };

        let constructor = context
            .intrinsics()
            .constructors()
            .data_view()
            .constructor()
            .into();

        let prototype =
            get_prototype_from_constructor(&constructor, StandardConstructors::data_view, context)?;

        let obj = JsObject::from_proto_and_data(
            prototype,
            ObjectData::data_view(DataView {
                viewed_array_buffer: (**array_buffer).clone(),
                byte_length,
                byte_offset,
            }),
        );

        Ok(Self { inner: obj })
    }

    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.borrow().is_data_view() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a DataView")
                .into())
        }
    }

    /// Returns the `viewed_array_buffer` field for [`JsDataView`]
    #[inline]
    pub fn buffer(&self, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_buffer(&self.inner.clone().into(), &[], context)
    }

    /// Returns the `byte_length` property of [`JsDataView`] as a u64 integer
    #[inline]
    pub fn byte_length(&self, context: &mut Context) -> JsResult<u64> {
        DataView::get_byte_length(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_number().expect("value should be a number") as u64)
    }

    /// Returns the `byte_offset` field property of [`JsDataView`] as a u64 integer
    #[inline]
    pub fn byte_offset(&self, context: &mut Context) -> JsResult<u64> {
        DataView::get_byte_offset(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_number().expect("byte_offset value must be a number") as u64)
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
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
            &self.inner.clone().into(),
            &[byte_offset.into(), value.into(), is_little_endian.into()],
            context,
        )
    }
}

impl From<JsDataView> for JsObject {
    #[inline]
    fn from(o: JsDataView) -> Self {
        o.inner.clone()
    }
}

impl From<JsDataView> for JsValue {
    #[inline]
    fn from(o: JsDataView) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsDataView {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsDataView {}
