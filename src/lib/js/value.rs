use gc::GcCell;
use js::function::Function;
use js::object::{ObjectData, Property, INSTANCE_PROTOTYPE, PROTOTYPE};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

/// The result of a Javascript expression is represented like this so it can succeed (`Ok`) or fail (`Err`)
pub type ResultValue = Result<Value, Value>;
/// A Garbage-collected Javascript value as represented in the interpreter
pub type Value = GcCell<ValueData>;

/// A Javascript value
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
    Object(RefCell<ObjectData>),
    /// `Function` - A runnable block of code, such as `Math.sqrt`, which can take some variables and return a useful value or act upon an object
    Function(RefCell<Function>),
}

impl ValueData {
    /// Returns a new empty object
    pub fn new_obj(global: Option<Value>) -> Value {
        let mut obj: ObjectData = HashMap::new();
        if global.is_some() {
            let obj_proto = global
                .unwrap()
                .borrow()
                .get_field_slice("Object")
                .borrow()
                .get_field_slice(PROTOTYPE);
            obj.insert(INSTANCE_PROTOTYPE.into_String(), Property::new(obj_proto));
        }
        GcCell::new(ValueData::Object(RefCell::new(obj)))
    }
    /// Returns true if the value is an object
    pub fn is_object(&self) -> bool {
        return match *self {
            ValueData::Object(_) => true,
            _ => false,
        };
    }
    /// Returns true if the value is undefined
    pub fn is_undefined(&self) -> bool {
        return match *self {
            ValueData::Undefined => true,
            _ => false,
        };
    }
    /// Returns true if the value is null
    pub fn is_null(&self) -> bool {
        return match *self {
            ValueData::Null => true,
            _ => false,
        };
    }
    /// Returns true if the value is null or undefined
    pub fn is_null_or_undefined(&self) -> bool {
        return match *self {
            ValueData::Null | ValueData::Undefined => true,
            _ => false,
        };
    }
    /// Returns true if the value is a 64-bit floating-point number
    pub fn is_double(&self) -> bool {
        return match *self {
            ValueData::Number(_) => true,
            _ => false,
        };
    }
    /// Returns true if the value is true
    pub fn is_true(&self) -> bool {
        return match *self {
            ValueData::Object(_) => true,
            ValueData::String(ref s) if s.as_slice() == "1" => true,
            ValueData::Number(n) if n >= 1.0 && n % 1.0 == 0.0 => true,
            ValueData::Integer(n) if n > 1 => true,
            ValueData::Boolean(v) => v,
            _ => false,
        };
    }
    /// Converts the value into a 64-bit floating point number
    pub fn to_num(&self) -> f64 {
        return match *self {
            ValueData::Object(_) | ValueData::Undefined | ValueData::Function(_) => f64::NAN,
            ValueData::String(ref str) => match FromStr::from_str(str.as_slice()) {
                Some(num) => num,
                None => f64::NAN,
            },
            ValueData::Number(num) => num,
            ValueData::Boolean(true) => 1.0,
            ValueData::Boolean(false) | ValueData::Null => 0.0,
            ValueData::Integer(num) => num as f64,
        };
    }
    /// Converts the value into a 32-bit integer
    pub fn to_int(&self) -> i32 {
        return match *self {
            ValueData::Object(_)
            | ValueData::Undefined
            | ValueData::Null
            | ValueData::Boolean(false)
            | ValueData::Function(_) => 0,
            ValueData::String(ref str) => match FromStr::from_str(str.as_slice()) {
                Some(num) => num,
                None => 0,
            },
            ValueData::Number(num) => num as i32,
            ValueData::Boolean(true) => 1,
            ValueData::Integer(num) => num,
        };
    }
    /// Resolve the property in the object
    pub fn get_prop(&self, field: String) -> Option<Property> {
        let obj: ObjectData = match *self {
            ValueData::Object(ref obj) => obj.borrow().clone(),
            ValueData::Function(ref func) => {
                let func = func.borrow().clone();
                match func {
                    Function::NativeFunc(f) => f.object.clone(),
                    Function::RegularFunc(f) => f.object.clone(),
                }
            }
            _ => return None,
        };
        match obj.find(&field) {
            Some(val) => Some(*val),
            None => match obj.find(&PROTOTYPE.into_String()) {
                Some(prop) => prop.value.borrow().get_prop(field),
                None => None,
            },
        }
    }
    /// Resolve the property in the object and get its value, or undefined if this is not an object or the field doesn't exist
    pub fn get_field(&self, field: String) -> Value {
        match self.get_prop(field) {
            Some(prop) => prop.value,
            None => GcCell::new(ValueData::Undefined),
        }
    }
    /// Resolve the property in the object and get its value, or undefined if this is not an object or the field doesn't exist
    pub fn get_field_slice<'t>(&self, field: &'t str) -> Value {
        self.get_field(field.into_String())
    }
    /// Set the field in the value
    pub fn set_field(&self, field: String, val: Value) -> Value {
        match *self {
            ValueData::Object(ref obj) => {
                obj.borrow_mut().insert(field.clone(), Property::new(val));
            }
            ValueData::Function(ref func) => {
                match *func.borrow_mut().deref_mut() {
                    Function::NativeFunc(ref mut f) => {
                        f.object.insert(field.clone(), Property::new(val))
                    }
                    Function::RegularFunc(ref mut f) => {
                        f.object.insert(field.clone(), Property::new(val))
                    }
                };
            }
            _ => (),
        }
        val
    }
    /// Set the field in the value
    pub fn set_field_slice<'t>(&self, field: &'t str, val: Value) -> Value {
        self.set_field(field.into_String(), val)
    }
    /// Set the property in the value
    pub fn set_prop(&self, field: String, prop: Property) -> Property {
        match *self {
            ValueData::Object(ref obj) => {
                obj.borrow_mut().insert(field.clone(), prop);
            }
            ValueData::Function(ref func) => {
                match *func.borrow_mut().deref_mut() {
                    Function::NativeFunc(ref mut f) => f.object.insert(field.clone(), prop),
                    Function::RegularFunc(ref mut f) => f.object.insert(field.clone(), prop),
                };
            }
            _ => (),
        }
        prop
    }
    /// Set the property in the value
    pub fn set_prop_slice<'t>(&self, field: &'t str, prop: Property) -> Property {
        self.set_prop(field.into_String(), prop)
    }
    /// Convert from a JSON value to a JS value
    pub fn from_json(json: serde_json::Value) -> ValueData {
        match json {
            serde_json::Value::Number(v) => ValueData::Number(v),
            serde_json::Value::String(v) => ValueData::String(v),
            serde_json::Value::Boolean(v) => ValueData::Boolean(v),
            serde_json::Value::List(vs) => {
                let mut i = 0;
                let mut data: ObjectData = FromIterator::from_iter(vs.iter().map(|json| {
                    i += 1;
                    (
                        (i - 1).to_str().into_String(),
                        Property::new(to_value(json.clone())),
                    )
                }));
                data.insert(
                    "length".into_String(),
                    Property::new(to_value(vs.len() as i32)),
                );
                ValueData::Object(RefCell::new(data))
            }
            serde_json::Value::Object(obj) => {
                let data: ObjectData = FromIterator::from_iter(
                    obj.iter()
                        .map(|(key, json)| (key.clone(), Property::new(to_value(json.clone())))),
                );
                ValueData::Object(RefCell::new(data))
            }
            Null => ValueData::Null,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match *self {
            ValueData::Null | ValueData::Undefined => serde_json::Value::Null,
            ValueData::Boolean(b) => serde_json::Value::Boolean(b),
            ValueData::Object(ref obj) => {
                let mut nobj = HashMap::new();
                for (k, v) in obj.borrow().iter() {
                    if k.as_slice() != INSTANCE_PROTOTYPE.as_slice() {
                        nobj.insert(k.clone(), v.value.borrow().to_json());
                    }
                }
                serde_json::Value::Object(Box::new(nobj))
            }
            ValueData::String(ref str) => serde_json::Value::String(str.clone()),
            ValueData::Number(num) => serde_json::Value::Number(num),
            ValueData::Integer(val) => serde_json::Value::Number(val as f64),
            ValueData::Function(_) => serde_json::Value::Null,
        }
    }
}

