use super::*;
use std::convert::TryFrom;

impl From<&Value> for Value {
    #[inline]
    fn from(value: &Value) -> Self {
        value.clone()
    }
}

impl From<String> for Value {
    #[inline]
    fn from(value: String) -> Self {
        let _timer = BoaProfiler::global().start_event("From<String>", "value");
        Self::string(value)
    }
}

impl From<Box<str>> for Value {
    #[inline]
    fn from(value: Box<str>) -> Self {
        Self::string(String::from(value))
    }
}

impl From<&str> for Value {
    #[inline]
    fn from(value: &str) -> Value {
        Value::string(value)
    }
}

impl From<&Box<str>> for Value {
    #[inline]
    fn from(value: &Box<str>) -> Self {
        Self::string(value.as_ref())
    }
}

impl From<char> for Value {
    #[inline]
    fn from(value: char) -> Self {
        Value::string(value.to_string())
    }
}

impl From<RcString> for Value {
    #[inline]
    fn from(value: RcString) -> Self {
        Value::String(value)
    }
}

impl From<RcSymbol> for Value {
    #[inline]
    fn from(value: RcSymbol) -> Self {
        Value::Symbol(value)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TryFromCharError;

impl Display for TryFromCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert value to a char type")
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::rational(value)
    }
}

impl From<u32> for Value {
    #[inline]
    fn from(value: u32) -> Value {
        if let Ok(integer) = i32::try_from(value) {
            Value::integer(integer)
        } else {
            Value::rational(value)
        }
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Value {
        Value::integer(value)
    }
}

impl From<BigInt> for Value {
    fn from(value: BigInt) -> Self {
        Value::bigint(value)
    }
}

impl From<RcBigInt> for Value {
    fn from(value: RcBigInt) -> Self {
        Value::BigInt(value)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Value {
        Value::integer(value as i32)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::boolean(value)
    }
}

impl From<&Value> for bool {
    fn from(value: &Value) -> Self {
        value.to_boolean()
    }
}

impl<T> From<&[T]> for Value
where
    T: Clone + Into<Value>,
{
    fn from(value: &[T]) -> Self {
        let mut array = Object::default();
        for (i, item) in value.iter().enumerate() {
            array.insert(i, Property::default().value(item.clone().into()));
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
            array.insert(i, Property::default().value(item.into()));
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

impl From<GcObject> for Value {
    fn from(object: GcObject) -> Self {
        let _timer = BoaProfiler::global().start_event("From<GcObject>", "value");
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TryFromObjectError;

impl Display for TryFromObjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert value to an Object type")
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
