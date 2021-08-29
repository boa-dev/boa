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
    object::{JsObject, Object, ObjectData},
    property::{PropertyDescriptor, PropertyKey},
    symbol::{JsSymbol, WellKnownSymbols},
    BoaProfiler, Context, JsBigInt, JsResult, JsString,
};
use gc::{Finalize, Trace};
use std::{
    collections::HashSet,
    convert::TryFrom,
    fmt::{self, Display},
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
    pub fn positive_inifnity() -> Self {
        Self::Rational(f64::INFINITY)
    }

    /// Creates a new number with `-Infinity` value.
    #[inline]
    pub fn negative_inifnity() -> Self {
        Self::Rational(f64::NEG_INFINITY)
    }

    /// Returns a new empty object
    pub(crate) fn new_object(context: &Context) -> Self {
        let _timer = BoaProfiler::global().start_event("new_object", "value");
        context.construct_object().into()
    }

    /// Returns true if the value is an object
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    #[inline]
    pub fn as_object(&self) -> Option<JsObject> {
        match *self {
            Self::Object(ref o) => Some(o.clone()),
            _ => None,
        }
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

    /// Returns true if the value is a function
    #[inline]
    pub fn is_function(&self) -> bool {
        matches!(self, Self::Object(o) if o.is_function())
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

                object.borrow().prototype_instance().get_property(key)
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
                return Err(context.construct_type_error("Cannot assign value to property"));
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
        if let JsValue::Object(obj) = self {
            if let Some(exotic_to_prim) =
                obj.get_method(context, WellKnownSymbols::to_primitive())?
            {
                let hint = match preferred_type {
                    PreferredType::String => "string",
                    PreferredType::Number => "number",
                    PreferredType::Default => "default",
                }
                .into();
                let result = exotic_to_prim.call(self, &[hint], context)?;
                return if result.is_object() {
                    Err(context.construct_type_error("Symbol.toPrimitive cannot return an object"))
                } else {
                    Ok(result)
                };
            }

            let mut hint = preferred_type;

            if hint == PreferredType::Default {
                hint = PreferredType::Number;
            };

            // g. Return ? OrdinaryToPrimitive(input, hint).
            obj.ordinary_to_primitive(context, hint)
        } else {
            // 3. Return input.
            Ok(self.clone())
        }
    }

    /// Converts the value to a `BigInt`.
    ///
    /// This function is equivelent to `BigInt(value)` in JavaScript.
    pub fn to_bigint(&self, context: &mut Context) -> JsResult<JsBigInt> {
        match self {
            JsValue::Null => Err(context.construct_type_error("cannot convert null to a BigInt")),
            JsValue::Undefined => {
                Err(context.construct_type_error("cannot convert undefined to a BigInt"))
            }
            JsValue::String(ref string) => {
                if let Some(value) = JsBigInt::from_string(string) {
                    Ok(value)
                } else {
                    Err(context.construct_syntax_error(format!(
                        "cannot convert string '{}' to bigint primitive",
                        string
                    )))
                }
            }
            JsValue::Boolean(true) => Ok(JsBigInt::one()),
            JsValue::Boolean(false) => Ok(JsBigInt::zero()),
            JsValue::Integer(num) => Ok(JsBigInt::new(*num)),
            JsValue::Rational(num) => {
                if let Ok(bigint) = JsBigInt::try_from(*num) {
                    return Ok(bigint);
                }
                Err(context.construct_type_error(format!(
                    "The number {} cannot be converted to a BigInt because it is not an integer",
                    num
                )))
            }
            JsValue::BigInt(b) => Ok(b.clone()),
            JsValue::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::Number)?;
                primitive.to_bigint(context)
            }
            JsValue::Symbol(_) => {
                Err(context.construct_type_error("cannot convert Symbol to a BigInt"))
            }
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
            JsValue::Symbol(_) => {
                Err(context.construct_type_error("can't convert symbol to string"))
            }
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
                Err(context.construct_type_error("cannot convert 'null' or 'undefined' to object"))
            }
            JsValue::Boolean(boolean) => {
                let prototype = context.standard_objects().boolean_object().prototype();
                Ok(JsObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::boolean(*boolean),
                )))
            }
            JsValue::Integer(integer) => {
                let prototype = context.standard_objects().number_object().prototype();
                Ok(JsObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::number(f64::from(*integer)),
                )))
            }
            JsValue::Rational(rational) => {
                let prototype = context.standard_objects().number_object().prototype();
                Ok(JsObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::number(*rational),
                )))
            }
            JsValue::String(ref string) => {
                let prototype = context.standard_objects().string_object().prototype();

                let object = JsObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::string(string.clone()),
                ));
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
                Ok(JsObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::symbol(symbol.clone()),
                )))
            }
            JsValue::BigInt(ref bigint) => {
                let prototype = context.standard_objects().bigint_object().prototype();
                Ok(JsObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::big_int(bigint.clone()),
                )))
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

    /// Converts a value to a non-negative integer if it is a valid integer index value.
    ///
    /// See: <https://tc39.es/ecma262/#sec-toindex>
    pub fn to_index(&self, context: &mut Context) -> JsResult<usize> {
        if self.is_undefined() {
            return Ok(0);
        }

        let integer_index = self.to_integer(context)?;

        if integer_index < 0.0 {
            return Err(context.construct_range_error("Integer index must be >= 0"));
        }

        if integer_index > Number::MAX_SAFE_INTEGER {
            return Err(
                context.construct_range_error("Integer index must be less than 2**(53) - 1")
            );
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
            JsValue::Symbol(_) => {
                Err(context.construct_type_error("argument must not be a symbol"))
            }
            JsValue::BigInt(_) => {
                Err(context.construct_type_error("argument must not be a bigint"))
            }
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
            Err(context.construct_type_error("cannot convert null or undefined to Object"))
        } else {
            Ok(self)
        }
    }

    #[inline]
    pub fn to_property_descriptor(&self, context: &mut Context) -> JsResult<PropertyDescriptor> {
        // 1. If Type(Obj) is not Object, throw a TypeError exception.
        match self {
            JsValue::Object(ref obj) => obj.to_property_descriptor(context),
            _ => Err(context
                .construct_type_error("Cannot construct a property descriptor from a non-object")),
        }
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

    /// Check if it is an array.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isarray
    pub(crate) fn is_array(&self, _context: &mut Context) -> JsResult<bool> {
        // 1. If Type(argument) is not Object, return false.
        if let Some(object) = self.as_object() {
            // 2. If argument is an Array exotic object, return true.
            //     a. If argument.[[ProxyHandler]] is null, throw a TypeError exception.
            // 3. If argument is a Proxy exotic object, then
            //     b. Let target be argument.[[ProxyTarget]].
            //     c. Return ? IsArray(target).
            // 4. Return false.
            Ok(object.is_array())
        } else {
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
