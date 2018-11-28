use gc::{Gc, GcCell};
use js::function::{Function, NativeFunction, NativeFunctionData};
use js::object::{ObjectData, Property, INSTANCE_PROTOTYPE, PROTOTYPE};
use serde_json::map::Map;
use serde_json::Number as JSONNumber;
use serde_json::Value as JSONValue;
use std::collections::HashMap;
use std::f64::NAN;
use std::fmt;
use std::fmt::Display;
use std::iter::FromIterator;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};
use std::str::FromStr;

#[must_use]
/// The result of a Javascript expression is represented like this so it can succeed (`Ok`) or fail (`Err`)
pub type ResultValue = Result<Value, Value>;
/// A Garbage-collected Javascript value as represented in the interpreter
pub type Value = Gc<ValueData>;

/// A Javascript value
#[derive(Trace, Finalize, Debug, Clone)]
pub enum ValueData {
    /// `null` - A null value, for when a value doesn't exist
    Null,
    /// `undefined` - An undefined value, for when a field or index doesn't exist
    Undefined,
    /// `boolean` - A `true` / `false` value, for if a certain criteria is met
    Boolean(bool),
    /// `String` - A UTF-8 string, such as `"Hello, world"`
    String(String),
    /// `Number` - A 64-bit floating point number, such as `3.1415`
    Number(f64),
    /// `Number` - A 32-bit integer, such as `42`
    Integer(i32),
    /// `Object` - An object, such as `Math`, represented by a binary tree of string keys to Javascript values
    Object(GcCell<ObjectData>),
    /// `Function` - A runnable block of code, such as `Math.sqrt`, which can take some variables and return a useful value or act upon an object
    Function(GcCell<Function>),
}

impl ValueData {
    /// Returns a new empty object
    pub fn new_obj(global: Option<Value>) -> Value {
        let mut obj: ObjectData = HashMap::new();
        if global.is_some() {
            let obj_proto = global
                .unwrap()
                .get_field_slice("Object")
                .get_field_slice(PROTOTYPE);
            obj.insert(INSTANCE_PROTOTYPE.to_string(), Property::new(obj_proto));
        }
        Gc::new(ValueData::Object(GcCell::new(obj)))
    }

    /// Similar to `new_obj`, but you can pass a prototype to create from
    pub fn new_obj_from_prototype(proto: Value) -> Value {
        let mut obj: ObjectData = HashMap::new();
        obj.insert(INSTANCE_PROTOTYPE.to_string(), Property::new(proto));
        Gc::new(ValueData::Object(GcCell::new(obj)))
    }

