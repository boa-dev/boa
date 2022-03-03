#![warn(unsafe_op_in_unsafe_fn)]
//! This module implements the JavaScript Value.
//!
//! Javascript values, utility methods and conversion between Javascript values and Rust values.

#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        number::{f64_to_int32, f64_to_uint32},
        Number,
    },
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyKey},
    symbol::{JsSymbol, WellKnownSymbols},
    Context, JsBigInt, JsResult, JsString,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::Zero;
use once_cell::sync::Lazy;
use std::{
    cell::Cell,
    collections::HashSet,
    fmt::{self, Display},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::Sub,
    str::FromStr,
};

mod conversions;
pub(crate) mod display;
mod equality;
mod hash;
mod integer;
mod operations;
mod serde_json;
mod r#type;

pub use conversions::*;
pub use display::ValueDisplay;
pub use equality::*;
pub use hash::*;
pub use integer::IntegerOrInfinity;
pub use operations::*;
pub use r#type::Type;

static TWO_E_64: Lazy<BigInt> = Lazy::new(|| {
    const TWO_E_64: u128 = 2u128.pow(64);
    BigInt::from(TWO_E_64)
});

static TWO_E_63: Lazy<BigInt> = Lazy::new(|| {
    const TWO_E_63: u128 = 2u128.pow(63);
    BigInt::from(TWO_E_63)
});

const SIGN_BIT: u64 = 0x8000000000000000;
const EXPONENT: u64 = 0x7FF0000000000000;
// const MANTISA: u64 = 0x0008000000000000;
const SIGNAL_BIT: u64 = 0x0008000000000000;
const QNAN: u64 = EXPONENT | SIGNAL_BIT; // 0x7FF8000000000000

pub const CANONICALIZED_NAN: u64 = QNAN;
// const PAYLOAD: u64 = 0x00007FFFFFFFFFFF;
// const TYPE: u64 = !PAYLOAD;

pub const TAG_MASK: u64 = 0xFFFF000000000000;

pub const DOUBLE_TYPE: u64 = QNAN;
pub const INTEGER_TYPE: u64 = QNAN | (0b001 << 48);
pub const BOOLEAN_TYPE: u64 = QNAN | (0b010 << 48);
pub const UNDEFINED_TYPE: u64 = QNAN | (0b011 << 48);
pub const NULL_TYPE: u64 = QNAN | (0b100 << 48);

pub const RESERVED1_TYPE: u64 = QNAN | (0b101 << 48);
pub const RESERVED2_TYPE: u64 = QNAN | (0b110 << 48);
pub const RESERVED3_TYPE: u64 = QNAN | (0b111 << 48);

pub const POINTER_TYPE: u64 = SIGN_BIT | QNAN;
pub const OBJECT_TYPE: u64 = POINTER_TYPE | (0b001 << 48);
pub const STRING_TYPE: u64 = POINTER_TYPE | (0b010 << 48);
pub const SYMBOL_TYPE: u64 = POINTER_TYPE | (0b011 << 48);
pub const BIGINT_TYPE: u64 = POINTER_TYPE | (0b100 << 48);

pub const RESERVED4_TYPE: u64 = POINTER_TYPE | (0b101 << 48);
pub const RESERVED5_TYPE: u64 = POINTER_TYPE | (0b110 << 48);
pub const RESERVED6_TYPE: u64 = POINTER_TYPE | (0b111 << 48);

pub const MASK_INT_PAYLOAD: u64 = 0x00000000FFFFFFFF;
pub const MASK_POINTER_PAYLOAD: u64 = 0x0000FFFFFFFFFFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ValueTag {
    Double = (DOUBLE_TYPE >> 48) as _,
    Integer = (INTEGER_TYPE >> 48) as _,
    Boolean = (BOOLEAN_TYPE >> 48) as _,
    Undefined = (UNDEFINED_TYPE >> 48) as _,
    Null = (NULL_TYPE >> 48) as _,
    Object = (OBJECT_TYPE >> 48) as _,
    String = (STRING_TYPE >> 48) as _,
    Symbol = (SYMBOL_TYPE >> 48) as _,
    BigInt = (BIGINT_TYPE >> 48) as _,
    // Reserved1 = RESERVED1_TYPE,
    // Reserved2 = RESERVED2_TYPE,
    // Reserved3 = RESERVED3_TYPE,
    // Reserved4 = RESERVED4_TYPE,
    // Reserved5 = RESERVED5_TYPE,
    // Reserved6 = RESERVED6_TYPE,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct JsValue {
    value: Cell<u64>,
}

// TODO: Remove this !!!!
unsafe impl Sync for JsValue {}

impl JsValue {
    /// Create a new [`JsValue`].
    #[inline]
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Self>,
    {
        value.into()
    }

    fn tag(&self) -> ValueTag {
        if self.is_rational() {
            return ValueTag::Double;
        }
        unsafe { std::mem::transmute(((self.value.get() & TAG_MASK) >> 48) as u16) }
    }

    /// Creates a new number with `NaN` value.
    #[inline]
    pub const fn nan() -> Self {
        Self {
            value: Cell::new(CANONICALIZED_NAN),
        }
    }

    pub fn is_nan(&self) -> bool {
        self.value.get() == CANONICALIZED_NAN
    }

    /// Creates a new `undefined` value.
    #[inline]
    pub const fn undefined() -> Self {
        Self {
            value: Cell::new(UNDEFINED_TYPE),
        }
    }

    /// Returns true if the value is undefined.
    #[inline]
    pub fn is_undefined(&self) -> bool {
        self.value.get() == UNDEFINED_TYPE
    }

    /// Creates a new `null` value.
    #[inline]
    pub const fn null() -> Self {
        Self {
            value: Cell::new(NULL_TYPE),
        }
    }

