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

use boa_gc::{Finalize, Trace};
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
mod inner;
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

/// A generic Javascript value. This can be any ECMAScript language valid value.
///
/// This is a wrapper around the actual value, which is stored in an opaque type.
/// This allows for internal changes to the value without affecting the public API.
///
/// ```
/// # use boa_engine::{js_string, Context, JsValue};
/// let mut context = Context::default();
/// let value = JsValue::new(3);
/// assert_eq!(value.to_string(&mut context), Ok(js_string!("3")));
/// ```
#[derive(Finalize, Debug, Clone, Trace)]
pub struct JsValue(inner::InnerValue);

impl JsValue {
    /// Create a new [`JsValue`] from an inner value.
    const fn from_inner(inner: inner::InnerValue) -> Self {
        Self(inner)
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
    pub const fn variant(&self) -> JsVariant<'_> {
        self.0.as_variant()
    }

    /// Creates a new `undefined` value.
    #[inline]
    #[must_use]
    pub const fn undefined() -> Self {
        Self::from_inner(inner::InnerValue::undefined())
    }

    /// Creates a new `null` value.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self::from_inner(inner::InnerValue::null())
    }

    /// Creates a new number with `NaN` value.
    #[inline]
    #[must_use]
    pub const fn nan() -> Self {
        Self::from_inner(inner::InnerValue::float64(f64::NAN))
    }

    /// Creates a new number with `Infinity` value.
    #[inline]
    #[must_use]
    pub const fn positive_infinity() -> Self {
        Self::from_inner(inner::InnerValue::float64(f64::INFINITY))
    }

    /// Creates a new number with `-Infinity` value.
    #[inline]
    #[must_use]
    pub const fn negative_infinity() -> Self {
        Self::from_inner(inner::InnerValue::float64(f64::NEG_INFINITY))
    }

    /// Creates a new number from a float.
    #[inline]
    #[must_use]
    pub const fn rational(rational: f64) -> Self {
        Self::from_inner(inner::InnerValue::float64(rational))
    }

    /// Returns true if the value is an object.
    #[inline]
    #[must_use]
    pub const fn is_object(&self) -> bool {
        self.0.is_object()
    }

    /// Returns the object if the value is object, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn as_object(&self) -> Option<&JsObject> {
        self.0.as_object()
    }

    /// Consumes the value and return the inner object if it was an object.
    #[inline]
    #[must_use]
    pub fn into_object(self) -> Option<JsObject> {
        self.0.as_object().cloned()
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
        self.as_object().map_or(false, |obj| obj.is_callable())
    }

    /// Returns the callable value if the value is callable, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn as_callable(&self) -> Option<&JsObject> {
        self.as_object().filter(|obj| obj.is_callable())
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
        self.as_object().map_or(false, |obj| obj.is_constructor())
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
        self.as_object().map_or(false, |obj| obj.is::<Promise>())
    }

    /// Returns the value as an object if the value is a promise, otherwise `None`.
    #[inline]
    #[must_use]
    pub(crate) fn as_promise_object(&self) -> Option<&JsObject> {
        self.as_object().filter(|obj| obj.is::<Promise>())
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
        self.as_object().map_or(false, |obj| obj.is::<RegExp>())
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
        self.0.is_symbol()
    }

    /// Returns the symbol if the value is a symbol, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn as_symbol(&self) -> Option<JsSymbol> {
        if let Some(symbol) = self.0.as_symbol() {
            Some(symbol.clone())
        } else {
            None
        }
    }

    /// Returns true if the value is undefined.
    #[inline]
    #[must_use]
    pub const fn is_undefined(&self) -> bool {
        self.0.is_undefined()
    }

    /// Returns true if the value is null.
    #[inline]
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.0.is_null()
    }

    /// Returns true if the value is null or undefined.
    #[inline]
    #[must_use]
    pub const fn is_null_or_undefined(&self) -> bool {
        self.is_undefined() || self.is_null()
    }

    /// Returns the number if the value is a finite integral Number value, otherwise `None`.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isintegralnumber
    #[inline]
    #[must_use]
    #[allow(clippy::float_cmp)]
    pub const fn as_i32(&self) -> Option<i32> {
        if let Some(integer) = self.0.as_integer32() {
            Some(integer)
        } else if let Some(rational) = self.0.as_float64() {
            // Use this poor-man's check as `[f64::fract]` isn't const.
            if rational == ((rational as i32) as f64) {
                Some(rational as i32)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns true if the value is a number.
    #[inline]
    #[must_use]
    pub const fn is_number(&self) -> bool {
        self.0.is_integer32() || self.0.is_float64()
    }

    /// Returns the number if the value is a number, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        if let Some(i) = self.as_i32() {
            Some(i as f64)
        } else {
            self.0.as_float64()
        }
    }

    /// Returns true if the value is a string.
    #[inline]
    #[must_use]
    pub const fn is_string(&self) -> bool {
        self.0.is_string()
    }

    /// Returns the string if the value is a string, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn as_string(&self) -> Option<&JsString> {
        self.0.as_string()
    }

    /// Returns true if the value is a boolean.
    #[inline]
    #[must_use]
    pub const fn is_boolean(&self) -> bool {
        self.0.is_bool()
    }

    /// Returns the boolean if the value is a boolean, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn as_boolean(&self) -> Option<bool> {
        self.0.as_bool()
    }

    /// Returns true if the value is a bigint.
    #[inline]
    #[must_use]
    pub const fn is_bigint(&self) -> bool {
        self.0.is_bigint()
    }

    /// Returns an optional reference to a `BigInt` if the value is a `BigInt` primitive.
    #[inline]
    #[must_use]
    pub const fn as_bigint(&self) -> Option<&JsBigInt> {
        self.0.as_bigint()
    }

    /// Converts the value to a `bool` type.
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-toboolean
    #[must_use]
    pub fn to_boolean(&self) -> bool {
        match self.variant() {
            JsVariant::Symbol(_) | JsVariant::Object(_) => true,
            JsVariant::String(s) if !s.is_empty() => true,
            JsVariant::Float64(n) if n != 0.0 && !n.is_nan() => true,
            JsVariant::Integer32(n) if n != 0 => true,
            JsVariant::BigInt(n) if !n.is_zero() => true,
            JsVariant::Boolean(v) => v,
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
        match self.variant() {
            JsVariant::Null => Err(JsNativeError::typ()
                .with_message("cannot convert null to a BigInt")
                .into()),
            JsVariant::Undefined => Err(JsNativeError::typ()
                .with_message("cannot convert undefined to a BigInt")
                .into()),
            JsVariant::String(ref string) => JsBigInt::from_js_string(string).map_or_else(
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
            JsVariant::Boolean(true) => Ok(JsBigInt::one()),
            JsVariant::Boolean(false) => Ok(JsBigInt::zero()),
            JsVariant::Integer32(_) | JsVariant::Float64(_) => Err(JsNativeError::typ()
                .with_message("cannot convert Number to a BigInt")
                .into()),
            JsVariant::BigInt(b) => Ok(b.clone()),
            JsVariant::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::Number)?;
                primitive.to_bigint(context)
            }
            JsVariant::Symbol(_) => Err(JsNativeError::typ()
                .with_message("cannot convert Symbol to a BigInt")
                .into()),
        }
    }

    /// Returns an object that implements `Display`.
    ///
    /// By default, the internals are not shown, but they can be toggled
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
        match self.variant() {
            JsVariant::Null => Ok(js_string!("null")),
            JsVariant::Undefined => Ok(js_string!("undefined")),
            JsVariant::Boolean(true) => Ok(js_string!("true")),
            JsVariant::Boolean(false) => Ok(js_string!("false")),
            JsVariant::Float64(rational) => Ok(JsString::from(rational)),
            JsVariant::Integer32(integer) => Ok(JsString::from(integer)),
            JsVariant::String(string) => Ok(string.clone()),
            JsVariant::Symbol(_) => Err(JsNativeError::typ()
                .with_message("can't convert symbol to string")
                .into()),
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
            JsVariant::Undefined | JsVariant::Null => Err(JsNativeError::typ()
                .with_message("cannot convert 'null' or 'undefined' to object")
                .into()),
            JsVariant::Boolean(boolean) => Ok(context
                .intrinsics()
                .templates()
                .boolean()
                .create(boolean, Vec::default())),
            JsVariant::Integer32(integer) => Ok(context
                .intrinsics()
                .templates()
                .number()
                .create(f64::from(integer), Vec::default())),
            JsVariant::Float64(rational) => Ok(context
                .intrinsics()
                .templates()
                .number()
                .create(rational, Vec::default())),
            JsVariant::String(string) => Ok(context
                .intrinsics()
                .templates()
                .string()
                .create(string.clone(), vec![string.len().into()])),
            JsVariant::Symbol(symbol) => Ok(context
                .intrinsics()
                .templates()
                .symbol()
                .create(symbol.clone(), Vec::default())),
            JsVariant::BigInt(bigint) => Ok(context
                .intrinsics()
                .templates()
                .bigint()
                .create(bigint.clone(), Vec::default())),
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
            JsVariant::Integer32(integer) => integer.into(),
            // Slow path:
            JsVariant::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::String)?;
                match primitive.variant() {
                    JsVariant::String(string) => string.clone().into(),
                    JsVariant::Symbol(symbol) => symbol.clone().into(),
                    JsVariant::Integer32(integer) => integer.into(),
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
        if let Some(number) = self.0.as_integer32() {
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
        if let Some(number) = self.0.as_integer32() {
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
        match self.variant() {
            JsVariant::Null => Ok(0.0),
            JsVariant::Undefined => Ok(f64::NAN),
            JsVariant::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            JsVariant::String(string) => Ok(string.to_number()),
            JsVariant::Float64(number) => Ok(number),
            JsVariant::Integer32(integer) => Ok(f64::from(integer)),
            JsVariant::Symbol(_) => Err(JsNativeError::typ()
                .with_message("argument must not be a symbol")
                .into()),
            JsVariant::BigInt(_) => Err(JsNativeError::typ()
                .with_message("argument must not be a bigint")
                .into()),
            JsVariant::Object(_) => {
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

    /// Maps a `JsValue` into `Option<T>` where T is the result of an
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

    /// Maps a `JsValue` into `T` where T is the result of an
    /// operation on a defined value. If the value is `JsValue::undefined`,
    /// then `JsValue::map` will return the provided default value.
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
    /// let defined_result = defined_value
    ///     .map_or(Ok(JsValue::new(true)), |v| v.add(&JsValue::from(5), &mut context))
    ///     .unwrap();
    /// let undefined_result = undefined
    ///     .map_or(Ok(JsValue::new(true)), |v| v.add(&JsValue::from(5), &mut context))
    ///     .unwrap();
    ///
    /// assert_eq!(defined_result, JsValue::new(10));
    /// assert_eq!(undefined_result, JsValue::new(true));
    ///
    /// ```
    #[inline]
    #[must_use]
    pub fn map_or<T, F>(&self, default: T, f: F) -> T
    where
        F: FnOnce(&JsValue) -> T,
    {
        if self.is_undefined() {
            return default;
        }
        f(self)
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
