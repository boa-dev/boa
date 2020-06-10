//! This module implements the JavaScript Value.
//!
//! Javascript values, utility methods and conversion between Javascript values and Rust values.

#[cfg(test)]
mod tests;

pub mod val_type;

pub use crate::builtins::value::val_type::Type;

use crate::builtins::{
    object::{
        internal_methods_trait::ObjectInternalMethods, InternalState, InternalStateCell, Object,
        ObjectKind, INSTANCE_PROTOTYPE, PROTOTYPE,
    },
    property::Property,
    BigInt, Function,
};
use crate::BoaProfiler;

use crate::exec::Interpreter;
use gc::{Finalize, Gc, GcCell, GcCellRef, Trace};
use serde_json::{map::Map, Number as JSONNumber, Value as JSONValue};
use std::{
    any::Any,
    collections::HashSet,
    convert::TryFrom,
    f64::NAN,
    fmt::{self, Display},
    ops::{Add, BitAnd, BitOr, BitXor, Deref, DerefMut, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub},
    str::FromStr,
};

pub mod conversions;
pub mod display;
pub mod equality;
pub mod operations;

pub use conversions::*;
pub(crate) use display::display_obj;
pub use equality::*;
pub use operations::*;

/// The result of a Javascript expression is represented like this so it can succeed (`Ok`) or fail (`Err`)
#[must_use]
pub type ResultValue = Result<Value, Value>;

/// A Garbage-collected Javascript value as represented in the interpreter.
#[derive(Debug, Clone, Trace, Finalize, Default)]
pub struct Value(pub(crate) Gc<ValueData>);

impl Value {
    /// Creates a new `undefined` value.
    #[inline]
    pub fn undefined() -> Self {
        Self(Gc::new(ValueData::Undefined))
    }

    /// Creates a new `null` value.
    #[inline]
    pub fn null() -> Self {
        Self(Gc::new(ValueData::Null))
    }

