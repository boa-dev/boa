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

use crate::{JsBigInt, JsObject, JsSymbol, JsVariant};
use boa_gc::{custom_trace, Finalize, Trace};
use boa_string::JsString;
use core::fmt;
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
///
/// All the bit masking, tagging and untagging is done in this type.
#[derive(Copy, Clone, Debug)]
#[repr(u64)]
enum NanBitTag {
    Undefined = 0x7FF4_0000_0000_0000,
    Null = 0x7FF5_0000_0000_0000,
    False = 0x7FF6_0000_0000_0000,
    True = 0x7FF6_0000_0000_0001,
    Integer32 = 0x7FF7_0000_0000_0000,

    /// A generic pointer.
    Pointer = 0x7FF8_0000_0000_0000,
    BigInt = 0x0000_0000_0000_0000,
    Object = 0x0000_0000_0000_0001,
    Symbol = 0x0000_0000_0000_0002,
    String = 0x0000_0000_0000_0003,

    // Masks
    TaggedMask = 0x7FFC_0000_0000_0000,
    PointerMask = 0x0007_FFFF_FFFF_FFFC,
}

// Verify that all representations of NanBitTag ARE NAN, but don't match static NAN.
// The only exception to this rule is BigInt, which assumes that the pointer is
// non-null. The static f64::NAN is equal to BigInt.
const_assert!(f64::from_bits(NanBitTag::Undefined as u64).is_nan());

impl NanBitTag {
    /// Checks that a value is a valid boolean (either true or false).
    #[inline]
    const fn is_bool(value: u64) -> bool {
        // We know that if the tag matches false, it is a boolean.
        (value & NanBitTag::False as u64) == NanBitTag::False as u64
    }

    /// Checks that a value is a valid float, not a tagged nan boxed value.
    #[inline]
    const fn is_float(value: u64) -> bool {
        // Either it is a constant float value,
        if value == f64::INFINITY.to_bits()
            || value == f64::NEG_INFINITY.to_bits()
            // or it is exactly a NaN value, which is the same as the BigInt tag.
            // Reminder that pointers cannot be null, so this is safe.
            || value == NanBitTag::Pointer as u64
        {
            return true;
        }

        // Or it is not tagged,
        match value & NanBitTag::TaggedMask as u64 {
            0x7FF4_0000_0000_0000 => false,
            0x7FF5_0000_0000_0000 => false,
            0x7FF6_0000_0000_0000 => false,
            0x7FF6_0000_0000_0001 => false,
            0x7FF7_0000_0000_0000 => false,
            0x7FF8_0000_0000_0000 => false,
            0x7FF9_0000_0000_0000 => false,
            0x7FFA_0000_0000_0000 => false,
            0x7FFB_0000_0000_0000 => false,
            0x7FFC_0000_0000_0000 => false,
            0x7FFD_0000_0000_0000 => false,
            0x7FFE_0000_0000_0000 => false,
            0x7FFF_0000_0000_0000 => false,
            _ => true,
        }
    }

    /// Checks that a value is a valid undefined.
    #[inline]
    const fn is_undefined(value: u64) -> bool {
        value == NanBitTag::Undefined as u64
    }

    /// Checks that a value is a valid null.
    #[inline]
    const fn is_null(value: u64) -> bool {
        value == NanBitTag::Null as u64
    }

    /// Checks that a value is a valid integer32.
    #[inline]
    const fn is_integer32(value: u64) -> bool {
        value & NanBitTag::Integer32 as u64 == NanBitTag::Integer32 as u64
    }

    /// Checks that a value is a valid BigInt.
    #[inline]
    const fn is_bigint(value: u64) -> bool {
        (value & NanBitTag::TaggedMask as u64 == NanBitTag::Pointer as u64)
            && (value & 0x3 == Self::BigInt as u64)
            && (value & NanBitTag::PointerMask as u64) != 0
    }

    /// Checks that a value is a valid Object.
    #[inline]
    const fn is_object(value: u64) -> bool {
        (value & NanBitTag::TaggedMask as u64 == NanBitTag::Pointer as u64)
            && (value & 0x3 == Self::Object as u64)
    }

    /// Checks that a value is a valid Symbol.
    #[inline]
    const fn is_symbol(value: u64) -> bool {
        (value & NanBitTag::TaggedMask as u64 == NanBitTag::Pointer as u64)
            && (value & 0x3 == Self::Symbol as u64)
    }

    /// Checks that a value is a valid String.
    #[inline]
    const fn is_string(value: u64) -> bool {
        (value & NanBitTag::TaggedMask as u64 == NanBitTag::Pointer as u64)
            && (value & 0x3 == Self::String as u64)
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
        Self::Integer32 as u64 | value as u64 & 0xFFFF_FFFFu64
    }

