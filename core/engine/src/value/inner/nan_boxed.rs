//! A NaN-boxed inner value for JavaScript values.
//!
//! This `[JsValue]` is a float using `NaN` values to represent inner
//! JavaScript value.
//!
//! # Assumptions
//!
//! This implementation makes exactly two architecture assumptions. Everything
//! else is independent of the arch where it's run.
//!
//! The first assumption is easy to verify: JavaScript numbers must be 64 bits
//! IEEE-754, which is guaranteed by Rust and JavaScript implementations.
//!
//! The second assumption is that pointers are 48 bits maximum. This is a bit
//! more complex to verify, but it is a safe assumption for all current
//! architectures. The only exception is RISC-V and Intel processors that
//! enable 5-level paging extensions.
//!
//! This is clarified here: <https://en.m.wikipedia.org/wiki/64-bit_computing>:
//!
//! > not all 64-bit instruction sets support full 64-bit virtual memory
//! > addresses; x86-64 and AArch64 for example, support only 48 bits of
//! > virtual address, with the remaining 16 bits of the virtual address
//! > required to be all zeros (000...) or all ones (111...), and several
//! > 64-bit instruction sets support fewer than 64 bits of physical
//! > memory address.
//!
//! ALL 32 bits architectures are compatible, of course, as their pointers
//! are 32 bits.
//!
//! WASM with MEMORY64 (which is very rare) follows the pointer structure
//! of its host architecture.
//! For more info, see
//! <https://spidermonkey.dev/blog/2025/01/15/is-memory64-actually-worth-using.html>
//!
//! This leaves RISC-V and processes that enable 5-level paging extensions
//! on Intel (<https://en.m.wikipedia.org/wiki/Intel_5-level_paging>).
//!
//! We could feature gate on RISC-V, but it's not worth it. The only
//! RISC-V processors that support 64-bit are the ones that support 64-bit
//! virtual memory addresses. So it's a safe assumption.
//!
//! There is no way to feature gate on 5-level paging as it's a software
//! trigger.
//!
//! There is a software assertion in the code that will panic if the pointer
//! uses more than 48 bits.
//!
//! # Design
//!
//! This `[JsValue]` inner type is a NaN-boxed value, which is a 64-bits value
//! that can represent any JavaScript value. If the integer is a non-NaN value,
//! it will be stored as a 64-bits float. If it is a `f64::NAN` value, it will
//! be stored as a quiet `NaN` value. Subnormal numbers are regular float.
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
//! | `BigInt`          | `7FF8:PPPP:PPPP:PPPP`    | 48-bits pointer. Assumes non-null pointer. |
//! | `Object`          | `7FFA:PPPP:PPPP:PPPP`    | 48-bits pointer. |
//! | `Symbol`          | `7FFC:PPPP:PPPP:PPPP`    | 48-bits pointer. |
//! | `String`          | `7FFE:PPPP:PPPP:PPPP`    | 48-bits pointer. |
//! | `Float64`         | Any other values.        | |
//!
//! Another way to vizualize this is by looking at the bit layout of a NaN-boxed
//! value:
//! ```text
//!                           ....--<| The type of inner value is represented by this.
//!                           |..|   | 1??0 - Pointer, where ?? is the subtype of pointer:
//!                           |..|   |        b00 - BigInt, b01 - Object,
//!                           |..|   |        b10 - Symbol, b11 - String.
//!                           |..|   |        If the pointer is null, then it is a NaN value.
//!                           |..|   | 0??? - Non-pointer, where ??? is the subtype:
//!                           |..|   |        b100 - Undefined, b101 - Null,
//!                           |..|   |        b011 - Boolean, b110 - Integer32.
//!                           vvvv
//! bit index: 63   59   55   51   47   43   39   35   31 .. 3  0
//!            0000 0000 0000 0000 0000 0000 0000 0000 0000 .. 0000
//! +Inf       0111 1111 1111 0000 0000 0000 0000 0000 0000 .. 0000
//! -Inf       1111 1111 1111 0000 0000 0000 0000 0000 0000 .. 0000
//! NaN (q)    0111 1111 1111 1000 0000 0000 0000 0000 0000 .. 0000
//! NaN (s)    1111 1111 1111 1000 0000 0000 0000 0000 0000 .. 0000
//! Undefined  0111 1111 1111 0100 0000 0000 0000 0000 0000 .. 0000
//! Null       0111 1111 1111 0101 0000 0000 0000 0000 0000 .. 0000
//! False      0111 1111 1111 0110 0000 0000 0000 0000 0000 .. 0000
//! True       0111 1111 1111 0110 0000 0000 0000 0000 0000 .. 0001
//! Integer32  0111 1111 1111 0111 0000 0000 0000 0000 IIII .. IIII
//! BigInt     0111 1111 1111 1000 PPPP PPPP PPPP PPPP PPPP .. PPPP
//! Object     0111 1111 1111 1010 PPPP PPPP PPPP PPPP PPPP .. PPPP
//! Symbol     0111 1111 1111 1100 PPPP PPPP PPPP PPPP PPPP .. PPPP
//! String     0111 1111 1111 1110 PPPP PPPP PPPP PPPP PPPP .. PPPP
//! Float64    Any other value.
//! ```
//!
//! The pointers are assumed to never be NULL, and as such no clash
//! with regular NAN should happen.
#![allow(clippy::inline_always)]