    /// Returns true if the value is an object
    pub fn is_object(&self) -> bool {
        match *self {
            ValueData::Object(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is undefined
    pub fn is_undefined(&self) -> bool {
        match *self {
            ValueData::Undefined => true,
            _ => false,
        }
    }

    /// Returns true if the value is null
    pub fn is_null(&self) -> bool {
        match *self {
            ValueData::Null => true,
            _ => false,
        }
    }

    /// Returns true if the value is null or undefined
    pub fn is_null_or_undefined(&self) -> bool {
        match *self {
            ValueData::Null | ValueData::Undefined => true,
            _ => false,
        }
    }

    /// Returns true if the value is a 64-bit floating-point number
    pub fn is_double(&self) -> bool {
        match *self {
            ValueData::Number(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a string
    pub fn is_string(&self) -> bool {
        match *self {
            ValueData::String(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is true
    /// [toBoolean](https://tc39.github.io/ecma262/#sec-toboolean)
    pub fn is_true(&self) -> bool {
        match *self {
            ValueData::Object(_) => true,
            ValueData::String(ref s) if !s.is_empty() => true,
            ValueData::Number(n) if n >= 1.0 && n % 1.0 == 0.0 => true,
            ValueData::Integer(n) if n > 1 => true,
            ValueData::Boolean(v) => v,
            _ => false,
        }
    }

    /// Converts the value into a 64-bit floating point number
    pub fn to_num(&self) -> f64 {
        match *self {
            ValueData::Object(_) | ValueData::Undefined | ValueData::Function(_) => NAN,
            ValueData::String(ref str) => match FromStr::from_str(str) {
                Ok(num) => num,
                Err(_) => NAN,
            },
            ValueData::Number(num) => num,
            ValueData::Boolean(true) => 1.0,
            ValueData::Boolean(false) | ValueData::Null => 0.0,
            ValueData::Integer(num) => num as f64,
        }
    }

    /// Converts the value into a 32-bit integer
    pub fn to_int(&self) -> i32 {
        match *self {
            ValueData::Object(_)
            | ValueData::Undefined
            | ValueData::Null
            | ValueData::Boolean(false)
            | ValueData::Function(_) => 0,
            ValueData::String(ref str) => match FromStr::from_str(str) {
                Ok(num) => num,
                Err(_) => 0,
            },
            ValueData::Number(num) => num as i32,
            ValueData::Boolean(true) => 1,
            ValueData::Integer(num) => num,
        }
    }

    /// Resolve the property in the object
    /// Returns a copy of the Property
    pub fn get_prop(&self, field: String) -> Option<Property> {
        // handle length
        // Spidermonkey has its own GetLengthProperty: https://searchfox.org/mozilla-central/source/js/src/vm/Interpreter-inl.h#154
        // TODO: Maybe we need a GetLengthProperty for value types
        if let ValueData::String(ref s) = *self {
            return Some(Property::new(to_value(s.len() as f64)))
        }

        let obj: ObjectData = match *self {
            ValueData::Object(ref obj) => {
                let hash = obj.clone();
                hash.into_inner()
            }
            // Accesing .object on borrow() seems to automatically dereference it, so we don't need the *
            // ValueData::Function(ref func) => func.clone().object,
            _ => return None,
        };
        match obj.get(&field) {
            Some(val) => Some(val.clone()),
            None => match obj.get(&PROTOTYPE.to_string()) {
                Some(prop) => prop.value.get_prop(field),
                None => None,
            },
        }
    }

    /// Resolve the property in the object and get its value, or undefined if this is not an object or the field doesn't exist
    pub fn get_field(&self, field: String) -> Value {
        match self.get_prop(field) {
            Some(prop) => prop.value.clone(),
            None => Gc::new(ValueData::Undefined),
        }
    }

    /// Resolve the property in the object and get its value, or undefined if this is not an object or the field doesn't exist
    pub fn get_field_slice<'a>(&self, field: &'a str) -> Value {
        self.get_field(field.to_string())
    }

    /// Set the field in the value
    pub fn set_field(&self, field: String, val: Value) -> Value {
        match *self {
            ValueData::Object(ref obj) => {
                obj.borrow_mut()
                    .insert(field.clone(), Property::new(val.clone()));
            }
            ValueData::Function(ref func) => {
                match *func.borrow_mut().deref_mut() {
                    Function::NativeFunc(ref mut f) => {
                        f.object.insert(field.clone(), Property::new(val.clone()))
                    }
                    Function::RegularFunc(ref mut f) => {
                        f.object.insert(field.clone(), Property::new(val.clone()))
                    }
                };
            }
            _ => (),
        }
        val
    }

    /// Set the field in the value
    pub fn set_field_slice<'a>(&self, field: &'a str, val: Value) -> Value {
        self.set_field(field.to_string(), val)
    }

    /// Set the property in the value
    pub fn set_prop(&self, field: String, prop: Property) -> Property {
        match *self {
            ValueData::Object(ref obj) => {
                obj.borrow_mut().insert(field.clone(), prop.clone());
            }
            ValueData::Function(ref func) => {
                match *func.borrow_mut().deref_mut() {
                    Function::NativeFunc(ref mut f) => f.object.insert(field.clone(), prop.clone()),
                    Function::RegularFunc(ref mut f) => {
                        f.object.insert(field.clone(), prop.clone())
                    }
                };
            }
            _ => (),
        }
        prop
    }

    /// Set the property in the value
    pub fn set_prop_slice<'t>(&self, field: &'t str, prop: Property) -> Property {
        self.set_prop(field.to_string(), prop)
    }

    /// Convert from a JSON value to a JS value
    pub fn from_json(json: JSONValue) -> ValueData {
        match json {
            JSONValue::Number(v) => ValueData::Number(v.as_f64().unwrap()),
            JSONValue::String(v) => ValueData::String(v),
            JSONValue::Bool(v) => ValueData::Boolean(v),
            JSONValue::Array(vs) => {
                let mut i = 0;
                let mut data: ObjectData = FromIterator::from_iter(vs.iter().map(|json| {
                    i += 1;
                    (
                        (i - 1).to_string().to_string(),
                        Property::new(to_value(json.clone())),
                    )
                }));
                data.insert(
                    "length".to_string(),
                    Property::new(to_value(vs.len() as i32)),
                );
                ValueData::Object(GcCell::new(data))
            }
            JSONValue::Object(obj) => {
                let data: ObjectData = FromIterator::from_iter(
                    obj.iter()
                        .map(|(key, json)| (key.clone(), Property::new(to_value(json.clone())))),
                );
                ValueData::Object(GcCell::new(data))
            }
            JSONValue::Null => ValueData::Null,
        }
    }

    pub fn to_json(&self) -> JSONValue {
        match *self {
            ValueData::Null | ValueData::Undefined => JSONValue::Null,
            ValueData::Boolean(b) => JSONValue::Bool(b),
            ValueData::Object(ref obj) => {
                let mut nobj = Map::new();
                for (k, v) in obj.borrow().iter() {
                    if k != INSTANCE_PROTOTYPE {
                        nobj.insert(k.clone(), v.value.to_json());
                    }
                }
                JSONValue::Object(nobj)
            }
            ValueData::String(ref str) => JSONValue::String(str.clone()),
            ValueData::Number(num) => JSONValue::Number(JSONNumber::from_f64(num).unwrap()),
            ValueData::Integer(val) => JSONValue::Number(JSONNumber::from(val)),
            ValueData::Function(_) => JSONValue::Null,
        }
    }

    /// Get the type of the value
    pub fn get_type(&self) -> &'static str {
        match *self {
            ValueData::Number(_) | ValueData::Integer(_) => "number",
            ValueData::String(_) => "string",
            ValueData::Boolean(_) => "boolean",
            ValueData::Null => "null",
            ValueData::Undefined => "undefined",
            _ => "object",
        }
    }
}

impl Display for ValueData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValueData::Null => write!(f, "null"),
            ValueData::Undefined => write!(f, "undefined"),
            ValueData::Boolean(v) => write!(f, "{}", v),
            ValueData::String(ref v) => write!(f, "{}", v),
            ValueData::Number(v) => write!(
                f,
                "{}",
                match v {
                    _ if v.is_nan() => "NaN".to_string(),
                    _ if v.is_infinite() && v.is_sign_negative() => "-Infinity".to_string(),
                    _ if v.is_infinite() => "Infinity".to_string(),
                    _ => v.to_string(),
                }
            ),
            ValueData::Object(ref v) => {
                try!(write!(f, "{}", "{"));
                match v.borrow().iter().last() {
                    Some((last_key, _)) => {
                        for (key, val) in v.borrow().iter() {
                            try!(write!(f, "{}: {}", key, val.value.clone()));
                            if key != last_key {
                                try!(write!(f, "{}", ", "));
                            }
                        }
                    }
                    None => (),
                }
                write!(f, "{}", "}")
            }
            ValueData::Integer(v) => write!(f, "{}", v),
            ValueData::Function(ref v) => match *v.borrow() {
                Function::NativeFunc(_) => write!(f, "{}", "function() { [native code] }"),
                Function::RegularFunc(ref rf) => {
                    write!(f, "function({}){}", rf.args.join(", "), rf.expr)
                }
            },
        }
    }
}

impl PartialEq for ValueData {
    fn eq(&self, other: &ValueData) -> bool {
        match (self.clone(), other.clone()) {
            // TODO: fix this
            // _ if self.ptr.to_inner() == &other.ptr.to_inner() => true,
            _ if self.is_null_or_undefined() && other.is_null_or_undefined() => true,
            (ValueData::String(_), _) | (_, ValueData::String(_)) => {
                self.to_string() == other.to_string()
            }
            (ValueData::Boolean(a), ValueData::Boolean(b)) if a == b => true,
            (ValueData::Number(a), ValueData::Number(b))
                if a == b && !a.is_nan() && !b.is_nan() =>
            {
                true
            }
            (ValueData::Number(a), _) if a == other.to_num() => true,
            (_, ValueData::Number(a)) if a == self.to_num() => true,
            (ValueData::Integer(a), ValueData::Integer(b)) if a == b => true,
            _ => false,
        }
    }
}

impl Add for ValueData {
    type Output = ValueData;
    fn add(self, other: ValueData) -> ValueData {
        return match (self.clone(), other.clone()) {
            (ValueData::String(ref s), ref other) | (ref other, ValueData::String(ref s)) => {
                ValueData::String(s.clone() + &other.to_string())
            }
            (_, _) => ValueData::Number(self.to_num() + other.to_num()),
        };
    }
}
impl Sub for ValueData {
    type Output = ValueData;
    fn sub(self, other: ValueData) -> ValueData {
        ValueData::Number(self.to_num() - other.to_num())
    }
}
impl Mul for ValueData {
    type Output = ValueData;
    fn mul(self, other: ValueData) -> ValueData {
        ValueData::Number(self.to_num() * other.to_num())
    }
}
impl Div for ValueData {
    type Output = ValueData;
    fn div(self, other: ValueData) -> ValueData {
        ValueData::Number(self.to_num() / other.to_num())
    }
}
impl Rem for ValueData {
    type Output = ValueData;
    fn rem(self, other: ValueData) -> ValueData {
        ValueData::Number(self.to_num() % other.to_num())
    }
}
impl BitAnd for ValueData {
    type Output = ValueData;
    fn bitand(self, other: ValueData) -> ValueData {
        ValueData::Integer(self.to_int() & other.to_int())
    }
}
impl BitOr for ValueData {
    type Output = ValueData;
    fn bitor(self, other: ValueData) -> ValueData {
        ValueData::Integer(self.to_int() | other.to_int())
    }
}
impl BitXor for ValueData {
    type Output = ValueData;
    fn bitxor(self, other: ValueData) -> ValueData {
        ValueData::Integer(self.to_int() ^ other.to_int())
    }
}
impl Shl for ValueData {
    type Output = ValueData;
    fn shl(self, other: ValueData) -> ValueData {
        ValueData::Integer(self.to_int() << other.to_int())
    }
}
impl Shr for ValueData {
    type Output = ValueData;
    fn shr(self, other: ValueData) -> ValueData {
        ValueData::Integer(self.to_int() >> other.to_int())
    }
}
impl Not for ValueData {
    type Output = ValueData;
    fn not(self) -> ValueData {
        ValueData::Boolean(!self.is_true())
    }
}

/// Conversion to Javascript values from Rust values
pub trait ToValue {
    /// Convert this value to a Rust value
    fn to_value(&self) -> Value;
}
/// Conversion to Rust values from Javascript values
pub trait FromValue {
    /// Convert this value to a Javascript value
    fn from_value(value: Value) -> Result<Self, &'static str>
    where
        Self: Sized;
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::String(self.clone()))
    }
}