    /// Returns a i32-bits from a tagged integer.
    #[inline]
    const fn untag_i32(value: u64) -> Option<i32> {
        if Self::is_integer32(value) {
            Some(((value & 0xFFFF_FFFFu64) | 0xFFFF_FFFF_0000_0000u64) as i32)
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
        Self::Pointer as u64 | Self::BigInt as u64 | value
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
        Self::Pointer as u64 | Self::Object as u64 | value
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
        Self::Pointer as u64 | Self::Symbol as u64 | value
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
        Self::Pointer as u64 | Self::String as u64 | value
    }

    /// Drops a value if it is a pointer, otherwise do nothing.
    #[inline]
    unsafe fn drop_pointer(value: u64) {
        let value_ptr = value & Self::PointerMask as u64;

        if value & NanBitTag::Pointer as u64 != 0 || value == NanBitTag::Pointer as u64 {
            return;
        }

        match value & 0x3 {
            0 => drop(unsafe { Box::from_raw(value_ptr as *mut JsBigInt) }),
            1 => drop(unsafe { Box::from_raw(value_ptr as *mut JsObject) }),
            2 => drop(unsafe { Box::from_raw(value_ptr as *mut JsSymbol) }),
            3 => drop(unsafe { Box::from_raw(value_ptr as *mut JsString) }),
            _ => unreachable!(),
        }
    }
}

/// A NaN-boxed `[super::JsValue]`'s inner.
#[derive(PartialEq)]
pub(super) struct InnerValue(u64);

impl fmt::Debug for InnerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_variant() {
            JsVariant::Undefined => f.debug_tuple("Undefined").finish(),
            JsVariant::Null => f.debug_tuple("Null").finish(),
            JsVariant::Boolean(b) => f.debug_tuple("Boolean").field(&b).finish(),
            JsVariant::Float64(n) => f.debug_tuple("Float64").field(&n).finish(),
            JsVariant::Integer32(n) => f.debug_tuple("Integer32").field(&n).finish(),
            JsVariant::BigInt(n) => f.debug_tuple("BigInt").field(&n).finish(),
            JsVariant::Object(n) => f.debug_tuple("Object").field(&n).finish(),
            JsVariant::Symbol(n) => f.debug_tuple("Symbol").field(&n).finish(),
            JsVariant::String(n) => f.debug_tuple("String").field(&n).finish(),
        }
    }
}

impl Finalize for InnerValue {}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl Trace for InnerValue {
    custom_trace! {this, mark, {
        if let JsVariant::Object(o) = this.as_variant() {
            mark(o);
        }
    }}
}

impl Clone for InnerValue {
    fn clone(&self) -> Self {
        match self.as_variant() {
            JsVariant::BigInt(n) => Self::bigint(n.clone()),
            JsVariant::Object(n) => Self::object(n.clone()),
            JsVariant::Symbol(n) => Self::symbol(n.clone()),
            JsVariant::String(n) => Self::string(n.clone()),
            _ => Self(self.0),
        }
    }
}

impl InnerValue {
    /// Creates a new `InnerValue` from an u64 value without checking the validity
    /// of the value.
    #[must_use]
    #[inline]
    const fn from_inner_unchecked(inner: u64) -> Self {
        Self(inner)
    }

    /// Returns a `InnerValue` from a Null.
    #[must_use]
    #[inline]
    pub(super) const fn null() -> Self {
        Self::from_inner_unchecked(NanBitTag::Null as u64)
    }

    /// Returns a `InnerValue` from an undefined.
    #[must_use]
    #[inline]
    pub(super) const fn undefined() -> Self {
        Self::from_inner_unchecked(NanBitTag::Undefined as u64)
    }

    /// Returns a `InnerValue` from a 64-bits float. If the float is `NaN`,
    /// it will be reduced to a canonical `NaN` representation.
    #[must_use]
    #[inline]
    pub(super) const fn float64(value: f64) -> Self {
        Self::from_inner_unchecked(NanBitTag::tag_f64(value))
    }

    /// Returns a `InnerValue` from a 32-bits integer.
    #[must_use]
    #[inline]
    pub(super) const fn integer32(value: i32) -> Self {
        Self::from_inner_unchecked(NanBitTag::tag_i32(value))
    }