    /// Creates a new string value.
    #[inline]
    pub fn string<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        Self(Gc::new(ValueData::String(value.into())))
    }

    /// Creates a new number value.
    #[inline]
    pub fn rational<N>(value: N) -> Self
    where
        N: Into<f64>,
    {
        Self(Gc::new(ValueData::Rational(value.into())))
    }

    /// Creates a new number value.
    #[inline]
    pub fn integer<I>(value: I) -> Self
    where
        I: Into<i32>,
    {
        Self(Gc::new(ValueData::Integer(value.into())))
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
    pub fn bigint(value: BigInt) -> Self {
        Self(Gc::new(ValueData::BigInt(value)))
    }

    /// Creates a new boolean value.
    #[inline]
    pub fn boolean(value: bool) -> Self {
        Self(Gc::new(ValueData::Boolean(value)))
    }

    /// Creates a new object value.
    #[inline]
    pub fn object(object: Object) -> Self {
        Self(Gc::new(ValueData::Object(Box::new(GcCell::new(object)))))
    }

    /// Gets the underlying `ValueData` structure.
    #[inline]
    pub fn data(&self) -> &ValueData {
        &*self.0
    }

    /// Helper function to convert the `Value` to a number and compute its power.
    pub fn as_num_to_power(&self, other: Self) -> Self {
        match (self.data(), other.data()) {
            (ValueData::BigInt(ref a), ValueData::BigInt(ref b)) => Self::bigint(a.clone().pow(b)),
            (a, b) => Self::rational(a.to_number().powf(b.to_number())),
        }
    }

    /// Returns a new empty object
    pub fn new_object(global: Option<&Value>) -> Self {
        let _timer = BoaProfiler::global().start_event("new_object", "value");
        if let Some(global) = global {
            let object_prototype = global.get_field("Object").get_field(PROTOTYPE);

            let object = Object::create(object_prototype);
            Self::object(object)
        } else {
            Self::object(Object::default())
        }
    }

    /// Similar to `new_object`, but you can pass a prototype to create from, plus a kind
    pub fn new_object_from_prototype(proto: Value, kind: ObjectKind) -> Self {
        let mut object = Object::default();
        object.kind = kind;

        object
            .internal_slots
            .insert(INSTANCE_PROTOTYPE.to_string(), proto);

        Self::object(object)
    }

    /// Convert from a JSON value to a JS value
    pub fn from_json(json: JSONValue, interpreter: &mut Interpreter) -> Self {
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
                let global_array_prototype = interpreter
                    .realm
                    .global_obj
                    .get_field("Array")
                    .get_field(PROTOTYPE);
                let new_obj =
                    Value::new_object_from_prototype(global_array_prototype, ObjectKind::Array);
                let length = vs.len();
                for (idx, json) in vs.into_iter().enumerate() {
                    new_obj.set_property(
                        idx.to_string(),
                        Property::default()
                            .value(Self::from_json(json, interpreter))
                            .writable(true)
                            .configurable(true),
                    );
                }
                new_obj.set_property(
                    "length".to_string(),
                    Property::default().value(Self::from(length)),
                );
                new_obj
            }
            JSONValue::Object(obj) => {
                let new_obj = Value::new_object(Some(&interpreter.realm.global_obj));
                for (key, json) in obj.into_iter() {
                    let value = Self::from_json(json, interpreter);
                    new_obj.set_property(
                        key,
                        Property::default()
                            .value(value)
                            .writable(true)
                            .configurable(true),
                    );
                }
                new_obj
            }
            JSONValue::Null => Self::null(),
        }
    }

    /// Conversts the `Value` to `JSON`.
    pub fn to_json(&self, interpreter: &mut Interpreter) -> Result<JSONValue, Value> {
        match *self.data() {
            ValueData::Null => Ok(JSONValue::Null),
            ValueData::Boolean(b) => Ok(JSONValue::Bool(b)),
            ValueData::Object(ref obj) => {
                if obj.borrow().kind == ObjectKind::Array {
                    let mut arr: Vec<JSONValue> = Vec::new();
                    for k in obj.borrow().properties.keys() {
                        if k != "length" {
                            let value = self.get_field(k.to_string());
                            if value.is_undefined() || value.is_function() {
                                arr.push(JSONValue::Null);
                            } else {
                                arr.push(self.get_field(k.to_string()).to_json(interpreter)?);
                            }
                        }
                    }
                    Ok(JSONValue::Array(arr))
                } else {
                    let mut new_obj = Map::new();
                    for k in obj.borrow().properties.keys() {
                        let key = k.clone();
                        let value = self.get_field(k.to_string());
                        if !value.is_undefined() && !value.is_function() {
                            new_obj.insert(key, value.to_json(interpreter)?);
                        }
                    }
                    Ok(JSONValue::Object(new_obj))
                }
            }
            ValueData::String(ref str) => Ok(JSONValue::String(str.clone())),
            ValueData::Rational(num) => Ok(JSONNumber::from_f64(num)
                .map(JSONValue::Number)
                .unwrap_or(JSONValue::Null)),
            ValueData::Integer(val) => Ok(JSONValue::Number(JSONNumber::from(val))),
            ValueData::BigInt(_) => Err(interpreter
                .throw_type_error("BigInt value can't be serialized in JSON")
                .expect_err("throw_type_error should always return an error")),
            ValueData::Symbol(_) | ValueData::Undefined => {
                unreachable!("Symbols and Undefined JSON Values depend on parent type");
            }
        }
    }
}

impl Deref for Value {
    type Target = ValueData;

    fn deref(&self) -> &Self::Target {
        self.data()
    }
}

/// A Javascript value
#[derive(Trace, Finalize, Debug, Clone)]
pub enum ValueData {
    /// `null` - A null value, for when a value doesn't exist.
    Null,
    /// `undefined` - An undefined value, for when a field or index doesn't exist.
    Undefined,
    /// `boolean` - A `true` / `false` value, for if a certain criteria is met.
    Boolean(bool),
    /// `String` - A UTF-8 string, such as `"Hello, world"`.
    String(String),
    /// `Number` - A 64-bit floating point number, such as `3.1415`
    Rational(f64),
    /// `Number` - A 32-bit integer, such as `42`.
    Integer(i32),
    /// `BigInt` - holds any arbitrary large signed integer.
    BigInt(BigInt),
    /// `Object` - An object, such as `Math`, represented by a binary tree of string keys to Javascript values.
    Object(Box<GcCell<Object>>),
    /// `Symbol` - A Symbol Type - Internally Symbols are similar to objects, except there are no properties, only internal slots.
    Symbol(Box<GcCell<Object>>),
}