use crate::{JsBigInt, JsObject, JsSymbol, JsVariant};
use boa_gc::{custom_trace, Finalize, Trace};
use boa_string::JsString;
use core::fmt;
use static_assertions::const_assert;

// We cannot NaN-box pointers larger than 64 bits.
const_assert!(size_of::<usize>() <= size_of::<u64>());

// We cannot NaN-box pointers that are not 4-bytes aligned.
const_assert!(align_of::<*mut ()>() >= 4);

/// Internal module for bit masks and constants.
///
/// All bit magic is done here.
mod bits {
    use boa_engine::{JsBigInt, JsObject, JsSymbol};
    use boa_string::JsString;
    use std::ptr::NonNull;

    /// Undefined value in `u64`.
    pub(super) const UNDEFINED: u64 = 0x7FF4_0000_0000_0000;

    /// Null value in `u64`.
    pub(super) const NULL: u64 = 0x7FF5_0000_0000_0000;

    /// False value in `u64`.
    pub(super) const FALSE: u64 = 0x7FF6_0000_0000_0000;

    /// True value in `u64`.
    pub(super) const TRUE: u64 = 0x7FF6_0000_0000_0001;

    /// Integer32 start (zero) value in `u64`.
    pub(super) const INTEGER32_ZERO: u64 = 0x7FF7_0000_0000_0000;

    /// Integer32 end (MAX) value in `u64`.
    pub(super) const INTEGER32_MAX: u64 = 0x7FF7_0000_FFFF_FFFF;

    /// Pointer starting point in `u64`.
    pub(super) const POINTER_START: u64 = 0x7FF8_0000_0000_0000;

    /// Pointer types mask in `u64`.
    pub(super) const POINTER_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

    /// Pointer start point for `BigInt` in `u64`.
    pub(super) const POINTER_BIGINT_START: u64 = POINTER_START | POINTER_TYPE_BIGINT;
    /// Pointer end point for `BigInt` in `u64`.
    pub(super) const POINTER_BIGINT_END: u64 = POINTER_START | POINTER_MASK | POINTER_TYPE_BIGINT;
    /// Pointer start point for `JsObject` in `u64`.
    pub(super) const POINTER_OBJECT_START: u64 = POINTER_START | POINTER_TYPE_OBJECT;
    /// Pointer end point for `JsObject` in `u64`.
    pub(super) const POINTER_OBJECT_END: u64 = POINTER_START | POINTER_MASK | POINTER_TYPE_OBJECT;
    /// Pointer start point for `JsSymbol` in `u64`.
    pub(super) const POINTER_SYMBOL_START: u64 = POINTER_START | POINTER_TYPE_SYMBOL;
    /// Pointer end point for `JsSymbol` in `u64`.
    pub(super) const POINTER_SYMBOL_END: u64 = POINTER_START | POINTER_MASK | POINTER_TYPE_SYMBOL;
    /// Pointer start point for `JsString` in `u64`.
    pub(super) const POINTER_STRING_START: u64 = POINTER_START | POINTER_TYPE_STRING;
    /// Pointer end point for `JsString` in `u64`.
    pub(super) const POINTER_STRING_END: u64 = POINTER_START | POINTER_MASK | POINTER_TYPE_STRING;

    /// Pointer mask for the type of the pointer.
    pub(super) const POINTER_TYPE_MASK: u64 = 0x0007_0000_0000_0000;

