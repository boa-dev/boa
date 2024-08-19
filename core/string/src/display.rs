//! Display implementations for [`crate::JsString`].
use crate::{JsStr, JsStrVariant};
use std::fmt;

/// Display implementation for [`crate::JsString`] that escapes unicode characters.
#[derive(Debug)]
pub struct JsStringDisplayEscaped<'a> {
    pub(crate) inner: JsStr<'a>,
}

impl fmt::Display for JsStringDisplayEscaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.variant() {
            // SAFETY: `JsStrVariant::Latin1` is always valid utf8, so no need to check.
            JsStrVariant::Latin1(v) => unsafe { std::str::from_utf8_unchecked(v) }.fmt(f),
            JsStrVariant::Utf16(_) => self.inner.code_points().try_for_each(|r| write!(f, "{r}")),
        }
    }
}
