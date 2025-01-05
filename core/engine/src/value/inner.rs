//! Module implementing the operations for the inner value of a `[super::JsValue]`.
mod nan_boxed;
pub(crate) use nan_boxed::NanBoxedValue as InnerValue;
