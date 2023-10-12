#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::cast_ptr_alignment)] // Invariants are checked by the caller.
#![allow(clippy::undocumented_unsafe_blocks)] // Invariants are checked by the caller.

use std::sync::atomic;

use bytemuck::{AnyBitPattern, NoUninit};
use num_traits::ToPrimitive;
use portable_atomic::{AtomicU16, AtomicU32, AtomicU64};

use crate::{
    builtins::{
        array_buffer::utils::{SliceRef, SliceRefMut},
        typed_array::TypedArrayElement,
    },
    value::Numeric,
    Context, JsResult, JsValue,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, AnyBitPattern, NoUninit)]
#[repr(transparent)]
pub(crate) struct ClampedU8(pub(crate) u8);

impl ClampedU8 {
    pub(crate) fn to_be(self) -> Self {
        Self(self.0.to_be())
    }

    pub(crate) fn to_le(self) -> Self {
        Self(self.0.to_le())
    }
}

impl From<ClampedU8> for Numeric {
    fn from(value: ClampedU8) -> Self {
        Numeric::Number(value.0.into())
    }
}

pub(crate) trait Element:
    Sized + Into<TypedArrayElement> + NoUninit + AnyBitPattern
{
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self>;

    /// Gets the little endian representation of `Self`.
    fn to_little_endian(self) -> Self;

    /// Gets the big endian representation of `Self`.
    fn to_big_endian(self) -> Self;

    /// Reads `Self` from the `buffer`.
    ///
    /// This will always read values in the native endianness of the target architecture.
    ///
    /// # Safety
    ///
    /// - `buffer` must be aligned to the native alignment of `Self`.
    /// - `buffer` must contain enough bytes to read `std::sizeof::<Self>` bytes.
    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self;

    /// Writes the bytes of this element into `buffer`.
    ///
    /// This will always write values in the native endianness of the target architecture.
    ///
    /// # Safety
    ///
    /// - `buffer` must be aligned to the native alignment of `Self`.
    /// - `buffer` must contain enough bytes to store `std::sizeof::<Self>` bytes.
    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering);
}

impl Element for u8 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        value.to_uint8(context)
    }

    fn to_big_endian(self) -> Self {
        self.to_be()
    }

    fn to_little_endian(self) -> Self {
        self.to_le()
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        debug_assert!(buffer.len() >= 1);

        match buffer {
            SliceRef::Common(buffer) => unsafe { *buffer.get_unchecked(0) },
            SliceRef::Atomic(buffer) => unsafe { buffer.get_unchecked(0).load(order) },
        }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        debug_assert!(buffer.len() >= 1);

        match buffer {
            SliceRefMut::Common(buffer) => unsafe {
                *buffer.get_unchecked_mut(0) = value;
            },
            SliceRefMut::Atomic(buffer) => unsafe { buffer.get_unchecked(0).store(value, order) },
        }
    }
}

impl Element for u16 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        value.to_uint16(context)
    }

    fn to_big_endian(self) -> Self {
        self.to_be()
    }

    fn to_little_endian(self) -> Self {
        self.to_le()
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        if cfg!(debug_assertions) {
            assert!(buffer.len() >= std::mem::size_of::<u16>());
            assert!(buffer.addr() % std::mem::align_of::<u16>() == 0);
        }

        match buffer {
            SliceRef::Common(buffer) => unsafe { *buffer.as_ptr().cast() },
            SliceRef::Atomic(buffer) => unsafe {
                (*buffer.as_ptr().cast::<AtomicU16>()).load(order)
            },
        }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        if cfg!(debug_assertions) {
            assert!(buffer.len() >= std::mem::size_of::<u16>());
            assert!(buffer.addr() % std::mem::align_of::<u16>() == 0);
        }

        match buffer {
            SliceRefMut::Common(buffer) => unsafe {
                *buffer.as_mut_ptr().cast() = value;
            },
            SliceRefMut::Atomic(buffer) => unsafe {
                (*buffer.as_ptr().cast::<AtomicU16>()).store(value, order);
            },
        }
    }
}

impl Element for u32 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        value.to_u32(context)
    }

    fn to_big_endian(self) -> Self {
        self.to_be()
    }

    fn to_little_endian(self) -> Self {
        self.to_le()
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        if cfg!(debug_assertions) {
            assert!(buffer.len() >= std::mem::size_of::<u32>());
            assert!(buffer.addr() % std::mem::align_of::<u32>() == 0);
        }

        match buffer {
            SliceRef::Common(buffer) => unsafe { *buffer.as_ptr().cast() },
            SliceRef::Atomic(buffer) => unsafe {
                (*buffer.as_ptr().cast::<AtomicU32>()).load(order)
            },
        }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        if cfg!(debug_assertions) {
            assert!(buffer.len() >= std::mem::size_of::<u32>());
            assert!(buffer.addr() % std::mem::align_of::<u32>() == 0);
        }

        match buffer {
            SliceRefMut::Common(buffer) => unsafe {
                *buffer.as_mut_ptr().cast() = value;
            },
            SliceRefMut::Atomic(buffer) => unsafe {
                (*buffer.as_ptr().cast::<AtomicU32>()).store(value, order);
            },
        }
    }
}