    /// Returns a `InnerValue` from a boolean.
    #[must_use]
    #[inline]
    pub(super) const fn boolean(value: bool) -> Self {
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
    pub(super) const fn is_undefined(&self) -> bool {
        NanBitTag::is_undefined(self.0)
    }

    /// Returns true if a value is null.
    #[must_use]
    #[inline]
    pub(super) const fn is_null(&self) -> bool {
        NanBitTag::is_null(self.0)
    }

    /// Returns true if a value is a boolean.
    #[must_use]
    #[inline]
    pub(super) const fn is_bool(&self) -> bool {
        NanBitTag::is_bool(self.0)
    }

    /// Returns true if a value is a 64-bits float.
    #[must_use]
    #[inline]
    pub(super) const fn is_float64(&self) -> bool {
        NanBitTag::is_float(self.0)
    }

    /// Returns true if a value is a 32-bits integer.
    #[must_use]
    #[inline]
    pub(super) const fn is_integer32(&self) -> bool {
        NanBitTag::is_integer32(self.0)
    }

    /// Returns true if a value is a `[JsBigInt]`. A `NaN` will not match here.
    #[must_use]
    #[inline]
    pub(super) const fn is_bigint(&self) -> bool {
        NanBitTag::is_bigint(self.0)
    }

    /// Returns true if a value is a boxed Object.
    #[must_use]
    #[inline]
    pub(super) const fn is_object(&self) -> bool {
        NanBitTag::is_object(self.0)
    }

    /// Returns true if a value is a boxed Symbol.
    #[must_use]
    #[inline]
    pub(super) const fn is_symbol(&self) -> bool {
        NanBitTag::is_symbol(self.0)
    }

    /// Returns true if a value is a boxed String.
    #[must_use]
    #[inline]
    pub(super) const fn is_string(&self) -> bool {
        NanBitTag::is_string(self.0)
    }

    /// Returns the value as an f64 if it is a float.
    #[must_use]
    #[inline]
    pub(super) const fn as_float64(&self) -> Option<f64> {
        if self.is_float64() {
            Some(f64::from_bits(self.0))
        } else {
            None
        }
    }

    /// Returns the value as an i32 if it is an integer.
    #[must_use]
    #[inline]
    pub(super) const fn as_integer32(&self) -> Option<i32> {
        NanBitTag::untag_i32(self.0)
    }

    /// Returns the value as a boolean if it is a boolean.
    #[must_use]
    #[inline]
    pub(super) const fn as_bool(&self) -> Option<bool> {
        if self.0 == NanBitTag::False as u64 {
            Some(false)
        } else if self.0 == NanBitTag::True as u64 {
            Some(true)
        } else {
            None
        }
    }

    /// Returns the value as a boxed `[JsBigInt]`.
    #[must_use]
    #[inline]
    pub(super) const fn as_bigint(&self) -> Option<&JsBigInt> {
        if self.is_bigint() {
            // This is safe because the boxed object will always be on the heap.
            let ptr = self.0 & NanBitTag::PointerMask as u64;
            unsafe { Some(&*(ptr as *const _)) }
        } else {
            None
        }
    }

    /// Returns the value as a boxed `[JsObject]`.
    #[must_use]
    #[inline]
    pub(super) const fn as_object(&self) -> Option<&JsObject> {
        if self.is_object() {
            // This is safe because the boxed object will always be on the heap.
            let ptr = self.0 & NanBitTag::PointerMask as u64;
            unsafe { Some(&*(ptr as *const _)) }
        } else {
            None
        }
    }

    /// Returns the value as a boxed `[JsSymbol]`.
    #[must_use]
    #[inline]
    pub(super) const fn as_symbol(&self) -> Option<&JsSymbol> {
        if self.is_symbol() {
            // This is safe because the boxed object will always be on the heap.
            let ptr = self.0 & NanBitTag::PointerMask as u64;
            unsafe { Some(&*(ptr as *const _)) }
        } else {
            None
        }
    }

    /// Returns the value as a boxed `[JsString]`.
    #[must_use]
    #[inline]
    pub(super) const fn as_string(&self) -> Option<&JsString> {
        if self.is_string() {
            // This is safe because the boxed object will always be on the heap.
            let ptr = self.0 & NanBitTag::PointerMask as u64;
            unsafe { Some(&*(ptr as *const _)) }
        } else {
            None
        }
    }

    /// Returns the `[JsVariant]` of this inner value.
    #[must_use]
    #[inline]
    pub(super) const fn as_variant(&self) -> JsVariant<'_> {
        if self.is_undefined() {
            JsVariant::Undefined
        } else if self.is_null() {
            JsVariant::Null
        } else if let Some(b) = self.as_bool() {
            JsVariant::Boolean(b)
        } else if let Some(f) = self.as_float64() {
            JsVariant::Float64(f)
        } else if let Some(i) = self.as_integer32() {
            JsVariant::Integer32(i)
        } else if let Some(bigint) = self.as_bigint() {
            JsVariant::BigInt(bigint)
        } else if let Some(obj) = self.as_object() {
            JsVariant::Object(obj)
        } else if let Some(sym) = self.as_symbol() {
            JsVariant::Symbol(sym)
        } else if let Some(str) = self.as_string() {
            JsVariant::String(str)
        } else {
            unreachable!()
        }
    }
}