impl ValueData {
    /// This will tell us if we can exten an object or not, not properly implemented yet
    ///
    /// For now always returns true.
    ///
    /// For scalar types it should be false, for objects check the private field for extensibilaty.
    /// By default true.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/seal would turn extensible to false/>
    /// <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/freeze would also turn extensible to false/>
    pub fn is_extensible(&self) -> bool {
        true
    }

    /// Returns true if the value is an object
    pub fn is_object(&self) -> bool {
        match *self {
            Self::Object(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a symbol
    pub fn is_symbol(&self) -> bool {
        match *self {
            Self::Symbol(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a function
    pub fn is_function(&self) -> bool {
        match *self {
            Self::Object(ref o) => {
                let borrowed_obj = o.borrow();
                borrowed_obj.is_callable() || borrowed_obj.is_constructable()
            }
            _ => false,
        }
    }

    /// Returns true if the value is undefined.
    pub fn is_undefined(&self) -> bool {
        match *self {
            Self::Undefined => true,
            _ => false,
        }
    }

    /// Returns true if the value is null.
    pub fn is_null(&self) -> bool {
        match *self {
            Self::Null => true,
            _ => false,
        }
    }

    /// Returns true if the value is null or undefined.
    pub fn is_null_or_undefined(&self) -> bool {
        match *self {
            Self::Null | Self::Undefined => true,
            _ => false,
        }
    }

    /// Returns true if the value is a 64-bit floating-point number.
    pub fn is_double(&self) -> bool {
        match *self {
            Self::Rational(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is integer.
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

    /// Returns true if the value is a number
    pub fn is_number(&self) -> bool {
        match self {
            Self::Rational(_) | Self::Integer(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a string
    pub fn is_string(&self) -> bool {
        match *self {
            Self::String(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a boolean
    pub fn is_boolean(&self) -> bool {
        match *self {
            Self::Boolean(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a bigint
    pub fn is_bigint(&self) -> bool {
        match *self {
            Self::BigInt(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is true
    ///
    /// [toBoolean](https://tc39.es/ecma262/#sec-toboolean)
    pub fn is_true(&self) -> bool {
        match *self {
            Self::Object(_) => true,
            Self::String(ref s) if !s.is_empty() => true,
            Self::Rational(n) if n != 0.0 && !n.is_nan() => true,
            Self::Integer(n) if n != 0 => true,
            Self::Boolean(v) => v,
            Self::BigInt(ref n) if *n != 0 => true,
            _ => false,
        }
    }

    /// Converts the value into a 64-bit floating point number
    pub fn to_number(&self) -> f64 {
        match *self {
            Self::Object(_) | Self::Symbol(_) | Self::Undefined => NAN,
            Self::String(ref str) => {
                if str.is_empty() {
                    return 0.0;
                }

                match FromStr::from_str(str) {
                    Ok(num) => num,
                    Err(_) => NAN,
                }
            }
            Self::Boolean(true) => 1.0,
            Self::Boolean(false) | Self::Null => 0.0,
            Self::Rational(num) => num,
            Self::Integer(num) => f64::from(num),
            Self::BigInt(_) => {
                panic!("TypeError: Cannot mix BigInt and other types, use explicit conversions")
            }
        }
    }

    /// Converts the value into a 32-bit integer
    pub fn to_integer(&self) -> i32 {
        match *self {
            Self::Object(_)
            | Self::Undefined
            | Self::Symbol(_)
            | Self::Null
            | Self::Boolean(false) => 0,
            Self::String(ref str) => match FromStr::from_str(str) {
                Ok(num) => num,
                Err(_) => 0,
            },
            Self::Rational(num) => num as i32,
            Self::Boolean(true) => 1,
            Self::Integer(num) => num,
            Self::BigInt(_) => {
                panic!("TypeError: Cannot mix BigInt and other types, use explicit conversions")
            }
        }
    }

    pub fn as_object(&self) -> Option<GcCellRef<'_, Object>> {
        match *self {
            ValueData::Object(ref o) => Some(o.borrow()),
            _ => None,
        }
    }

    /// Removes a property from a Value object.
    ///
    /// It will return a boolean based on if the value was removed, if there was no value to remove false is returned
    pub fn remove_property(&self, field: &str) -> bool {
        let removed = match *self {
            Self::Object(ref obj) => obj.borrow_mut().deref_mut().properties.remove(field),
            _ => None,
        };

        removed.is_some()
    }

    /// Resolve the property in the object.
    ///
    /// A copy of the Property is returned.
    pub fn get_property(&self, field: &str) -> Option<Property> {
        let _timer = BoaProfiler::global().start_event("Value::get_property", "value");
        // Spidermonkey has its own GetLengthProperty: https://searchfox.org/mozilla-central/source/js/src/vm/Interpreter-inl.h#154
        // This is only for primitive strings, String() objects have their lengths calculated in string.rs
        if self.is_string() && field == "length" {
            if let Self::String(ref s) = *self {
                return Some(Property::default().value(Value::from(s.len())));
            }
        }

        if self.is_undefined() {
            return None;
        }

        let obj: Object = match *self {
            Self::Object(ref obj) => {
                let hash = obj.clone();
                // TODO: This will break, we should return a GcCellRefMut instead
                // into_inner will consume the wrapped value and remove it from the hashmap
                hash.into_inner()
            }
            Self::Symbol(ref obj) => {
                let hash = obj.clone();
                hash.into_inner()
            }
            _ => return None,
        };

        match obj.properties.get(field) {
            Some(val) => Some(val.clone()),
            None => match obj.internal_slots.get(&INSTANCE_PROTOTYPE.to_string()) {
                Some(value) => value.get_property(field),
                None => None,
            },
        }
    }

    /// update_prop will overwrite individual [Property] fields, unlike
    /// Set_prop, which will overwrite prop with a new Property
    /// Mostly used internally for now
    pub fn update_property(
        &self,
        field: &str,
        value: Option<Value>,
        enumerable: Option<bool>,
        writable: Option<bool>,
        configurable: Option<bool>,
    ) {
        let _timer = BoaProfiler::global().start_event("Value::update_property", "value");
        let obj: Option<Object> = match self {
            Self::Object(ref obj) => Some(obj.borrow_mut().deref_mut().clone()),
            _ => None,
        };

        if let Some(mut obj_data) = obj {
            // Use value, or walk up the prototype chain
            if let Some(ref mut prop) = obj_data.properties.get_mut(field) {
                prop.value = value;
                prop.enumerable = enumerable;
                prop.writable = writable;
                prop.configurable = configurable;
            }
        }
    }

    /// Resolve the property in the object.
    ///
    /// Returns a copy of the Property.
    pub fn get_internal_slot(&self, field: &str) -> Value {
        let _timer = BoaProfiler::global().start_event("Value::get_internal_slot", "value");
        let obj: Object = match *self {
            Self::Object(ref obj) => {
                let hash = obj.clone();
                hash.into_inner()
            }
            Self::Symbol(ref obj) => {
                let hash = obj.clone();
                hash.into_inner()
            }
            _ => return Value::undefined(),
        };

        match obj.internal_slots.get(field) {
            Some(val) => val.clone(),
            None => Value::undefined(),
        }
    }

    /// Resolve the property in the object and get its value, or undefined if this is not an object or the field doesn't exist
    /// get_field recieves a Property from get_prop(). It should then return the [[Get]] result value if that's set, otherwise fall back to [[Value]]
    /// TODO: this function should use the get Value if its set
    pub fn get_field<F>(&self, field: F) -> Value
    where
        F: Into<Value>,
    {
        let _timer = BoaProfiler::global().start_event("Value::get_field", "value");
        match *field.into() {
            // Our field will either be a String or a Symbol
            Self::String(ref s) => {
                match self.get_property(s) {
                    Some(prop) => {
                        // If the Property has [[Get]] set to a function, we should run that and return the Value
                        let prop_getter = match prop.get {
                            Some(_) => None,
                            None => None,
                        };

                        // If the getter is populated, use that. If not use [[Value]] instead
                        if let Some(val) = prop_getter {
                            val
                        } else {
                            let val = prop
                                .value
                                .as_ref()
                                .expect("Could not get property as reference");
                            val.clone()
                        }
                    }
                    None => Value::undefined(),
                }
            }
            Self::Symbol(_) => unimplemented!(),
            _ => Value::undefined(),
        }
    }

    /// Check whether an object has an internal state set.
    pub fn has_internal_state(&self) -> bool {
        if let Self::Object(ref obj) = *self {
            obj.borrow().state.is_some()
        } else {
            false
        }
    }

    /// Get the internal state of an object.
    pub fn get_internal_state(&self) -> Option<InternalStateCell> {
        if let Self::Object(ref obj) = *self {
            obj.borrow().state.as_ref().cloned()
        } else {
            None
        }
    }

    /// Run a function with a reference to the internal state.
    ///
    /// # Panics
    ///
    /// This will panic if this value doesn't have an internal state or if the internal state doesn't
    /// have the concrete type `S`.
    pub fn with_internal_state_ref<S: Any + InternalState, R, F: FnOnce(&S) -> R>(
        &self,
        f: F,
    ) -> R {
        if let Self::Object(ref obj) = *self {
            let o = obj.borrow();
            let state = o
                .state
                .as_ref()
                .expect("no state")
                .downcast_ref()
                .expect("wrong state type");
            f(state)
        } else {
            panic!("not an object");
        }
    }

    /// Run a function with a mutable reference to the internal state.
    ///
    /// # Panics
    ///
    /// This will panic if this value doesn't have an internal state or if the internal state doesn't
    /// have the concrete type `S`.
    pub fn with_internal_state_mut<S: Any + InternalState, R, F: FnOnce(&mut S) -> R>(
        &self,
        f: F,
    ) -> R {
        if let Self::Object(ref obj) = *self {
            let mut o = obj.borrow_mut();
            let state = o
                .state
                .as_mut()
                .expect("no state")
                .downcast_mut()
                .expect("wrong state type");
            f(state)
        } else {
            panic!("not an object");
        }
    }

    /// Check to see if the Value has the field, mainly used by environment records
    pub fn has_field(&self, field: &str) -> bool {
        let _timer = BoaProfiler::global().start_event("Value::has_field", "value");
        self.get_property(field).is_some()
    }

    /// Set the field in the value
    /// Field could be a Symbol, so we need to accept a Value (not a string)
    pub fn set_field<F, V>(&self, field: F, val: V) -> Value
    where
        F: Into<Value>,
        V: Into<Value>,
    {
        let _timer = BoaProfiler::global().start_event("Value::set_field", "value");
        let field = field.into();
        let val = val.into();

        if let Self::Object(ref obj) = *self {
            if obj.borrow().kind == ObjectKind::Array {
                if let Ok(num) = field.to_string().parse::<usize>() {
                    if num > 0 {
                        let len = i32::from(&self.get_field("length"));
                        if len < (num + 1) as i32 {
                            self.set_field("length", Value::from(num + 1));
                        }
                    }
                }
            }

            // Symbols get saved into a different bucket to general properties
            if field.is_symbol() {
                obj.borrow_mut().set(field, val.clone());
            } else {
                obj.borrow_mut()
                    .set(Value::from(field.to_string()), val.clone());
            }
        }

        val
    }

    /// Set the private field in the value
    pub fn set_internal_slot(&self, field: &str, val: Value) -> Value {
        let _timer = BoaProfiler::global().start_event("Value::set_internal_slot", "exec");
        if let Self::Object(ref obj) = *self {
            obj.borrow_mut()
                .internal_slots
                .insert(field.to_string(), val.clone());
        }
        val
    }

    /// Set the kind of an object
    pub fn set_kind(&self, kind: ObjectKind) {
        if let Self::Object(ref obj) = *self {
            (*obj.deref().borrow_mut()).kind = kind;
        }
    }

    /// Set the property in the value
    pub fn set_property(&self, field: String, prop: Property) -> Property {
        if let Self::Object(ref obj) = *self {
            obj.borrow_mut().properties.insert(field, prop.clone());
        }
        prop
    }

    /// Set the property in the value
    pub fn set_property_slice(&self, field: &str, prop: Property) -> Property {
        self.set_property(field.to_string(), prop)
    }

    /// Set internal state of an Object. Discards the previous state if it was set.
    pub fn set_internal_state<T: Any + InternalState>(&self, state: T) {
        if let Self::Object(ref obj) = *self {
            obj.borrow_mut()
                .state
                .replace(InternalStateCell::new(state));
        }
    }

    /// Consume the function and return a Value
    pub fn from_func(native_func: Function) -> Value {
        // Object with Kind set to function
        let mut new_func = crate::builtins::object::Object::function();
        // Get Length
        let length = native_func.params.len();
        // Set [[Call]] internal slot
        new_func.set_func(native_func);
        // Wrap Object in GC'd Value
        let new_func_val = Value::from(new_func);
        // Set length to parameters
        new_func_val.set_field("length", Value::from(length));
        new_func_val
    }
}

impl Default for ValueData {
    fn default() -> Self {
        Self::Undefined
    }
}
