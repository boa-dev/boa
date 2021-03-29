//! This module implements the JavaScript Value.
//!
//! Javascript values, utility methods and conversion between Javascript values and Rust values.

#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        number::{f64_to_int32, f64_to_uint32},
        string::is_trimmable_whitespace,
        BigInt, Number,
    },
    object::{GcObject, Object, ObjectData},
    property::{Attribute, DataDescriptor, PropertyDescriptor, PropertyKey},
    BoaProfiler, Context, Result,
};
use gc::{Finalize, Trace};
use serde_json::{Number as JSONNumber, Value as JSONValue};
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
mod rcbigint;
mod rcstring;
mod rcsymbol;
mod r#type;

pub use conversions::*;
pub use display::ValueDisplay;
pub use equality::*;
pub use hash::*;
pub use operations::*;
pub use r#type::Type;
pub use rcbigint::RcBigInt;
pub use rcstring::RcString;
pub use rcsymbol::RcSymbol;

/// A Javascript value
#[derive(Trace, Finalize, Debug, Clone)]
pub enum Value {
    /// `null` - A null value, for when a value doesn't exist.
    Null,
    /// `undefined` - An undefined value, for when a field or index doesn't exist.
    Undefined,
    /// `boolean` - A `true` / `false` value, for if a certain criteria is met.
    Boolean(bool),
    /// `String` - A UTF-8 string, such as `"Hello, world"`.
    String(RcString),
    /// `Number` - A 64-bit floating point number, such as `3.1415`
    Rational(f64),
    /// `Number` - A 32-bit integer, such as `42`.
    Integer(i32),
    /// `BigInt` - holds any arbitrary large signed integer.
    BigInt(RcBigInt),
    /// `Object` - An object, such as `Math`, represented by a binary tree of string keys to Javascript values.
    Object(GcObject),
    /// `Symbol` - A Symbol Primitive type.
    Symbol(RcSymbol),
}

/// Represents the result of ToIntegerOrInfinity operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerOrInfinity {
    Integer(i64),
    PositiveInfinity,
    NegativeInfinity,
}