impl FromValue for String {
    fn from_value(v: Value) -> Result<String, &'static str> {
        Ok(v.to_string())
    }
}

impl<'s> ToValue for &'s str {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::String(String::from_str(*self).unwrap()))
    }
}

impl ToValue for char {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::String(self.to_string()))
    }
}
impl FromValue for char {
    fn from_value(v: Value) -> Result<char, &'static str> {
        Ok(v.to_string().chars().next().unwrap())
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Number(*self))
    }
}
impl FromValue for f64 {
    fn from_value(v: Value) -> Result<f64, &'static str> {
        Ok(v.to_num())
    }
}

impl ToValue for i32 {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Integer(*self))
    }
}
impl FromValue for i32 {
    fn from_value(v: Value) -> Result<i32, &'static str> {
        Ok(v.to_int())
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Boolean(*self))
    }
}
impl FromValue for bool {
    fn from_value(v: Value) -> Result<bool, &'static str> {
        Ok(v.is_true())
    }
}

impl<'s, T: ToValue> ToValue for &'s [T] {
    fn to_value(&self) -> Value {
        let mut arr = HashMap::new();
        let mut i = 0;
        for item in self.iter() {
            arr.insert(i.to_string(), Property::new(item.to_value()));
            i += 1;
        }
        to_value(arr)
    }
}
impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(&self) -> Value {
        let mut arr = HashMap::new();
        let mut i = 0;
        for item in self.iter() {
            arr.insert(i.to_string(), Property::new(item.to_value()));
            i += 1;
        }
        to_value(arr)
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(v: Value) -> Result<Vec<T>, &'static str> {
        let len = v.get_field_slice("length").to_int();
        let mut vec = Vec::with_capacity(len as usize);
        for i in 0..len {
            vec.push(try!(from_value(v.get_field(i.to_string()))))
        }
        Ok(vec)
    }
}

