//! Module implementing the operations for the inner value of a `[super::JsValue]`.
//! The `[InnerValue]` type is a NaN-boxed value, which is a 64-bits value that
//! can represent any JavaScript value. If the integer is a non-NaN value, it
//! will be stored as a 64-bits float. If it is a `f64::NAN` value, it will be
//! stored as a quiet `NaN` value. Subnormal numbers are regular float.
//!
//! For any other type of values, the value will be stored as a 51-bits non-zero
//! integer.
//!
//! In short, the memory layout of a NaN-boxed value is as follows:
//!
//! | Type of           | Bit Layout | Comment |
//! |-------------------|------------|---------|
//! | `+Infinity`       | `7FF0:0000:0000:0000`    | |
//! | `-Infinity`       | `FFF0:0000:0000:0000`    | |
//! | `NAN` (quiet)     | `7FF8:0000:0000:0000`    | |
//! | `NAN` (signaling) | `FFF8:0000:0000:0000`    | |
//! | `Undefined`       | `7FF4:0000:0000:0000`    | |
//! | `Null`            | `7FF5:0000:0000:0000`    | |
//! | `False`           | `7FF6:0000:0000:0000`    | |
//! | `True`            | `7FF6:0000:0000:0001`    | |
//! | `Integer32`       | `7FF7:0000:IIII:IIII`    | 32-bits integer. |
//! | `BigInt`          | `7FF[8-F]:PPPP:PPPP:PPPP \| 0` | 51-bits pointer. Assumes non-null pointer. |
//! | `Object`          | `7FF[8-F]:PPPP:PPPP:PPPP \| 1` | 51-bits pointer. |
//! | `Symbol`          | `7FF[8-F]:PPPP:PPPP:PPPP \| 2` | 51-bits pointer. |
//! | `String`          | `7FF[8-F]:PPPP:PPPP:PPPP \| 3` | 51-bits pointer. |
//! | `Float64`         | Any other values.        | |
//!
//! Pointers have the highest bit (in the `NaN` tag) set to 1, so they
//! can represent any value from `0x8000_0000_0000` to `0xFFFF_FFFF_FFFF`.
//! The last 2 bits of the pointer is used to store the type of the value.
//!
//! The pointers are assumed to never be NULL, and as such no clash
//! with regular NAN should happen.
//!
//! This only works on 4-bits aligned values, which is asserted when the
//! `InnerValue` is created.

use crate::{JsBigInt, JsObject, JsSymbol};
use boa_string::JsString;
use num_traits::ToBytes;
use static_assertions::const_assert;

// We cannot NaN-box pointers larger than 64 bits.
const_assert!(size_of::<usize>() <= size_of::<u64>());

// We cannot NaN-box pointers that are not 4-bytes aligned.
const_assert!(align_of::<*mut ()>() >= 4);

/// The bit tags and masks for NaN-boxed values. Masks are applied when creating
/// the value.
///
/// This is a utility type that allows to create NaN-boxed values, and to check
/// the type of a NaN-boxed value.
#[derive(Copy, Clone)]
#[repr(u64)]
enum NanBitTag {
    Undefined = 0x7FF4_0000_0000_0000,
    Null = 0x7FF5_0000_0000_0000,
    False = 0x7FF6_0000_0000_0000,
    True = 0x7FF6_0000_0000_0001,
    Integer32 = 0x7FF7_0000_0000_0000,
    BigInt = 0x7FF8_0000_0000_0000,
    Object = 0x7FF8_0000_0000_0001,
    Symbol = 0x7FF8_0000_0000_0002,
    String = 0x7FF8_0000_0000_0003,

    // Masks
    TaggedMask = 0x7FFF_0000_0000_0000,
    PointerMask = 0x0007_FFFF_FFFF_FFFC,
    PointerTypeMask = 0x0000_0000_0000_0003,
}

// Verify that all representations of NanBitTag ARE NAN, but don't match static NAN.
// The only exception to this rule is BigInt, which assumes that the pointer is
// non-null. The static f64::NAN is equal to BigInt.
const_assert!(f64::from_bits(NanBitTag::Undefined as u64).is_nan());

