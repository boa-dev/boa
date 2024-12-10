//! Boa's ECMAScript Value implementation.
//!
//! Javascript values, utility methods and conversion between Javascript values and Rust values.

use std::{
    collections::HashSet,
    fmt::{self, Display},
    ops::Sub,
};

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{ToPrimitive, Zero};
use once_cell::sync::Lazy;

use boa_gc::{custom_trace, Finalize, Trace};
#[doc(inline)]
pub use boa_macros::TryFromJs;
pub use boa_macros::TryIntoJs;
use boa_profiler::Profiler;
#[doc(inline)]
pub use conversions::convert::Convert;

pub(crate) use self::conversions::IntoOrUndefined;
#[doc(inline)]
pub use self::{
    conversions::try_from_js::TryFromJs, conversions::try_into_js::TryIntoJs,
    display::ValueDisplay, integer::IntegerOrInfinity, operations::*, r#type::Type,
    variant::JsVariant,
};
use crate::builtins::RegExp;
use crate::object::{JsFunction, JsPromise, JsRegExp};
use crate::{
    builtins::{
        number::{f64_to_int32, f64_to_uint32},
        Number, Promise,
    },
    error::JsNativeError,
    js_string,
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
    symbol::JsSymbol,
    Context, JsBigInt, JsResult, JsString,
};

mod conversions;
pub(crate) mod display;
mod equality;
mod hash;
mod integer;
mod operations;
mod r#type;
mod variant;

#[cfg(test)]
mod tests;

static TWO_E_64: Lazy<BigInt> = Lazy::new(|| {
    const TWO_E_64: u128 = 2u128.pow(64);
    BigInt::from(TWO_E_64)
});

static TWO_E_63: Lazy<BigInt> = Lazy::new(|| {
    const TWO_E_63: u128 = 2u128.pow(63);
    BigInt::from(TWO_E_63)
});

/// The Inner type of a [`JsValue`]. This is the actual value that the `JsValue` holds.
/// This is not a public API and should not be used directly.
///
/// If you need access to the variant, use [`JsValue::variant`] instead.
#[derive(Finalize, Debug, Clone, PartialEq)]
enum InnerValue {
    Null,
    Undefined,
    Boolean(bool),
    Float64(f64),
    Integer32(i32),
    BigInt(JsBigInt),
    String(JsString),
    Symbol(JsSymbol),
    Object(JsObject),
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe impl Trace for InnerValue {
    custom_trace! {this, mark, {
        if let Self::Object(o) = this {
            mark(o);
        }
    }}
}

/// A Javascript value
#[derive(Finalize, Debug, Clone, Trace)]
pub struct JsValue {
    inner: InnerValue,
}

impl JsValue {
    /// The integer zero as a [`JsValue`] constant, for convenience.
    pub const ZERO: Self = Self {
        inner: InnerValue::Integer32(0),
    };

    /// The integer one as a [`JsValue`] constant, for convenience.
    pub const ONE: Self = Self {
        inner: InnerValue::Integer32(1),
    };

    /// `NaN` as a [`JsValue`] constant, for convenience.
    pub const NAN: Self = Self {
        inner: InnerValue::Float64(f64::NAN),
    };

    /// Positive infinity as a [`JsValue`] constant, for convenience.
    pub const POSITIVE_INFINITY: Self = Self {
        inner: InnerValue::Float64(f64::INFINITY),
    };

    /// Negative infinity as a [`JsValue`] constant, for convenience.
    pub const NEGATIVE_INFINITY: Self = Self {
        inner: InnerValue::Float64(f64::NEG_INFINITY),
    };

    /// Undefined as a [`JsValue`] constant, for convenience.
    pub const UNDEFINED: Self = Self {
        inner: InnerValue::Undefined,
    };

    /// Null as a [`JsValue`] constant, for convenience.
    pub const NULL: Self = Self {
        inner: InnerValue::Null,
    };

    /// Create a new [`JsValue`] from an inner value.
    const fn from_inner(inner: InnerValue) -> Self {
        Self { inner }
    }

    /// Create a new [`JsValue`].
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Self>,
    {
        value.into()
    }

    /// Return the variant of this value.
    #[inline]
    #[must_use]
    pub fn variant(&self) -> JsVariant<'_> {
        (&self.inner).into()
    }

    /// Creates a new `undefined` value.
    #[inline]
    #[must_use]
    pub const fn undefined() -> Self {
        Self {
            inner: InnerValue::Undefined,
        }
    }

