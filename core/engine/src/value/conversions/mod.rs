//! Conversions from JavaScript values into Rust values, and the other way around.

use crate::{js_string, string::JsStr};

use super::{InnerValue, JsBigInt, JsObject, JsString, JsSymbol, JsValue, Profiler};

mod either;
mod serde_json;
pub(super) mod try_from_js;
pub(super) mod try_into_js;

pub(super) mod convert;

impl From<JsStr<'_>> for JsValue {
    fn from(value: JsStr<'_>) -> Self {
        let _timer = Profiler::global().start_event("From<JsStr<'_>>", "value");

        Self::from_inner(InnerValue::String(value.into()))
    }
}

impl From<JsString> for JsValue {
    fn from(value: JsString) -> Self {
        let _timer = Profiler::global().start_event("From<JsString>", "value");

        Self::from_inner(InnerValue::String(value))
    }
}

impl From<char> for JsValue {
    #[inline]
    fn from(value: char) -> Self {
        let _timer = Profiler::global().start_event("From<char>", "value");

        let mut buf: [u16; 2] = [0; 2];

        let out = value.encode_utf16(&mut buf);

        Self::from(js_string!(&*out))
    }
}

impl From<JsSymbol> for JsValue {
    #[inline]
    fn from(value: JsSymbol) -> Self {
        let _timer = Profiler::global().start_event("From<JsSymbol>", "value");

        Self::from_inner(InnerValue::Symbol(value))
    }
}

macro_rules! impl_from_float {
    ( $( $type_:ty ),* ) => {
        $(
            impl From<$type_> for JsValue {
                #[inline]
                #[allow(trivial_numeric_casts)]
                #[allow(clippy::cast_lossless)]
                fn from(value: $type_) -> Self {
                    let _timer = Profiler::global().start_event(concat!("From<", stringify!($type_), ">"), "value");

                    if value != -0.0 && value.fract() == 0.0 && value <= i32::MAX as $type_ && value >= i32::MIN as $type_ {
                        Self::from_inner(InnerValue::Integer32(value as i32))
                    } else {
                        Self::from_inner(InnerValue::Float64(f64::from(value)))
                    }
                }
            }
        )*
    };
}

macro_rules! impl_from_integer {
    ( $( $type_:ty ),* ) => {
        $(
            impl From<$type_> for JsValue {
                #[inline]
                #[allow(clippy::cast_lossless)]
                fn from(value: $type_) -> Self {
                    let _timer = Profiler::global().start_event(concat!("From<", stringify!($type_), ">"), "value");

                    i32::try_from(value)
                        .map_or_else(
                            |_| Self::from(value as f64),
                            |value| Self::from_inner(InnerValue::Integer32(value)),
                        )
                }
            }
        )*
    };
}

impl_from_float!(f32, f64);
impl_from_integer!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);

impl From<JsBigInt> for JsValue {
    #[inline]
    fn from(value: JsBigInt) -> Self {
        let _timer = Profiler::global().start_event("From<JsBigInt>", "value");

        Self::from_inner(InnerValue::BigInt(value))
    }
}

impl From<bool> for JsValue {
    #[inline]
    fn from(value: bool) -> Self {
        let _timer = Profiler::global().start_event("From<bool>", "value");

        Self::from_inner(InnerValue::Boolean(value))
    }
}

impl From<JsObject> for JsValue {
    #[inline]
    fn from(object: JsObject) -> Self {
        let _timer = Profiler::global().start_event("From<JsObject>", "value");

        Self::from_inner(InnerValue::Object(object))
    }
}

impl From<()> for JsValue {
    #[inline]
    #[allow(clippy::pedantic)] // didn't want to increase our MSRV for just a lint.
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
        match self {
            Some(value) => value.into(),
            None => JsValue::undefined(),
        }
    }
}
