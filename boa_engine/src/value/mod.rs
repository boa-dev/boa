#![warn(unsafe_op_in_unsafe_fn)]
//! This module implements the JavaScript [`JsValue`] type.
//!
//! [`JsValue`] implements several utility methods and conversions between Javascript
//! values and Rust values.
//!
//! # Notes
//!
//! We recommend using [`JsValue::from`], [`JsValue::new`] or the [`Into::into`]
//! trait if you need to convert from float types ([`f64`], [`f32`]) to [`JsValue`],
//! since there are some checks implemented that convert float values to signed
//! integer values when there's no loss of precision involved.
//!
//! Alternatively, you can use the [`JsValue::float64`] method if you do need a pure
//! [`f64`] value. The constructor skips all conversions from [`f64`]s to
//! [`i32`]s, even if the value can be represented by an [`i32`] without loss of
//! precision.
//!
//! # Alternative implementations
//!
//! `boa` right now implements two versions of [`JsValue`]:
//! - The default implementation using a simple `enum`.
//! - A NaN-boxed implementation for x86-64 platforms that can be enabled using the `nan_boxing` feature.
//! The feature flag is ignored on incompatible platforms.

#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        number::{f64_to_int32, f64_to_uint32},
        Number,
    },
    object::{JsObject, ObjectData},
    property::{PropertyDescriptor, PropertyKey},
    symbol::WellKnownSymbols,
    Context, JsBigInt, JsResult, JsString,
};
use boa_profiler::Profiler;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::Zero;
use once_cell::sync::Lazy;
use std::{
    collections::HashSet,
    fmt::{self, Display},
    ops::Sub,
    str::FromStr,
};

mod sys;

#[doc(inline)]
pub use sys::{JsValue, JsVariant, Ref};

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
pub(crate) use sys::PointerType;

static TWO_E_64: Lazy<BigInt> = Lazy::new(|| {
    const TWO_E_64: u128 = 2u128.pow(64);
    BigInt::from(TWO_E_64)
});

static TWO_E_63: Lazy<BigInt> = Lazy::new(|| {
    const TWO_E_63: u128 = 2u128.pow(63);
    BigInt::from(TWO_E_63)
});

impl JsValue {
    /// Create a new [`JsValue`].
    #[inline]
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Self>,
    {
        value.into()
    }

    /// Returns true if the value is null or undefined.
    #[inline]
    pub fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.is_undefined()
    }

    /// Creates a new number with `NaN` value.
    #[inline]
    pub fn nan() -> Self {
        Self::new(f64::NAN)
    }

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

    /// Returns true if the value is a constructor object.
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

    /// Returns true if the value is a promise object.
    #[inline]
    pub fn is_promise(&self) -> bool {
        self.as_object()
            .as_deref()
            .map_or(false, JsObject::is_promise)
    }

    #[inline]
    pub fn as_promise(&self) -> Option<Ref<'_, JsObject>> {
        self.as_object().filter(|obj| obj.is_promise())
    }

    /// Returns true if the value is an integer, even if it's represented by an [`f64`].
    #[inline]
    #[allow(clippy::float_cmp)]
    pub fn is_integer(&self) -> bool {
        if self.is_integer32() {
            return true;
        }

        self.as_float64()
            // If it can fit in a i32 and the trucated version is
            // equal to the original then it is an integer.
            .map(|num| num == f64::from(num as i32))
            .unwrap_or_default()
    }

    /// Returns true if the value is a number.
    #[inline]
    pub fn is_number(&self) -> bool {
        self.is_integer32() || self.is_float64()
    }

    #[inline]
    pub fn as_number(&self) -> Option<f64> {
        match self.variant() {
            JsVariant::Integer32(integer) => Some(integer.into()),
            JsVariant::Float64(rational) => Some(rational),
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
            JsVariant::Symbol(_) | JsVariant::Object(_) => true,
            JsVariant::String(s) if !s.is_empty() => true,
            JsVariant::Float64(n) if n != 0.0 && !n.is_nan() => true,
            JsVariant::Integer32(n) if n != 0 => true,
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
            let property = object.borrow().properties().get(&key);
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
            JsVariant::Integer32(_) | JsVariant::Float64(_) => {
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
            JsVariant::Float64(rational) => Ok(Number::to_native_string(rational).into()),
            JsVariant::Integer32(integer) => Ok(integer.to_string().into()),
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
                let prototype = context.intrinsics().constructors().boolean().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::boolean(boolean),
                ))
            }
            JsVariant::Integer32(integer) => {
                let prototype = context.intrinsics().constructors().number().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::number(f64::from(integer)),
                ))
            }
            JsVariant::Float64(rational) => {
                let prototype = context.intrinsics().constructors().number().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::number(rational),
                ))
            }
            JsVariant::String(string) => {
                let prototype = context.intrinsics().constructors().string().prototype();

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
                let prototype = context.intrinsics().constructors().symbol().prototype();
                Ok(JsObject::from_proto_and_data(
                    prototype,
                    ObjectData::symbol(symbol.clone()),
                ))
            }
            JsVariant::BigInt(bigint) => {
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
        },
            _ => self.to_string(context)?.into(),
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
        if let Some(number) = self.as_integer32() {
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
        if let Some(number) = self.as_integer32() {
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
            return context.throw_range_error("Index must be between 0 and  2^53 - 1");
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
            JsVariant::Float64(number) => Ok(number),
            JsVariant::Integer32(integer) => Ok(f64::from(integer)),
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
            JsVariant::Float64(_) | JsVariant::Integer32(_) => "number",
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