    /// Pointer type value for `BigInt`.
    pub(super) const POINTER_TYPE_BIGINT: u64 = 0x0000_0000_0000_0000;

    /// Pointer type value for `JsObject`.
    pub(super) const POINTER_TYPE_OBJECT: u64 = 0x0004_0000_0000_0000;

    /// Pointer type value for `JsSymbol`.
    pub(super) const POINTER_TYPE_SYMBOL: u64 = 0x0005_0000_0000_0000;

    /// Pointer type value for `JsString`.
    pub(super) const POINTER_TYPE_STRING: u64 = 0x0006_0000_0000_0000;

    /// NAN value in `u64`.
    pub(super) const NAN: u64 = 0x7FF8_0000_0000_0000;

    /// Checks that a value is a valid boolean (either true or false).
    #[inline(always)]
    pub(super) const fn is_bool(value: u64) -> bool {
        value == TRUE || value == FALSE
    }

    /// Checks that a value is a valid float, not a tagged nan boxed value.
    #[inline(always)]
    pub(super) const fn is_float(value: u64) -> bool {
        let as_float = f64::from_bits(value);
        !as_float.is_nan() || value == NAN
    }

    /// Checks that a value is a valid undefined.
    #[inline(always)]
    pub(super) const fn is_undefined(value: u64) -> bool {
        value == UNDEFINED
    }

    /// Checks that a value is a valid null.
    #[inline(always)]
    pub(super) const fn is_null(value: u64) -> bool {
        value == NULL
    }

    /// Checks that a value is a valid integer32.
    #[inline(always)]
    pub(super) const fn is_integer32(value: u64) -> bool {
        value & INTEGER32_ZERO == INTEGER32_ZERO
    }

    /// Untag a value as a pointer.
    #[inline(always)]
    pub(super) const fn is_pointer(value: u64) -> bool {
        value & POINTER_START == POINTER_START
    }

    /// Checks that a value is a valid `BigInt`.
    #[inline(always)]
    #[allow(clippy::verbose_bit_mask)]
    pub(super) const fn is_bigint(value: u64) -> bool {
        // If `(value & POINTER_MASK)` is zero, then it is NaN.
        is_pointer(value)
            && (value & POINTER_TYPE_MASK == POINTER_TYPE_BIGINT)
            && (value & POINTER_MASK) != 0
    }

    /// Checks that a value is a valid Object.
    #[inline(always)]
    pub(super) const fn is_object(value: u64) -> bool {
        is_pointer(value) && (value & POINTER_TYPE_MASK == POINTER_TYPE_OBJECT)
    }

    /// Checks that a value is a valid Symbol.
    #[inline(always)]
    pub(super) const fn is_symbol(value: u64) -> bool {
        is_pointer(value) && (value & POINTER_TYPE_MASK == POINTER_TYPE_SYMBOL)
    }

    /// Checks that a value is a valid String.
    #[inline(always)]
    pub(super) const fn is_string(value: u64) -> bool {
        is_pointer(value) && (value & POINTER_TYPE_MASK == POINTER_TYPE_STRING)
    }

    /// Returns a tagged u64 of a 64-bits float.
    #[inline(always)]
    pub(super) const fn tag_f64(value: f64) -> u64 {
        if value.is_nan() {
            // Reduce any NAN to a canonical NAN representation.
            f64::NAN.to_bits()
        } else {
            value.to_bits()
        }
    }

    /// Returns a tagged u64 of a 32-bits integer.
    #[inline(always)]
    pub(super) const fn tag_i32(value: i32) -> u64 {
        INTEGER32_ZERO | value as u64 & 0xFFFF_FFFFu64
    }

    /// Returns a i32-bits from a tagged integer.
    #[inline(always)]
    pub(super) const fn untag_i32(value: u64) -> i32 {
        ((value & 0xFFFF_FFFFu64) | 0xFFFF_FFFF_0000_0000u64) as i32
    }