impl NanBitTag {
    /// Checks if the value is a specific tagged value.
    #[inline]
    const fn is(self, value: u64) -> bool {
        (value & self as u64) == self as u64
    }

    /// Checks that a value is a valid boolean (either true or false).
    #[inline]
    const fn is_bool(value: u64) -> bool {
        // We know that if the tag matches false, it is a boolean.
        (value & NanBitTag::False as u64) == NanBitTag::False as u64
    }

    /// Checks that a value is a valid float, not a tagged nan boxed value.
    #[inline]
    const fn is_float(value: u64) -> bool {
        (value & NanBitTag::TaggedMask as u64) != NanBitTag::TaggedMask as u64
    }

    /// Return the tag of this value.
    #[inline]
    const fn tag_of(value: u64) -> Option<Self> {
        match value & NanBitTag::TaggedMask as u64 {
            0x7FF4_0000_0000_0000 => Some(NanBitTag::Undefined),
            0x7FF5_0000_0000_0000 => Some(NanBitTag::Null),
            0x7FF6_0000_0000_0000 => Some(NanBitTag::False),
            0x7FF6_0000_0000_0001 => Some(NanBitTag::True),
            0x7FF7_0000_0000_0000 => Some(NanBitTag::Integer32),
            // Verify this is not a NULL pointer.
            0x7FF8_0000_0000_0000 if (value & NanBitTag::PointerMask as u64) != 0 => {
                Some(NanBitTag::BigInt)
            }
            0x7FF8_0000_0000_0001 => Some(NanBitTag::Object),
            0x7FF8_0000_0000_0002 => Some(NanBitTag::Symbol),
            0x7FF8_0000_0000_0003 => Some(NanBitTag::String),
            _ => None,
        }
    }

    /// Returns a tagged u64 of a 64-bits float.
    #[inline]
    const fn tag_f64(value: f64) -> u64 {
        if value.is_nan() {
            // Reduce any NAN to a canonical NAN representation.
            f64::NAN.to_bits()
        } else {
            value.to_bits()
        }
    }

    /// Returns a tagged u64 of a 32-bits integer.
    #[inline]
    const fn tag_i32(value: i32) -> u64 {
        // Get the 32-bits integer value inside an u64 as is, in native endian.
        let mut tagged = (Self::Integer32 as u64).to_ne_bytes();
        let bytes = value.to_ne_bytes();

        tagged[4] = bytes[0];
        tagged[5] = bytes[1];
        tagged[6] = bytes[2];
        tagged[7] = bytes[3];

        u64::from_ne_bytes(tagged)
    }

