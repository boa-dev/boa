//! Types and functions for applying Coercing rules to [`JsValue`] when
//! converting.

use boa_engine::JsNativeError;

use crate::value::TryFromJs;
use crate::{Context, JsResult, JsString, JsValue};

/// A wrapper type that allows coercing a `JsValue` to a specific type.
/// This is useful when you want to coerce a `JsValue` to a Rust type.
///
/// # Example
/// Convert a string to number.
/// ```
/// # use boa_engine::{Context, js_string, JsValue};
/// # use boa_engine::value::{Convert, TryFromJs};
/// # let mut context = Context::default();
/// let value = JsValue::from(js_string!("42"));
/// let Convert(coerced): Convert<i32> = Convert::try_from_js(&value, &mut context).unwrap();
///
/// assert_eq!(coerced, 42);
/// ```
///
/// Convert a number to a bool.
/// ```
/// # use boa_engine::{Context, js_string, JsValue};
/// # use boa_engine::value::{Convert, TryFromJs};
/// # let mut context = Context::default();
/// let value0 = JsValue::Integer(0);
/// let value1 = JsValue::Integer(1);
/// let value_nan = JsValue::Rational(f64::NAN);
/// let Convert(coerced0): Convert<bool> = Convert::try_from_js(&value0, &mut context).unwrap();
/// let Convert(coerced1): Convert<bool> = Convert::try_from_js(&value1, &mut context).unwrap();
/// let Convert(coerced_nan): Convert<bool> = Convert::try_from_js(&value_nan, &mut context).unwrap();
///
/// assert_eq!(coerced0, false);
/// assert_eq!(coerced1, true);
/// assert_eq!(coerced_nan, false);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Convert<T: TryFromJs>(pub T);

impl<T: TryFromJs> From<T> for Convert<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

macro_rules! decl_coerce_to_int {
    ($($ty:ty),*) => {
        $(
            impl TryFromJs for Convert<$ty> {
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
                                Ok(Convert(num.round() as $ty))
                            } else {
                                Ok(Convert(num as $ty))
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
            impl TryFromJs for Convert<$ty> {
                fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
                    value.to_numeric_number(context).and_then(|num| Ok(Convert(<$ty>::try_from(num).map_err(|_| {
                        JsNativeError::typ()
                            .with_message("cannot coerce value to float")
                    })?)))
                }
            }
        )*
    };
}

decl_coerce_to_float!(f64);

impl TryFromJs for Convert<String> {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        value
            .to_string(context)
            .and_then(|s| s.to_std_string().map_err(|_| JsNativeError::typ().into()))
            .map(Convert)
    }
}

impl TryFromJs for Convert<JsString> {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        value.to_string(context).map(Convert)
    }
}

impl TryFromJs for Convert<bool> {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        Ok(Self(value.to_boolean()))
    }
}