    /// Creates a new `null` value.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self {
            inner: InnerValue::Null,
        }
    }

    /// Creates a new number with `NaN` value.
    #[inline]
    #[must_use]
    pub const fn nan() -> Self {
        Self::NAN
    }

    /// Creates a new number with `Infinity` value.
    #[inline]
    #[must_use]
    pub const fn positive_infinity() -> Self {
        Self::POSITIVE_INFINITY
    }

    /// Creates a new number with `-Infinity` value.
    #[inline]
    #[must_use]
    pub const fn negative_infinity() -> Self {
        Self::NEGATIVE_INFINITY
    }

    /// Returns true if the value is an object.
    #[inline]
    #[must_use]
    pub const fn is_object(&self) -> bool {
        matches!(self.inner, InnerValue::Object(_))
    }

    /// Returns the object if the value is object, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn as_object(&self) -> Option<&JsObject> {
        if let InnerValue::Object(obj) = &self.inner {
            Some(obj)
        } else {
            None
        }
    }

    /// It determines if the value is a callable function with a `[[Call]]` internal method.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iscallable
    #[inline]
    #[must_use]
    pub fn is_callable(&self) -> bool {
        if let InnerValue::Object(obj) = &self.inner {
            obj.is_callable()
        } else {
            false
        }
    }

    /// Returns the callable value if the value is callable, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn as_callable(&self) -> Option<&JsObject> {
        if let InnerValue::Object(obj) = &self.inner {
            obj.is_callable().then_some(obj)
        } else {
            None
        }
    }

    /// Returns a [`JsFunction`] if the value is callable, otherwise `None`.
    /// This is equivalent to `JsFunction::from_object(value.as_callable()?)`.
    #[inline]
    #[must_use]
    pub fn as_function(&self) -> Option<JsFunction> {
        self.as_callable()
            .cloned()
            .and_then(JsFunction::from_object)
    }

    /// Returns true if the value is a constructor object.
    #[inline]
    #[must_use]
    pub fn is_constructor(&self) -> bool {
        matches!(&self.inner, InnerValue::Object(obj) if obj.is_constructor())
    }

    /// Returns the constructor if the value is a constructor, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn as_constructor(&self) -> Option<&JsObject> {
        self.as_object().filter(|obj| obj.is_constructor())
    }

    /// Returns true if the value is a promise object.
    #[inline]
    #[must_use]
    pub fn is_promise(&self) -> bool {
        if let InnerValue::Object(obj) = &self.inner {
            obj.is::<Promise>()
        } else {
            false
        }
    }

    /// Returns the value as an object if the value is a promise, otherwise `None`.
    #[inline]
    #[must_use]
    pub(crate) fn as_promise_object(&self) -> Option<&JsObject> {
        if let InnerValue::Object(obj) = &self.inner {
            obj.is::<Promise>().then_some(obj)
        } else {
            None
        }
    }

    /// Returns the value as a promise if the value is a promise, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn as_promise(&self) -> Option<JsPromise> {
        self.as_promise_object()
            .cloned()
            .and_then(|o| JsPromise::from_object(o).ok())
    }

    /// Returns true if the value is a regular expression object.
    #[inline]
    #[must_use]
    pub fn is_regexp(&self) -> bool {
        if let InnerValue::Object(obj) = &self.inner {
            obj.is::<RegExp>()
        } else {
            false
        }
    }

    /// Returns the value as a regular expression if the value is a regexp, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn as_regexp(&self) -> Option<JsRegExp> {
        self.as_object()
            .filter(|obj| obj.is::<RegExp>())
            .cloned()
            .and_then(|o| JsRegExp::from_object(o).ok())
    }

    /// Returns true if the value is a symbol.
    #[inline]
    #[must_use]
    pub const fn is_symbol(&self) -> bool {
        matches!(self.inner, InnerValue::Symbol(_))
    }

    /// Returns the symbol if the value is a symbol, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn as_symbol(&self) -> Option<JsSymbol> {
        if let InnerValue::Symbol(symbol) = &self.inner {
            Some(symbol.clone())
        } else {
            None
        }
    }

    /// Returns true if the value is undefined.
    #[inline]
    #[must_use]
    pub const fn is_undefined(&self) -> bool {
        matches!(&self.inner, InnerValue::Undefined)
    }

    /// Returns `()` if the value is undefined, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn as_undefined(&self) -> Option<()> {
        if self.is_undefined() {
            Some(())
        } else {
            None
        }
    }

    /// Returns `Some(self)` if the value is defined, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn as_defined(&self) -> Option<&Self> {
        if self.is_undefined() {
            None
        } else {
            Some(self)
        }
    }

    /// Returns true if the value is null.
    #[inline]
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self.inner, InnerValue::Null)
    }

    /// Returns true if the value is null or undefined.
    #[inline]
    #[must_use]
    pub const fn is_null_or_undefined(&self) -> bool {
        self.is_undefined() || self.is_null()
    }

    /// Determines if argument is a finite integral Number value.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isintegralnumber
    #[must_use]
    #[allow(clippy::float_cmp)]
    pub fn is_integral_number(&self) -> bool {
        // When creating the inner value, we verify that the float is rational or
        // an integer, so we can safely unwrap here.
        matches!(&self.inner, InnerValue::Integer32(_))
    }

    /// Returns the number if the value is an integer, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn as_integer32(&self) -> Option<i32> {
        if let InnerValue::Integer32(i) = self.inner {
            Some(i)
        } else {
            None
        }
    }

    /// Returns true if the value is a number.
    #[inline]
    #[must_use]
    pub const fn is_number(&self) -> bool {
        matches!(
            self.inner,
            InnerValue::Float64(_) | InnerValue::Integer32(_)
        )
    }

    /// Returns the number if the value is a number, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        match self.inner {
            InnerValue::Integer32(integer) => Some(integer.into()),
            InnerValue::Float64(rational) => Some(rational),
            _ => None,
        }
    }

    /// Returns true if the value is a string.
    #[inline]
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self.inner, InnerValue::String(_))
    }

    /// Returns the string if the value is a string, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn as_string(&self) -> Option<&JsString> {
        if let InnerValue::String(string) = &self.inner {
            Some(string)
        } else {
            None
        }
    }

    /// Returns true if the value is a boolean.
    #[inline]
    #[must_use]
    pub const fn is_boolean(&self) -> bool {
        matches!(self.inner, InnerValue::Boolean(_))
    }

    /// Returns the boolean if the value is a boolean, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn as_boolean(&self) -> Option<bool> {
        match &self.inner {
            InnerValue::Boolean(boolean) => Some(*boolean),
            _ => None,
        }
    }

    /// Returns true if the value is a bigint.
    #[inline]
    #[must_use]
    pub const fn is_bigint(&self) -> bool {
        matches!(self.inner, InnerValue::BigInt(_))
    }

    /// Returns an optional reference to a `BigInt` if the value is a `BigInt` primitive.
    #[inline]
    #[must_use]
    pub const fn as_bigint(&self) -> Option<&JsBigInt> {
        match &self.inner {
            InnerValue::BigInt(bigint) => Some(bigint),
            _ => None,
        }
    }

    /// Converts the value to a `bool` type.
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-toboolean
    #[must_use]
    pub fn to_boolean(&self) -> bool {
        match self.inner {
            InnerValue::Symbol(_) | InnerValue::Object(_) => true,
            InnerValue::String(ref s) if !s.is_empty() => true,
            InnerValue::Float64(n) if !n.is_nan() => true,
            InnerValue::Integer32(n) if n != 0 => true,
            InnerValue::BigInt(ref n) if !n.is_zero() => true,
            InnerValue::Boolean(v) => v,
            _ => false,
        }
    }

    /// The abstract operation `ToPrimitive` takes an input argument and an optional argument
    /// `PreferredType`.
    ///
    /// <https://tc39.es/ecma262/#sec-toprimitive>
    pub fn to_primitive(
        &self,
        context: &mut Context,
        preferred_type: PreferredType,
    ) -> JsResult<Self> {
        // 1. Assert: input is an ECMAScript language value. (always a value not need to check)
        // 2. If Type(input) is Object, then
        if let Some(input) = self.as_object() {
            // a. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
            let exotic_to_prim = input.get_method(JsSymbol::to_primitive(), context)?;

            // b. If exoticToPrim is not undefined, then
            if let Some(exotic_to_prim) = exotic_to_prim {
                // i. If preferredType is not present, let hint be "default".
                // ii. Else if preferredType is string, let hint be "string".
                // iii. Else,
                //     1. Assert: preferredType is number.
                //     2. Let hint be "number".
                let hint = match preferred_type {
                    PreferredType::Default => js_string!("default"),
                    PreferredType::String => js_string!("string"),
                    PreferredType::Number => js_string!("number"),
                }
                    .into();

                // iv. Let result be ? Call(exoticToPrim, input, « hint »).
                let result = exotic_to_prim.call(self, &[hint], context)?;
                // v. If Type(result) is not Object, return result.
                // vi. Throw a TypeError exception.
                return if result.is_object() {
                    Err(JsNativeError::typ()
                        .with_message("Symbol.toPrimitive cannot return an object")
                        .into())
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
            return input.ordinary_to_primitive(context, preferred_type);
        }

        // 3. Return input.
        Ok(self.clone())
    }

    /// `7.1.13 ToBigInt ( argument )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-tobigint
    pub fn to_bigint(&self, context: &mut Context) -> JsResult<JsBigInt> {
        match &self.inner {
            InnerValue::Null => Err(JsNativeError::typ()
                .with_message("cannot convert null to a BigInt")
                .into()),
            InnerValue::Undefined => Err(JsNativeError::typ()
                .with_message("cannot convert undefined to a BigInt")
                .into()),
            InnerValue::String(ref string) => JsBigInt::from_js_string(string).map_or_else(
                || {
                    Err(JsNativeError::syntax()
                        .with_message(format!(
                            "cannot convert string '{}' to bigint primitive",
                            string.to_std_string_escaped()
                        ))
                        .into())
                },
                Ok,
            ),
            InnerValue::Boolean(true) => Ok(JsBigInt::one()),
            InnerValue::Boolean(false) => Ok(JsBigInt::zero()),
            InnerValue::Integer32(_) | InnerValue::Float64(_) => Err(JsNativeError::typ()
                .with_message("cannot convert Number to a BigInt")
                .into()),
            InnerValue::BigInt(b) => Ok(b.clone()),
            InnerValue::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::Number)?;
                primitive.to_bigint(context)
            }
            InnerValue::Symbol(_) => Err(JsNativeError::typ()
                .with_message("cannot convert Symbol to a BigInt")
                .into()),
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
    #[must_use]
    #[inline]
    pub const fn display(&self) -> ValueDisplay<'_> {
        ValueDisplay {
            value: self,
            internals: false,
        }
    }

    /// Converts the value to a string.
    ///
    /// This function is equivalent to `String(value)` in JavaScript.
    pub fn to_string(&self, context: &mut Context) -> JsResult<JsString> {
        match self.inner {
            InnerValue::Null => Ok(js_string!("null")),
            InnerValue::Undefined => Ok(js_string!("undefined")),
            InnerValue::Boolean(true) => Ok(js_string!("true")),
            InnerValue::Boolean(false) => Ok(js_string!("false")),
            InnerValue::Float64(rational) => Ok(Number::to_js_string_radix(rational, 10)),
            InnerValue::Integer32(integer) => Ok(integer.to_string().into()),
            InnerValue::String(ref string) => Ok(string.clone()),
            InnerValue::Symbol(_) => Err(JsNativeError::typ()
                .with_message("can't convert symbol to string")
                .into()),
            InnerValue::BigInt(ref bigint) => Ok(bigint.to_string().into()),
            InnerValue::Object(_) => {
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
        match &self.inner {
            InnerValue::Undefined | InnerValue::Null => Err(JsNativeError::typ()
                .with_message("cannot convert 'null' or 'undefined' to object")
                .into()),
            InnerValue::Boolean(boolean) => Ok(context
                .intrinsics()
                .templates()
                .boolean()
                .create(*boolean, Vec::default())),
            InnerValue::Integer32(integer) => Ok(context
                .intrinsics()
                .templates()
                .number()
                .create(f64::from(*integer), Vec::default())),
            InnerValue::Float64(rational) => Ok(context
                .intrinsics()
                .templates()
                .number()
                .create(*rational, Vec::default())),
            InnerValue::String(ref string) => Ok(context
                .intrinsics()
                .templates()
                .string()
                .create(string.clone(), vec![string.len().into()])),
            InnerValue::Symbol(ref symbol) => Ok(context
                .intrinsics()
                .templates()
                .symbol()
                .create(symbol.clone(), Vec::default())),
            InnerValue::BigInt(ref bigint) => Ok(context
                .intrinsics()
                .templates()
                .bigint()
                .create(bigint.clone(), Vec::default())),
            InnerValue::Object(jsobject) => Ok(jsobject.clone()),
        }
    }

    /// Converts the value to a `PropertyKey`, that can be used as a key for properties.
    ///
    /// See <https://tc39.es/ecma262/#sec-topropertykey>
    pub fn to_property_key(&self, context: &mut Context) -> JsResult<PropertyKey> {
        Ok(match &self.inner {
            // Fast path:
            InnerValue::String(string) => string.clone().into(),
            InnerValue::Symbol(symbol) => symbol.clone().into(),
            InnerValue::Integer32(integer) => (*integer).into(),
            // Slow path:
            InnerValue::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::String)?;
                match primitive.inner {
                    InnerValue::String(ref string) => string.clone().into(),
                    InnerValue::Symbol(ref symbol) => symbol.clone().into(),
                    InnerValue::Integer32(integer) => integer.into(),
                    _ => primitive.to_string(context)?.into(),
                }
            }
            _ => self.to_string(context)?.into(),
        })
    }

    /// It returns value converted to a numeric value of type `Number` or `BigInt`.
    ///
    /// See: <https://tc39.es/ecma262/#sec-tonumeric>
    pub fn to_numeric(&self, context: &mut Context) -> JsResult<Numeric> {
        // 1. Let primValue be ? ToPrimitive(value, number).
        let primitive = self.to_primitive(context, PreferredType::Number)?;

        // 2. If primValue is a BigInt, return primValue.
        if let Some(bigint) = primitive.as_bigint() {
            return Ok(bigint.clone().into());
        }

        // 3. Return ? ToNumber(primValue).
        Ok(primitive.to_number(context)?.into())
    }

    /// Converts a value to an integral 32-bit unsigned integer.
    ///
    /// This function is equivalent to `value | 0` in JavaScript
    ///
    /// See: <https://tc39.es/ecma262/#sec-touint32>
    pub fn to_u32(&self, context: &mut Context) -> JsResult<u32> {
        // This is the fast path, if the value is Integer we can just return it.
        if let InnerValue::Integer32(number) = self.inner {
            if let Ok(number) = u32::try_from(number) {
                return Ok(number);
            }
        }
        let number = self.to_number(context)?;

        Ok(f64_to_uint32(number))
    }

    /// Converts a value to an integral 32-bit signed integer.
    ///
    /// See: <https://tc39.es/ecma262/#sec-toint32>
    pub fn to_i32(&self, context: &mut Context) -> JsResult<i32> {
        // This is the fast path, if the value is Integer we can just return it.
        if let InnerValue::Integer32(number) = self.inner {
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
    pub fn to_big_int64(&self, context: &mut Context) -> JsResult<i64> {
        // 1. Let n be ? ToBigInt(argument).
        let n = self.to_bigint(context)?;

        // 2. Let int64bit be ℝ(n) modulo 2^64.
        let int64_bit = n.as_inner().mod_floor(&TWO_E_64);

        // 3. If int64bit ≥ 2^63, return ℤ(int64bit - 2^64); otherwise return ℤ(int64bit).
        let value = if int64_bit >= *TWO_E_63 {
            int64_bit.sub(&*TWO_E_64)
        } else {
            int64_bit
        };

        Ok(value
            .to_i64()
            .expect("should be within the range of `i64` by the mod operation"))
    }

    /// `7.1.16 ToBigUint64 ( argument )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-tobiguint64
    pub fn to_big_uint64(&self, context: &mut Context) -> JsResult<u64> {
        // 1. Let n be ? ToBigInt(argument).
        let n = self.to_bigint(context)?;

        // 2. Let int64bit be ℝ(n) modulo 2^64.
        // 3. Return ℤ(int64bit).
        Ok(n.as_inner()
            .mod_floor(&TWO_E_64)
            .to_u64()
            .expect("should be within the range of `u64` by the mod operation"))
    }

    /// Converts a value to a non-negative integer if it is a valid integer index value.
    ///
    /// See: <https://tc39.es/ecma262/#sec-toindex>
    pub fn to_index(&self, context: &mut Context) -> JsResult<u64> {
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
            return Err(JsNativeError::range()
                .with_message("Index must be between 0 and  2^53 - 1")
                .into());
        }

        // d. Assert: 0 ≤ integer ≤ 2^53 - 1.
        debug_assert!(0 <= clamped && clamped <= Number::MAX_SAFE_INTEGER as i64);

        // e. Return integer.
        Ok(clamped as u64)
    }

    /// Converts argument to an integer suitable for use as the length of an array-like object.
    ///
    /// See: <https://tc39.es/ecma262/#sec-tolength>
    pub fn to_length(&self, context: &mut Context) -> JsResult<u64> {
        // 1. Let len be ? ToInteger(argument).
        // 2. If len ≤ +0, return +0.
        // 3. Return min(len, 2^53 - 1).
        Ok(self
            .to_integer_or_infinity(context)?
            .clamp_finite(0, Number::MAX_SAFE_INTEGER as i64) as u64)
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

        // Continues on `IntegerOrInfinity::from::<f64>`
        Ok(IntegerOrInfinity::from(number))
    }

    /// Converts a value to a double precision floating point.
    ///
    /// This function is equivalent to the unary `+` operator (`+value`) in JavaScript
    ///
    /// See: <https://tc39.es/ecma262/#sec-tonumber>
    pub fn to_number(&self, context: &mut Context) -> JsResult<f64> {
        match self.inner {
            InnerValue::Null => Ok(0.0),
            InnerValue::Undefined => Ok(f64::NAN),
            InnerValue::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            InnerValue::String(ref string) => Ok(string.to_number()),
            InnerValue::Float64(number) => Ok(number),
            InnerValue::Integer32(integer) => Ok(f64::from(integer)),
            InnerValue::Symbol(_) => Err(JsNativeError::typ()
                .with_message("argument must not be a symbol")
                .into()),
            InnerValue::BigInt(_) => Err(JsNativeError::typ()
                .with_message("argument must not be a bigint")
                .into()),
            InnerValue::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::Number)?;
                primitive.to_number(context)
            }
        }
    }

    /// Converts a value to a 32 bit floating point.
    pub fn to_f32(&self, context: &mut Context) -> JsResult<f32> {
        self.to_number(context).map(|n| n as f32)
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
    pub fn require_object_coercible(&self) -> JsResult<&Self> {
        if self.is_null_or_undefined() {
            Err(JsNativeError::typ()
                .with_message("cannot convert null or undefined to Object")
                .into())
        } else {
            Ok(self)
        }
    }

    /// The abstract operation `ToPropertyDescriptor`.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-topropertydescriptor
    #[inline]
    pub fn to_property_descriptor(&self, context: &mut Context) -> JsResult<PropertyDescriptor> {
        // 1. If Type(Obj) is not Object, throw a TypeError exception.
        self.as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("Cannot construct a property descriptor from a non-object")
                    .into()
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
    #[must_use]
    pub fn type_of(&self) -> &'static str {
        self.variant().type_of()
    }

    /// Same as [`JsValue::type_of`], but returning a [`JsString`] instead.
    #[must_use]
    pub fn js_type_of(&self) -> JsString {
        self.variant().js_type_of()
    }

    /// Maps a `JsValue` into a `Option<T>` where T is the result of an
    /// operation on a defined value. If the value is `JsValue::undefined`,
    /// then `JsValue::map` will return None.
    ///
    /// # Example
    ///
    /// ```
    /// use boa_engine::{JsValue, Context};
    ///
    /// let mut context = Context::default();
    ///
    /// let defined_value = JsValue::from(5);
    /// let undefined = JsValue::undefined();
    ///
    /// let defined_result = defined_value.map(|v| v.add(&JsValue::from(5), &mut context)).transpose().unwrap();
    /// let undefined_result = undefined.map(|v| v.add(&JsValue::from(5), &mut context)).transpose().unwrap();
    ///
    /// assert_eq!(defined_result, Some(JsValue::from(10u8)));
    /// assert_eq!(undefined_result, None);
    ///
    /// ```
    ///
    #[inline]
    #[must_use]
    pub fn map<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&JsValue) -> T,
    {
        if self.is_undefined() {
            return None;
        }
        Some(f(self))
    }

    /// Abstract operation `IsArray ( argument )`
    ///
    /// Check if a value is an array.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isarray
    pub(crate) fn is_array(&self) -> JsResult<bool> {
        // Note: The spec specifies this function for JsValue.
        // The main part of the function is implemented for JsObject.

        // 1. If Type(argument) is not Object, return false.
        self.as_object()
            .map_or(Ok(false), JsObject::is_array_abstract)
    }
}

impl Default for JsValue {
    fn default() -> Self {
        Self::undefined()
    }
}

/// The preferred type to convert an object to a primitive `Value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PreferredType {
    /// Prefer to convert to a `String` primitive.
    String,

    /// Prefer to convert to a `Number` primitive.
    Number,

    /// Do not prefer a type to convert to.
    Default,
}

/// Numeric value which can be of two types `Number`, `BigInt`.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Numeric {
    /// Double precision floating point number.
    Number(f64),
    /// `BigInt` an integer of arbitrary size.
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