    /// Returns a i32-bits from a tagged integer.
    #[inline]
    const fn untag_i32(value: u64) -> Option<i32> {
        if value & NanBitTag::Integer32 as u64 == NanBitTag::Integer32 as u64 {
            // Get the 32-bits integer value inside an u64 as is.
            let bytes = value.to_ne_bytes();
            Some(i32::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))
        } else {
            None
        }
    }

    /// Returns a tagged u64 of a boolean.
    #[inline]
    const fn tag_bool(value: bool) -> u64 {
        if value {
            Self::True as u64
        } else {
            Self::False as u64
        }
    }

    /// Returns a tagged u64 of a boxed `[JsBigInt]`.
    ///
    /// # Safety
    /// The pointer must be 4-bits aligned and cannot exceed 51-bits. This will
    /// result in a panic. Also, the object is not checked for validity.
    ///
    /// The box is forgotten after this operation. It must be dropped separately,
    /// by calling `[Self::drop_pointer]`.
    #[inline]
    unsafe fn tag_bigint(value: Box<JsBigInt>) -> u64 {
        let value = Box::into_raw(value) as u64;
        let value_masked: u64 = value & Self::PointerMask as u64;

        // Assert alignment and location of the pointer.
        assert_eq!(
            value_masked, value,
            "Pointer is not 4-bits aligned or over 51-bits."
        );
        // Cannot have a null pointer for bigint.
        assert_ne!(value & Self::PointerMask as u64, 0, "Pointer is NULL.");

        // Simply cast for bits.
        Self::BigInt as u64 | value
    }

    /// Returns a tagged u64 of a boxed `[JsObject]`.
    ///
    /// # Safety
    /// The pointer must be 4-bits aligned and cannot exceed 51-bits. This will
    /// result in a panic. Also, the object is not checked for validity.
    ///
    /// The box is forgotten after this operation. It must be dropped separately,
    /// by calling `[Self::drop_pointer]`.
    #[inline]
    unsafe fn tag_object(value: Box<JsObject>) -> u64 {
        let value = Box::into_raw(value) as u64;
        let value_masked: u64 = value & Self::PointerMask as u64;

        // Assert alignment and location of the pointer.
        assert_eq!(
            value_masked, value,
            "Pointer is not 4-bits aligned or over 51-bits."
        );

        // Simply cast for bits.
        Self::Object as u64 | value
    }

    /// Returns a tagged u64 of a boxed `[JsSymbol]`.
    ///
    /// # Safety
    /// The pointer must be 4-bits aligned and cannot exceed 51-bits. This will
    /// result in a panic. Also, the object is not checked for validity.
    ///
    /// The box is forgotten after this operation. It must be dropped separately,
    /// by calling `[Self::drop_pointer]`.
    #[inline]
    unsafe fn tag_symbol(value: Box<JsSymbol>) -> u64 {
        let value = Box::into_raw(value) as u64;
        let value_masked: u64 = value & Self::PointerMask as u64;

        // Assert alignment and location of the pointer.
        assert_eq!(
            value_masked, value,
            "Pointer is not 4-bits aligned or over 51-bits."
        );

        // Simply cast for bits.
        Self::Symbol as u64 | value
    }

    /// Returns a tagged u64 of a boxed `[JsString]`.
    ///
    /// # Safety
    /// The pointer must be 4-bits aligned and cannot exceed 51-bits. This will
    /// result in a panic. Also, the object is not checked for validity.
    ///
    /// The box is forgotten after this operation. It must be dropped separately,
    /// by calling `[Self::drop_pointer]`.
    #[inline]
    unsafe fn tag_string(value: Box<JsString>) -> u64 {
        let value = Box::into_raw(value) as u64;
        let value_masked: u64 = value & Self::PointerMask as u64;

        // Assert alignment and location of the pointer.
        assert_eq!(
            value_masked, value,
            "Pointer is not 4-bits aligned or over 51-bits."
        );

        // Simply cast for bits.
        Self::String as u64 | value
    }

    /// Drops a value if it is a pointer, otherwise do nothing.
    #[inline]
    unsafe fn drop_pointer(value: u64) {
        let value_ptr = value & Self::PointerMask as u64;

        match Self::tag_of(value) {
            Some(Self::BigInt) => {
                drop(unsafe { Box::from_raw(value_ptr as *mut JsBigInt) });
            }
            Some(Self::Object) => {
                drop(unsafe { Box::from_raw(value_ptr as *mut JsObject) });
            }
            Some(Self::Symbol) => {
                drop(unsafe { Box::from_raw(value_ptr as *mut JsSymbol) });
            }
            Some(Self::String) => {
                drop(unsafe { Box::from_raw(value_ptr as *mut JsString) });
            }
            _ => {}
        }
    }
}

/// A NaN-boxed `[super::JsValue]`'s inner.
pub(super) struct InnerValue {
    inner: u64,
}

impl InnerValue {
    /// Creates a new `InnerValue` from an u64 value without checking the validity
    /// of the value.
    #[must_use]
    #[inline]
    fn from_inner_unchecked(inner: u64) -> Self {
        Self { inner }
    }

    /// Returns a `InnerValue` from a Null.
    #[must_use]
    #[inline]
    pub(super) fn null() -> Self {
        Self::from_inner_unchecked(NanBitTag::Null as u64)
    }

    /// Returns a `InnerValue` from an undefined.
    #[must_use]
    #[inline]
    pub(super) fn undefined() -> Self {
        Self::from_inner_unchecked(NanBitTag::Undefined as u64)
    }

    /// Returns a `InnerValue` from a 64-bits float. If the float is `NaN`,
    /// it will be reduced to a canonical `NaN` representation.
    #[must_use]
    #[inline]
    pub(super) fn float64(value: f64) -> Self {
        Self::from_inner_unchecked(NanBitTag::tag_f64(value))
    }

