use gc::{Gc, GcCell};
use js::object::{ObjectData, Property, INSTANCE_PROTOTYPE, PROTOTYPE};
use std::collections::HashMap;

/// The result of a Javascript expression is represented like this so it can succeed (`Ok`) or fail (`Err`)
pub type ResultValue = Result<Value, Value>;
/// A Garbage-collected Javascript value as represented in the interpreter
#[derive(Trace, Finalize)]
pub struct Value {
    /// The garbage-collected pointer
    pub ptr: Gc<ValueData>,
}

/// A Javascript value
#[derive(Trace, Finalize)]
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
}

impl Value {
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
        Value {
            ptr: Gc::new(ValueData::Object(GcCell::new(obj))),
        }
    }

    /// Returns true if the value is an object
    pub fn is_object(&self) -> bool {
        match *self.ptr {
            ValueData::Object(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is undefined
    pub fn is_undefined(&self) -> bool {
        match *self.ptr {
            ValueData::Undefined => true,
            _ => false,
        }
    }

    /// Returns true if the value is null
    pub fn is_null(&self) -> bool {
        match *self.ptr {
            ValueData::Null => true,
            _ => false,
        }
    }

    /// Returns true if the value is null or undefined
    pub fn is_null_or_undefined(&self) -> bool {
        match *self.ptr {
            ValueData::Null | ValueData::Undefined => true,
            _ => false,
        }
    }

    /// Returns true if the value is a 64-bit floating-point number
    pub fn is_double(&self) -> bool {
        match *self.ptr {
            ValueData::Number(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is a string
    pub fn is_string(&self) -> bool {
        match *self.ptr {
            ValueData::String(_) => true,
            _ => false,
        }
    }

    /// Returns true if the value is true
    /// [toBoolean](https://tc39.github.io/ecma262/#sec-toboolean)
    pub fn is_true(&self) -> bool {
        match *self.ptr {
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
        match *self.ptr {
            ValueData::Object(_) | ValueData::Undefined | ValueData::Function(_) => std::f64::NAN,
            ValueData::String(ref str) => match from_str(str) {
                Some(num) => num,
                None => std::f64::NAN,
            },
            ValueData::Number(num) => num,
            ValueData::Boolean(true) => 1.0,
            ValueData::Boolean(false) | ValueData::Null => 0.0,
            ValueData::Integer(num) => num as f64,
        }
    }
}
