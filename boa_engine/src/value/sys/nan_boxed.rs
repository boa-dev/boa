#![allow(unstable_name_collisions)]

use num_enum::TryFromPrimitive;
use std::cell::Cell;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use boa_gc::{Finalize, Trace};

// TODO: Remove if/when https://github.com/rust-lang/rust/issues/95228 gets stabilized
use sptr::Strict;

use super::JsVariant;

use crate::{object::JsObject, value::PointerType, JsBigInt, JsString, JsSymbol};

// Our `cfg` options must ensure `usize == u64`.
// Using `usize`s only makes it more convenient to use in this module

const SIGN_BIT: usize = 0x8000_0000_0000_0000;
const EXPONENT: usize = 0x7FF0_0000_0000_0000;
// const MANTISA: usize = 0x000F_FFFF_FFFF_FFFF;
const SIGNAL_BIT: usize = 0x0008_0000_0000_0000;
const QNAN: usize = EXPONENT | SIGNAL_BIT; // 0x7FF8000000000000

const CANONICALIZED_NAN: usize = QNAN;
// const PAYLOAD: usize = 0x0000_7FFF_FFFF_FFFF;
// const TYPE: usize = !PAYLOAD;

const TAG_MASK: usize = 0xFFFF_0000_0000_0000;

const DOUBLE_TYPE: usize = QNAN;
const INTEGER_TYPE: usize = QNAN | (0b001 << 48);
const BOOLEAN_TYPE: usize = QNAN | (0b010 << 48);
const UNDEFINED_TYPE: usize = QNAN | (0b011 << 48);
const NULL_TYPE: usize = QNAN | (0b100 << 48);

#[allow(unused)]
const RESERVED1_TYPE: usize = QNAN | (0b101 << 48);
#[allow(unused)]
const RESERVED2_TYPE: usize = QNAN | (0b110 << 48);
#[allow(unused)]
const RESERVED3_TYPE: usize = QNAN | (0b111 << 48);

const POINTER_TYPE: usize = SIGN_BIT | QNAN;
const OBJECT_TYPE: usize = POINTER_TYPE | (0b001 << 48);
const STRING_TYPE: usize = POINTER_TYPE | (0b010 << 48);
const SYMBOL_TYPE: usize = POINTER_TYPE | (0b011 << 48);
const BIGINT_TYPE: usize = POINTER_TYPE | (0b100 << 48);

#[allow(unused)]
const RESERVED4_TYPE: usize = POINTER_TYPE | (0b101 << 48);
#[allow(unused)]
const RESERVED5_TYPE: usize = POINTER_TYPE | (0b110 << 48);
#[allow(unused)]
const RESERVED6_TYPE: usize = POINTER_TYPE | (0b111 << 48);

const MASK_INT_PAYLOAD: usize = 0x00000000FFFFFFFF;
const MASK_POINTER_PAYLOAD: usize = 0x0000FFFFFFFFFFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
enum ValueTag {
    Float64 = (DOUBLE_TYPE >> 48) as _,
    Integer32 = (INTEGER_TYPE >> 48) as _,
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

/// A Javascript value
///
/// Check the [`value`][`super::super`] module for more information.
#[derive(Debug)]
#[repr(transparent)]
pub struct JsValue {
    value: Cell<*mut ()>,
}

impl JsValue {
    /// `null` - A null value, for when a value doesn't exist.
    #[inline]
    pub const fn null() -> Self {
        Self {
            value: Cell::new(sptr::invalid_mut(NULL_TYPE)),
        }
    }

    /// `undefined` - An undefined value, for when a field or index doesn't exist
    #[inline]
    pub const fn undefined() -> Self {
        Self {
            value: Cell::new(sptr::invalid_mut(UNDEFINED_TYPE)),
        }
    }

    /// `boolean` - A `true` / `false` value.
    #[inline]
    pub fn boolean(boolean: bool) -> Self {
        let value = Self {
            value: Cell::new(sptr::invalid_mut(usize::from(boolean) | BOOLEAN_TYPE)),
        };
        debug_assert!(value.is_boolean());
        debug_assert_eq!(value.tag(), ValueTag::Boolean);
        value
    }