impl Drop for InnerValue {
    fn drop(&mut self) {
        // Drop the pointer if it is a pointer.
        unsafe {
            NanBitTag::drop_pointer(self.0);
        }
    }
}

#[test]
fn float() {
    fn assert_float(f: f64) {
        let v = InnerValue::float64(f);

        assert!(!v.is_undefined());
        assert!(!v.is_null());
        assert!(!v.is_bool());
        assert!(!v.is_integer32());
        assert!(v.is_float64());
        assert!(!v.is_bigint());
        assert!(!v.is_string());
        assert!(!v.is_object());
        assert!(!v.is_symbol());

        assert_eq!(v.as_bool(), None);
        assert_eq!(v.as_integer32(), None);
        assert_eq!(v.as_float64(), Some(f));
        assert_eq!(v.as_bigint(), None);
        assert_eq!(v.as_object(), None);
        assert_eq!(v.as_string(), None);
        assert_eq!(v.as_symbol(), None);
    }

    assert_float(0.0);
    assert_float(-0.0);
    assert_float(3.14);
    assert_float(-3.14);
    assert_float(f64::INFINITY);
    assert_float(f64::NEG_INFINITY);

    // Special care has to be taken for NaN, because NaN != NaN.
    let v = InnerValue::float64(f64::NAN);
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(!v.is_bool());
    assert!(!v.is_integer32());
    assert!(v.is_float64());
    assert!(!v.is_bigint());
    assert!(!v.is_string());
    assert!(!v.is_object());
    assert!(!v.is_symbol());

    assert_eq!(v.as_bool(), None);
    assert_eq!(v.as_integer32(), None);
    assert!(v.as_float64().unwrap().is_nan());
    assert_eq!(v.as_bigint(), None);
    assert_eq!(v.as_object(), None);
    assert_eq!(v.as_string(), None);
    assert_eq!(v.as_symbol(), None);
}

#[test]
fn integer() {
    let int = 42;
    let v = InnerValue::integer32(int);
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(!v.is_bool());
    assert!(v.is_integer32());
    assert!(!v.is_float64());
    assert!(!v.is_bigint());
    assert!(!v.is_string());
    assert!(!v.is_object());
    assert!(!v.is_symbol());

    assert_eq!(v.as_bool(), None);
    assert_eq!(v.as_integer32(), Some(int));
    assert_eq!(v.as_float64(), None);
    assert_eq!(v.as_bigint(), None);
    assert_eq!(v.as_object(), None);
    assert_eq!(v.as_string(), None);
    assert_eq!(v.as_symbol(), None);

    let int = -42;
    let v = InnerValue::integer32(int);
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(!v.is_bool());
    assert!(v.is_integer32());
    assert!(!v.is_float64());
    assert!(!v.is_bigint());
    assert!(!v.is_string());
    assert!(!v.is_object());
    assert!(!v.is_symbol());

    assert_eq!(v.as_bool(), None);
    assert_eq!(v.as_integer32(), Some(int));
    assert_eq!(v.as_float64(), None);
    assert_eq!(v.as_bigint(), None);
    assert_eq!(v.as_object(), None);
    assert_eq!(v.as_string(), None);
    assert_eq!(v.as_symbol(), None);

    let int = 0;
    let v = InnerValue::integer32(int);
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(!v.is_bool());
    assert!(v.is_integer32());
    assert!(!v.is_float64());
    assert!(!v.is_bigint());
    assert!(!v.is_string());
    assert!(!v.is_object());
    assert!(!v.is_symbol());

    assert_eq!(v.as_bool(), None);
    assert_eq!(v.as_integer32(), Some(int));
    assert_eq!(v.as_float64(), None);
    assert_eq!(v.as_bigint(), None);
    assert_eq!(v.as_object(), None);
    assert_eq!(v.as_string(), None);
    assert_eq!(v.as_symbol(), None);
}