    /// Returns true if the value is null.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.value.get() == NULL_TYPE
    }

    /// Returns true if the value is null or undefined.
    #[inline]
    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.is_undefined()
    }

    pub fn is_rational(&self) -> bool {
        (self.value.get() & !SIGN_BIT) <= QNAN
    }

    fn as_rational_unchecked(&self) -> f64 {
        f64::from_bits(self.value.get())
    }

    pub fn as_rational(&self) -> Option<f64> {
        if self.is_rational() {
            return Some(self.as_rational_unchecked());
        }

        None
    }

    pub fn is_i32(&self) -> bool {
        self.value.get() & TAG_MASK == INTEGER_TYPE
    }

    pub fn as_i32_uncheched(&self) -> i32 {
        (self.value.get() & MASK_INT_PAYLOAD) as u32 as i32
    }

    pub fn as_i32(&self) -> Option<i32> {
        if self.is_i32() {
            return Some(self.as_i32_uncheched());
        }

        None
    }

    /// Returns true if the value is a boolean.
    #[inline]
    pub fn is_boolean(&self) -> bool {
        self.value.get() & TAG_MASK == BOOLEAN_TYPE
    }

    pub fn as_boolean_uncheched(&self) -> bool {
        (self.value.get() & 0xFF) != 0
    }

    #[inline]
    pub fn as_boolean(&self) -> Option<bool> {
        if self.is_boolean() {
            return Some(self.as_boolean_uncheched());
        }

        None
    }

    pub fn as_pointer(&self) -> *mut () {
        (self.value.get() & MASK_POINTER_PAYLOAD) as *mut ()
    }

    /// Returns true if the value is an object
    #[inline]
    pub fn is_object(&self) -> bool {
        self.value.get() & TAG_MASK == OBJECT_TYPE
    }

    /// Returns a reference to the boxed [`JsObject`] without checking
    /// if the tag of `self` is valid.
    ///
    /// # Safety
    ///
    /// Calling this method with a [`JsValue`] that doesn't box
    /// a [`JsObject`] is undefined behaviour.
    pub unsafe fn as_object_unchecked(&self) -> Ref<'_, JsObject> {
        unsafe { Ref::new(JsObject::from_void_ptr(self.as_pointer())) }
    }

    pub fn as_object(&self) -> Option<Ref<'_, JsObject>> {
        if self.is_object() {
            return unsafe { Some(self.as_object_unchecked()) };
        }

        None
    }

    /// Returns true if the value is a string.
    #[inline]
    pub fn is_string(&self) -> bool {
        self.value.get() & TAG_MASK == STRING_TYPE
    }

    /// Returns a reference to the boxed [`JsString`] without checking
    /// if the tag of `self` is valid.
    ///
    /// # Safety
    ///
    /// Calling this method with a [`JsValue`] that doesn't box
    /// a [`JsString`] is undefined behaviour.
    pub unsafe fn as_string_unchecked(&self) -> Ref<'_, JsString> {
        unsafe { Ref::new(JsString::from_void_ptr(self.as_pointer())) }
    }

    /// Returns the string if the values is a string, otherwise `None`.
    #[inline]
    pub fn as_string(&self) -> Option<Ref<'_, JsString>> {
        if self.is_string() {
            return unsafe { Some(self.as_string_unchecked()) };
        }

        None
    }

    pub fn is_symbol(&self) -> bool {
        self.value.get() & TAG_MASK == SYMBOL_TYPE
    }

    /// Returns a reference to the boxed [`JsSymbol`] without checking
    /// if the tag of `self` is valid.
    ///
    /// # Safety
    ///
    /// Calling this method with a [`JsValue`] that doesn't box
    /// a [`JsSymbol`] is undefined behaviour.
    pub unsafe fn as_symbol_unchecked(&self) -> Ref<'_, JsSymbol> {
        unsafe { Ref::new(JsSymbol::from_void_ptr(self.as_pointer())) }
    }

    pub fn as_symbol(&self) -> Option<Ref<'_, JsSymbol>> {
        if self.is_symbol() {
            return unsafe { Some(self.as_symbol_unchecked()) };
        }

        None
    }

    /// Returns true if the value is a bigint.
    #[inline]
    pub fn is_bigint(&self) -> bool {
        self.value.get() & TAG_MASK == BIGINT_TYPE
    }

    /// Returns a reference to the boxed [`JsBigInt`] without checking
    /// if the tag of `self` is valid.
    ///
    /// # Safety
    ///
    /// Calling this method with a [`JsValue`] that doesn't box
    /// a [`JsBigInt`] is undefined behaviour.
    #[inline]
    pub unsafe fn as_bigint_unchecked(&self) -> Ref<'_, JsBigInt> {
        // SAFETY: The safety contract must be upheld by the caller
        unsafe { Ref::new(JsBigInt::from_void_ptr(self.as_pointer())) }
    }

    pub fn as_bigint(&self) -> Option<Ref<'_, JsBigInt>> {
        if self.is_bigint() {
            return unsafe { Some(self.as_bigint_unchecked()) };
        }

        None
    }

    pub fn variant(&self) -> JsVariant<'_> {
        unsafe {
            match self.tag() {
                ValueTag::Null => JsVariant::Null,
                ValueTag::Undefined => JsVariant::Undefined,
                ValueTag::Integer => JsVariant::Integer(self.as_i32_uncheched()),
                ValueTag::Double => JsVariant::Rational(self.as_rational_unchecked()),
                ValueTag::Boolean => JsVariant::Boolean(self.as_boolean_uncheched()),
                ValueTag::Object => JsVariant::Object(self.as_object_unchecked()),
                ValueTag::String => JsVariant::String(self.as_string_unchecked()),
                ValueTag::Symbol => JsVariant::Symbol(self.as_symbol_unchecked()),
                ValueTag::BigInt => JsVariant::BigInt(self.as_bigint_unchecked()),
            }
        }
    }
}

