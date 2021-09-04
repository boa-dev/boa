use super::*;
use std::convert::TryFrom;

impl From<&JsValue> for JsValue {
    #[inline]
    fn from(value: &JsValue) -> Self {
        value.clone()
    }
}

impl<T> From<T> for JsValue
where
    T: Into<JsString>,
{
    #[inline]
    fn from(value: T) -> Self {
        let _timer = BoaProfiler::global().start_event("From<String>", "value");

        Self::String(value.into())
    }
}

impl From<char> for JsValue {
    #[inline]
    fn from(value: char) -> Self {
        JsValue::new(value.to_string())
    }
}

impl From<JsSymbol> for JsValue {
    #[inline]
    fn from(value: JsSymbol) -> Self {
        JsValue::Symbol(value)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TryFromCharError;

impl Display for TryFromCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert value to a char type")
    }
}

impl From<f64> for JsValue {
    #[allow(clippy::float_cmp)]
    #[inline]
    fn from(value: f64) -> Self {
        // if value as i32 as f64 == value {
        //     JsValue::Integer(value as i32)
        // } else {
        JsValue::Rational(value)
        // }
    }
}

impl From<u32> for JsValue {
    #[inline]
    fn from(value: u32) -> JsValue {
        if let Ok(integer) = i32::try_from(value) {
            JsValue::Integer(integer)
        } else {
            JsValue::Rational(value.into())
        }
    }
}

impl From<i32> for JsValue {
    #[inline]
    fn from(value: i32) -> JsValue {
        JsValue::Integer(value)
    }
}

impl From<JsBigInt> for JsValue {
    #[inline]
    fn from(value: JsBigInt) -> Self {
        JsValue::BigInt(value)
    }
}

impl From<usize> for JsValue {
    #[inline]
    fn from(value: usize) -> JsValue {
        if let Ok(value) = i32::try_from(value) {
            JsValue::Integer(value)
        } else {
            JsValue::Rational(value as f64)
        }
    }
}

impl From<u64> for JsValue {
    #[inline]
    fn from(value: u64) -> JsValue {
        if let Ok(value) = i32::try_from(value) {
            JsValue::Integer(value)
        } else {
            JsValue::Rational(value as f64)
        }
    }
}

impl From<i64> for JsValue {
    #[inline]
    fn from(value: i64) -> JsValue {
        if let Ok(value) = i32::try_from(value) {
            JsValue::Integer(value)
        } else {
            JsValue::Rational(value as f64)
        }
    }
}

impl From<bool> for JsValue {
    #[inline]
    fn from(value: bool) -> Self {
        JsValue::Boolean(value)
    }
}

impl<T> From<&[T]> for JsValue
where
    T: Clone + Into<JsValue>,
{
    fn from(value: &[T]) -> Self {
        let mut array = Object::default();
        for (i, item) in value.iter().enumerate() {
            array.insert(
                i,
                PropertyDescriptor::builder()
                    .value(item.clone())
                    .writable(true)
                    .enumerable(true)
                    .configurable(true),
            );
        }
        Self::from(array)
    }
}

impl<T> From<Vec<T>> for JsValue
where
    T: Into<JsValue>,
{
    fn from(value: Vec<T>) -> Self {
        let mut array = Object::default();
        for (i, item) in value.into_iter().enumerate() {
            array.insert(
                i,
                PropertyDescriptor::builder()
                    .value(item)
                    .writable(true)
                    .enumerable(true)
                    .configurable(true),
            );
        }
        JsValue::new(array)
    }
}

impl From<Object> for JsValue {
    #[inline]
    fn from(object: Object) -> Self {
        let _timer = BoaProfiler::global().start_event("From<Object>", "value");
        JsValue::Object(JsObject::new(object))
    }
}

impl From<JsObject> for JsValue {
    #[inline]
    fn from(object: JsObject) -> Self {
        let _timer = BoaProfiler::global().start_event("From<JsObject>", "value");
        JsValue::Object(object)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TryFromObjectError;

impl Display for TryFromObjectError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert value to an Object type")
    }
}

impl From<()> for JsValue {
    #[inline]
    fn from(_: ()) -> Self {
        JsValue::null()
    }
}

impl<T> From<Option<T>> for JsValue
where
    T: Into<JsValue>,
{
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => JsValue::null(),
        }
    }
}