    /// `integer32` - A 32-bit integer value, such as `42`.
    #[inline]
    pub fn integer32(integer32: i32) -> Self {
        let value = Self {
            value: Cell::new(sptr::invalid_mut(integer32 as u32 as usize | INTEGER_TYPE)),
        };
        debug_assert!(value.is_integer32());
        debug_assert_eq!(value.tag(), ValueTag::Integer32);
        value
    }

    /// `float64` - A 64-bit floating point number value, such as `3.1415`
    #[inline]
    pub fn float64(float64: f64) -> Self {
        if float64.is_nan() {
            return Self {
                value: Cell::new(sptr::invalid_mut(CANONICALIZED_NAN)),
            };
        }

        let value = Self {
            value: Cell::new(sptr::invalid_mut(float64.to_bits() as usize)),
        };
        debug_assert!(value.is_float64());
        debug_assert_eq!(value.tag(), ValueTag::Float64);
        value
    }

    /// `String` - A [`JsString`] value, such as `"Hello, world"`.
    #[inline]
    pub fn string(string: JsString) -> Self {
        let string = ManuallyDrop::new(string);
        let pointer = unsafe { JsString::into_void_ptr(string) };
        debug_assert_eq!(pointer.addr() & MASK_POINTER_PAYLOAD, pointer.addr());
        let value = Self {
            value: Cell::new(pointer.map_addr(|addr| addr | STRING_TYPE)),
        };
        debug_assert!(value.is_string());
        debug_assert_eq!(value.tag(), ValueTag::String);
        value
    }

    /// `BigInt` - A [`JsBigInt`] value, an arbitrarily large signed integer.
    #[inline]
    pub fn bigint(bigint: JsBigInt) -> Self {
        let bigint = ManuallyDrop::new(bigint);
        let pointer = unsafe { JsBigInt::into_void_ptr(bigint) };
        debug_assert_eq!(pointer.addr() & MASK_POINTER_PAYLOAD, pointer.addr());
        let value = Self {
            value: Cell::new(pointer.map_addr(|addr| addr | BIGINT_TYPE)),
        };
        debug_assert!(value.is_bigint());
        debug_assert_eq!(value.tag(), ValueTag::BigInt);
        value
    }

    /// `Symbol` - A [`JsSymbol`] value.
    #[inline]
    pub fn symbol(symbol: JsSymbol) -> Self {
        let symbol = ManuallyDrop::new(symbol);
        let pointer = unsafe { JsSymbol::into_void_ptr(symbol) };
        debug_assert_eq!(pointer.addr() & MASK_POINTER_PAYLOAD, pointer.addr());
        let value = Self {
            value: Cell::new(pointer.map_addr(|addr| addr | SYMBOL_TYPE)),
        };
        debug_assert!(value.is_symbol());
        debug_assert_eq!(value.tag(), ValueTag::Symbol);
        value
    }

    /// `Object` - A [`JsObject`], such as `Math`, represented by a binary tree of string keys to Javascript values.
    #[inline]
    pub fn object(object: JsObject) -> Self {
        let object = ManuallyDrop::new(object);
        let pointer = unsafe { JsObject::into_void_ptr(object) };
        debug_assert_eq!(pointer.addr() & MASK_POINTER_PAYLOAD, pointer.addr());
        debug_assert_eq!(OBJECT_TYPE & MASK_POINTER_PAYLOAD, 0);
        let value = Self {
            value: Cell::new(pointer.map_addr(|addr| addr | OBJECT_TYPE)),
        };
        debug_assert!(value.is_object());
        debug_assert_eq!(value.tag(), ValueTag::Object);
        value
    }

    /// Returns the internal [`bool`] if the value is a boolean, or
    /// [`None`] otherwise.
    #[inline]
    pub fn as_boolean(&self) -> Option<bool> {
        if self.is_boolean() {
            return Some(self.as_boolean_uncheched());
        }

        None
    }

