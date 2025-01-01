//! Module implementing the operations for the inner value of a `[super::JsValue]`.
//!
//! The `[InnerValue]` type is an opaque type that can be either an enum of possible
//! JavaScript value types, or a 64-bits float that represents a NaN-boxed JavaScript
//! value, depending on feature flags. By default, the behaviour is to use the
//! enumeration.

#[cfg(feature = "nan-box-jsvalue")]
mod nan_boxed;

#[cfg(feature = "nan-box-jsvalue")]
pub(crate) use nan_boxed::NanBoxedValue as InnerValue;

#[cfg(not(feature = "nan-box-jsvalue"))]
mod enum_value;

#[cfg(not(feature = "nan-box-jsvalue"))]
pub(crate) use enum_value::EnumBasedValue as InnerValue;