impl fmt::Display for ValueData {
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
                    _ if v.is_nan() => "NaN".into_String(),
                    f64::INFINITY => "Infinity".into_String(),
                    f64::NEG_INFINITY => "-Infinity".into_String(),
                    _ => f64::to_str_digits(v, 15),
                }
            ),
            ValueData::Object(ref v) => {
                try!(write!(f, "{}", "{"));
                match v.borrow().iter().last() {
                    Some((last_key, _)) => {
                        for (key, val) in v.borrow().iter() {
                            try!(write!(f, "{}: {}", key, val.value.borrow()));
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
            ValueData::Function(ref v) => match v.borrow().clone() {
                Function::NativeFunc(_) => write!(f, "{}", "function() { [native code] }"),
                Function::RegularFunc(rf) => {
                    write!(f, "function({}){}", rf.args.connect(", "), rf.expr)
                }
            },
        }
    }
}
impl PartialEq for ValueData {
    fn eq(&self, other: &ValueData) -> bool {
        match (self.clone(), other.clone()) {
            (ref a, ref b) if a.is_null_or_undefined() && b.is_null_or_undefined() => true,
            (ValueData::String(ref a), ValueData::String(ref b)) if a == b => true,
            (ValueData::String(ref a), ref b) | (ref b, ValueData::String(ref a))
                if *a == b.to_str() =>
            {
                true
            }
            (ValueData::Boolean(a), ValueData::Boolean(b)) if a == b => true,
            (ValueData::Number(a), ValueData::Number(b))
                if a == b && !a.is_nan() && !b.is_nan() =>
            {
                true
            }
            (ValueData::Number(a), ref b) | (ref b, ValueData::Number(a)) if a == b.to_num() => {
                true
            }
            (ValueData::Integer(a), ValueData::Integer(b)) if a == b => true,
            _ => false,
        }
    }
}

// impl Add<ValueData, ValueData> for ValueData {
//     fn add(&self, other: &ValueData) -> ValueData {
//         return match (self.clone(), other.clone()) {
//             (ValueData::String(s), other) | (other, ValueData::String(s)) => {
//                 ValueData::String(s.clone().append(other.to_str().as_slice()))
//             }
//             (_, _) => ValueData::Number(self.to_num() + other.to_num()),
//         };
//     }
// }
// impl Sub<ValueData, ValueData> for ValueData {
//     fn sub(&self, other: &ValueData) -> ValueData {
//         ValueData::Number(self.to_num() - other.to_num())
//     }
// }
// impl Mul<ValueData, ValueData> for ValueData {
//     fn mul(&self, other: &ValueData) -> ValueData {
//         ValueData::Number(self.to_num() * other.to_num())
//     }
// }
// impl Div<ValueData, ValueData> for ValueData {
//     fn div(&self, other: &ValueData) -> ValueData {
//         ValueData::Number(self.to_num() / other.to_num())
//     }
// }
// impl Rem<ValueData, ValueData> for ValueData {
//     fn rem(&self, other: &ValueData) -> ValueData {
//         ValueData::Number(self.to_num() % other.to_num())
//     }
// }
// impl BitAnd<ValueData, ValueData> for ValueData {
//     fn bitand(&self, other: &ValueData) -> ValueData {
//         ValueData::Integer(self.to_int() & other.to_int())
//     }
// }
// impl BitOr<ValueData, ValueData> for ValueData {
//     fn bitor(&self, other: &ValueData) -> ValueData {
//         ValueData::Integer(self.to_int() | other.to_int())
//     }
// }
// impl BitXor<ValueData, ValueData> for ValueData {
//     fn bitxor(&self, other: &ValueData) -> ValueData {
//         ValueData::Integer(self.to_int() ^ other.to_int())
//     }
// }
// impl Shl<ValueData, ValueData> for ValueData {
//     fn shl(&self, other: &ValueData) -> ValueData {
//         ValueData::Integer(self.to_int() << other.to_int())
//     }
// }
// impl Shr<ValueData, ValueData> for ValueData {
//     fn shr(&self, other: &ValueData) -> ValueData {
//         ValueData::Integer(self.to_int() >> other.to_int())
//     }
// }
// impl Not<ValueData> for ValueData {
//     fn not(&self) -> ValueData {
//         ValueData::Boolean(!self.is_true())
//     }
// }
/// Conversion to Javascript values from Rust values
pub trait ToValue {
    /// Convert this value to a Rust value
    fn to_value(&self) -> Value;
}
/// Conversion to Rust values from Javascript values
pub trait FromValue<T: std::marker::Sized> {
    /// Convert this value to a Javascript value
    fn from_value(value: Value) -> Result<T, &'static str>;
}
impl ToValue for String {
    fn to_value(&self) -> Value {
        GcCell::new(ValueData::String(self.clone()))
    }
}
impl<T> FromValue<T> for String {
    fn from_value(v: Value) -> Result<String, &'static str> {
        Ok(v.borrow().to_str())
    }
}
impl<'s> ToValue for &'s str {
    fn to_value(&self) -> Value {
        GcCell::new(ValueData::String(String::from_str(*self)))
    }
}
impl ToValue for char {
    fn to_value(&self) -> Value {
        GcCell::new(ValueData::String(String::from_char(1, *self)))
    }
}
impl<T> FromValue<T> for char {
    fn from_value(v: Value) -> Result<char, &'static str> {
        Ok(v.borrow().to_str().as_slice().char_at(0))
    }
}
impl ToValue for f64 {
    fn to_value(&self) -> Value {
        GcCell::new(ValueData::Number(self.clone()))
    }
}
impl<T> FromValue<T> for f64 {
    fn from_value(v: Value) -> Result<f64, &'static str> {
        Ok(v.borrow().to_num())
    }
}
impl ToValue for i32 {
    fn to_value(&self) -> Value {
        GcCell::new(ValueData::Integer(self.clone()))
    }
}
impl<T> FromValue<T> for i32 {
    fn from_value(v: Value) -> Result<i32, &'static str> {
        Ok(v.borrow().to_int())
    }
}
impl ToValue for bool {
    fn to_value(&self) -> Value {
        GcCell::new(ValueData::Boolean(self.clone()))
    }
}
impl<T> FromValue<T> for bool {
    fn from_value(v: Value) -> Result<bool, &'static str> {
        Ok(v.borrow().is_true())
    }
}
impl<'s, T: ToValue> ToValue for &'s [T] {
    fn to_value(&self) -> Value {
        let mut arr = HashMap::new();
        let mut i = 0;
        for item in self.iter() {
            arr.insert(i.to_str().into_String(), Property::new(item.to_value()));
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
            arr.insert(i.to_str().into_String(), Property::new(item.to_value()));
            i += 1;
        }
        to_value(arr)
    }
}
impl<T: FromValue<T>, R> FromValue<R> for Vec<T> {
    fn from_value(v: Value) -> Result<Vec<T>, &'static str> {
        let len = v.borrow().get_field_slice("length").borrow().to_int();
        let mut vec = Vec::with_capacity(len);
        for i in 0..len {
            vec.push(try!(from_value(v.borrow().get_field(i.to_str()))))
        }
        Ok(vec)
    }
}
impl ToValue for Function {
    fn to_value(&self) -> Value {
        GcCell::new(ValueData::Function(RefCell::new(Function::NativeFunc(
            Function::NativeFunction::new(*self),
        ))))
    }
}
impl<T> FromValue<T> for Function {
    fn from_value(v: Value) -> Result<Function, &'static str> {
        match *v.borrow() {
            ValueData::Function(ref func) => match *func.borrow() {
                Function::NativeFunc(ref data) => Ok(data.data),
                _ => Err("Value is not a native function"),
            },
            _ => Err("Value is not a function"),
        }
    }
}
impl ToValue for ObjectData {
    fn to_value(&self) -> Value {
        GcCell::new(ValueData::Object(RefCell::new(self.clone())))
    }
}
impl<T> FromValue<T> for ObjectData {
    fn from_value(v: Value) -> Result<ObjectData, &'static str> {
        match *v.borrow() {
            ValueData::Object(ref obj) => Ok(obj.clone().borrow().deref().clone()),
            ValueData::Function(ref func) => Ok(match *func.borrow().deref() {
                Function::NativeFunc(ref data) => data.object.clone(),
                Function::RegularFunc(ref data) => data.object.clone(),
            }),
            _ => Err("Value is not a valid object"),
        }
    }
}

impl ToValue for () {
    fn to_value(&self) -> Value {
        GcCell::new(ValueData::Null)
    }
}
impl<T> FromValue<T> for () {
    fn from_value(_: Value) -> Result<(), &'static str> {
        Ok(())
    }
}
/// A utility function that just calls FromValue::from_value
pub fn from_value<A: FromValue<T>>(v: Value) -> Result<A, &'static str> {
    FromValue::from_value(v)
}

/// A utility function that just calls ToValue::to_value
pub fn to_value<A: ToValue>(v: A) -> Value {
    v.to_value()
}