#[derive(Debug)]
pub enum JsVariant<'a> {
    Null,
    Undefined,
    Rational(f64),
    Integer(i32),
    Boolean(bool),
    String(Ref<'a, JsString>),
    Symbol(Ref<'a, JsSymbol>),
    BigInt(Ref<'a, JsBigInt>),
    Object(Ref<'a, JsObject>),
}

impl From<bool> for JsValue {
    #[inline]
    fn from(value: bool) -> Self {
        let value = Self {
            value: Cell::new(BOOLEAN_TYPE | u64::from(value)),
        };
        debug_assert!(value.is_boolean());
        debug_assert_eq!(value.tag(), ValueTag::Boolean);
        value
    }
}

impl From<i32> for JsValue {
    #[inline]
    fn from(value: i32) -> Self {
        let value = Self {
            value: Cell::new(INTEGER_TYPE | u64::from(value as u32)),
        };
        debug_assert!(value.is_integer());
        debug_assert_eq!(value.tag(), ValueTag::Integer);
        value
    }
}

impl From<u32> for JsValue {
    #[inline]
    fn from(value: u32) -> Self {
        if let Ok(integer) = i32::try_from(value) {
            Self::new(integer)
        } else {
            Self::new(f64::from(value))
        }
    }
}

impl From<usize> for JsValue {
    #[inline]
    fn from(value: usize) -> Self {
        if let Ok(value) = i32::try_from(value) {
            Self::new(value)
        } else {
            Self::new(value as f64)
        }
    }
}

impl From<u64> for JsValue {
    #[inline]
    fn from(value: u64) -> Self {
        if let Ok(value) = i32::try_from(value) {
            Self::new(value)
        } else {
            Self::new(value as f64)
        }
    }
}

impl From<i64> for JsValue {
    #[inline]
    fn from(value: i64) -> Self {
        if let Ok(value) = i32::try_from(value) {
            Self::new(value)
        } else {
            Self::new(value as f64)
        }
    }
}

impl From<f64> for JsValue {
    #[inline]
    fn from(value: f64) -> Self {
        if value.is_nan() {
            return Self {
                value: Cell::new(CANONICALIZED_NAN),
            };
        }

        let value = Self {
            value: Cell::new(value.to_bits()),
        };
        debug_assert!(value.is_rational());
        debug_assert_eq!(value.tag(), ValueTag::Double);
        value
    }
}

impl From<()> for JsValue {
    #[inline]
    fn from(_: ()) -> Self {
        let value = Self::null();
        debug_assert!(value.is_null());
        debug_assert_eq!(value.tag(), ValueTag::Null);
        value
    }
}

impl From<&str> for JsValue {
    #[inline]
    fn from(string: &str) -> Self {
        From::<JsString>::from(JsString::new(string))
    }
}

impl From<&String> for JsValue {
    #[inline]
    fn from(string: &String) -> Self {
        From::<JsString>::from(JsString::new(string.as_str()))
    }
}

impl From<String> for JsValue {
    #[inline]
    fn from(string: String) -> Self {
        From::<JsString>::from(JsString::new(string))
    }
}

impl From<Box<str>> for JsValue {
    #[inline]
    fn from(string: Box<str>) -> Self {
        From::<JsString>::from(JsString::new(string))
    }
}

impl From<JsString> for JsValue {
    #[inline]
    fn from(string: JsString) -> Self {
        let string = ManuallyDrop::new(string);
        let pointer = unsafe { JsString::into_void_ptr(string) } as u64;
        debug_assert_eq!(pointer & MASK_POINTER_PAYLOAD, pointer);
        let value = Self {
            value: Cell::new(STRING_TYPE | pointer),
        };
        debug_assert!(value.is_string());
        debug_assert_eq!(value.tag(), ValueTag::String);
        value
    }
}

impl From<JsObject> for JsValue {
    #[inline]
    fn from(object: JsObject) -> Self {
        let object = ManuallyDrop::new(object);
        let pointer = unsafe { JsObject::into_void_ptr(object) } as u64;
        debug_assert_eq!(pointer & MASK_POINTER_PAYLOAD, pointer);
        debug_assert_eq!(OBJECT_TYPE & MASK_POINTER_PAYLOAD, 0);
        let value = Self {
            value: Cell::new(OBJECT_TYPE | pointer),
        };
        debug_assert!(value.is_object());
        debug_assert_eq!(value.tag(), ValueTag::Object);
        value
    }
}

impl From<JsSymbol> for JsValue {
    #[inline]
    fn from(symbol: JsSymbol) -> Self {
        let symbol = ManuallyDrop::new(symbol);
        let pointer = unsafe { JsSymbol::into_void_ptr(symbol) as u64 };
        debug_assert_eq!(pointer & MASK_POINTER_PAYLOAD, pointer);
        let value = Self {
            value: Cell::new(SYMBOL_TYPE | pointer),
        };
        debug_assert!(value.is_symbol());
        debug_assert_eq!(value.tag(), ValueTag::Symbol);
        value
    }
}

impl From<JsBigInt> for JsValue {
    #[inline]
    fn from(bigint: JsBigInt) -> Self {
        let bigint = ManuallyDrop::new(bigint);
        let pointer = unsafe { JsBigInt::into_void_ptr(bigint) as u64 };
        debug_assert_eq!(pointer & MASK_POINTER_PAYLOAD, pointer);
        let value = Self {
            value: Cell::new(BIGINT_TYPE | pointer),
        };
        debug_assert!(value.is_bigint());
        debug_assert_eq!(value.tag(), ValueTag::BigInt);
        value
    }
}

/// This abstracts over every pointer type boxed inside `NaN` values.
///
/// # Safety
///
/// Non-exhaustive list of situations that could cause undefined behaviour:
/// - Returning an invalid `*mut ()`.
/// - Returning a `ManuallyDrop<Self>` that doesn't correspond with the provided
/// `ptr`.
/// - Dropping `ty` before returning its pointer.
pub(crate) unsafe trait PointerType {
    unsafe fn from_void_ptr(ptr: *mut ()) -> ManuallyDrop<Self>;

    unsafe fn into_void_ptr(ty: ManuallyDrop<Self>) -> *mut ();
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ref<'a, T> {
    inner: ManuallyDrop<T>,
    _marker: PhantomData<&'a T>,
}

impl<T> Ref<'_, T> {
    #[inline]
    fn new(inner: ManuallyDrop<T>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<T> std::borrow::Borrow<T> for Ref<'_, T> {
    fn borrow(&self) -> &T {
        &*self.inner
    }
}

impl<T> AsRef<T> for Ref<'_, T> {
    fn as_ref(&self) -> &T {
        &*self.inner
    }
}

impl<T> std::ops::Deref for Ref<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<T: PartialEq> PartialEq<T> for Ref<'_, T> {
    fn eq(&self, other: &T) -> bool {
        &*self.inner == other
    }
}

impl Drop for JsValue {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            match self.tag() {
                ValueTag::Object => {
                    ManuallyDrop::into_inner(JsObject::from_void_ptr(self.as_pointer()));
                }
                ValueTag::String => {
                    ManuallyDrop::into_inner(JsString::from_void_ptr(self.as_pointer()));
                }
                ValueTag::Symbol => {
                    ManuallyDrop::into_inner(JsSymbol::from_void_ptr(self.as_pointer()));
                }
                ValueTag::BigInt => {
                    ManuallyDrop::into_inner(JsBigInt::from_void_ptr(self.as_pointer()));
                }
                _ => {}
            }
        }
    }
}

impl Clone for JsValue {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            match self.tag() {
                ValueTag::Object => Self::new(self.as_object_unchecked().clone()),
                ValueTag::String => Self::new(self.as_string_unchecked().clone()),
                ValueTag::Symbol => Self::new(self.as_symbol_unchecked().clone()),
                ValueTag::BigInt => Self::new(self.as_bigint_unchecked().clone()),
                _ => Self {
                    value: Cell::new(self.value.get()),
                },
            }
        }
    }
}

