//! Module implementing the operations for the inner value of a `[super::JsValue]`.
#[cfg(feature = "legacy-jsvalue")]
mod nan_boxed;
#[cfg(feature = "legacy-jsvalue")]
pub(crate) use nan_boxed::NanBoxedValue as InnerValue;

#[cfg(not(feature = "legacy-jsvalue"))]
mod legacy;
#[cfg(not(feature = "legacy-jsvalue"))]
pub(crate) use legacy::EnumBasedValue as InnerValue;