    /// Returns a tagged u64 of a boolean.
    #[inline(always)]
    pub(super) const fn tag_bool(value: bool) -> u64 {
        if value {
            TRUE
        } else {
            FALSE
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
    #[inline(always)]
    #[allow(clippy::identity_op)]
    pub(super) unsafe fn tag_bigint(value: Box<JsBigInt>) -> u64 {
        let value = Box::into_raw(value) as u64;
        let value_masked: u64 = value & POINTER_MASK;

        // Assert alignment and location of the pointer.
        assert_eq!(
            value_masked, value,
            "Pointer is not 4-bits aligned or over 51-bits."
        );
        // Cannot have a null pointer for bigint.
        assert_ne!(value_masked, 0, "Pointer is NULL.");

        // Simply cast for bits.
        POINTER_BIGINT_START | value_masked
    }

    /// Returns a tagged u64 of a boxed `[JsObject]`.
    ///
    /// # Safety
    /// The pointer must be 4-bits aligned and cannot exceed 51-bits. This will
    /// result in a panic. Also, the object is not checked for validity.
    ///
    /// The box is forgotten after this operation. It must be dropped separately,
    /// by calling `[Self::drop_pointer]`.
    #[inline(always)]
    pub(super) unsafe fn tag_object(value: Box<JsObject>) -> u64 {
        let value = Box::into_raw(value) as u64;
        let value_masked: u64 = value & POINTER_MASK;

        // Assert alignment and location of the pointer.
        assert_eq!(
            value_masked, value,
            "Pointer is not 4-bits aligned or over 51-bits."
        );
        // Cannot have a null pointer for bigint.
        assert_ne!(value_masked, 0, "Pointer is NULL.");

        // Simply cast for bits.
        POINTER_OBJECT_START | value_masked
    }

    /// Returns a tagged u64 of a boxed `[JsSymbol]`.
    ///
    /// # Safety
    /// The pointer must be 4-bits aligned and cannot exceed 51-bits. This will
    /// result in a panic. Also, the object is not checked for validity.
    ///
    /// The box is forgotten after this operation. It must be dropped separately,
    /// by calling `[Self::drop_pointer]`.
    #[inline(always)]
    pub(super) unsafe fn tag_symbol(value: Box<JsSymbol>) -> u64 {
        let value = Box::into_raw(value) as u64;
        let value_masked: u64 = value & POINTER_MASK;

        // Assert alignment and location of the pointer.
        assert_eq!(
            value_masked, value,
            "Pointer is not 4-bits aligned or over 51-bits."
        );
        // Cannot have a null pointer for bigint.
        assert_ne!(value_masked, 0, "Pointer is NULL.");

        // Simply cast for bits.
        POINTER_SYMBOL_START | value_masked
    }

    /// Returns a tagged u64 of a boxed `[JsString]`.
    ///
    /// # Safety
    /// The pointer must be 4-bits aligned and cannot exceed 51-bits. This will
    /// result in a panic. Also, the object is not checked for validity.
    ///
    /// The box is forgotten after this operation. It must be dropped separately,
    /// by calling `[Self::drop_pointer]`.
    #[inline(always)]
    pub(super) unsafe fn tag_string(value: Box<JsString>) -> u64 {
        let value = Box::into_raw(value) as u64;
        let value_masked: u64 = value & POINTER_MASK;

        // Assert alignment and location of the pointer.
        assert_eq!(
            value_masked, value,
            "Pointer is not 4-bits aligned or over 51-bits."
        );
        // Cannot have a null pointer for bigint.
        assert_ne!(value_masked, 0, "Pointer is NULL.");

        // Simply cast for bits.
        POINTER_STRING_START | value_masked
    }

    /// Returns a reference to T from a tagged value.
    ///
    /// # Safety
    /// The pointer must be a valid pointer to a T on the heap, otherwise this
    /// will result in undefined behavior.
    #[inline(always)]
    pub(super) const unsafe fn untag_pointer<'a, T>(value: u64) -> &'a T {
        // This is safe since we already checked the pointer is not null as this point.
        unsafe { NonNull::new_unchecked((value & POINTER_MASK) as *mut T).as_ref() }
    }
}

// Verify that all representations of NanBitTag ARE NAN, but don't match static NAN.
// The only exception to this rule is BigInt, which assumes that the pointer is
// non-null. The static f64::NAN is equal to BigInt.
const_assert!(f64::from_bits(bits::UNDEFINED).is_nan());
const_assert!(f64::from_bits(bits::NULL).is_nan());
const_assert!(f64::from_bits(bits::FALSE).is_nan());
const_assert!(f64::from_bits(bits::TRUE).is_nan());
const_assert!(f64::from_bits(bits::INTEGER32_ZERO).is_nan());
const_assert!(f64::from_bits(bits::POINTER_BIGINT_START).is_nan());
const_assert!(f64::from_bits(bits::POINTER_BIGINT_END).is_nan());
const_assert!(f64::from_bits(bits::POINTER_OBJECT_START).is_nan());
const_assert!(f64::from_bits(bits::POINTER_OBJECT_END).is_nan());
const_assert!(f64::from_bits(bits::POINTER_SYMBOL_START).is_nan());
const_assert!(f64::from_bits(bits::POINTER_SYMBOL_END).is_nan());
const_assert!(f64::from_bits(bits::POINTER_STRING_START).is_nan());
const_assert!(f64::from_bits(bits::POINTER_STRING_END).is_nan());

/// A NaN-boxed `[JsValue]`'s inner.
pub(crate) struct NanBoxedValue(pub u64);

impl fmt::Debug for NanBoxedValue {
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

impl Finalize for NanBoxedValue {
    fn finalize(&self) {
        if let Some(o) = self.as_object() {
            o.finalize();
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl Trace for NanBoxedValue {
    custom_trace! {this, mark, {
        if let Some(o) = this.as_object() {
            mark(o);
        }
    }}
}

impl Clone for NanBoxedValue {
    #[inline(always)]
    fn clone(&self) -> Self {
        if let Some(o) = self.as_object() {
            Self::object(o.clone())
        } else if let Some(b) = self.as_bigint() {
            Self::bigint(b.clone())
        } else if let Some(s) = self.as_symbol() {
            Self::symbol(s.clone())
        } else if let Some(s) = self.as_string() {
            Self::string(s.clone())
        } else {
            Self(self.0)
        }
    }
}

impl NanBoxedValue {
    /// Creates a new `InnerValue` from an u64 value without checking the validity
    /// of the value.
    #[must_use]
    #[inline(always)]
    const fn from_inner_unchecked(inner: u64) -> Self {
        Self(inner)
    }

    /// Returns a `InnerValue` from a Null.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn null() -> Self {
        Self::from_inner_unchecked(bits::NULL)
    }

    /// Returns a `InnerValue` from an undefined.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn undefined() -> Self {
        Self::from_inner_unchecked(bits::UNDEFINED)
    }

    /// Returns a `InnerValue` from a 64-bits float. If the float is `NaN`,
    /// it will be reduced to a canonical `NaN` representation.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn float64(value: f64) -> Self {
        Self::from_inner_unchecked(bits::tag_f64(value))
    }

    /// Returns a `InnerValue` from a 32-bits integer.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn integer32(value: i32) -> Self {
        Self::from_inner_unchecked(bits::tag_i32(value))
    }

    /// Returns a `InnerValue` from a boolean.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn boolean(value: bool) -> Self {
        Self::from_inner_unchecked(bits::tag_bool(value))
    }

    /// Returns a `InnerValue` from a boxed `[JsBigInt]`.
    #[must_use]
    #[inline(always)]
    pub(crate) fn bigint(value: JsBigInt) -> Self {
        Self::from_inner_unchecked(unsafe { bits::tag_bigint(Box::new(value)) })
    }

    /// Returns a `InnerValue` from a boxed `[JsObject]`.
    #[must_use]
    #[inline(always)]
    pub(crate) fn object(value: JsObject) -> Self {
        Self::from_inner_unchecked(unsafe { bits::tag_object(Box::new(value)) })
    }

    /// Returns a `InnerValue` from a boxed `[JsSymbol]`.
    #[must_use]
    #[inline(always)]
    pub(crate) fn symbol(value: JsSymbol) -> Self {
        Self::from_inner_unchecked(unsafe { bits::tag_symbol(Box::new(value)) })
    }

    /// Returns a `InnerValue` from a boxed `[JsString]`.
    #[must_use]
    #[inline(always)]
    pub(crate) fn string(value: JsString) -> Self {
        Self::from_inner_unchecked(unsafe { bits::tag_string(Box::new(value)) })
    }

    /// Returns true if a value is undefined.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_undefined(&self) -> bool {
        bits::is_undefined(self.0)
    }