impl ToValue for ObjectData {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Object(GcCell::new(self.clone())))
    }
}

impl FromValue for ObjectData {
    fn from_value(v: Value) -> Result<ObjectData, &'static str> {
        match *v {
            ValueData::Object(ref obj) => Ok(obj.clone().into_inner()),
            ValueData::Function(ref func) => Ok(match *func.borrow().deref() {
                Function::NativeFunc(ref data) => data.object.clone(),
                Function::RegularFunc(ref data) => data.object.clone(),
            }),
            _ => Err("Value is not a valid object"),
        }
    }
}

impl ToValue for JSONValue {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::from_json(self.clone()))
    }
}

impl FromValue for JSONValue {
    fn from_value(v: Value) -> Result<JSONValue, &'static str> {
        Ok(v.to_json())
    }
}

impl ToValue for () {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Null)
    }
}
impl FromValue for () {
    fn from_value(_: Value) -> Result<(), &'static str> {
        Ok(())
    }
}

impl<T: ToValue> ToValue for Option<T> {
    fn to_value(&self) -> Value {
        match *self {
            Some(ref v) => v.to_value(),
            None => Gc::new(ValueData::Null),
        }
    }
}
impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: Value) -> Result<Option<T>, &'static str> {
        Ok(if value.is_null_or_undefined() {
            None
        } else {
            Some(try!(FromValue::from_value(value)))
        })
    }
}

impl ToValue for NativeFunctionData {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Function(GcCell::new(Function::NativeFunc(
            NativeFunction::new(*self),
        ))))
    }
}
impl FromValue for NativeFunctionData {
    fn from_value(v: Value) -> Result<NativeFunctionData, &'static str> {
        match *v {
            ValueData::Function(ref func) => match *func.borrow() {
                Function::NativeFunc(ref data) => Ok(data.data),
                _ => Err("Value is not a native function"),
            },
            _ => Err("Value is not a function"),
        }
    }
}

/// A utility function that just calls FromValue::from_value
pub fn from_value<A: FromValue>(v: Value) -> Result<A, &'static str> {
    FromValue::from_value(v)
}

/// A utility function that just calls ToValue::to_value
pub fn to_value<A: ToValue>(v: A) -> Value {
    v.to_value()
}