impl Value {
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
        Self::number(f64::NAN)
    }

    /// Creates a new string value.
    #[inline]
    pub fn string<S>(value: S) -> Self
    where
        S: Into<RcString>,
    {
        Self::String(value.into())
    }

    /// Creates a new number value.
    #[inline]
    pub fn rational<N>(value: N) -> Self
    where
        N: Into<f64>,
    {
        Self::Rational(value.into())
    }

    /// Creates a new number value.
    #[inline]
    pub fn integer<I>(value: I) -> Self
    where
        I: Into<i32>,
    {
        Self::Integer(value.into())
    }

    /// Creates a new number value.
    #[inline]
    pub fn number<N>(value: N) -> Self
    where
        N: Into<f64>,
    {
        Self::rational(value.into())
    }

    /// Creates a new bigint value.
    #[inline]
    pub fn bigint<B>(value: B) -> Self
    where
        B: Into<RcBigInt>,
    {
        Self::BigInt(value.into())
    }

    /// Creates a new boolean value.
    #[inline]
    pub fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    /// Creates a new object value.
    #[inline]
    pub fn object(object: Object) -> Self {
        Self::Object(GcObject::new(object))
    }

    /// Creates a new symbol value.
    #[inline]
    pub fn symbol(symbol: RcSymbol) -> Self {
        Self::Symbol(symbol)
    }

    /// Returns a new empty object
    pub fn new_object(context: &Context) -> Self {
        let _timer = BoaProfiler::global().start_event("new_object", "value");
        context.construct_object().into()
    }

    /// Convert from a JSON value to a JS value
    pub fn from_json(json: JSONValue, context: &mut Context) -> Self {
        match json {
            JSONValue::Number(v) => {
                if let Some(Ok(integer_32)) = v.as_i64().map(i32::try_from) {
                    Self::integer(integer_32)
                } else {
                    Self::rational(v.as_f64().expect("Could not convert value to f64"))
                }
            }
            JSONValue::String(v) => Self::string(v),
            JSONValue::Bool(v) => Self::boolean(v),
            JSONValue::Array(vs) => {
                let array_prototype = context.standard_objects().array_object().prototype();
                let new_obj: Value =
                    Object::with_prototype(array_prototype.into(), ObjectData::Array).into();
                let length = vs.len();
                for (idx, json) in vs.into_iter().enumerate() {
                    new_obj.set_property(
                        idx.to_string(),
                        DataDescriptor::new(
                            Self::from_json(json, context),
                            Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
                        ),
                    );
                }
                new_obj.set_property(
                    "length",
                    // TODO: Fix length attribute
                    DataDescriptor::new(length, Attribute::all()),
                );
                new_obj
            }
            JSONValue::Object(obj) => {
                let new_obj = Value::new_object(context);
                for (key, json) in obj.into_iter() {
                    let value = Self::from_json(json, context);
                    new_obj.set_property(
                        key,
                        DataDescriptor::new(
                            value,
                            Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
                        ),
                    );
                }
                new_obj
            }
            JSONValue::Null => Self::null(),
        }
    }

    /// Converts the `Value` to `JSON`.
    pub fn to_json(&self, context: &mut Context) -> Result<JSONValue> {
        let to_json = self.get_field("toJSON", context)?;
        if to_json.is_function() {
            let json_value = context.call(&to_json, self, &[])?;
            return json_value.to_json(context);
        }

        match *self {
            Self::Null => Ok(JSONValue::Null),
            Self::Boolean(b) => Ok(JSONValue::Bool(b)),
            Self::Object(ref obj) => obj.to_json(context),
            Self::String(ref str) => Ok(JSONValue::String(str.to_string())),
            Self::Rational(num) => {
                if num.is_finite() {
                    Ok(JSONValue::Number(
                        JSONNumber::from_str(&Number::to_native_string(num))
                            .expect("invalid number found"),
                    ))
                } else {
                    Ok(JSONValue::Null)
                }
            }
            Self::Integer(val) => Ok(JSONValue::Number(JSONNumber::from(val))),
            Self::BigInt(_) => {
                Err(context.construct_type_error("BigInt value can't be serialized in JSON"))
            }
            Self::Symbol(_) | Self::Undefined => {
                unreachable!("Symbols and Undefined JSON Values depend on parent type");
            }
        }
    }

    /// This will tell us if we can exten an object or not, not properly implemented yet
    ///
    /// For now always returns true.
    ///
    /// For scalar types it should be false, for objects check the private field for extensibilaty.
    /// By default true.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/seal> would turn `extensible` to `false`
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/freeze> would also turn `extensible` to `false`
    pub fn is_extensible(&self) -> bool {
        true
    }

    /// Returns true if the value is an object
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    #[inline]
    pub fn as_object(&self) -> Option<GcObject> {
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

    pub fn as_symbol(&self) -> Option<RcSymbol> {
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
    pub fn as_string(&self) -> Option<&RcString> {
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
    pub fn as_bigint(&self) -> Option<&BigInt> {
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
            Self::BigInt(ref n) if *n.as_inner() != 0 => true,
            Self::Boolean(v) => v,
            _ => false,
        }
    }

    /// Removes a property from a Value object.
    ///
    /// It will return a boolean based on if the value was removed, if there was no value to remove false is returned.
    pub fn remove_property<Key>(&self, key: Key) -> bool
    where
        Key: Into<PropertyKey>,
    {
        self.as_object()
            .map(|mut x| x.remove(&key.into()))
            .is_some()
    }

    /// Resolve the property in the object.
    ///
    /// A copy of the Property is returned.
    pub fn get_property<Key>(&self, key: Key) -> Option<PropertyDescriptor>
    where
        Key: Into<PropertyKey>,
    {
        let key = key.into();
        let _timer = BoaProfiler::global().start_event("Value::get_property", "value");
        match self {
            Self::Object(ref object) => {
                let property = object.get_own_property(&key);
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
    pub fn get_field<K>(&self, key: K, context: &mut Context) -> Result<Self>
    where
        K: Into<PropertyKey>,
    {
        let _timer = BoaProfiler::global().start_event("Value::get_field", "value");
        if let Self::Object(ref obj) = *self {
            obj.clone().get(&key.into(), obj.clone().into(), context)
        } else {
            Ok(Value::undefined())
        }
    }

    /// Check to see if the Value has the field, mainly used by environment records.
    #[inline]
    pub fn has_field<K>(&self, key: K) -> bool
    where
        K: Into<PropertyKey>,
    {
        let _timer = BoaProfiler::global().start_event("Value::has_field", "value");
        self.as_object()
            .map(|object| object.has_property(&key.into()))
            .unwrap_or(false)
    }

    /// Set the field in the value
    #[inline]
    pub fn set_field<K, V>(&self, key: K, value: V, context: &mut Context) -> Result<Value>
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        let key = key.into();
        let value = value.into();
        let _timer = BoaProfiler::global().start_event("Value::set_field", "value");
        if let Self::Object(ref obj) = *self {
            obj.clone()
                .set(key, value.clone(), obj.clone().into(), context)?;
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
    pub fn set_property<K, P>(&self, key: K, property: P)
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        if let Some(mut object) = self.as_object() {
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
    ) -> Result<Value> {
        // 1. Assert: input is an ECMAScript language value. (always a value not need to check)
        // 2. If Type(input) is Object, then
        if let Value::Object(obj) = self {
            if let Some(exotic_to_prim) =
                obj.get_method(context, context.well_known_symbols().to_primitive_symbol())?
            {
                let hint = match preferred_type {
                    PreferredType::String => "string",
                    PreferredType::Number => "number",
                    PreferredType::Default => "default",
                }
                .into();
                let result = exotic_to_prim.call(&self, &[hint], context)?;
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
    pub fn to_bigint(&self, context: &mut Context) -> Result<RcBigInt> {
        match self {
            Value::Null => Err(context.construct_type_error("cannot convert null to a BigInt")),
            Value::Undefined => {
                Err(context.construct_type_error("cannot convert undefined to a BigInt"))
            }
            Value::String(ref string) => Ok(RcBigInt::from(BigInt::from_string(string, context)?)),
            Value::Boolean(true) => Ok(RcBigInt::from(BigInt::from(1))),
            Value::Boolean(false) => Ok(RcBigInt::from(BigInt::from(0))),
            Value::Integer(num) => Ok(RcBigInt::from(BigInt::from(*num))),
            Value::Rational(num) => {
                if let Ok(bigint) = BigInt::try_from(*num) {
                    return Ok(RcBigInt::from(bigint));
                }
                Err(context.construct_type_error(format!(
                    "The number {} cannot be converted to a BigInt because it is not an integer",
                    num
                )))
            }
            Value::BigInt(b) => Ok(b.clone()),
            Value::Object(_) => {
                let primitive = self.to_primitive(context, PreferredType::Number)?;
                primitive.to_bigint(context)
            }
            Value::Symbol(_) => {
                Err(context.construct_type_error("cannot convert Symbol to a BigInt"))
            }
        }
    }

    /// Returns an object that implements `Display`.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa::Value;
    ///
    /// let value = Value::number(3);
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
    pub fn to_string(&self, context: &mut Context) -> Result<RcString> {
        match self {
            Value::Null => Ok("null".into()),
            Value::Undefined => Ok("undefined".into()),
            Value::Boolean(boolean) => Ok(boolean.to_string().into()),
            Value::Rational(rational) => Ok(Number::to_native_string(*rational).into()),
            Value::Integer(integer) => Ok(integer.to_string().into()),
            Value::String(string) => Ok(string.clone()),
            Value::Symbol(_) => Err(context.construct_type_error("can't convert symbol to string")),
            Value::BigInt(ref bigint) => Ok(bigint.to_string().into()),
            Value::Object(_) => {
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
    pub fn to_object(&self, context: &mut Context) -> Result<GcObject> {
        match self {
            Value::Undefined | Value::Null => {
                Err(context.construct_type_error("cannot convert 'null' or 'undefined' to object"))
            }
            Value::Boolean(boolean) => {
                let prototype = context.standard_objects().boolean_object().prototype();
                Ok(GcObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::Boolean(*boolean),
                )))
            }
            Value::Integer(integer) => {
                let prototype = context.standard_objects().number_object().prototype();
                Ok(GcObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::Number(f64::from(*integer)),
                )))
            }
            Value::Rational(rational) => {
                let prototype = context.standard_objects().number_object().prototype();
                Ok(GcObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::Number(*rational),
                )))
            }
            Value::String(ref string) => {
                let prototype = context.standard_objects().string_object().prototype();

                let mut object = GcObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::String(string.clone()),
                ));
                // Make sure the correct length is set on our new string object
                object.insert_property(
                    PropertyKey::String("length".into()),
                    Value::from(string.encode_utf16().count()),
                    Attribute::NON_ENUMERABLE,
                );
                Ok(object)
            }
            Value::Symbol(ref symbol) => {
                let prototype = context.standard_objects().symbol_object().prototype();
                Ok(GcObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::Symbol(symbol.clone()),
                )))
            }
            Value::BigInt(ref bigint) => {
                let prototype = context.standard_objects().bigint_object().prototype();
                Ok(GcObject::new(Object::with_prototype(
                    prototype.into(),
                    ObjectData::BigInt(bigint.clone()),
                )))
            }
            Value::Object(gcobject) => Ok(gcobject.clone()),
        }
    }

    /// Converts the value to a `PropertyKey`, that can be used as a key for properties.
    ///
    /// See <https://tc39.es/ecma262/#sec-topropertykey>
    pub fn to_property_key(&self, context: &mut Context) -> Result<PropertyKey> {
        Ok(match self {
            // Fast path:
            Value::String(string) => string.clone().into(),
            Value::Symbol(symbol) => symbol.clone().into(),
            // Slow path:
            _ => match self.to_primitive(context, PreferredType::String)? {
                Value::String(ref string) => string.clone().into(),
                Value::Symbol(ref symbol) => symbol.clone().into(),
                primitive => primitive.to_string(context)?.into(),
            },
        })
    }

    /// It returns value converted to a numeric value of type `Number` or `BigInt`.
    ///
    /// See: <https://tc39.es/ecma262/#sec-tonumeric>
    pub fn to_numeric(&self, context: &mut Context) -> Result<Numeric> {
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
    pub fn to_u32(&self, context: &mut Context) -> Result<u32> {
        // This is the fast path, if the value is Integer we can just return it.
        if let Value::Integer(number) = *self {
            return Ok(number as u32);
        }
        let number = self.to_number(context)?;

        Ok(f64_to_uint32(number))
    }

    /// Converts a value to an integral 32 bit signed integer.
    ///
    /// See: <https://tc39.es/ecma262/#sec-toint32>
    pub fn to_i32(&self, context: &mut Context) -> Result<i32> {
        // This is the fast path, if the value is Integer we can just return it.
        if let Value::Integer(number) = *self {
            return Ok(number);
        }
        let number = self.to_number(context)?;

        Ok(f64_to_int32(number))
    }

    /// Converts a value to a non-negative integer if it is a valid integer index value.
    ///
    /// See: <https://tc39.es/ecma262/#sec-toindex>
    pub fn to_index(&self, context: &mut Context) -> Result<usize> {
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
    pub fn to_length(&self, context: &mut Context) -> Result<usize> {
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
    pub fn to_integer(&self, context: &mut Context) -> Result<f64> {
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
    pub fn to_number(&self, context: &mut Context) -> Result<f64> {
        match *self {
            Value::Null => Ok(0.0),
            Value::Undefined => Ok(f64::NAN),
            Value::Boolean(b) => Ok(if b { 1.0 } else { 0.0 }),
            Value::String(ref string) => {
                let string = string.trim_matches(is_trimmable_whitespace);

                // TODO: write our own lexer to match syntax StrDecimalLiteral
                match string {
                    "" => Ok(0.0),
                    "Infinity" | "+Infinity" => Ok(f64::INFINITY),
                    "-Infinity" => Ok(f64::NEG_INFINITY),
                    _ if matches!(
                        string
                            .chars()
                            .take(4)
                            .collect::<String>()
                            .to_ascii_lowercase()
                            .as_str(),
                        "inf" | "+inf" | "-inf" | "nan" | "+nan" | "-nan"
                    ) =>
                    {
                        // Prevent fast_float from parsing "inf", "+inf" as Infinity and "-inf" as -Infinity
                        Ok(f64::NAN)
                    }
                    _ => Ok(fast_float::parse(string).unwrap_or(f64::NAN)),
                }
            }
            Value::Rational(number) => Ok(number),
            Value::Integer(integer) => Ok(f64::from(integer)),
            Value::Symbol(_) => Err(context.construct_type_error("argument must not be a symbol")),
            Value::BigInt(_) => Err(context.construct_type_error("argument must not be a bigint")),
            Value::Object(_) => {
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
    pub fn to_numeric_number(&self, context: &mut Context) -> Result<f64> {
        let primitive = self.to_primitive(context, PreferredType::Number)?;
        if let Some(ref bigint) = primitive.as_bigint() {
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
    pub fn require_object_coercible(&self, context: &mut Context) -> Result<&Value> {
        if self.is_null_or_undefined() {
            Err(context.construct_type_error("cannot convert null or undefined to Object"))
        } else {
            Ok(self)
        }
    }

    #[inline]
    pub fn to_property_descriptor(&self, context: &mut Context) -> Result<PropertyDescriptor> {
        if let Self::Object(ref object) = self {
            object.to_property_descriptor(context)
        } else {
            Err(context.construct_type_error("Property description must be an object"))
        }
    }

    /// Converts argument to an integer, +∞, or -∞.
    ///
    /// See: <https://tc39.es/ecma262/#sec-tointegerorinfinity>
    pub fn to_integer_or_infinity(&self, context: &mut Context) -> Result<IntegerOrInfinity> {
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
}

impl Default for Value {
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
    BigInt(RcBigInt),
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

impl From<BigInt> for Numeric {
    #[inline]
    fn from(value: BigInt) -> Self {
        Self::BigInt(value.into())
    }
}

impl From<RcBigInt> for Numeric {
    #[inline]
    fn from(value: RcBigInt) -> Self {
        Self::BigInt(value)
    }
}

impl From<Numeric> for Value {
    fn from(value: Numeric) -> Self {
        match value {
            Numeric::Number(number) => Self::rational(number),
            Numeric::BigInt(bigint) => Self::bigint(bigint),
        }
    }
}