#[test]
fn boolean() {
    let v = InnerValue::boolean(true);
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(v.is_bool());
    assert!(!v.is_integer32());
    assert!(!v.is_float64());
    assert!(!v.is_bigint());
    assert!(!v.is_string());
    assert!(!v.is_object());
    assert!(!v.is_symbol());

    assert_eq!(v.as_bool(), Some(true));
    assert_eq!(v.as_integer32(), None);
    assert_eq!(v.as_float64(), None);
    assert_eq!(v.as_bigint(), None);
    assert_eq!(v.as_object(), None);
    assert_eq!(v.as_string(), None);
    assert_eq!(v.as_symbol(), None);

    let v = InnerValue::boolean(false);
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(v.is_bool());
    assert!(!v.is_integer32());
    assert!(!v.is_float64());
    assert!(!v.is_bigint());
    assert!(!v.is_string());
    assert!(!v.is_object());
    assert!(!v.is_symbol());

    assert_eq!(v.as_bool(), Some(false));
    assert_eq!(v.as_integer32(), None);
    assert_eq!(v.as_float64(), None);
    assert_eq!(v.as_bigint(), None);
    assert_eq!(v.as_object(), None);
    assert_eq!(v.as_string(), None);
    assert_eq!(v.as_symbol(), None);
}

#[test]
fn bigint() {
    let bigint = JsBigInt::from(42);
    let v = InnerValue::bigint(bigint.clone());
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(!v.is_bool());
    assert!(!v.is_integer32());
    assert!(!v.is_float64());
    assert!(v.is_bigint());
    assert!(!v.is_string());
    assert!(!v.is_object());
    assert!(!v.is_symbol());

    assert_eq!(v.as_bool(), None);
    assert_eq!(v.as_integer32(), None);
    assert_eq!(v.as_float64(), None);
    assert_eq!(v.as_bigint(), Some(&bigint));
    assert_eq!(v.as_object(), None);
    assert_eq!(v.as_string(), None);
    assert_eq!(v.as_symbol(), None);
}

#[test]
fn object() {
    let object = JsObject::with_null_proto();
    let v = InnerValue::object(object.clone());
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(!v.is_bool());
    assert!(!v.is_integer32());
    assert!(!v.is_float64());
    assert!(!v.is_bigint());
    assert!(!v.is_string());
    assert!(v.is_object());
    assert!(!v.is_symbol());

    assert_eq!(v.as_bool(), None);
    assert_eq!(v.as_integer32(), None);
    assert_eq!(v.as_float64(), None);
    assert_eq!(v.as_bigint(), None);
    assert_eq!(v.as_object(), Some(&object));
    assert_eq!(v.as_string(), None);
    assert_eq!(v.as_symbol(), None);
}

#[test]
fn string() {
    let str = crate::js_string!("Hello World");
    let v = InnerValue::string(str.clone());
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(!v.is_bool());
    assert!(!v.is_integer32());
    assert!(!v.is_float64());
    assert!(!v.is_bigint());
    assert!(v.is_string());
    assert!(!v.is_object());
    assert!(!v.is_symbol());

    assert_eq!(v.as_bool(), None);
    assert_eq!(v.as_integer32(), None);
    assert_eq!(v.as_float64(), None);
    assert_eq!(v.as_bigint(), None);
    assert_eq!(v.as_object(), None);
    assert_eq!(v.as_string(), Some(&str));
    assert_eq!(v.as_symbol(), None);
}

#[test]
fn symbol() {
    let sym = JsSymbol::new(Some(JsString::from("Hello World"))).unwrap();
    let v = InnerValue::symbol(sym.clone());
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(!v.is_bool());
    assert!(!v.is_integer32());
    assert!(!v.is_float64());
    assert!(!v.is_bigint());
    assert!(!v.is_string());
    assert!(!v.is_object());
    assert!(v.is_symbol());

    assert_eq!(v.as_bool(), None);
    assert_eq!(v.as_integer32(), None);
    assert_eq!(v.as_float64(), None);
    assert_eq!(v.as_bigint(), None);
    assert_eq!(v.as_object(), None);
    assert_eq!(v.as_string(), None);
    assert_eq!(v.as_symbol(), Some(&sym));

    let sym = JsSymbol::new(None).unwrap();
    let v = InnerValue::symbol(sym.clone());
    assert!(!v.is_undefined());
    assert!(!v.is_null());
    assert!(!v.is_bool());
    assert!(!v.is_integer32());
    assert!(!v.is_float64());
    assert!(!v.is_bigint());
    assert!(!v.is_string());
    assert!(!v.is_object());
    assert!(v.is_symbol());

    assert_eq!(v.as_bool(), None);
    assert_eq!(v.as_integer32(), None);
    assert_eq!(v.as_float64(), None);
    assert_eq!(v.as_bigint(), None);
    assert_eq!(v.as_object(), None);
    assert_eq!(v.as_string(), None);
    assert_eq!(v.as_symbol(), Some(&sym));
}