    /// Returns the internal [`i32`] if the value is a 32-bit signed integer number, or
    /// [`None`] otherwise.
    pub fn as_integer32(&self) -> Option<i32> {
        if self.is_integer32() {
            return Some(self.as_integer32_uncheched());
        }

        None
    }

    /// Returns the internal [`f64`] if the value is a 64-bit floating-point number, or
    /// [`None`] otherwise.
    pub fn as_float64(&self) -> Option<f64> {
        if self.is_float64() {
            return Some(self.as_float64_unchecked());
        }

        None
    }

    /// Returns a reference to the internal [`JsString`] if the value is a string, or
    /// [`None`] otherwise.
    #[inline]
    pub fn as_string(&self) -> Option<Ref<'_, JsString>> {
        if self.is_string() {
            return unsafe { Some(self.as_string_unchecked()) };
        }

        None
    }

    /// Returns a reference to the internal [`JsBigInt`] if the value is a big int, or
    /// [`None`] otherwise.
    pub fn as_bigint(&self) -> Option<Ref<'_, JsBigInt>> {
        if self.is_bigint() {
            return unsafe { Some(self.as_bigint_unchecked()) };
        }

        None
    }

    /// Returns a reference to the internal [`JsSymbol`] if the value is a symbol, or
    /// [`None`] otherwise.
    pub fn as_symbol(&self) -> Option<Ref<'_, JsSymbol>> {
        if self.is_symbol() {
            return unsafe { Some(self.as_symbol_unchecked()) };
        }

        None
    }

    /// Returns a reference to the internal [`JsObject`] if the value is an object, or
    /// [`None`] otherwise.
    pub fn as_object(&self) -> Option<Ref<'_, JsObject>> {
        if self.is_object() {
            return unsafe { Some(self.as_object_unchecked()) };
        }

        None
    }

    /// Returns true if the value is null.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.value.get().addr() == NULL_TYPE
    }

    /// Returns true if the value is undefined.
    #[inline]
    pub fn is_undefined(&self) -> bool {
        self.value.get().addr() == UNDEFINED_TYPE
    }

    /// Returns true if the value is a boolean.
    #[inline]
    pub fn is_boolean(&self) -> bool {
        self.value.get().addr() & TAG_MASK == BOOLEAN_TYPE
    }

    /// Returns true if the value is a 32-bit signed integer number.
    pub fn is_integer32(&self) -> bool {
        self.value.get().addr() & TAG_MASK == INTEGER_TYPE
    }

    /// Returns true if the value is a 64-bit floating-point number.
    pub fn is_float64(&self) -> bool {
        (self.value.get().addr() & !SIGN_BIT) <= QNAN
    }

    /// Returns true if the value is a 64-bit floating-point `NaN` number.
    pub fn is_nan(&self) -> bool {
        self.value.get().addr() == CANONICALIZED_NAN
    }

    /// Returns true if the value is a string.
    #[inline]
    pub fn is_string(&self) -> bool {
        self.value.get().addr() & TAG_MASK == STRING_TYPE
    }

    /// Returns true if the value is a bigint.
    #[inline]
    pub fn is_bigint(&self) -> bool {
        self.value.get().addr() & TAG_MASK == BIGINT_TYPE
    }

    /// Returns true if the value is a symbol.
    pub fn is_symbol(&self) -> bool {
        self.value.get().addr() & TAG_MASK == SYMBOL_TYPE
    }

    /// Returns true if the value is an object
    #[inline]
    pub fn is_object(&self) -> bool {
        self.value.get().addr() & TAG_MASK == OBJECT_TYPE
    }

    /// Returns a [`JsVariant`] enum representing the current variant of the value.
    ///
    /// # Note
    ///
    /// More exotic implementations of [`JsValue`] cannot use direct references to
    /// heap based types, so [`JsVariant`] instead returns [`Ref`]s on those cases.
    pub fn variant(&self) -> JsVariant<'_> {
        unsafe {
            match self.tag() {
                ValueTag::Null => JsVariant::Null,
                ValueTag::Undefined => JsVariant::Undefined,
                ValueTag::Integer32 => JsVariant::Integer32(self.as_integer32_uncheched()),
                ValueTag::Float64 => JsVariant::Float64(self.as_float64_unchecked()),
                ValueTag::Boolean => JsVariant::Boolean(self.as_boolean_uncheched()),
                ValueTag::Object => JsVariant::Object(self.as_object_unchecked()),
                ValueTag::String => JsVariant::String(self.as_string_unchecked()),
                ValueTag::Symbol => JsVariant::Symbol(self.as_symbol_unchecked()),
                ValueTag::BigInt => JsVariant::BigInt(self.as_bigint_unchecked()),
            }
        }
    }

    fn as_pointer(&self) -> *mut () {
        self.value
            .get()
            .map_addr(|addr| addr & MASK_POINTER_PAYLOAD)
    }

    fn tag(&self) -> ValueTag {
        if self.is_float64() {
            return ValueTag::Float64;
        }
        let tag = ((self.value.get().addr() & TAG_MASK) >> 48) as u16;
        ValueTag::try_from(tag).expect("Implementation must never construct an invalid tag")
    }

    fn as_boolean_uncheched(&self) -> bool {
        (self.value.get().addr() & 0xFF) != 0
    }

    fn as_integer32_uncheched(&self) -> i32 {
        (self.value.get().addr() & MASK_INT_PAYLOAD) as u32 as i32
    }

    fn as_float64_unchecked(&self) -> f64 {
        f64::from_bits(self.value.get().addr() as u64)
    }

    /// Returns a reference to the boxed [`JsString`] without checking
    /// if the tag of `self` is valid.
    ///
    /// # Safety
    ///
    /// Calling this method with a [`JsValue`] that doesn't box
    /// a [`JsString`] is undefined behaviour.
    unsafe fn as_string_unchecked(&self) -> Ref<'_, JsString> {
        unsafe { Ref::new(JsString::from_void_ptr(self.as_pointer())) }
    }

    /// Returns a reference to the boxed [`JsBigInt`] without checking
    /// if the tag of `self` is valid.
    ///
    /// # Safety
    ///
    /// Calling this method with a [`JsValue`] that doesn't box
    /// a [`JsBigInt`] is undefined behaviour.
    #[inline]
    unsafe fn as_bigint_unchecked(&self) -> Ref<'_, JsBigInt> {
        // SAFETY: The safety contract must be upheld by the caller
        unsafe { Ref::new(JsBigInt::from_void_ptr(self.as_pointer())) }
    }

    /// Returns a reference to the boxed [`JsSymbol`] without checking
    /// if the tag of `self` is valid.
    ///
    /// # Safety
    ///
    /// Calling this method with a [`JsValue`] that doesn't box
    /// a [`JsSymbol`] is undefined behaviour.
    unsafe fn as_symbol_unchecked(&self) -> Ref<'_, JsSymbol> {
        unsafe { Ref::new(JsSymbol::from_void_ptr(self.as_pointer())) }
    }

    /// Returns a reference to the boxed [`JsObject`] without checking
    /// if the tag of `self` is valid.
    ///
    /// # Safety
    ///
    /// Calling this method with a [`JsValue`] that doesn't box
    /// a [`JsObject`] is undefined behaviour.
    unsafe fn as_object_unchecked(&self) -> Ref<'_, JsObject> {
        unsafe { Ref::new(JsObject::from_void_ptr(self.as_pointer())) }
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
                    .set(JsObject::into_void_ptr(o).map_addr(|addr| addr | OBJECT_TYPE));
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
                    .set(JsObject::into_void_ptr(o).map_addr(|addr| addr | OBJECT_TYPE));
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

