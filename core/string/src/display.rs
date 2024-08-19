//! Display implementations for [`crate::JsString`].
use crate::{JsStr, JsStrVariant};
use std::fmt;
use std::fmt::Write;

/// Display implementation for [`crate::JsString`] that escapes unicode characters.
#[derive(Debug)]
pub struct JsStringDisplayEscaped<'a> {
    pub(crate) inner: JsStr<'a>,
}

impl fmt::Display for JsStringDisplayEscaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.variant() {
            // SAFETY: `JsStrVariant::Latin1` is always valid utf8, so no need to check.
            JsStrVariant::Latin1(v) => v
                .iter()
                .copied()
                .map(char::from)
                .try_for_each(|c| f.write_char(c)),
            JsStrVariant::Utf16(_) => self.inner.code_points().try_for_each(|r| write!(f, "{r}")),
        }
    }
}

#[test]
fn latin1() {
    let mut s_bytes = b"Hello ".to_vec();
    s_bytes.push(0x82);
    s_bytes.append(b" world!".to_vec().as_mut());
    let s = JsStr::latin1(&s_bytes);

    let rust_str = format!("{}", JsStringDisplayEscaped { inner: s });
    assert_eq!(rust_str, "Hello é world!");
}
