//! Types and functions for applying Coercing rules to [`JsValue`] when
//! converting.

use crate::value::TryFromJs;
use crate::{Context, JsResult, JsValue};
use boa_engine::JsNativeError;

/// A wrapper type that allows coercing a `JsValue` to a specific type.
/// This is useful when you want to coerce a `JsValue` to a Rust type.
///
/// # Example
/// ```
/// # use boa_engine::{Context, js_string, JsValue};
/// # use boa_engine::value::{Coerce, TryFromJs};
/// # let mut context = Context::default();
/// let value = JsValue::from(js_string!("42"));
/// let Coerce(coerced): Coerce<i32> = Coerce::try_from_js(&value, &mut context).unwrap();
///
/// assert_eq!(coerced, 42);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coerce<T: TryFromJs>(pub T);

impl<T: TryFromJs> From<T> for Coerce<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

macro_rules! decl_coerce_to_int {
    ($($ty:ty),*) => {
        $(
            impl TryFromJs for Coerce<$ty> {
                fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
                    value.to_numeric_number(context).and_then(|num| {
                        if num.is_finite() {
                            if num >= f64::from(<$ty>::MAX) {
                                Err(JsNativeError::typ()
                                    .with_message("cannot coerce value to integer, it is too large")
                                    .into())
                            } else if num <= f64::from(<$ty>::MIN) {
                                Err(JsNativeError::typ()
                                    .with_message("cannot coerce value to integer, it is too small")
                                    .into())
                                // Only round if it differs from the next integer by an epsilon
                            } else if num.abs().fract() >= (1.0 - f64::EPSILON) {
                                Ok(Coerce(num.round() as $ty))
                            } else {
                                Ok(Coerce(num as $ty))
                            }
                        } else if num.is_nan() {
                            Err(JsNativeError::typ()
                                .with_message("cannot coerce NaN to integer")
                                .into())
                        } else if num.is_infinite() {
                            Err(JsNativeError::typ()
                                .with_message("cannot coerce Infinity to integer")
                                .into())
                        } else {
                            Err(JsNativeError::typ()
                                .with_message("cannot coerce non-finite number to integer")
                                .into())
                        }
                    })
                }
            }
        )*
    };
}

decl_coerce_to_int!(i8, i16, i32, u8, u16, u32);

macro_rules! decl_coerce_to_float {
    ($($ty:ty),*) => {
        $(
            impl TryFromJs for Coerce<$ty> {
                fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
                    value.to_numeric_number(context).and_then(|num| Ok(Coerce(<$ty>::try_from(num).map_err(|_| {
                        JsNativeError::typ()
                            .with_message("cannot coerce value to float")
                    })?)))
                }
            }
        )*
    };
}

decl_coerce_to_float!(f64);

impl TryFromJs for Coerce<String> {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        value
            .to_string(context)
            .and_then(|s| s.to_std_string().map_err(|_| JsNativeError::typ().into()))
            .map(Coerce)
    }
}

impl TryFromJs for Coerce<JsString> {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        value.to_string(context).map(Coerce)
    }
}

impl TryFromJs for Coerce<bool> {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        Ok(Self(value.to_boolean()))
    }
}