impl Element for u64 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        Ok(value.to_big_uint64(context)?.to_u64().unwrap_or(u64::MAX))
    }

    fn to_big_endian(self) -> Self {
        self.to_be()
    }

    fn to_little_endian(self) -> Self {
        self.to_le()
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        if cfg!(debug_assertions) {
            assert!(buffer.len() >= std::mem::size_of::<u64>());
            assert!(buffer.addr() % std::mem::align_of::<u64>() == 0);
        }

        match buffer {
            SliceRef::Common(buffer) => unsafe { *buffer.as_ptr().cast() },
            SliceRef::Atomic(buffer) => unsafe {
                (*buffer.as_ptr().cast::<AtomicU64>()).load(order)
            },
        }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        if cfg!(debug_assertions) {
            assert!(buffer.len() >= std::mem::size_of::<u64>());
            assert!(buffer.addr() % std::mem::align_of::<u64>() == 0);
        }

        match buffer {
            SliceRefMut::Common(buffer) => unsafe {
                *buffer.as_mut_ptr().cast() = value;
            },
            SliceRefMut::Atomic(buffer) => unsafe {
                (*buffer.as_ptr().cast::<AtomicU64>()).store(value, order);
            },
        }
    }
}

impl Element for i8 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        value.to_int8(context)
    }

    fn to_big_endian(self) -> Self {
        self.to_be()
    }

    fn to_little_endian(self) -> Self {
        self.to_le()
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        unsafe { u8::read_from_buffer(buffer, order) as i8 }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        unsafe { u8::write_to_buffer(buffer, value as u8, order) }
    }
}

impl Element for ClampedU8 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        value.to_uint8_clamp(context).map(ClampedU8)
    }

    fn to_big_endian(self) -> Self {
        self.to_be()
    }

    fn to_little_endian(self) -> Self {
        self.to_le()
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        unsafe { ClampedU8(u8::read_from_buffer(buffer, order)) }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        unsafe { u8::write_to_buffer(buffer, value.0, order) }
    }
}

impl Element for i16 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        value.to_int16(context)
    }

    fn to_big_endian(self) -> Self {
        self.to_be()
    }

    fn to_little_endian(self) -> Self {
        self.to_le()
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        unsafe { u16::read_from_buffer(buffer, order) as i16 }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        unsafe { u16::write_to_buffer(buffer, value as u16, order) }
    }
}

impl Element for i32 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        value.to_i32(context)
    }

    fn to_big_endian(self) -> Self {
        self.to_be()
    }

    fn to_little_endian(self) -> Self {
        self.to_le()
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        unsafe { u32::read_from_buffer(buffer, order) as i32 }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        unsafe { u32::write_to_buffer(buffer, value as u32, order) }
    }
}

impl Element for i64 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        let big_int = value.to_big_int64(context)?;

        Ok(big_int.to_i64().unwrap_or_else(|| {
            if big_int.is_positive() {
                i64::MAX
            } else {
                i64::MIN
            }
        }))
    }

    fn to_big_endian(self) -> Self {
        self.to_be()
    }

    fn to_little_endian(self) -> Self {
        self.to_le()
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        unsafe { u64::read_from_buffer(buffer, order) as i64 }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        unsafe { u64::write_to_buffer(buffer, value as u64, order) }
    }
}

impl Element for f32 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        value.to_number(context).map(|f| f as f32)
    }

    fn to_big_endian(self) -> Self {
        f32::from_bits(self.to_bits().to_be())
    }

    fn to_little_endian(self) -> Self {
        f32::from_bits(self.to_bits().to_le())
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        unsafe { f32::from_bits(u32::read_from_buffer(buffer, order)) }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        unsafe { u32::write_to_buffer(buffer, value.to_bits(), order) }
    }
}

impl Element for f64 {
    fn from_js_value(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        value.to_number(context)
    }

    fn to_big_endian(self) -> Self {
        f64::from_bits(self.to_bits().to_be())
    }

    fn to_little_endian(self) -> Self {
        f64::from_bits(self.to_bits().to_le())
    }

    unsafe fn read_from_buffer(buffer: SliceRef<'_>, order: atomic::Ordering) -> Self {
        unsafe { f64::from_bits(u64::read_from_buffer(buffer, order)) }
    }

    unsafe fn write_to_buffer(buffer: SliceRefMut<'_>, value: Self, order: atomic::Ordering) {
        unsafe { u64::write_to_buffer(buffer, value.to_bits(), order) }
    }
}