impl Finalize for JsValue {}

unsafe impl Trace for JsValue {
    unsafe fn trace(&self) {
        if let Some(o) = self.as_object() {
            // SAFETY: `self.as_object()` must always return a valid `JsObject
            unsafe {
                o.trace();
            }
        }
    }

    unsafe fn root(&self) {
        if self.tag() == ValueTag::Object {
            // SAFETY: Implementors of `PointerType` must guarantee the
            // safety of both `from_void_ptr` and `into_void_ptr`
            unsafe {
                let o = JsObject::from_void_ptr(self.as_pointer());
                o.root();
                self.value
                    .set(OBJECT_TYPE | (JsObject::into_void_ptr(o) as u64));
            }
        }
    }

    unsafe fn unroot(&self) {
        if self.tag() == ValueTag::Object {
            // SAFETY: Implementors of `PointerType` must guarantee the
            // safety of both `from_void_ptr` and `into_void_ptr`
            unsafe {
                let o = JsObject::from_void_ptr(self.as_pointer());
                o.unroot();
                self.value
                    .set(OBJECT_TYPE | (JsObject::into_void_ptr(o) as u64));
            }
        }
    }

    #[inline]
    fn finalize_glue(&self) {
        if let Some(o) = self.as_object() {
            o.finalize_glue();
        }
    }
}
#[cfg(test)]
mod tests_nan_box {
    use super::*;

    #[test]
    fn bigint() {
        let value = JsValue::new(JsBigInt::new(12345));

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_i32());
        assert!(!value.is_rational());
        assert!(!value.is_boolean());
        assert!(!value.is_object());
        assert!(!value.is_string());
        assert!(!value.is_symbol());

        assert!(value.is_bigint());

        let bigint = value.as_bigint().unwrap();

        println!("pass!");

        assert_eq!(&bigint, &JsBigInt::new(12345));
    }

    #[test]
    fn symbol() {
        let value = JsValue::new(JsSymbol::new(Some("description...".into())));

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_i32());
        assert!(!value.is_rational());
        assert!(!value.is_boolean());
        assert!(!value.is_object());
        assert!(!value.is_string());

        assert!(value.is_symbol());

        let symbol = value.as_symbol().unwrap();

        assert_eq!(symbol.description(), Some("description...".into()));
    }

    #[test]
    fn string() {
        let value = JsValue::new("I am a string :)");

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_i32());
        assert!(!value.is_rational());
        assert!(!value.is_boolean());
        assert!(!value.is_object());

        assert!(value.is_string());

        let string = value.as_string().unwrap();

        assert_eq!(JsString::refcount(&string), 1);

        assert_eq!(*string, "I am a string :)");
    }

    #[test]
    fn object() {
        //let context = Context::default();

        let o1 = JsObject::from_proto_and_data(None, ObjectData::ordinary());

        // let value = JsValue::new(context.construct_object());
        let value = JsValue::new(o1.clone());

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_i32());
        assert!(!value.is_rational());
        assert!(!value.is_boolean());

        assert!(value.is_object());

        let o2 = value.as_object().unwrap();
        assert!(JsObject::equals(&o1, &o2));
    }

    #[test]
    fn boolean() {
        let value = JsValue::new(true);

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_i32());
        assert!(!value.is_rational());

        assert!(value.is_boolean());
        assert_eq!(value.as_boolean(), Some(true));

        let value = JsValue::new(false);

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_i32());
        assert!(!value.is_rational());

        assert!(value.is_boolean());
        assert_eq!(value.as_boolean(), Some(false));
    }

    #[test]
    fn rational() {
        let value = JsValue::new(1.3);

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_i32());

        assert!(value.is_rational());
        assert_eq!(value.as_rational(), Some(1.3));

        let value = JsValue::new(f64::MAX);
        assert!(value.is_rational());
        assert_eq!(value.as_rational(), Some(f64::MAX));

        let value = JsValue::new(f64::MIN);
        assert!(value.is_rational());
        assert_eq!(value.as_rational(), Some(f64::MIN));

        let value = JsValue::nan();
        assert!(value.is_rational());
        assert!(value.as_rational().unwrap().is_nan());

        let value = JsValue::new(12345);
        assert!(!value.is_rational());
        assert_eq!(value.as_rational(), None);

        let value = JsValue::undefined();
        assert!(!value.is_rational());
        assert_eq!(value.as_rational(), None);

        let value = JsValue::null();
        assert!(!value.is_rational());
        assert_eq!(value.as_rational(), None);
    }

    #[test]
    fn undefined() {
        let value = JsValue::undefined();

        println!("{:?}", value);
        println!("{:?}", UNDEFINED_TYPE);

        assert!(value.is_undefined());
    }

    #[test]
    fn null() {
        let value = JsValue::null();

        assert!(value.is_null());
        assert!(value.is_null_or_undefined());
        assert!(!value.is_undefined());
    }

    #[test]
    fn integer() {
        let value = JsValue::new(-0xcafe);

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());

        assert!(value.is_i32());

        assert_eq!(value.as_i32(), Some(-0xcafe));

        let value = JsValue::null();
        assert_eq!(value.as_i32(), None);
    }
}

impl JsValue {
    /// Creates a new number with `Infinity` value.
    #[inline]
    pub fn positive_infinity() -> Self {
        Self::new(f64::INFINITY)
    }

    /// Creates a new number with `-Infinity` value.
    #[inline]
    pub fn negative_infinity() -> Self {
        Self::new(f64::NEG_INFINITY)
    }

    /// It determines if the value is a callable function with a `[[Call]]` internal method.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iscallable
    #[inline]
    pub fn is_callable(&self) -> bool {
        self.as_object()
            .as_deref()
            .map_or(false, JsObject::is_callable)
    }

