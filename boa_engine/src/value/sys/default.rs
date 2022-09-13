use boa_gc::{custom_trace, Finalize, Trace};

use super::JsVariant;

use crate::{object::JsObject, JsBigInt, JsString, JsSymbol};

#[derive(Finalize, Debug, Clone)]
enum Value {
    Null,
    Undefined,
    Boolean(bool),
    Integer32(i32),
    Float64(f64),
    String(JsString),
    BigInt(JsBigInt),
    Symbol(JsSymbol),
    Object(JsObject),
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl Trace for Value {
    custom_trace! {this, {
        if let Self::Object(o) = this {
            mark(o);
        }
    }}
}

/// A Javascript value
///
/// Check the [`value`][`super::super`] module for more information.
#[derive(Trace, Finalize, Debug, Clone)]
pub struct JsValue(Value);

impl JsValue {
    /// `null` - A null value, for when a value doesn't exist.
    #[inline]
    pub const fn null() -> Self {
        Self(Value::Null)
    }

    /// `undefined` - An undefined value, for when a field or index doesn't exist
    #[inline]
    pub const fn undefined() -> Self {
        Self(Value::Undefined)
    }

    /// `boolean` - A `true` / `false` value.
    #[inline]
    pub fn boolean(boolean: bool) -> Self {
        Self(Value::Boolean(boolean))
    }

    /// `integer32` - A 32-bit integer value, such as `42`.
    #[inline]
    pub fn integer32(integer: i32) -> Self {
        Self(Value::Integer32(integer))
    }

    /// `float64` - A 64-bit floating point number value, such as `3.1415`
    #[inline]
    pub fn float64(float64: f64) -> Self {
        Self(Value::Float64(float64))
    }

    /// `String` - A [`JsString`] value, such as `"Hello, world"`.
    #[inline]
    pub fn string(string: JsString) -> Self {
        Self(Value::String(string))
    }

    /// `BigInt` - A [`JsBigInt`] value, an arbitrarily large signed integer.
    #[inline]
    pub fn bigint(bigint: JsBigInt) -> Self {
        Self(Value::BigInt(bigint))
    }

    /// `Symbol` - A [`JsSymbol`] value.
    #[inline]
    pub fn symbol(symbol: JsSymbol) -> Self {
        Self(Value::Symbol(symbol))
    }

    /// `Object` - A [`JsObject`], such as `Math`, represented by a binary tree of string keys to Javascript values.
    #[inline]
    pub fn object(object: JsObject) -> Self {
        Self(Value::Object(object))
    }

    /// Returns the internal [`bool`] if the value is a boolean, or
    /// [`None`] otherwise.
    #[inline]
    pub fn as_boolean(&self) -> Option<bool> {
        match self.0 {
            Value::Boolean(boolean) => Some(boolean),
            _ => None,
        }
    }

    /// Returns the internal [`i32`] if the value is a 32-bit signed integer number, or
    /// [`None`] otherwise.
    #[inline]
    pub fn as_integer32(&self) -> Option<i32> {
        match self.0 {
            Value::Integer32(integer) => Some(integer),
            _ => None,
        }
    }

    /// Returns the internal [`f64`] if the value is a 64-bit floating-point number, or
    /// [`None`] otherwise.
    #[inline]
    pub fn as_float64(&self) -> Option<f64> {
        match self.0 {
            Value::Float64(rational) => Some(rational),
            _ => None,
        }
    }

    /// Returns a reference to the internal [`JsString`] if the value is a string, or
    /// [`None`] otherwise.
    #[inline]
    pub fn as_string(&self) -> Option<Ref<'_, JsString>> {
        match self.0 {
            Value::String(ref inner) => Some(Ref { inner }),
            _ => None,
        }
    }

