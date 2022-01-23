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
    gc::{Finalize, Trace},
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyKey},
    symbol::{JsSymbol, WellKnownSymbols},
    BoaProfiler, Context, JsBigInt, JsResult, JsString,
};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::Zero;
use once_cell::sync::Lazy;
use std::{
    collections::HashSet,
    convert::TryFrom,
    fmt::{self, Display},
    ops::{Deref, Sub},
    str::FromStr,
};

mod conversions;
pub(crate) mod display;
mod equality;
mod hash;
mod operations;
mod r#type;

pub use conversions::*;
pub use display::ValueDisplay;
pub use equality::*;
pub use hash::*;
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
#[derive(Trace, Finalize, Debug, Clone)]
pub enum JsValue {
    /// `null` - A null value, for when a value doesn't exist.
    Null,
    /// `undefined` - An undefined value, for when a field or index doesn't exist.
    Undefined,
    /// `boolean` - A `true` / `false` value, for if a certain criteria is met.
    Boolean(bool),
    /// `String` - A UTF-8 string, such as `"Hello, world"`.
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

/// Represents the result of ToIntegerOrInfinity operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerOrInfinity {
    Integer(i64),
    PositiveInfinity,
    NegativeInfinity,
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
    /// - [EcmaScript reference][spec]
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

    /// Returns true if the value is a constructor object
    #[inline]
    pub fn is_constructor(&self) -> bool {
        matches!(self, Self::Object(obj) if obj.is_constructor())
    }

    #[inline]
    pub fn as_constructor(&self) -> Option<&JsObject> {
        self.as_object().filter(|obj| obj.is_constructor())
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
        // If it can fit in a i32 and the trucated version is
        // equal to the original then it is an integer.
        let is_racional_intiger = |n: f64| n == ((n as i32) as f64);

        match *self {
            Self::Integer(_) => true,
            Self::Rational(n) if is_racional_intiger(n) => true,
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

    /// Returns an optional reference to a `BigInt` if the value is a BigInt primitive.
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
            Self::Undefined | Self::Null => false,
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
        let _timer = BoaProfiler::global().start_event("Value::get_property", "value");
        match self {
            Self::Object(ref object) => {
                // TODO: had to skip `__get_own_properties__` since we don't have context here
                let property = object.borrow().properties().get(&key).cloned();
                if property.is_some() {
                    return property;
                }

                object
                    .prototype()
                    .as_ref()
                    .map_or(JsValue::Null, |obj| obj.clone().into())
                    .get_property(key)
            }
            _ => None,
        }
    }

    /// Resolve the property in the object and get its value, or undefined if this is not an object or the field doesn't exist
    /// get_field receives a Property from get_prop(). It should then return the `[[Get]]` result value if that's set, otherwise fall back to `[[Value]]`
    pub(crate) fn get_field<K>(&self, key: K, context: &mut Context) -> JsResult<Self>
    where
        K: Into<PropertyKey>,
    {
        let _timer = BoaProfiler::global().start_event("Value::get_field", "value");
        if let Self::Object(ref obj) = *self {
            obj.clone()
                .__get__(&key.into(), obj.clone().into(), context)
        } else {
            Ok(JsValue::undefined())
        }
    }

    /// Set the field in the value
    ///
    /// Similar to `7.3.4 Set ( O, P, V, Throw )`, but returns the value instead of a boolean.
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set-o-p-v-throw
    #[inline]
    pub(crate) fn set_field<K, V>(
        &self,
        key: K,
        value: V,
        throw: bool,
        context: &mut Context,
    ) -> JsResult<JsValue>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        // 1. Assert: Type(O) is Object.
        // TODO: Currently the value may not be an object.
        //       In that case this function does nothing.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Assert: Type(Throw) is Boolean.

        let key = key.into();
        let value = value.into();
        let _timer = BoaProfiler::global().start_event("Value::set_field", "value");
        if let Self::Object(ref obj) = *self {
            // 4. Let success be ? O.[[Set]](P, V, O).
            let success = obj
                .clone()
                .__set__(key, value.clone(), obj.clone().into(), context)?;

            // 5. If success is false and Throw is true, throw a TypeError exception.
            // 6. Return success.
            if !success && throw {
                return context.throw_type_error("Cannot assign value to property");
            } else {
                return Ok(value);
            }
        }
        Ok(value)
    }

    /// Set the kind of an object.
    #[inline]
    pub fn set_data(&self, data: ObjectData) {
        if let Self::Object(ref obj) = *self {
            obj.borrow_mut().data = data;
        }
    }

