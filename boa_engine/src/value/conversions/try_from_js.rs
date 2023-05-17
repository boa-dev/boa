//! This module contains the [`TryFromJs`] trait, and conversions to basic Rust types.

use crate::{Context, JsBigInt, JsNativeError, JsResult, JsValue};
use num_bigint::BigInt;

/// This trait adds a fallible and efficient conversions from a [`JsValue`] to Rust types.
pub trait TryFromJs: Sized {
    /// This function tries to convert a JavaScript value into `Self`.
    fn try_from_js(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self>;
}

impl JsValue {
    /// This function is the inverse of [`TryFromJs`]. It tries to convert a [`JsValue`] to a given
    /// Rust type.
    pub fn try_js_into<T>(&self, context: &mut Context<'_>) -> JsResult<T>
    where
        T: TryFromJs,
    {
        T::try_from_js(self, context)
    }
}

impl TryFromJs for bool {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Boolean(b) => Ok(*b),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a boolean")
                .into()),
        }
    }
}

impl TryFromJs for String {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::String(s) => s.to_std_string().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("could not convert JsString to Rust string, since it has UTF-16 characters: {e}"))
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a String")
                .into()),
        }
    }
}

impl<T> TryFromJs for Option<T>
where
    T: TryFromJs,
{
    fn try_from_js(value: &JsValue, context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Null | JsValue::Undefined => Ok(None),
            value => Ok(Some(T::try_from_js(value, context)?)),
        }
    }
}

impl TryFromJs for JsBigInt {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::BigInt(b) => Ok(b.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a BigInt")
                .into()),
        }
    }
}

impl TryFromJs for BigInt {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::BigInt(b) => Ok(b.as_inner().clone()),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a BigInt")
                .into()),
        }
    }
}

impl TryFromJs for JsValue {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        Ok(value.clone())
    }
}

impl TryFromJs for f64 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => Ok((*i).into()),
            JsValue::Rational(r) => Ok(*r),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a f64")
                .into()),
        }
    }
}

impl TryFromJs for i8 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a i8: {e}"))
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a i8")
                .into()),
        }
    }
}

impl TryFromJs for u8 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a u8: {e}"))
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a u8")
                .into()),
        }
    }
}

impl TryFromJs for i16 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a i16: {e}"))
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a i16")
                .into()),
        }
    }
}

impl TryFromJs for u16 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a iu16: {e}"))
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a u16")
                .into()),
        }
    }
}

impl TryFromJs for i32 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => Ok(*i),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a i32")
                .into()),
        }
    }
}

impl TryFromJs for u32 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a u32: {e}"))
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a u32")
                .into()),
        }
    }
}

impl TryFromJs for i64 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => Ok((*i).into()),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a i64")
                .into()),
        }
    }
}

impl TryFromJs for u64 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a u64: {e}"))
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a u64")
                .into()),
        }
    }
}

impl TryFromJs for usize {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a usize: {e}"))
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a usize")
                .into()),
        }
    }
}

impl TryFromJs for i128 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => Ok((*i).into()),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a i128")
                .into()),
        }
    }
}

impl TryFromJs for u128 {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Integer(i) => (*i).try_into().map_err(|e| {
                JsNativeError::typ()
                    .with_message(format!("cannot convert value to a u128: {e}"))
                    .into()
            }),
            _ => Err(JsNativeError::typ()
                .with_message("cannot convert value to a u128")
                .into()),
        }
    }
}