    /// Returns true if a value is null.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_null(&self) -> bool {
        bits::is_null(self.0)
    }

    /// Returns true if a value is a boolean.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_bool(&self) -> bool {
        bits::is_bool(self.0)
    }

    /// Returns true if a value is a 64-bits float.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_float64(&self) -> bool {
        bits::is_float(self.0)
    }

    /// Returns true if a value is a 32-bits integer.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_integer32(&self) -> bool {
        bits::is_integer32(self.0)
    }

    /// Returns true if a value is a pointer type.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_pointer(&self) -> bool {
        bits::is_pointer(self.0)
    }

    /// Returns true if a value is a `[JsBigInt]`. A `NaN` will not match here.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_bigint(&self) -> bool {
        bits::is_bigint(self.0)
    }

    /// Returns true if a value is a boxed Object.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_object(&self) -> bool {
        bits::is_object(self.0)
    }

    /// Returns true if a value is a boxed Symbol.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_symbol(&self) -> bool {
        bits::is_symbol(self.0)
    }

    /// Returns true if a value is a boxed String.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn is_string(&self) -> bool {
        bits::is_string(self.0)
    }

    /// Returns the value as a f64 if it is a float.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn as_float64(&self) -> Option<f64> {
        if self.is_float64() {
            Some(f64::from_bits(self.0))
        } else {
            None
        }
    }

    /// Returns the value as an i32 if it is an integer.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn as_integer32(&self) -> Option<i32> {
        if self.is_integer32() {
            Some(bits::untag_i32(self.0))
        } else {
            None
        }
    }

    /// Returns the value as a boolean if it is a boolean.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn as_bool(&self) -> Option<bool> {
        match self.0 {
            bits::FALSE => Some(false),
            bits::TRUE => Some(true),
            _ => None,
        }
    }

    /// Returns the value as a boxed `[JsBigInt]`.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn as_bigint(&self) -> Option<&JsBigInt> {
        if self.is_bigint() {
            Some(unsafe { bits::untag_pointer::<'_, JsBigInt>(self.0) })
        } else {
            None
        }
    }

    /// Returns the value as a boxed `[JsObject]`.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn as_object(&self) -> Option<&JsObject> {
        if self.is_object() {
            Some(unsafe { bits::untag_pointer::<'_, JsObject>(self.0) })
        } else {
            None
        }
    }

    /// Returns the value as a boxed `[JsSymbol]`.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn as_symbol(&self) -> Option<&JsSymbol> {
        if self.is_symbol() {
            Some(unsafe { bits::untag_pointer::<'_, JsSymbol>(self.0) })
        } else {
            None
        }
    }

    /// Returns the value as a boxed `[JsString]`.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn as_string(&self) -> Option<&JsString> {
        if self.is_string() {
            Some(unsafe { bits::untag_pointer::<'_, JsString>(self.0) })
        } else {
            None
        }
    }

    /// Returns the `[JsVariant]` of this inner value.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn as_variant(&self) -> JsVariant<'_> {
        #[allow(clippy::match_overlapping_arm)]
        match self.0 {
            bits::UNDEFINED => JsVariant::Undefined,
            bits::NULL => JsVariant::Null,
            bits::FALSE => JsVariant::Boolean(false),
            bits::TRUE => JsVariant::Boolean(true),
            bits::INTEGER32_ZERO..=bits::INTEGER32_MAX => {
                JsVariant::Integer32(bits::untag_i32(self.0))
            }
            bits::NAN => JsVariant::Float64(f64::NAN),
            bits::POINTER_BIGINT_START..=bits::POINTER_BIGINT_END => {
                JsVariant::BigInt(unsafe { bits::untag_pointer(self.0) })
            }
            bits::POINTER_OBJECT_START..=bits::POINTER_OBJECT_END => {
                JsVariant::Object(unsafe { bits::untag_pointer(self.0) })
            }
            bits::POINTER_SYMBOL_START..=bits::POINTER_SYMBOL_END => {
                JsVariant::Symbol(unsafe { bits::untag_pointer(self.0) })
            }
            bits::POINTER_STRING_START..=bits::POINTER_STRING_END => {
                JsVariant::String(unsafe { bits::untag_pointer(self.0) })
            }
            _ => JsVariant::Float64(f64::from_bits(self.0)),
        }
    }
}

impl Drop for NanBoxedValue {
    fn drop(&mut self) {
        let maybe_ptr = self.0 & bits::POINTER_MASK;

        // Drop the pointer if it is a pointer.
        if self.is_pointer() {
            if self.is_string() {
                drop(unsafe { Box::from_raw(maybe_ptr as *mut JsString) });
            } else if self.is_object() {
                drop(unsafe { Box::from_raw(maybe_ptr as *mut JsObject) });
            } else if self.is_bigint() {
                drop(unsafe { Box::from_raw(maybe_ptr as *mut JsBigInt) });
            } else if self.is_symbol() {
                drop(unsafe { Box::from_raw(maybe_ptr as *mut JsSymbol) });
            }
        }
    }
}

#[cfg(test)]
macro_rules! assert_type {
    (@@is $value: ident, $u: literal, $n: literal, $b: literal, $i: literal, $f: literal, $bi: literal, $s: literal, $o: literal, $sy: literal) => {
        assert_eq!($u  != 0, $value.is_undefined());
        assert_eq!($n  != 0, $value.is_null());
        assert_eq!($b  != 0, $value.is_bool());
        assert_eq!($i  != 0, $value.is_integer32());
        assert_eq!($f  != 0, $value.is_float64());
        assert_eq!($bi != 0, $value.is_bigint());
        assert_eq!($s  != 0, $value.is_string());
        assert_eq!($o  != 0, $value.is_object());
        assert_eq!($sy != 0, $value.is_symbol());
    };
    (@@as $value: ident, $u: literal, $n: literal, $b: literal, $i: literal, $f: literal, $bi: literal, $s: literal, $o: literal, $sy: literal) => {
        if $b  == 0 { assert_eq!($value.as_bool(), None); }
        if $i  == 0 { assert_eq!($value.as_integer32(), None); }
        if $f  == 0 { assert_eq!($value.as_float64(), None); }
        if $bi == 0 { assert_eq!($value.as_bigint(), None); }
        if $s  == 0 { assert_eq!($value.as_string(), None); }
        if $o  == 0 { assert_eq!($value.as_object(), None); }
        if $sy == 0 { assert_eq!($value.as_symbol(), None); }
    };
    ($value: ident is undefined) => {
        assert_type!(@@is $value, 1, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!($value.as_variant(), JsVariant::Undefined);
    };
    ($value: ident is null) => {
        assert_type!(@@is $value, 0, 1, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!($value.as_variant(), JsVariant::Null);
    };
    ($value: ident is bool($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 1, 0, 0, 0, 0, 0, 0);
        assert_type!(@@as $value, 0, 0, 1, 0, 0, 0, 0, 0, 0);
        assert_eq!(Some($scalar), $value.as_bool());
        assert_eq!($value.as_variant(), JsVariant::Boolean($scalar));
    };
    ($value: ident is integer($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 1, 0, 0, 0, 0, 0);
        assert_type!(@@as $value, 0, 0, 0, 1, 0, 0, 0, 0, 0);
        assert_eq!(Some($scalar), $value.as_integer32());
        assert_eq!($value.as_variant(), JsVariant::Integer32($scalar));
    };
    ($value: ident is float($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 1, 0, 0, 0, 0);
        assert_type!(@@as $value, 0, 0, 0, 0, 1, 0, 0, 0, 0);
        assert_eq!(Some($scalar), $value.as_float64());
        // Verify parity.
        assert_eq!(Some(1.0 / $scalar), $value.as_float64().map(|f| 1.0 / f));
        assert_eq!($value.as_variant(), JsVariant::Float64($scalar));

        // Verify that the clone is still the same.
        let new_value = $value.clone();

        assert_eq!(Some($scalar), new_value.as_float64());
        assert_eq!($value.as_float64(), new_value.as_float64());
        // Verify parity.
        assert_eq!(Some(1.0 / $scalar), new_value.as_float64().map(|f| 1.0 / f));
        assert_eq!(new_value.as_variant(), JsVariant::Float64($scalar));

        let JsVariant::Float64(new_scalar) = new_value.as_variant() else {
            panic!("Expected Float64, got {:?}", new_value.as_variant());
        };
        assert_eq!(Some(new_scalar), new_value.as_float64());
        assert_eq!($value.as_float64(), new_value.as_float64());
        // Verify parity.
        assert_eq!(Some(1.0 / new_scalar), new_value.as_float64().map(|f| 1.0 / f));
        assert_eq!(new_value.as_variant(), JsVariant::Float64(new_scalar));
    };
    ($value: ident is nan) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 1, 0, 0, 0, 0);
        assert_type!(@@as $value, 0, 0, 0, 0, 1, 0, 0, 0, 0);
        assert!($value.as_float64().unwrap().is_nan());
        assert!(matches!($value.as_variant(), JsVariant::Float64(f) if f.is_nan()));
    };
    ($value: ident is bigint($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 0, 1, 0, 0, 0);
        assert_type!(@@as $value, 0, 0, 0, 0, 0, 1, 0, 0, 0);
        assert_eq!(Some(&$scalar), $value.as_bigint());
        assert_eq!($value.as_variant(), JsVariant::BigInt(&$scalar));
    };
    ($value: ident is object($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 0, 0, 0, 1, 0);
        assert_type!(@@as $value, 0, 0, 0, 0, 0, 0, 0, 1, 0);
        assert_eq!(Some(&$scalar), $value.as_object());
        assert_eq!($value.as_variant(), JsVariant::Object(&$scalar));
    };
    ($value: ident is symbol($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 0, 0, 0, 0, 1);
        assert_type!(@@as $value, 0, 0, 0, 0, 0, 0, 0, 0, 1);
        assert_eq!(Some(&$scalar), $value.as_symbol());
        assert_eq!($value.as_variant(), JsVariant::Symbol(&$scalar));
    };
    ($value: ident is string($scalar: ident)) => {
        assert_type!(@@is $value, 0, 0, 0, 0, 0, 0, 1, 0, 0);
        assert_type!(@@as $value, 0, 0, 0, 0, 0, 0, 1, 0, 0);
        assert_eq!(Some(&$scalar), $value.as_string());
        assert_eq!($value.as_variant(), JsVariant::String(&$scalar));
    };
}

#[test]
fn null() {
    let v = NanBoxedValue::null();
    assert_type!(v is null);
}

#[test]
fn undefined() {
    let v = NanBoxedValue::undefined();
    assert_type!(v is undefined);
}

#[test]
fn boolean() {
    let v = NanBoxedValue::boolean(true);
    assert_type!(v is bool(true));

    let v = NanBoxedValue::boolean(false);
    assert_type!(v is bool(false));
}

#[test]
fn integer() {
    fn assert_integer(i: i32) {
        let v = NanBoxedValue::integer32(i);
        assert_type!(v is integer(i));
    }

    assert_integer(0);
    assert_integer(1);
    assert_integer(-1);
    assert_integer(42);
    assert_integer(-42);
    assert_integer(i32::MAX);
    assert_integer(i32::MIN);
    assert_integer(i32::MAX - 1);
    assert_integer(i32::MIN + 1);
}

#[test]
#[allow(clippy::float_cmp)]
fn float() {
    fn assert_float(f: f64) {
        let v = NanBoxedValue::float64(f);
        assert_type!(v is float(f));
    }

    assert_float(0.0);
    assert_float(-0.0);
    assert_float(0.1 + 0.2);
    assert_float(-42.123);
    assert_float(f64::INFINITY);
    assert_float(f64::NEG_INFINITY);

    // Some edge cases around zeroes.
    let neg_zero = NanBoxedValue::float64(-0.0);
    assert!(neg_zero.as_float64().unwrap().is_sign_negative());
    assert_eq!(0.0f64, neg_zero.as_float64().unwrap());

    let pos_zero = NanBoxedValue::float64(0.0);
    assert!(!pos_zero.as_float64().unwrap().is_sign_negative());
    assert_eq!(0.0f64, pos_zero.as_float64().unwrap());

    assert_eq!(pos_zero.as_float64(), neg_zero.as_float64());

    let nan = NanBoxedValue::float64(f64::NAN);
    assert_type!(nan is nan);
}

#[test]
fn bigint() {
    let bigint = JsBigInt::from(42);
    let v = NanBoxedValue::bigint(bigint.clone());
    assert_type!(v is bigint(bigint));
}

#[test]
fn object() {
    let object = JsObject::with_null_proto();
    let v = NanBoxedValue::object(object.clone());
    assert_type!(v is object(object));
}

#[test]
fn string() {
    let str = crate::js_string!("Hello World");
    let v = NanBoxedValue::string(str.clone());
    assert_type!(v is string(str));
}

#[test]
fn symbol() {
    let sym = JsSymbol::new(Some(JsString::from("Hello World"))).unwrap();
    let v = NanBoxedValue::symbol(sym.clone());
    assert_type!(v is symbol(sym));

    let sym = JsSymbol::new(None).unwrap();
    let v = NanBoxedValue::symbol(sym.clone());
    assert_type!(v is symbol(sym));
}