    /// Set the property in the value.
    #[inline]
    pub(crate) fn set_property<K, P>(&self, key: K, property: P)
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        if let Some(object) = self.as_object() {
            object.insert(key.into(), property.into());
        }
    }

    /// The abstract operation ToPrimitive takes an input argument and an optional argument PreferredType.
    ///
    /// <https://tc39.es/ecma262/#sec-toprimitive>
    pub fn to_primitive(
        &self,
        context: &mut Context,
        preferred_type: PreferredType,
    ) -> JsResult<JsValue> {
        // 1. Assert: input is an ECMAScript language value. (always a value not need to check)
        // 2. If Type(input) is Object, then
        if self.is_object() {
            // a. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
            let exotic_to_prim = self.get_method(WellKnownSymbols::to_primitive(), context)?;

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
            self.as_object()
                .expect("self was not an object")
                .ordinary_to_primitive(context, preferred_type)
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
        match self {
            JsValue::Null => context.throw_type_error("cannot convert null to a BigInt"),
            JsValue::Undefined => context.throw_type_error("cannot convert undefined to a BigInt"),
            JsValue::String(ref string) => {
                if let Some(value) = JsBigInt::from_string(string) {
                    Ok(value)
                } else {
                    context.throw_syntax_error(format!(
                        "cannot convert string '{}' to bigint primitive",
                        string
                    ))
                }
            }
            JsValue::Boolean(true) => Ok(JsBigInt::one()),
            JsValue::Boolean(false) => Ok(JsBigInt::zero()),
            JsValue::Integer(_) | JsValue::Rational(_) => {
                context.throw_type_error("cannot convert Number to a BigInt")
            }
            JsValue::BigInt(b) => Ok(b.clone()),
            JsValue::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::Number)?;
                primitive.to_bigint(context)
            }
            JsValue::Symbol(_) => context.throw_type_error("cannot convert Symbol to a BigInt"),
        }
    }

    /// Returns an object that implements `Display`.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa::JsValue;
    ///
    /// let value = JsValue::new(3);
    ///
    /// println!("{}", value.display());
    /// ```
    #[inline]
    pub fn display(&self) -> ValueDisplay<'_> {
        ValueDisplay { value: self }
    }

    /// Converts the value to a string.
    ///
    /// This function is equivalent to `String(value)` in JavaScript.
    pub fn to_string(&self, context: &mut Context) -> JsResult<JsString> {
        match self {
            JsValue::Null => Ok("null".into()),
            JsValue::Undefined => Ok("undefined".into()),
            JsValue::Boolean(boolean) => Ok(boolean.to_string().into()),
            JsValue::Rational(rational) => Ok(Number::to_native_string(*rational).into()),
            JsValue::Integer(integer) => Ok(integer.to_string().into()),
            JsValue::String(string) => Ok(string.clone()),
            JsValue::Symbol(_) => context.throw_type_error("can't convert symbol to string"),
            JsValue::BigInt(ref bigint) => Ok(bigint.to_string().into()),
            JsValue::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::String)?;
                primitive.to_string(context)
            }
        }
    }

    /// Converts the value to an Object.
    ///
    /// This function is equivalent to `Object(value)` in JavaScript
    ///
    /// See: <https://tc39.es/ecma262/#sec-toobject>
    pub fn to_object(&self, context: &mut Context) -> JsResult<JsObject> {
        match self {
            JsValue::Undefined | JsValue::Null => {
                context.throw_type_error("cannot convert 'null' or 'undefined' to object")
            }
            JsValue::Boolean(boolean) => {
                let prototype = context.standard_objects().boolean_object().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::boolean(*boolean),
                ))
            }
            JsValue::Integer(integer) => {
                let prototype = context.standard_objects().number_object().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::number(f64::from(*integer)),
                ))
            }
            JsValue::Rational(rational) => {
                let prototype = context.standard_objects().number_object().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::number(*rational),
                ))
            }
            JsValue::String(ref string) => {
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
            JsValue::Symbol(ref symbol) => {
                let prototype = context.standard_objects().symbol_object().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::symbol(symbol.clone()),
                ))
            }
            JsValue::BigInt(ref bigint) => {
                let prototype = context.standard_objects().bigint_object().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::big_int(bigint.clone()),
                ))
            }
            JsValue::Object(jsobject) => Ok(jsobject.clone()),
        }
    }

    /// Converts the value to a `PropertyKey`, that can be used as a key for properties.
    ///
    /// See <https://tc39.es/ecma262/#sec-topropertykey>
    pub fn to_property_key(&self, context: &mut Context) -> JsResult<PropertyKey> {
        Ok(match self {
            // Fast path:
            JsValue::String(string) => string.clone().into(),
            JsValue::Symbol(symbol) => symbol.clone().into(),
            // Slow path:
            _ => match self.to_primitive(context, PreferredType::String)? {
                JsValue::String(ref string) => string.clone().into(),
                JsValue::Symbol(ref symbol) => symbol.clone().into(),
                primitive => primitive.to_string(context)?.into(),
            },
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
        let int = number.floor() as i64;

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
        let int = number.floor() as i64;

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
        let int = number.floor() as i64;

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
        let int = number.floor() as i64;

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
        if &int64_bit >= TWO_E_63.deref() {
            Ok(int64_bit.sub(TWO_E_64.deref()))
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
        if self.is_undefined() {
            return Ok(0);
        }

        let integer_index = self.to_integer(context)?;

        if integer_index < 0.0 {
            return context.throw_range_error("Integer index must be >= 0");
        }

        if integer_index > Number::MAX_SAFE_INTEGER {
            return context.throw_range_error("Integer index must be less than 2**(53) - 1");
        }

        Ok(integer_index as usize)
    }

    /// Converts argument to an integer suitable for use as the length of an array-like object.
    ///
    /// See: <https://tc39.es/ecma262/#sec-tolength>
    pub fn to_length(&self, context: &mut Context) -> JsResult<usize> {
        // 1. Let len be ? ToInteger(argument).
        let len = self.to_integer(context)?;

        // 2. If len ≤ +0, return +0.
        if len < 0.0 {
            return Ok(0);
        }

        // 3. Return min(len, 2^53 - 1).
        Ok(len.min(Number::MAX_SAFE_INTEGER) as usize)
    }

    /// Converts a value to an integral Number value.
    ///
    /// See: <https://tc39.es/ecma262/#sec-tointeger>
    pub fn to_integer(&self, context: &mut Context) -> JsResult<f64> {
        // 1. Let number be ? ToNumber(argument).
        let number = self.to_number(context)?;

        // 2. If number is +∞ or -∞, return number.
        if !number.is_finite() {
            // 3. If number is NaN, +0, or -0, return +0.
            if number.is_nan() {
                return Ok(0.0);
            }
            return Ok(number);
        }

        // 4. Let integer be the Number value that is the same sign as number and whose magnitude is floor(abs(number)).
        // 5. If integer is -0, return +0.
        // 6. Return integer.
        Ok(number.trunc() + 0.0) // We add 0.0 to convert -0.0 to +0.0
    }

    /// Converts a value to a double precision floating point.
    ///
    /// This function is equivalent to the unary `+` operator (`+value`) in JavaScript
    ///
    /// See: <https://tc39.es/ecma262/#sec-tonumber>
    pub fn to_number(&self, context: &mut Context) -> JsResult<f64> {
        match *self {
            JsValue::Null => Ok(0.0),
            JsValue::Undefined => Ok(f64::NAN),
            JsValue::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            JsValue::String(ref string) => Ok(string.string_to_number()),
            JsValue::Rational(number) => Ok(number),
            JsValue::Integer(integer) => Ok(f64::from(integer)),
            JsValue::Symbol(_) => context.throw_type_error("argument must not be a symbol"),
            JsValue::BigInt(_) => context.throw_type_error("argument must not be a bigint"),
            JsValue::Object(_) => {
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
    pub fn require_object_coercible(&self, context: &mut Context) -> JsResult<&JsValue> {
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

    /// Converts argument to an integer, +∞, or -∞.
    ///
    /// See: <https://tc39.es/ecma262/#sec-tointegerorinfinity>
    pub fn to_integer_or_infinity(&self, context: &mut Context) -> JsResult<IntegerOrInfinity> {
        // 1. Let number be ? ToNumber(argument).
        let number = self.to_number(context)?;

        // 2. If number is NaN, +0𝔽, or -0𝔽, return 0.
        if number.is_nan() || number == 0.0 || number == -0.0 {
            Ok(IntegerOrInfinity::Integer(0))
        } else if number.is_infinite() && number.is_sign_positive() {
            // 3. If number is +∞𝔽, return +∞.
            Ok(IntegerOrInfinity::PositiveInfinity)
        } else if number.is_infinite() && number.is_sign_negative() {
            // 4. If number is -∞𝔽, return -∞.
            Ok(IntegerOrInfinity::NegativeInfinity)
        } else {
            // 5. Let integer be floor(abs(ℝ(number))).
            let integer = number.abs().floor();
            let integer = integer.min(Number::MAX_SAFE_INTEGER) as i64;

            // 6. If number < +0𝔽, set integer to -integer.
            // 7. Return integer.
            if number < 0.0 {
                Ok(IntegerOrInfinity::Integer(-integer))
            } else {
                Ok(IntegerOrInfinity::Integer(integer))
            }
        }
    }

    /// `typeof` operator. Returns a string representing the type of the
    /// given ECMA Value.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-typeof-operator
    pub fn type_of(&self) -> JsString {
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
        Self::Undefined
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
