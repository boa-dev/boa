//! Conversions from JavaScript values into Rust values, and the other way around.

use super::{JsBigInt, JsObject, JsString, JsSymbol, JsValue, Profiler};

mod serde_json;
pub(super) mod try_from_js;

impl<T> From<T> for JsValue
where
    T: Into<JsString>,
{
    fn from(value: T) -> Self {
        let _timer = Profiler::global().start_event("From<String>", "value");

        Self::String(value.into())
    }
}

impl From<char> for JsValue {
    #[inline]
    fn from(value: char) -> Self {
        let _timer = Profiler::global().start_event("From<char>", "value");

        Self::new(value.to_string())
    }
}

impl From<JsSymbol> for JsValue {
    #[inline]
    fn from(value: JsSymbol) -> Self {
        let _timer = Profiler::global().start_event("From<JsSymbol>", "value");

        Self::Symbol(value)
    }
}

impl From<f32> for JsValue {
    #[inline]
    fn from(value: f32) -> Self {
        let _timer = Profiler::global().start_event("From<f32>", "value");

        Self::Rational(value.into())
    }
}

impl From<f64> for JsValue {
    #[inline]
    fn from(value: f64) -> Self {
        let _timer = Profiler::global().start_event("From<f64>", "value");

        Self::Rational(value)
    }
}

impl From<u8> for JsValue {
    #[inline]
    fn from(value: u8) -> Self {
        let _timer = Profiler::global().start_event("From<u8>", "value");

        Self::Integer(value.into())
    }
}

impl From<i8> for JsValue {
    #[inline]
    fn from(value: i8) -> Self {
        let _timer = Profiler::global().start_event("From<i8>", "value");

        Self::Integer(value.into())
    }
}

impl From<u16> for JsValue {
    #[inline]
    fn from(value: u16) -> Self {
        let _timer = Profiler::global().start_event("From<u16>", "value");

        Self::Integer(value.into())
    }
}

impl From<i16> for JsValue {
    #[inline]
    fn from(value: i16) -> Self {
        let _timer = Profiler::global().start_event("From<i16>", "value");

        Self::Integer(value.into())
    }
}

impl From<u32> for JsValue {
    #[inline]
    fn from(value: u32) -> Self {
        let _timer = Profiler::global().start_event("From<u32>", "value");

        i32::try_from(value).map_or_else(|_| Self::Rational(value.into()), Self::Integer)
    }
}

impl From<i32> for JsValue {
    #[inline]
    fn from(value: i32) -> Self {
        let _timer = Profiler::global().start_event("From<i32>", "value");

        Self::Integer(value)
    }
}

impl From<JsBigInt> for JsValue {
    #[inline]
    fn from(value: JsBigInt) -> Self {
        let _timer = Profiler::global().start_event("From<JsBigInt>", "value");

        Self::BigInt(value)
    }
}

impl From<usize> for JsValue {
    #[inline]
    fn from(value: usize) -> Self {
        let _timer = Profiler::global().start_event("From<usize>", "value");

        i32::try_from(value).map_or(Self::Rational(value as f64), Self::Integer)
    }
}

impl From<u64> for JsValue {
    #[inline]
    fn from(value: u64) -> Self {
        let _timer = Profiler::global().start_event("From<u64>", "value");

        i32::try_from(value).map_or(Self::Rational(value as f64), Self::Integer)
    }
}

impl From<i64> for JsValue {
    #[inline]
    fn from(value: i64) -> Self {
        let _timer = Profiler::global().start_event("From<i64>", "value");

        i32::try_from(value).map_or(Self::Rational(value as f64), Self::Integer)
    }
}

impl From<bool> for JsValue {
    #[inline]
    fn from(value: bool) -> Self {
        let _timer = Profiler::global().start_event("From<bool>", "value");

        Self::Boolean(value)
    }
}

impl From<JsObject> for JsValue {
    #[inline]
    fn from(object: JsObject) -> Self {
        let _timer = Profiler::global().start_event("From<JsObject>", "value");

        Self::Object(object)
    }
}

impl From<()> for JsValue {
    #[inline]
    fn from(_: ()) -> Self {
        let _timer = Profiler::global().start_event("From<()>", "value");

        Self::null()
    }
}

/// Converts an `Option<T>` into a `JsValue`.
///
/// It will convert the `None` variant to `JsValue::undefined()`, and the `Some()` variant into a
/// `JsValue` using the `Into` trait.
pub(crate) trait IntoOrUndefined {
    /// Converts an `Option<T>` into a `JsValue`.
    fn into_or_undefined(self) -> JsValue;
}

impl<T> IntoOrUndefined for Option<T>
where
    T: Into<JsValue>,
{
    #[inline]
    fn into_or_undefined(self) -> JsValue {
        self.map_or_else(JsValue::undefined, Into::into)
    }
}