    /// Returns a `InnerValue` from a 32-bits integer.
    #[must_use]
    #[inline]
    pub(super) fn integer32(value: i32) -> Self {
        Self::from_inner_unchecked(NanBitTag::tag_i32(value))
    }

    /// Returns a `InnerValue` from a boolean.
    #[must_use]
    #[inline]
    pub(super) fn boolean(value: bool) -> Self {
        Self::from_inner_unchecked(NanBitTag::tag_bool(value))
    }

    /// Returns a `InnerValue` from a boxed `[JsBigInt]`.
    #[must_use]
    #[inline]
    pub(super) fn bigint(value: JsBigInt) -> Self {
        Self::from_inner_unchecked(unsafe { NanBitTag::tag_bigint(Box::new(value)) })
    }

    /// Returns a `InnerValue` from a boxed `[JsObject]`.
    #[must_use]
    #[inline]
    pub(super) fn object(value: JsObject) -> Self {
        Self::from_inner_unchecked(unsafe { NanBitTag::tag_object(Box::new(value)) })
    }

    /// Returns a `InnerValue` from a boxed `[JsSymbol]`.
    #[must_use]
    #[inline]
    pub(super) fn symbol(value: JsSymbol) -> Self {
        Self::from_inner_unchecked(unsafe { NanBitTag::tag_symbol(Box::new(value)) })
    }

    /// Returns a `InnerValue` from a boxed `[JsString]`.
    #[must_use]
    #[inline]
    pub(super) fn string(value: JsString) -> Self {
        Self::from_inner_unchecked(unsafe { NanBitTag::tag_string(Box::new(value)) })
    }

    /// Returns true if a value is undefined.
    #[must_use]
    #[inline]
    pub(super) fn is_undefined(&self) -> bool {
        NanBitTag::Undefined.is(self.inner)
    }

    /// Returns true if a value is null.
    #[must_use]
    #[inline]
    pub(super) fn is_null(&self) -> bool {
        NanBitTag::Null.is(self.inner)
    }

    /// Returns true if a value is a boolean.
    #[must_use]
    #[inline]
    pub(super) fn is_bool(&self) -> bool {
        NanBitTag::is_bool(self.inner)
    }

    /// Returns true if a value is a 64-bits float.
    #[must_use]
    #[inline]
    pub(super) fn is_float64(&self) -> bool {
        NanBitTag::is_float(self.inner)
    }

    /// Returns true if a value is a 32-bits integer.
    #[must_use]
    #[inline]
    pub(super) fn is_integer32(&self) -> bool {
        NanBitTag::Integer32.is(self.inner)
    }

    /// Returns true if a value is a `[JsBigInt]`. A `NaN` will not match here.
    #[must_use]
    #[inline]
    pub(super) fn is_bigint(&self) -> bool {
        NanBitTag::BigInt.is(self.inner) && (self.inner & NanBitTag::PointerMask as u64) != 0
    }

    /// Returns true if a value is a boxed Object.
    #[must_use]
    #[inline]
    pub(super) fn is_object(&self) -> bool {
        NanBitTag::Object.is(self.inner)
    }

    /// Returns true if a value is a boxed Symbol.
    #[must_use]
    #[inline]
    pub(super) fn is_symbol(&self) -> bool {
        NanBitTag::Symbol.is(self.inner)
    }

    /// Returns true if a value is a boxed String.
    #[must_use]
    #[inline]
    pub(super) fn is_string(&self) -> bool {
        NanBitTag::String.is(self.inner)
    }

    /// Returns the value as an f64 if it is a float.
    #[must_use]
    #[inline]
    pub(super) fn as_float64(&self) -> Option<f64> {
        if self.is_float64() {
            Some(f64::from_bits(self.inner))
        } else {
            None
        }
    }

    /// Returns the value as an i32 if it is an integer.
    #[must_use]
    #[inline]
    pub(super) fn as_integer32(&self) -> Option<i32> {
        if self.is_integer32() {
            Some(self.inner as i32)
        } else {
            None
        }
    }
}

impl Drop for InnerValue {
    fn drop(&mut self) {
        // Drop the pointer if it is a pointer.
        unsafe {
            NanBitTag::drop_pointer(self.inner);
        }
    }
}
