//! Display implementations for [`crate::JsString`].
use crate::{JsStr, JsStrVariant};
use std::fmt;
use std::fmt::Write;

/// Display implementation for `JsString` that escapes unicode characters.
#[derive(Debug)]
pub struct JsStringDisplayEscaped<'a> {
    pub(crate) inner: JsStr<'a>,
}

impl fmt::Display for JsStringDisplayEscaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.variant() {
            // SAFETY: `JsStrVariant::Latin1` is always valid latin1, so no need to check.
            JsStrVariant::Latin1(v) => unsafe { std::str::from_utf8_unchecked(v) }.fmt(f),
            JsStrVariant::Utf16(v) => {
                char::decode_utf16(v.iter().copied()).try_for_each(|r| match r {
                    Ok(c) => f.write_char(c),
                    Err(e) => write!(f, "\\u{:04X}", e.unpaired_surrogate()),
                })
            }
        }
    }
}
