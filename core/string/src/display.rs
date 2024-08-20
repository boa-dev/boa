//! Display implementations for [`crate::JsString`].
use crate::{CodePoint, JsStr, JsStrVariant};
use std::fmt;
use std::fmt::Write;

/// Display implementation for [`crate::JsString`] that escapes unicode characters.
#[derive(Debug)]
pub struct JsStrDisplayEscaped<'a> {
    inner: JsStr<'a>,
}

impl fmt::Display for JsStrDisplayEscaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner.variant() {
            // SAFETY: `JsStrVariant::Latin1` is always valid utf8, so no need to check.
            JsStrVariant::Latin1(v) => v
                .iter()
                .copied()
                .map(char::from)
                .try_for_each(|c| f.write_char(c)),
            JsStrVariant::Utf16(_) => self.inner.code_points().try_for_each(|r| match r {
                CodePoint::Unicode(c) => f.write_char(c),
                CodePoint::UnpairedSurrogate(c) => {
                    f.write_str("\\u")?;
                    write!(f, "{c:04X}")
                }
            }),
        }
    }
}

impl<'a> From<JsStr<'a>> for JsStrDisplayEscaped<'a> {
    fn from(inner: JsStr<'a>) -> Self {
        Self { inner }
    }
}

#[test]
fn latin1() {
    // 0xE9 is `é` in ISO-8859-1 (see https://www.ascii-code.com/ISO-8859-1).
    let s = JsStr::latin1(b"Hello \xE9 world!");

    let rust_str = format!("{}", JsStrDisplayEscaped { inner: s });
    assert_eq!(rust_str, "Hello é world!");
}