    #[inline]
    pub fn as_callable(&self) -> Option<Ref<'_, JsObject>> {
        self.as_object().filter(|obj| obj.is_callable())
    }

    /// Returns true if the value is a constructor object
    #[inline]
    pub fn is_constructor(&self) -> bool {
        self.as_object()
            .as_deref()
            .map_or(false, JsObject::is_constructor)
    }

    #[inline]
    pub fn as_constructor(&self) -> Option<Ref<'_, JsObject>> {
        self.as_object().filter(|obj| obj.is_constructor())
    }

    /// Returns true if the value is a 64-bit floating-point number.
    #[inline]
    pub fn is_double(&self) -> bool {
        self.is_rational()
    }

    /// Returns true if the value is integer.
    #[inline]
    #[allow(clippy::float_cmp)]
    pub fn is_integer(&self) -> bool {
        // If it can fit in a i32 and the trucated version is
        // equal to the original then it is an integer.
        let is_racional_intiger = |n: f64| n == f64::from(n as i32);

        if self.is_i32() {
            true
        } else if self.is_rational() {
            is_racional_intiger(self.as_rational_unchecked())
        } else {
            false
        }
    }

    /// Returns true if the value is a number.
    #[inline]
    pub fn is_number(&self) -> bool {
        self.is_i32() || self.is_rational()
    }

    #[inline]
    pub fn as_number(&self) -> Option<f64> {
        match self.variant() {
            JsVariant::Integer(integer) => Some(integer.into()),
            JsVariant::Rational(rational) => Some(rational),
            _ => None,
        }
    }

    /// Converts the value to a `bool` type.
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-toboolean
    pub fn to_boolean(&self) -> bool {
        match self.variant() {
            JsVariant::Undefined | JsVariant::Null => false,
            JsVariant::Symbol(_) | JsVariant::Object(_) => true,
            JsVariant::String(s) if !s.is_empty() => true,
            JsVariant::Rational(n) if n != 0.0 && !n.is_nan() => true,
            JsVariant::Integer(n) if n != 0 => true,
            JsVariant::BigInt(n) if !n.is_zero() => true,
            JsVariant::Boolean(v) => v,
            _ => false,
        }
    }

    /// Resolve the property in the object.
    ///
    /// A copy of the Property is returned.
    pub(crate) fn get_property<Key>(&self, key: Key) -> Option<PropertyDescriptor>
    where
        Key: Into<PropertyKey>,
    {
        let key = key.into();
        let _timer = Profiler::global().start_event("Value::get_property", "value");
        if let Some(object) = self.as_object() {
            // TODO: had to skip `__get_own_properties__` since we don't have context here
            let property = object.borrow().properties().get(&key).cloned();
            if property.is_some() {
                return property;
            }

            object
                .prototype()
                .as_ref()
                .map_or(Self::null(), |obj| obj.clone().into())
                .get_property(key)
        } else {
            None
        }
    }

    /// Set the kind of an object.
    #[inline]
    pub fn set_data(&self, data: ObjectData) {
        if let Some(obj) = self.as_object() {
            obj.borrow_mut().data = data;
        }
    }

    /// The abstract operation `ToPrimitive` takes an input argument and an optional argumen`PreferredType`pe.
    ///
    /// <https://tc39.es/ecma262/#sec-toprimitive>
    pub fn to_primitive(
        &self,
        context: &mut Context,
        preferred_type: PreferredType,
    ) -> JsResult<Self> {
        // 1. Assert: input is an ECMAScript language value. (always a value not need to check)
        // 2. If Type(input) is Object, then
        if let Some(object) = self.as_object() {
            // a. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
            let exotic_to_prim = object.get_method(WellKnownSymbols::to_primitive(), context)?;

            // b. If exoticToPrim is not undefined, then
            if let Some(exotic_to_prim) = exotic_to_prim {
                // i. If preferredType is not present, let hint be "default".
                // ii. Else if preferredType is string, let hint be "string".
                // iii. Else,
                //     1. Assert: preferredType is number.
                //     2. Let hint be "number".
                let hint = match preferred_type {
                    PreferredType::Default => "default",
                    PreferredType::String => "string",
                    PreferredType::Number => "number",
                }
                .into();

                // iv. Let result be ? Call(exoticToPrim, input, « hint »).
                let result = exotic_to_prim.call(self, &[hint], context)?;
                // v. If Type(result) is not Object, return result.
                // vi. Throw a TypeError exception.
                return if result.is_object() {
                    context.throw_type_error("Symbol.toPrimitive cannot return an object")
                } else {
                    Ok(result)
                };
            }

            // c. If preferredType is not present, let preferredType be number.
            let preferred_type = match preferred_type {
                PreferredType::Default | PreferredType::Number => PreferredType::Number,
                PreferredType::String => PreferredType::String,
            };

            // d. Return ? OrdinaryToPrimitive(input, preferredType).
            object.ordinary_to_primitive(context, preferred_type)
        } else {
            // 3. Return input.
            Ok(self.clone())
        }
    }

    /// `7.1.13 ToBigInt ( argument )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-tobigint
    pub fn to_bigint(&self, context: &mut Context) -> JsResult<JsBigInt> {
        match self.variant() {
            JsVariant::Null => context.throw_type_error("cannot convert null to a BigInt"),
            JsVariant::Undefined => {
                context.throw_type_error("cannot convert undefined to a BigInt")
            }
            JsVariant::String(string) => {
                let string = &*string;
                if let Some(value) = JsBigInt::from_string(string) {
                    Ok(value)
                } else {
                    context.throw_syntax_error(format!(
                        "cannot convert string '{string}' to bigint primitive",
                    ))
                }
            }
            JsVariant::Boolean(true) => Ok(JsBigInt::one()),
            JsVariant::Boolean(false) => Ok(JsBigInt::zero()),
            JsVariant::Integer(_) | JsVariant::Rational(_) => {
                context.throw_type_error("cannot convert Number to a BigInt")
            }
            JsVariant::BigInt(b) => Ok(b.clone()),
            JsVariant::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::Number)?;
                primitive.to_bigint(context)
            }
            JsVariant::Symbol(_) => context.throw_type_error("cannot convert Symbol to a BigInt"),
        }
    }

    /// Returns an object that implements `Display`.
    ///
    /// By default the internals are not shown, but they can be toggled
    /// with [`ValueDisplay::internals`] method.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_engine::JsValue;
    ///
    /// let value = JsValue::new(3);
    ///
    /// println!("{}", value.display());
    /// ```
    #[inline]
    pub fn display(&self) -> ValueDisplay<'_> {
        ValueDisplay {
            value: self,
            internals: false,
        }
    }

    /// Converts the value to a string.
    ///
    /// This function is equivalent to `String(value)` in JavaScript.
    pub fn to_string(&self, context: &mut Context) -> JsResult<JsString> {
        match self.variant() {
            JsVariant::Null => Ok("null".into()),
            JsVariant::Undefined => Ok("undefined".into()),
            JsVariant::Boolean(boolean) => Ok(boolean.to_string().into()),
            JsVariant::Rational(rational) => Ok(Number::to_native_string(rational).into()),
            JsVariant::Integer(integer) => Ok(integer.to_string().into()),
            JsVariant::String(string) => Ok(string.clone()),
            JsVariant::Symbol(_) => context.throw_type_error("can't convert symbol to string"),
            JsVariant::BigInt(bigint) => Ok(bigint.to_string().into()),
            JsVariant::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::String)?;
                primitive.to_string(context)
            }
        }
    }

    /// Converts the value to an Object.
    ///
    /// This function is equivalent to `Object(value)` in JavaScript.
    ///
    /// See: <https://tc39.es/ecma262/#sec-toobject>
    pub fn to_object(&self, context: &mut Context) -> JsResult<JsObject> {
        match self.variant() {
            JsVariant::Undefined | JsVariant::Null => {
                context.throw_type_error("cannot convert 'null' or 'undefined' to object")
            }
            JsVariant::Boolean(boolean) => {
                let prototype = context.standard_objects().boolean_object().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::boolean(boolean),
                ))
            }
            JsVariant::Integer(integer) => {
                let prototype = context.standard_objects().number_object().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::number(f64::from(integer)),
                ))
            }
            JsVariant::Rational(rational) => {
                let prototype = context.standard_objects().number_object().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::number(rational),
                ))
            }
            JsVariant::String(string) => {
                let prototype = context.standard_objects().string_object().prototype();

                let object =
                    JsObject::from_proto_and_data(prototype, ObjectData::string(string.clone()));
                // Make sure the correct length is set on our new string object
                object.insert_property(
                    "length",
                    PropertyDescriptor::builder()
                        .value(string.encode_utf16().count())
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                );
                Ok(object)
            }
            JsVariant::Symbol(symbol) => {
                let prototype = context.standard_objects().symbol_object().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::symbol(symbol.clone()),
                ))
            }
            JsVariant::BigInt(bigint) => {
                let prototype = context.standard_objects().bigint_object().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::big_int(bigint.clone()),
                ))
            }
            JsVariant::Object(jsobject) => Ok(jsobject.clone()),
        }
    }

    /// Converts the value to a `PropertyKey`, that can be used as a key for properties.
    ///
    /// See <https://tc39.es/ecma262/#sec-topropertykey>
    pub fn to_property_key(&self, context: &mut Context) -> JsResult<PropertyKey> {
        Ok(match self.variant() {
            // Fast path:
            JsVariant::String(string) => string.clone().into(),
            JsVariant::Symbol(symbol) => symbol.clone().into(),
            // Slow path:
            _ => {
                let primitive = self.to_primitive(context, PreferredType::String)?;
                match primitive.variant() {
                    JsVariant::String(string) => string.clone().into(),
                    JsVariant::Symbol(symbol) => symbol.clone().into(),
                    _ => primitive.to_string(context)?.into(),
                }
            }
        })
    }

    /// It returns value converted to a numeric value of type `Number` or `BigInt`.
    ///
    /// See: <https://tc39.es/ecma262/#sec-tonumeric>
    pub fn to_numeric(&self, context: &mut Context) -> JsResult<Numeric> {
        let primitive = self.to_primitive(context, PreferredType::Number)?;
        if let Some(bigint) = primitive.as_bigint() {
            return Ok(bigint.clone().into());
        }
        Ok(self.to_number(context)?.into())
    }

    /// Converts a value to an integral 32 bit unsigned integer.
    ///
    /// This function is equivalent to `value | 0` in JavaScript
    ///
    /// See: <https://tc39.es/ecma262/#sec-touint32>
    pub fn to_u32(&self, context: &mut Context) -> JsResult<u32> {
        // This is the fast path, if the value is Integer we can just return it.
        if let Some(number) = self.as_i32() {
            return Ok(number as u32);
        }
        let number = self.to_number(context)?;

        Ok(f64_to_uint32(number))
    }

    /// Converts a value to an integral 32 bit signed integer.
    ///
    /// See: <https://tc39.es/ecma262/#sec-toint32>
    pub fn to_i32(&self, context: &mut Context) -> JsResult<i32> {
        // This is the fast path, if the value is Integer we can just return it.
        if let Some(number) = self.as_i32() {
            return Ok(number);
        }
        let number = self.to_number(context)?;

        Ok(f64_to_int32(number))
    }

    /// `7.1.10 ToInt8 ( argument )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-toint8
    pub fn to_int8(&self, context: &mut Context) -> JsResult<i8> {
        // 1. Let number be ? ToNumber(argument).
        let number = self.to_number(context)?;

        // 2. If number is NaN, +0𝔽, -0𝔽, +∞𝔽, or -∞𝔽, return +0𝔽.
        if number.is_nan() || number.is_zero() || number.is_infinite() {
            return Ok(0);
        }

        // 3. Let int be the mathematical value whose sign is the sign of number and whose magnitude is floor(abs(ℝ(number))).
        let int = number.abs().floor().copysign(number) as i64;

        // 4. Let int8bit be int modulo 2^8.
        let int_8_bit = int % 2i64.pow(8);

        // 5. If int8bit ≥ 2^7, return 𝔽(int8bit - 2^8); otherwise return 𝔽(int8bit).
        if int_8_bit >= 2i64.pow(7) {
            Ok((int_8_bit - 2i64.pow(8)) as i8)
        } else {
            Ok(int_8_bit as i8)
        }
    }

    /// `7.1.11 ToUint8 ( argument )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-touint8
    pub fn to_uint8(&self, context: &mut Context) -> JsResult<u8> {
        // 1. Let number be ? ToNumber(argument).
        let number = self.to_number(context)?;

        // 2. If number is NaN, +0𝔽, -0𝔽, +∞𝔽, or -∞𝔽, return +0𝔽.
        if number.is_nan() || number.is_zero() || number.is_infinite() {
            return Ok(0);
        }

        // 3. Let int be the mathematical value whose sign is the sign of number and whose magnitude is floor(abs(ℝ(number))).
        let int = number.abs().floor().copysign(number) as i64;

        // 4. Let int8bit be int modulo 2^8.
        let int_8_bit = int % 2i64.pow(8);

        // 5. Return 𝔽(int8bit).
        Ok(int_8_bit as u8)
    }

    /// `7.1.12 ToUint8Clamp ( argument )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-touint8clamp
    pub fn to_uint8_clamp(&self, context: &mut Context) -> JsResult<u8> {
        // 1. Let number be ? ToNumber(argument).
        let number = self.to_number(context)?;

        // 2. If number is NaN, return +0𝔽.
        if number.is_nan() {
            return Ok(0);
        }

        // 3. If ℝ(number) ≤ 0, return +0𝔽.
        if number <= 0.0 {
            return Ok(0);
        }

        // 4. If ℝ(number) ≥ 255, return 255𝔽.
        if number >= 255.0 {
            return Ok(255);
        }

        // 5. Let f be floor(ℝ(number)).
        let f = number.floor();

        // 6. If f + 0.5 < ℝ(number), return 𝔽(f + 1).
        if f + 0.5 < number {
            return Ok(f as u8 + 1);
        }

        // 7. If ℝ(number) < f + 0.5, return 𝔽(f).
        if number < f + 0.5 {
            return Ok(f as u8);
        }

        // 8. If f is odd, return 𝔽(f + 1).
        if f as u8 % 2 != 0 {
            return Ok(f as u8 + 1);
        }

        // 9. Return 𝔽(f).
        Ok(f as u8)
    }

    /// `7.1.8 ToInt16 ( argument )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-toint16
    pub fn to_int16(&self, context: &mut Context) -> JsResult<i16> {
        // 1. Let number be ? ToNumber(argument).
        let number = self.to_number(context)?;

        // 2. If number is NaN, +0𝔽, -0𝔽, +∞𝔽, or -∞𝔽, return +0𝔽.
        if number.is_nan() || number.is_zero() || number.is_infinite() {
            return Ok(0);
        }

        // 3. Let int be the mathematical value whose sign is the sign of number and whose magnitude is floor(abs(ℝ(number))).
        let int = number.abs().floor().copysign(number) as i64;

        // 4. Let int16bit be int modulo 2^16.
        let int_16_bit = int % 2i64.pow(16);

        // 5. If int16bit ≥ 2^15, return 𝔽(int16bit - 2^16); otherwise return 𝔽(int16bit).
        if int_16_bit >= 2i64.pow(15) {
            Ok((int_16_bit - 2i64.pow(16)) as i16)
        } else {
            Ok(int_16_bit as i16)
        }
    }

    /// `7.1.9 ToUint16 ( argument )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-touint16
    pub fn to_uint16(&self, context: &mut Context) -> JsResult<u16> {
        // 1. Let number be ? ToNumber(argument).
        let number = self.to_number(context)?;

        // 2. If number is NaN, +0𝔽, -0𝔽, +∞𝔽, or -∞𝔽, return +0𝔽.
        if number.is_nan() || number.is_zero() || number.is_infinite() {
            return Ok(0);
        }

        // 3. Let int be the mathematical value whose sign is the sign of number and whose magnitude is floor(abs(ℝ(number))).
        let int = number.abs().floor().copysign(number) as i64;

        // 4. Let int16bit be int modulo 2^16.
        let int_16_bit = int % 2i64.pow(16);

        // 5. Return 𝔽(int16bit).
        Ok(int_16_bit as u16)
    }

    /// `7.1.15 ToBigInt64 ( argument )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-tobigint64
    pub fn to_big_int64(&self, context: &mut Context) -> JsResult<BigInt> {
        // 1. Let n be ? ToBigInt(argument).
        let n = self.to_bigint(context)?;

        // 2. Let int64bit be ℝ(n) modulo 2^64.
        let int64_bit = n.as_inner().mod_floor(&TWO_E_64);

        // 3. If int64bit ≥ 2^63, return ℤ(int64bit - 2^64); otherwise return ℤ(int64bit).
        if int64_bit >= *TWO_E_63 {
            Ok(int64_bit.sub(&*TWO_E_64))
        } else {
            Ok(int64_bit)
        }
    }

    /// `7.1.16 ToBigUint64 ( argument )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-tobiguint64
    pub fn to_big_uint64(&self, context: &mut Context) -> JsResult<BigInt> {
        let two_e_64: u128 = 0x1_0000_0000_0000_0000;
        let two_e_64 = BigInt::from(two_e_64);

        // 1. Let n be ? ToBigInt(argument).
        let n = self.to_bigint(context)?;

        // 2. Let int64bit be ℝ(n) modulo 2^64.
        // 3. Return ℤ(int64bit).
        Ok(n.as_inner().mod_floor(&two_e_64))
    }

    /// Converts a value to a non-negative integer if it is a valid integer index value.
    ///
    /// See: <https://tc39.es/ecma262/#sec-toindex>
    pub fn to_index(&self, context: &mut Context) -> JsResult<usize> {
        // 1. If value is undefined, then
        if self.is_undefined() {
            // a. Return 0.
            return Ok(0);
        }

        // 2. Else,
        // a. Let integer be ? ToIntegerOrInfinity(value).
        let integer = self.to_integer_or_infinity(context)?;

        // b. Let clamped be ! ToLength(𝔽(integer)).
        let clamped = integer.clamp_finite(0, Number::MAX_SAFE_INTEGER as i64);

        // c. If ! SameValue(𝔽(integer), clamped) is false, throw a RangeError exception.
        if integer != clamped {
            return context.throw_range_error("Index must be between 0 and  2^53 - 1");
        }

        // d. Assert: 0 ≤ integer ≤ 2^53 - 1.
        debug_assert!(0 <= clamped && clamped <= Number::MAX_SAFE_INTEGER as i64);

        // e. Return integer.
        Ok(clamped as usize)
    }

    /// Converts argument to an integer suitable for use as the length of an array-like object.
    ///
    /// See: <https://tc39.es/ecma262/#sec-tolength>
    pub fn to_length(&self, context: &mut Context) -> JsResult<usize> {
        // 1. Let len be ? ToInteger(argument).
        // 2. If len ≤ +0, return +0.
        // 3. Return min(len, 2^53 - 1).
        Ok(self
            .to_integer_or_infinity(context)?
            .clamp_finite(0, Number::MAX_SAFE_INTEGER as i64) as usize)
    }

    /// Abstract operation `ToIntegerOrInfinity ( argument )`
    ///
    /// This method converts a `Value` to an integer representing its `Number` value with
    /// fractional part truncated, or to +∞ or -∞ when that `Number` value is infinite.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-tointegerorinfinity
    pub fn to_integer_or_infinity(&self, context: &mut Context) -> JsResult<IntegerOrInfinity> {
        // 1. Let number be ? ToNumber(argument).
        let number = self.to_number(context)?;

        if number.is_nan() || number == 0.0 {
            // 2. If number is NaN, +0𝔽, or -0𝔽, return 0.
            Ok(IntegerOrInfinity::Integer(0))
        } else if number == f64::INFINITY {
            // 3. If number is +∞𝔽, return +∞.
            Ok(IntegerOrInfinity::PositiveInfinity)
        } else if number == f64::NEG_INFINITY {
            // 4. If number is -∞𝔽, return -∞.
            Ok(IntegerOrInfinity::NegativeInfinity)
        } else {
            // 5. Let integer be floor(abs(ℝ(number))).
            // 6. If number < +0𝔽, set integer to -integer.
            let integer = number.abs().floor().copysign(number) as i64;

            // 7. Return integer.
            Ok(IntegerOrInfinity::Integer(integer))
        }
    }

    /// Converts a value to a double precision floating point.
    ///
    /// This function is equivalent to the unary `+` operator (`+value`) in JavaScript
    ///
    /// See: <https://tc39.es/ecma262/#sec-tonumber>
    pub fn to_number(&self, context: &mut Context) -> JsResult<f64> {
        match self.variant() {
            JsVariant::Null => Ok(0.0),
            JsVariant::Undefined => Ok(f64::NAN),
            JsVariant::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            JsVariant::String(string) => Ok(string.string_to_number()),
            JsVariant::Rational(number) => Ok(number),
            JsVariant::Integer(integer) => Ok(f64::from(integer)),
            JsVariant::Symbol(_) => context.throw_type_error("argument must not be a symbol"),
            JsVariant::BigInt(_) => context.throw_type_error("argument must not be a bigint"),
            JsVariant::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::Number)?;
                primitive.to_number(context)
            }
        }
    }

    /// This is a more specialized version of `to_numeric`, including `BigInt`.
    ///
    /// This function is equivalent to `Number(value)` in JavaScript
    ///
    /// See: <https://tc39.es/ecma262/#sec-tonumeric>
    pub fn to_numeric_number(&self, context: &mut Context) -> JsResult<f64> {
        let primitive = self.to_primitive(context, PreferredType::Number)?;
        if let Some(bigint) = primitive.as_bigint() {
            return Ok(bigint.to_f64());
        }
        primitive.to_number(context)
    }

    /// Check if the `Value` can be converted to an `Object`
    ///
    /// The abstract operation `RequireObjectCoercible` takes argument argument.
    /// It throws an error if argument is a value that cannot be converted to an Object using `ToObject`.
    /// It is defined by [Table 15][table]
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [table]: https://tc39.es/ecma262/#table-14
    /// [spec]: https://tc39.es/ecma262/#sec-requireobjectcoercible
    #[inline]
    pub fn require_object_coercible(&self, context: &mut Context) -> JsResult<&Self> {
        if self.is_null_or_undefined() {
            context.throw_type_error("cannot convert null or undefined to Object")
        } else {
            Ok(self)
        }
    }

    #[inline]
    pub fn to_property_descriptor(&self, context: &mut Context) -> JsResult<PropertyDescriptor> {
        // 1. If Type(Obj) is not Object, throw a TypeError exception.
        self.as_object()
            .ok_or_else(|| {
                context.construct_type_error(
                    "Cannot construct a property descriptor from a non-object",
                )
            })
            .and_then(|obj| obj.to_property_descriptor(context))
    }

    /// `typeof` operator. Returns a string representing the type of the
    /// given ECMA Value.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typeof-operator
    pub fn type_of(&self) -> JsString {
        match self.variant() {
            JsVariant::Rational(_) | JsVariant::Integer(_) => "number",
            JsVariant::String(_) => "string",
            JsVariant::Boolean(_) => "boolean",
            JsVariant::Symbol(_) => "symbol",
            JsVariant::Null => "object",
            JsVariant::Undefined => "undefined",
            JsVariant::BigInt(_) => "bigint",
            JsVariant::Object(object) => {
                if object.is_callable() {
                    "function"
                } else {
                    "object"
                }
            }
        }
        .into()
    }

    /// Abstract operation `IsArray ( argument )`
    ///
    /// Check if a value is an array.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isarray
    pub(crate) fn is_array(&self, context: &mut Context) -> JsResult<bool> {
        // Note: The spec specifies this function for JsValue.
        // The main part of the function is implemented for JsObject.

        // 1. If Type(argument) is not Object, return false.
        if let Some(object) = self.as_object() {
            object.is_array_abstract(context)
        }
        // 4. Return false.
        else {
            Ok(false)
        }
    }
}

