use super::InnerValue;
use crate::{JsBigInt, JsObject, JsSymbol, JsValue};
use boa_string::JsString;

/// A non-mutable variant of a JsValue.
/// Represents either a primitive value ([`bool`], [`f64`], [`i32`]) or a reference
/// to a heap allocated value ([`JsString`], [`JsSymbol`]).
///
/// References to heap allocated values are represented by [`Ref`], since
/// more exotic implementations of [`JsValue`] such as nan-boxed ones cannot
/// effectively return references.
#[derive(Debug)]
pub enum JsVariant<'a> {
    /// `null` - A null value, for when a value doesn't exist.
    Null,
    /// `undefined` - An undefined value, for when a field or index doesn't exist.
    Undefined,
    /// `boolean` - A `true` / `false` value, for if a certain criteria is met.
    Boolean(bool),
    /// `String` - A UTF-16 string, such as `"Hello, world"`.
    String(&'a JsString),
    /// `Number` - A 64-bit floating point number, such as `3.1415` or `Infinity`.
    /// This is the default representation of a number. If a number can be represented
    /// as an integer, it will be stored as an `Integer` variant instead.
    Float64(f64),
    /// `Number` - A 32-bit integer, such as `42`.
    Integer32(i32),
    /// `BigInt` - holds any arbitrary large signed integer.
    BigInt(&'a JsBigInt),
    /// `Object` - An object, such as `Math`, represented by a binary tree of string keys to Javascript values.
    Object(&'a JsObject),
    /// `Symbol` - A Symbol Primitive type.
    Symbol(&'a JsSymbol),
}

impl<'a> From<&'a InnerValue> for JsVariant<'a> {
    fn from(value: &'a InnerValue) -> Self {
        match value {
            InnerValue::Null => JsVariant::Null,
            InnerValue::Undefined => JsVariant::Undefined,
            InnerValue::Integer32(i) => JsVariant::Integer32(*i),
            InnerValue::Float64(d) => JsVariant::Float64(*d),
            InnerValue::Boolean(b) => JsVariant::Boolean(*b),
            InnerValue::Object(inner) => JsVariant::Object(inner),
            InnerValue::String(inner) => JsVariant::String(inner),
            InnerValue::Symbol(inner) => JsVariant::Symbol(inner),
            InnerValue::BigInt(inner) => JsVariant::BigInt(inner),
        }
    }
}

impl<'a> From<JsVariant<'a>> for JsValue {
    fn from(value: JsVariant<'a>) -> Self {
        match value {
            JsVariant::Null => JsValue::null(),
            JsVariant::Undefined => JsValue::undefined(),
            JsVariant::Boolean(b) => JsValue::new(b),
            JsVariant::String(s) => JsValue::new(s.clone()),
            JsVariant::Float64(f) => JsValue::new(f),
            JsVariant::Integer32(i) => JsValue::new(i),
            JsVariant::BigInt(b) => JsValue::new(b.clone()),
            JsVariant::Object(o) => JsValue::new(o.clone()),
            JsVariant::Symbol(s) => JsValue::new(s.clone()),
        }
    }
}