/// Represents a reference to a boxed pointer type inside a [`JsValue`]
///
/// This is exclusively used to return references to [`JsString`], [`JsObject`],
/// [`JsSymbol`] and [`JsBigInt`], since some [`JsValue`] implementations makes
/// returning proper references difficult.
/// It is mainly returned by the [`JsValue::variant`] method and the
/// `as_` methods for checked casts to pointer types.
///
/// [`Ref`] implements [`Deref`][`std::ops::Deref`], which facilitates conversion
/// to a proper [`reference`] by using the `ref` keyword or the
/// [`Option::as_deref`][`std::option::Option::as_deref`] method.
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

// Lift `Ref` over `AsRef`, since implementing `AsRef<T>` would override the
// `as_ref` implementations of `T`.
impl<U, T> AsRef<U> for Ref<'_, T>
where
    T: AsRef<U>,
{
    #[inline]
    fn as_ref(&self) -> &U {
        <T as AsRef<U>>::as_ref(&*self)
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
    #[inline]
    fn eq(&self, other: &T) -> bool {
        &**self == other
    }
}

impl<T> std::borrow::Borrow<T> for Ref<'_, T> {
    #[inline]
    fn borrow(&self) -> &T {
        &**self
    }
}

#[cfg(test)]
mod tests_nan_box {
    use crate::object::ObjectData;