    /// Returns a reference to the internal [`JsBigInt`] if the value is a big int, or
    /// [`None`] otherwise.
    #[inline]
    pub fn as_bigint(&self) -> Option<Ref<'_, JsBigInt>> {
        match self.0 {
            Value::BigInt(ref inner) => Some(Ref { inner }),
            _ => None,
        }
    }

    /// Returns a reference to the internal [`JsSymbol`] if the value is a symbol, or
    /// [`None`] otherwise.
    #[inline]
    pub fn as_symbol(&self) -> Option<Ref<'_, JsSymbol>> {
        match self.0 {
            Value::Symbol(ref inner) => Some(Ref { inner }),
            _ => None,
        }
    }

    /// Returns a reference to the internal [`JsObject`] if the value is an object, or
    /// [`None`] otherwise.
    #[inline]
    pub fn as_object(&self) -> Option<Ref<'_, JsObject>> {
        match self.0 {
            Value::Object(ref inner) => Some(Ref { inner }),
            _ => None,
        }
    }

    /// Returns true if the value is null.
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self.0, Value::Null)
    }

    /// Returns true if the value is undefined.
    #[inline]
    pub fn is_undefined(&self) -> bool {
        matches!(self.0, Value::Undefined)
    }

    /// Returns true if the value is a boolean.
    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(self.0, Value::Boolean(_))
    }

    /// Returns true if the value is a 32-bit signed integer number.
    #[inline]
    pub fn is_integer32(&self) -> bool {
        matches!(self.0, Value::Integer32(_))
    }

    /// Returns true if the value is a 64-bit floating-point number.
    #[inline]
    pub fn is_float64(&self) -> bool {
        matches!(self.0, Value::Float64(_))
    }

    /// Returns true if the value is a 64-bit floating-point `NaN` number.
    #[inline]
    pub fn is_nan(&self) -> bool {
        matches!(self.0, Value::Float64(r) if r.is_nan())
    }

    /// Returns true if the value is a string.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self.0, Value::String(_))
    }

    /// Returns true if the value is a bigint.
    #[inline]
    pub fn is_bigint(&self) -> bool {
        matches!(self.0, Value::BigInt(_))
    }

    /// Returns true if the value is a symbol.
    #[inline]
    pub fn is_symbol(&self) -> bool {
        matches!(self.0, Value::Symbol(_))
    }

    /// Returns true if the value is an object
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self.0, Value::Object(_))
    }

    /// Returns a [`JsVariant`] enum representing the current variant of the value.
    ///
    /// # Note
    ///
    /// More exotic implementations of [`JsValue`] cannot use direct references to
    /// heap based types, so [`JsVariant`] instead returns [`Ref`]s on those cases.
    pub fn variant(&self) -> JsVariant<'_> {
        match self.0 {
            Value::Null => JsVariant::Null,
            Value::Undefined => JsVariant::Undefined,
            Value::Integer32(i) => JsVariant::Integer32(i),
            Value::Float64(d) => JsVariant::Float64(d),
            Value::Boolean(b) => JsVariant::Boolean(b),
            Value::Object(ref inner) => JsVariant::Object(Ref { inner }),
            Value::String(ref inner) => JsVariant::String(Ref { inner }),
            Value::Symbol(ref inner) => JsVariant::Symbol(Ref { inner }),
            Value::BigInt(ref inner) => JsVariant::BigInt(Ref { inner }),
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
    inner: &'a T,
}

// Lift `Ref` over `AsRef`, since implementing `AsRef<T>` would override the
// `as_ref` implementations of `T`.
impl<U, T> AsRef<U> for Ref<'_, T>
where
    T: AsRef<U>,
{
    #[inline]
    fn as_ref(&self) -> &U {
        <T as AsRef<U>>::as_ref(self.inner)
    }
}

impl<T> std::ops::Deref for Ref<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T: PartialEq> PartialEq<T> for Ref<'_, T> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.inner == other
    }
}

impl<T> std::borrow::Borrow<T> for Ref<'_, T> {
    #[inline]
    fn borrow(&self) -> &T {
        self.inner
    }
}
