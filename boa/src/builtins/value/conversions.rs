use super::*;

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

impl ToValue for Value {
    fn to_value(&self) -> Value {
        self.clone()
    }
}

impl FromValue for Value {
    fn from_value(value: Value) -> Result<Self, &'static str> {
        Ok(value)
    }
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::String(self.clone()))
    }
}

impl FromValue for String {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_string())
    }
}

impl<'s> ToValue for &'s str {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::String(
            String::from_str(*self).expect("Could not convert string to self to String"),
        ))
    }
}

impl ToValue for char {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::String(self.to_string()))
    }
}
impl FromValue for char {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_string()
            .chars()
            .next()
            .expect("Could not get next char"))
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Rational(*self))
    }
}
impl FromValue for f64 {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_number())
    }
}

impl ToValue for i32 {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Integer(*self))
    }
}
impl FromValue for i32 {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_integer())
    }
}

impl ToValue for usize {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Integer(*self as i32))
    }
}
impl FromValue for usize {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.to_integer() as Self)
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Boolean(*self))
    }
}
impl FromValue for bool {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(v.is_true())
    }
}

impl<'s, T: ToValue> ToValue for &'s [T] {
    fn to_value(&self) -> Value {
        let mut arr = Object::default();
        for (i, item) in self.iter().enumerate() {
            arr.properties
                .insert(i.to_string(), Property::default().value(item.to_value()));
        }
        to_value(arr)
    }
}
impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(&self) -> Value {
        let mut arr = Object::default();
        for (i, item) in self.iter().enumerate() {
            arr.properties
                .insert(i.to_string(), Property::default().value(item.to_value()));
        }
        to_value(arr)
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        let len = v.get_field_slice("length").to_integer();
        let mut vec = Self::with_capacity(len as usize);
        for i in 0..len {
            vec.push(from_value(v.get_field_slice(&i.to_string()))?)
        }
        Ok(vec)
    }
}

impl ToValue for Object {
    fn to_value(&self) -> Value {
        Gc::new(ValueData::Object(Box::new(GcCell::new(self.clone()))))
    }
}

impl FromValue for Object {
    fn from_value(v: Value) -> Result<Self, &'static str> {
        match *v {
            ValueData::Object(ref obj) => Ok(obj.clone().into_inner()),
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
    fn from_value(v: Value) -> Result<Self, &'static str> {
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
    fn from_value(value: Value) -> Result<Self, &'static str> {
        Ok(if value.is_null_or_undefined() {
            None
        } else {
            Some(FromValue::from_value(value)?)
        })
    }
}

/// A utility function that just calls `FromValue::from_value`
pub fn from_value<A: FromValue>(v: Value) -> Result<A, &'static str> {
    FromValue::from_value(v)
}

/// A utility function that just calls `ToValue::to_value`
pub fn to_value<A: ToValue>(v: A) -> Value {
    v.to_value()
}
