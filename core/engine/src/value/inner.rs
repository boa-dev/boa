//! Module implementing the operations for the inner value of a `[super::JsValue]`.
//! The [InnerValue] type is a NaN-boxed value, which is a 64-bits value that
//! can represent any JavaScript value. If the integer is a non-NaN value, it
//! will be stored as a 64-bits float. If it is a f64::NAN value, it will be
//! stored as a quiet NaN value. Subnormal numbers are regular float.
//!
//! For any other type of values, the value will be stored as a 51-bits non-zero
//! integer.
//!
//! In short, the memory layout of a NaN-boxed value is as follows:
//!
//! | Type of   | Bit Layout | Comment |
//! |-----------|------------|---------|
//! | +Infinity | `7FF0:0000:0000:0000` | |
//! | -Infinity | `FFF0:0000:0000:0000` | |
//! | Undefined | `7FF4:0000:0000:0000` | |
//! | Null      | `7FF5:0000:0000:0000` | |
//! | False     | `7FF6:0000:0000:0000` | |
//! | True      | `7FF6:0000:0000:0001` | |
//! | Integer32 | `7FF7:0000:IIII:IIII` | 32-bits integer. |
//! | BigInt    | `7FF[8-F]:PPPP:PPPP:PPPP | 0` | 51-bits pointer. |
//! | Object    | `7FF[8-F]:PPPP:PPPP:PPPP | 1` | 51-bits pointer. |
//! | Symbol    | `7FF[8-F]:PPPP:PPPP:PPPP | 2` | 51-bits pointer. |
//! | String    | `7FF[8-F]:PPPP:PPPP:PPPP | 3` | 51-bits pointer. |
//! | Float64   | Any other values.     | |
//!
//! Pointers have the highest bit (in the NaN tag) set to 1, so they
//! can represent any value from `0x8000_0000_0000` to `0xFFFF_FFFF_FFFF`.
//! The last 2 bits of the pointer is used to store the type of the value.
//!
//! This only works on 4-bits aligned values, which is asserted when the
//! `NanBox` is created.

use crate::JsObject;

/// The bit mask for a quiet NaN in f64.
const QUIET_NAN: u64 = 0x7FF8_0000_0000_0000;

/// The bit tag for NaN-boxed values. Masks are applied when creating
/// the value.
#[derive(Copy)]
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
}

impl NanBitTag {
    /// Checks if the value is a specific tagged value.
    #[inline]
    fn is(self, value: u64) -> bool {
        (value & self as u64) == self as u64
    }

    /// Returns a tagged u64 of a 32-bits integer.
    #[inline]
    fn tag_i32(value: i32) -> u64 {
        // Get the 32-bits integer value inside a u64 as is.
        let value: u64 = ((value as i64) & 0xFFFF_FFFFi64) as u64;
        Self::Integer32 as u64 | value
    }

    /// Returns a tagged u64 of a boxed JsObject.
    ///
    /// # Safety
    /// The pointer must be 4-bits aligned and cannot exceed 51-bits. This will
    /// result in a panic. Also, the object is not checked for validity.
    ///
    /// The object is forgotten after this operation. It must be dropped
    /// separately.
    #[inline]
    unsafe fn tag_object(value: Box<JsObject>) -> u64 {
        let value = Box::into_raw(value) as u64;
        let value_masked: u64 = value & 0x0007_FFFF_FFFF_FFFC_u64;

        // Assert alignment and location of the pointer.
        assert_eq!(value, value_masked);

        // Simply cast for bits.
        Self::Object as u64 | value
    }

    /// Drops a value if it is a pointer, otherwise do nothing.
    #[inline]
    fn drop_pointer(value: u64) {
        let value = value & 0x0007_FFFF_FFFF_FFFC_u64;
        let _ = Box::from_raw(value as *mut JsObject);
    }
}

// We cannot NaN-box pointers larger than 64 bits.
static_assertions::const_assert!(size_of::<usize>() <= size_of::<u64>());

/// A NaN-boxed [super::JsValue]'s inner.
pub(super) struct InnerValue {
    inner: u64,
    // Forces invariance of T.
    _marker: std::marker::PhantomData<*mut T>,
}

impl InnerValue {
    /// Creates a new `NanBox` from an inner without checking the validity
    /// of the value.
    #[must_use]
    #[inline]
    fn from_inner_unchecked(inner: u64) -> Self {
        Self {
            inner,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns a `NanBox` from a 64-bits float. If the float is NaN,
    /// it will be reduced to a canonical NaN representation.
    #[must_use]
    #[inline]
    pub(super) fn float64(value: f64) -> Self {
        // Reduce any NAN to a canonical NAN representation.
        if value.is_nan() {
            Self::from_inner_unchecked(QUIET_NAN)
        } else {
            Self::from_inner_unchecked(value.to_bits())
        }
    }

    /// Returns a `NanBox` from a 32-bits integer.
    #[must_use]
    #[inline]
    pub(super) fn integer32(value: i32) -> Self {
        Self::from_inner_unchecked(NanBitTag::Integer32 as u64 | (value as u32 as u64))
    }
}