impl Default for JsValue {
    fn default() -> Self {
        Self::undefined()
    }
}

/// The preffered type to convert an object to a primitive `Value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PreferredType {
    String,
    Number,
    Default,
}

/// Numeric value which can be of two types `Number`, `BigInt`.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Numeric {
    /// Double precision floating point number.
    Number(f64),
    /// BigInt an integer of arbitrary size.
    BigInt(JsBigInt),
}

impl From<f64> for Numeric {
    #[inline]
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<f32> for Numeric {
    #[inline]
    fn from(value: f32) -> Self {
        Self::Number(value.into())
    }
}

impl From<i64> for Numeric {
    #[inline]
    fn from(value: i64) -> Self {
        Self::BigInt(value.into())
    }
}

impl From<i32> for Numeric {
    #[inline]
    fn from(value: i32) -> Self {
        Self::Number(value.into())
    }
}

impl From<i16> for Numeric {
    #[inline]
    fn from(value: i16) -> Self {
        Self::Number(value.into())
    }
}

impl From<i8> for Numeric {
    #[inline]
    fn from(value: i8) -> Self {
        Self::Number(value.into())
    }
}

impl From<u64> for Numeric {
    #[inline]
    fn from(value: u64) -> Self {
        Self::BigInt(value.into())
    }
}

impl From<u32> for Numeric {
    #[inline]
    fn from(value: u32) -> Self {
        Self::Number(value.into())
    }
}

impl From<u16> for Numeric {
    #[inline]
    fn from(value: u16) -> Self {
        Self::Number(value.into())
    }
}

impl From<u8> for Numeric {
    #[inline]
    fn from(value: u8) -> Self {
        Self::Number(value.into())
    }
}

impl From<JsBigInt> for Numeric {
    #[inline]
    fn from(value: JsBigInt) -> Self {
        Self::BigInt(value)
    }
}

impl From<Numeric> for JsValue {
    fn from(value: Numeric) -> Self {
        match value {
            Numeric::Number(number) => Self::new(number),
            Numeric::BigInt(bigint) => Self::new(bigint),
        }
    }
}
