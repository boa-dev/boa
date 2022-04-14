use super::{Display, JsBigInt, JsObject, JsString, JsSymbol, JsValue, Profiler};

impl From<&Self> for JsValue {
    #[inline]
    fn from(value: &Self) -> Self {
        value.clone()
    }
}

impl<T> From<T> for JsValue
where
    T: Into<JsString>,
{
    #[inline]
    fn from(value: T) -> Self {
        let _timer = Profiler::global().start_event("From<String>", "value");

        Self::String(value.into())
    }
}

impl From<char> for JsValue {
    #[inline]
    fn from(value: char) -> Self {
        Self::new(value.to_string())
    }
}

impl From<JsSymbol> for JsValue {
    #[inline]
    fn from(value: JsSymbol) -> Self {
        Self::Symbol(value)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TryFromCharError;

impl Display for TryFromCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert value to a char type")
    }
}

impl From<f32> for JsValue {
    #[allow(clippy::float_cmp)]
    #[inline]
    fn from(value: f32) -> Self {
        // if value as i32 as f64 == value {
        //     Self::Integer(value as i32)
        // } else {
        Self::Rational(value.into())
        // }
    }
}

impl From<f64> for JsValue {
    #[allow(clippy::float_cmp)]
    #[inline]
    fn from(value: f64) -> Self {
        // if value as i32 as f64 == value {
        //     Self::Integer(value as i32)
        // } else {
        Self::Rational(value)
        // }
    }
}

impl From<u8> for JsValue {
    #[inline]
    fn from(value: u8) -> Self {
        Self::Integer(value.into())
    }
}

impl From<i8> for JsValue {
    #[inline]
    fn from(value: i8) -> Self {
        Self::Integer(value.into())
    }
}

impl From<u16> for JsValue {
    #[inline]
    fn from(value: u16) -> Self {
        Self::Integer(value.into())
    }
}

impl From<i16> for JsValue {
    #[inline]
    fn from(value: i16) -> Self {
        Self::Integer(value.into())
    }
}

impl From<u32> for JsValue {
    #[inline]
    fn from(value: u32) -> Self {
        if let Ok(integer) = i32::try_from(value) {
            Self::Integer(integer)
        } else {
            Self::Rational(value.into())
        }
    }
}

impl From<i32> for JsValue {
    #[inline]
    fn from(value: i32) -> Self {
        Self::Integer(value)
    }
}

impl From<JsBigInt> for JsValue {
    #[inline]
    fn from(value: JsBigInt) -> Self {
        Self::BigInt(value)
    }
}

impl From<usize> for JsValue {
    #[inline]
    fn from(value: usize) -> Self {
        if let Ok(value) = i32::try_from(value) {
            Self::Integer(value)
        } else {
            Self::Rational(value as f64)
        }
    }
}

impl From<u64> for JsValue {
    #[inline]
    fn from(value: u64) -> Self {
        if let Ok(value) = i32::try_from(value) {
            Self::Integer(value)
        } else {
            Self::Rational(value as f64)
        }
    }
}

impl From<i64> for JsValue {
    #[inline]
    fn from(value: i64) -> Self {
        if let Ok(value) = i32::try_from(value) {
            Self::Integer(value)
        } else {
            Self::Rational(value as f64)
        }
    }
}

impl From<bool> for JsValue {
    #[inline]
    fn from(value: bool) -> Self {
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
        Self::null()
    }
}

pub(crate) trait IntoOrUndefined {
    fn into_or_undefined(self) -> JsValue;
}

impl<T> IntoOrUndefined for Option<T>
where
    T: Into<JsValue>,
{
    fn into_or_undefined(self) -> JsValue {
        self.map_or_else(JsValue::undefined, Into::into)
    }
}
