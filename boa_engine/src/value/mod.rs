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
    error::JsNativeError,
    js_string,
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyKey},
    symbol::{JsSymbol, WellKnownSymbols},
    Context, JsBigInt, JsResult, JsString,
};
use boa_gc::{custom_trace, Finalize, Trace};
use boa_profiler::Profiler;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::Zero;
use once_cell::sync::Lazy;
use std::{
    collections::HashSet,
    fmt::{self, Display},
    ops::Sub,
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

/// A Javascript value
#[derive(Finalize, Debug, Clone)]
pub enum JsValue {
    /// `null` - A null value, for when a value doesn't exist.
    Null,
    /// `undefined` - An undefined value, for when a field or index doesn't exist.
    Undefined,
    /// `boolean` - A `true` / `false` value, for if a certain criteria is met.
    Boolean(bool),
    /// `String` - A UTF-16 string, such as `"Hello, world"`.
    String(JsString),
    /// `Number` - A 64-bit floating point number, such as `3.1415`
    Rational(f64),
    /// `Number` - A 32-bit integer, such as `42`.
    Integer(i32),
    /// `BigInt` - holds any arbitrary large signed integer.
    BigInt(JsBigInt),
    /// `Object` - An object, such as `Math`, represented by a binary tree of string keys to Javascript values.
    Object(JsObject),
    /// `Symbol` - A Symbol Primitive type.
    Symbol(JsSymbol),
}

unsafe impl Trace for JsValue {
    custom_trace! {this, {
        if let Self::Object(o) = this {
            mark(o);
        }
    }}
}

impl JsValue {
    /// Create a new [`JsValue`].
    #[inline]
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Self>,
    {
        value.into()
    }

    /// Creates a new `undefined` value.
    #[inline]
    pub fn undefined() -> Self {
        Self::Undefined
    }

    /// Creates a new `null` value.
    #[inline]
    pub fn null() -> Self {
        Self::Null
    }

    /// Creates a new number with `NaN` value.
    #[inline]
    pub fn nan() -> Self {
        Self::Rational(f64::NAN)
    }

    /// Creates a new number with `Infinity` value.
    #[inline]
    pub fn positive_infinity() -> Self {
        Self::Rational(f64::INFINITY)
    }

    /// Creates a new number with `-Infinity` value.
    #[inline]
    pub fn negative_infinity() -> Self {
        Self::Rational(f64::NEG_INFINITY)
    }

    /// Returns true if the value is an object
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    #[inline]
    pub fn as_object(&self) -> Option<&JsObject> {
        match *self {
            Self::Object(ref o) => Some(o),
            _ => None,
        }
    }

    /// It determines if the value is a callable function with a `[[Call]]` internal method.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iscallable
    #[inline]
    pub fn is_callable(&self) -> bool {
        matches!(self, Self::Object(obj) if obj.is_callable())
    }

    #[inline]
    pub fn as_callable(&self) -> Option<&JsObject> {
        self.as_object().filter(|obj| obj.is_callable())
    }

    /// Returns true if the value is a constructor object.
    #[inline]
    pub fn is_constructor(&self) -> bool {
        matches!(self, Self::Object(obj) if obj.is_constructor())
    }

    #[inline]
    pub fn as_constructor(&self) -> Option<&JsObject> {
        self.as_object().filter(|obj| obj.is_constructor())
    }

    /// Returns true if the value is a promise object.
    #[inline]
    pub fn is_promise(&self) -> bool {
        matches!(self, Self::Object(obj) if obj.is_promise())
    }

    #[inline]
    pub fn as_promise(&self) -> Option<&JsObject> {
        self.as_object().filter(|obj| obj.is_promise())
    }

    /// Returns true if the value is a symbol.
    #[inline]
    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(_))
    }

    pub fn as_symbol(&self) -> Option<JsSymbol> {
        match self {
            Self::Symbol(symbol) => Some(symbol.clone()),
            _ => None,
        }
    }

    /// Returns true if the value is undefined.
    #[inline]
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }

    /// Returns true if the value is null.
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns true if the value is null or undefined.
    #[inline]
    pub fn is_null_or_undefined(&self) -> bool {
        matches!(self, Self::Null | Self::Undefined)
    }

    /// Returns true if the value is a 64-bit floating-point number.
    #[inline]
    pub fn is_double(&self) -> bool {
        matches!(self, Self::Rational(_))
    }

    /// Returns true if the value is integer.
    #[inline]
    #[allow(clippy::float_cmp)]
    pub fn is_integer(&self) -> bool {
        // If it can fit in a i32 and the truncated version is
        // equal to the original then it is an integer.
        let is_rational_integer = |n: f64| n == f64::from(n as i32);

        match *self {
            Self::Integer(_) => true,
            Self::Rational(n) if is_rational_integer(n) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a number.
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Rational(_) | Self::Integer(_))
    }

    #[inline]
    pub fn as_number(&self) -> Option<f64> {
        match *self {
            Self::Integer(integer) => Some(integer.into()),
            Self::Rational(rational) => Some(rational),
            _ => None,
        }
    }

    /// Returns true if the value is a string.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns the string if the values is a string, otherwise `None`.
    #[inline]
    pub fn as_string(&self) -> Option<&JsString> {
        match self {
            Self::String(ref string) => Some(string),
            _ => None,
        }
    }

    /// Returns true if the value is a boolean.
    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    #[inline]
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(boolean) => Some(*boolean),
            _ => None,
        }
    }

    /// Returns true if the value is a bigint.
    #[inline]
    pub fn is_bigint(&self) -> bool {
        matches!(self, Self::BigInt(_))
    }

    /// Returns an optional reference to a `BigInt` if the value is a `BigInt` primitive.
    #[inline]
    pub fn as_bigint(&self) -> Option<&JsBigInt> {
        match self {
            Self::BigInt(bigint) => Some(bigint),
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
        match *self {
            Self::Symbol(_) | Self::Object(_) => true,
            Self::String(ref s) if !s.is_empty() => true,
            Self::Rational(n) if n != 0.0 && !n.is_nan() => true,
            Self::Integer(n) if n != 0 => true,
            Self::BigInt(ref n) if !n.is_zero() => true,
            Self::Boolean(v) => v,
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
        match self {
            Self::Object(ref object) => {
                // TODO: had to skip `__get_own_properties__` since we don't have context here
                let property = object.borrow().properties().get(&key);
                if property.is_some() {
                    return property;
                }

                object
                    .prototype()
                    .as_ref()
                    .map_or(Self::Null, |obj| obj.clone().into())
                    .get_property(key)
            }
            _ => None,
        }
    }

    /// Set the kind of an object.
    #[inline]
    pub fn set_data(&self, data: ObjectData) {
        if let Self::Object(ref obj) = *self {
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
        if let Some(input) = self.as_object() {
            // a. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
            let exotic_to_prim = input.get_method(WellKnownSymbols::to_primitive(), context)?;

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
        match self {
            Self::Null => Err(JsNativeError::typ()
                .with_message("cannot convert null to a BigInt")
                .into()),
            Self::Undefined => Err(JsNativeError::typ()
                .with_message("cannot convert undefined to a BigInt")
                .into()),
            Self::String(ref string) => {
                if let Some(value) = string.to_big_int() {
                    Ok(value)
                } else {
                    Err(JsNativeError::syntax()
                        .with_message(format!(
                            "cannot convert string '{}' to bigint primitive",
                            string.to_std_string_escaped()
                        ))
                        .into())
                }
            }
            Self::Boolean(true) => Ok(JsBigInt::one()),
            Self::Boolean(false) => Ok(JsBigInt::zero()),
            Self::Integer(_) | Self::Rational(_) => Err(JsNativeError::typ()
                .with_message("cannot convert Number to a BigInt")
                .into()),
            Self::BigInt(b) => Ok(b.clone()),
            Self::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::Number)?;
                primitive.to_bigint(context)
            }
            Self::Symbol(_) => Err(JsNativeError::typ()
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
        match self {
            Self::Null => Ok("null".into()),
            Self::Undefined => Ok("undefined".into()),
            Self::Boolean(boolean) => Ok(boolean.to_string().into()),
            Self::Rational(rational) => Ok(Number::to_native_string(*rational).into()),
            Self::Integer(integer) => Ok(integer.to_string().into()),
            Self::String(string) => Ok(string.clone()),
            Self::Symbol(_) => Err(JsNativeError::typ()
                .with_message("can't convert symbol to string")
                .into()),
            Self::BigInt(ref bigint) => Ok(bigint.to_string().into()),
            Self::Object(_) => {
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
        match self {
            Self::Undefined | Self::Null => Err(JsNativeError::typ()
                .with_message("cannot convert 'null' or 'undefined' to object")
                .into()),
            Self::Boolean(boolean) => {
                let prototype = context.intrinsics().constructors().boolean().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::boolean(*boolean),
                ))
            }
            Self::Integer(integer) => {
                let prototype = context.intrinsics().constructors().number().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::number(f64::from(*integer)),
                ))
            }
            Self::Rational(rational) => {
                let prototype = context.intrinsics().constructors().number().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::number(*rational),
                ))
            }
            Self::String(ref string) => {
                let prototype = context.intrinsics().constructors().string().prototype();

                let object =
                    JsObject::from_proto_and_data(prototype, ObjectData::string(string.clone()));
                // Make sure the correct length is set on our new string object
                object.insert_property(
                    js_string!("length"),
                    PropertyDescriptor::builder()
                        .value(string.len())
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                );
                Ok(object)
            }
            Self::Symbol(ref symbol) => {
                let prototype = context.intrinsics().constructors().symbol().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::symbol(symbol.clone()),
                ))
            }
            Self::BigInt(ref bigint) => {
                let prototype = context
                    .intrinsics()
                    .constructors()
                    .bigint_object()
                    .prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::big_int(bigint.clone()),
                ))
            }
            Self::Object(jsobject) => Ok(jsobject.clone()),
        }
    }

    /// Converts the value to a `PropertyKey`, that can be used as a key for properties.
    ///
    /// See <https://tc39.es/ecma262/#sec-topropertykey>
    pub fn to_property_key(&self, context: &mut Context) -> JsResult<PropertyKey> {
        Ok(match self {
            // Fast path:
            Self::String(string) => string.clone().into(),
            Self::Symbol(symbol) => symbol.clone().into(),
            Self::Integer(integer) => (*integer).into(),
            // Slow path:
            Self::Object(_) => match self.to_primitive(context, PreferredType::String)? {
                Self::String(ref string) => string.clone().into(),
                Self::Symbol(ref symbol) => symbol.clone().into(),
                Self::Integer(integer) => integer.into(),
                primitive => primitive.to_string(context)?.into(),
            },
            primitive => primitive.to_string(context)?.into(),
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

    /// Converts a value to an integral 32 bit unsigned integer.
    ///
    /// This function is equivalent to `value | 0` in JavaScript
    ///
    /// See: <https://tc39.es/ecma262/#sec-touint32>
    pub fn to_u32(&self, context: &mut Context) -> JsResult<u32> {
        // This is the fast path, if the value is Integer we can just return it.
        if let JsValue::Integer(number) = *self {
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
        if let JsValue::Integer(number) = *self {
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
        match *self {
            Self::Null => Ok(0.0),
            Self::Undefined => Ok(f64::NAN),
            Self::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            Self::String(ref string) => Ok(string.to_number()),
            Self::Rational(number) => Ok(number),
            Self::Integer(integer) => Ok(f64::from(integer)),
            Self::Symbol(_) => Err(JsNativeError::typ()
                .with_message("argument must not be a symbol")
                .into()),
            Self::BigInt(_) => Err(JsNativeError::typ()
                .with_message("argument must not be a bigint")
                .into()),
            Self::Object(_) => {
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
    pub fn require_object_coercible(&self) -> JsResult<&Self> {
        if self.is_null_or_undefined() {
            Err(JsNativeError::typ()
                .with_message("cannot convert null or undefined to Object")
                .into())
        } else {
            Ok(self)
        }
    }

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
    pub fn type_of(&self) -> &'static str {
        match *self {
            Self::Rational(_) | Self::Integer(_) => "number",
            Self::String(_) => "string",
            Self::Boolean(_) => "boolean",
            Self::Symbol(_) => "symbol",
            Self::Null => "object",
            Self::Undefined => "undefined",
            Self::BigInt(_) => "bigint",
            Self::Object(ref object) => {
                if object.is_callable() {
                    "function"
                } else {
                    "object"
                }
            }
        }
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
        if let Some(object) = self.as_object() {
            object.is_array_abstract()
        }
        // 4. Return false.
        else {
            Ok(false)
        }
    }
}

impl Default for JsValue {
    fn default() -> Self {
        Self::Undefined
    }
}

/// The preferred type to convert an object to a primitive `Value`.
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