    use super::*;

    #[test]
    fn bigint() {
        let value = JsValue::new(JsBigInt::new(12345));

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_integer32());
        assert!(!value.is_float64());
        assert!(!value.is_boolean());
        assert!(!value.is_object());
        assert!(!value.is_string());
        assert!(!value.is_symbol());

        assert!(value.is_bigint());

        let bigint = value.as_bigint().unwrap();

        assert_eq!(&bigint, &JsBigInt::new(12345));
    }

    #[test]
    fn symbol() {
        let value = JsValue::new(JsSymbol::new(Some("description...".into())));

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_integer32());
        assert!(!value.is_float64());
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
        assert!(!value.is_integer32());
        assert!(!value.is_float64());
        assert!(!value.is_boolean());
        assert!(!value.is_object());

        assert!(value.is_string());

        let string = value.as_string().unwrap();

        assert_eq!(JsString::refcount(&string), Some(1));

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
        assert!(!value.is_integer32());
        assert!(!value.is_float64());
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
        assert!(!value.is_integer32());
        assert!(!value.is_float64());

        assert!(value.is_boolean());
        assert_eq!(value.as_boolean(), Some(true));

        let value = JsValue::new(false);

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_integer32());
        assert!(!value.is_float64());

        assert!(value.is_boolean());
        assert_eq!(value.as_boolean(), Some(false));
    }

    #[test]
    fn float64() {
        let value = JsValue::new(1.3);

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());
        assert!(!value.is_integer32());

        assert!(value.is_float64());
        assert_eq!(value.as_float64(), Some(1.3));

        let value = JsValue::new(f64::MAX);
        assert!(value.is_float64());
        assert_eq!(value.as_float64(), Some(f64::MAX));

        let value = JsValue::new(f64::MIN);
        assert!(value.is_float64());
        assert_eq!(value.as_float64(), Some(f64::MIN));

        let value = JsValue::nan();
        assert!(value.is_float64());
        assert!(value.as_float64().unwrap().is_nan());

        let value = JsValue::new(12345);
        assert!(!value.is_float64());
        assert_eq!(value.as_float64(), None);

        let value = JsValue::undefined();
        assert!(!value.is_float64());
        assert_eq!(value.as_float64(), None);

        let value = JsValue::null();
        assert!(!value.is_float64());
        assert_eq!(value.as_float64(), None);
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
    fn integer32() {
        let value = JsValue::new(-0xcafe);

        assert!(!value.is_null());
        assert!(!value.is_null_or_undefined());
        assert!(!value.is_undefined());

        assert!(value.is_integer32());

        assert_eq!(value.as_integer32(), Some(-0xcafe));

        let value = JsValue::null();
        assert_eq!(value.as_integer32(), None);
    }
}
