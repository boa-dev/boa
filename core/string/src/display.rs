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
            // SAFETY: `JsStrVariant::Latin1` does not contain any unpaired surrogates, so need to check.
            JsStrVariant::Latin1(v) => v
                .iter()
                .copied()
                .map(char::from)
                .try_for_each(|c| f.write_char(c)),
            JsStrVariant::Utf16(_) => self.inner.code_points().try_for_each(|r| match r {
                CodePoint::Unicode(c) => f.write_char(c),
                CodePoint::UnpairedSurrogate(c) => {
                    write!(f, "\\u{c:04X}")
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

/// Display implementation for [`crate::JsString`] that escapes unicode characters.
#[derive(Debug)]
pub struct JsStrDisplayLossy<'a> {
    inner: JsStr<'a>,
}

impl fmt::Display for JsStrDisplayLossy<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No need to optimize latin1.
        self.inner
            .code_points_lossy()
            .try_for_each(|c| f.write_char(c))
    }
}

impl<'a> From<JsStr<'a>> for JsStrDisplayLossy<'a> {
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

    let rust_str = format!("{}", JsStrDisplayLossy { inner: s });
    assert_eq!(rust_str, "Hello é world!");
}

#[test]
fn emoji() {
    // 0x1F600 is `😀` (see https://www.fileformat.info/info/unicode/char/1f600/index.htm).
    let s = JsStr::utf16(&[0xD83D, 0xDE00]);

    let rust_str = format!("{}", JsStrDisplayEscaped { inner: s });
    assert_eq!(rust_str, "😀");

    let rust_str = format!("{}", JsStrDisplayLossy { inner: s });
    assert_eq!(rust_str, "😀");
}

#[test]
fn unpaired_surrogates() {
    // 0xD800 is an unpaired surrogate (see https://www.fileformat.info/info/unicode/char/d800/index.htm).
    let s = JsStr::utf16(&[0xD800]);

    let rust_str = format!("{}", JsStrDisplayEscaped { inner: s });
    assert_eq!(rust_str, "\\uD800");

    let rust_str = format!("{}", JsStrDisplayLossy { inner: s });
    assert_eq!(rust_str, "�");
}
