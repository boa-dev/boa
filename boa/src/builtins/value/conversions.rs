use super::*;
use std::convert::TryFrom;

impl From<&Value> for Value {
    fn from(value: &Value) -> Self {
        value.clone()
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        let _timer = BoaProfiler::global().start_event("From<String>", "value");
        Self::string(value)
    }
}

impl From<Box<str>> for Value {
    fn from(value: Box<str>) -> Self {
        Self::string(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Value {
        Value::string(value)
    }
}

impl From<&Box<str>> for Value {
    fn from(value: &Box<str>) -> Self {
        Self::string(value.as_ref())
    }
}

impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::string(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TryFromCharError;

impl Display for TryFromCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert value to a char type")
    }
}

impl TryFrom<&Value> for char {
    type Error = TryFromCharError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        if let Some(c) = value.to_string().chars().next() {
            Ok(c)
        } else {
            Err(TryFromCharError)
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::rational(value)
    }
}

impl From<&Value> for f64 {
    fn from(value: &Value) -> Self {
        value.to_number()
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Value {
        Value::integer(value)
    }
}

impl From<&Value> for i32 {
    fn from(value: &Value) -> i32 {
        value.to_integer()
    }
}

impl From<BigInt> for Value {
    fn from(value: BigInt) -> Self {
        Value::bigint(value)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Value {
        Value::integer(value as i32)
    }
}
impl From<&Value> for usize {
    fn from(value: &Value) -> usize {
        value.to_integer() as Self
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::boolean(value)
    }
}

impl From<&Value> for bool {
    fn from(value: &Value) -> Self {
        value.is_true()
    }
}

impl<T> From<&[T]> for Value
where
    T: Clone + Into<Value>,
{
    fn from(value: &[T]) -> Self {
        let mut array = Object::default();
        for (i, item) in value.iter().enumerate() {
            array.properties_mut().insert(
                i.to_string(),
                Property::default().value(item.clone().into()),
            );
        }
        Self::from(array)
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Vec<T>) -> Self {
        let mut array = Object::default();
        for (i, item) in value.into_iter().enumerate() {
            array
                .properties_mut()
                .insert(i.to_string(), Property::default().value(item.into()));
        }
        Value::from(array)
    }
}

impl From<Object> for Value {
    fn from(object: Object) -> Self {
        let _timer = BoaProfiler::global().start_event("From<Object>", "value");
        Value::object(object)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TryFromObjectError;

impl Display for TryFromObjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert value to an Object type")
    }
}

impl TryFrom<&Value> for Object {
    type Error = TryFromObjectError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.data() {
            ValueData::Object(ref object) => Ok(object.clone().into_inner()),
            _ => Err(TryFromObjectError),
        }
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::null()
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Value::null(),
        }
    }
}
