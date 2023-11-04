use crate::{builtins::string::is_trimmable_whitespace, JsString};

use super::{is_ascii, JsStr, JsStrVariant};

#[derive(Debug, Clone, Copy)]
pub enum JsStringSliceVariant<'a> {
    U8Ascii(&'a str),
    U8NonAscii(&'a str, usize),
    U16Ascii(&'a [u16]),
    U16NonAscii(&'a [u16]),
}

#[derive(Debug, Clone, Copy)]
pub struct JsStringSlice<'a> {
    inner: JsStringSliceVariant<'a>,
}

impl<'a> JsStringSlice<'a> {
    pub(crate) unsafe fn u8_ascii_unchecked(value: &'a str) -> Self {
        debug_assert!(value.is_ascii(), "string must be ascii");

        Self {
            inner: JsStringSliceVariant::U8Ascii(value),
        }
    }

    pub(crate) unsafe fn u16_ascii_unchecked(value: &'a [u16]) -> Self {
        debug_assert!(is_ascii(value), "string must be ascii");

        Self {
            inner: JsStringSliceVariant::U16Ascii(value),
        }
    }

    pub(crate) unsafe fn u8_non_ascii_unchecked(value: &'a str) -> Self {
        debug_assert!(!value.is_ascii(), "string must not be ascii");
        let len = value.encode_utf16().count();

        Self {
            inner: JsStringSliceVariant::U8NonAscii(value, len),
        }
    }

    pub(crate) unsafe fn u16_non_ascii_unchecked(value: &'a [u16]) -> Self {
        debug_assert!(!is_ascii(value), "string must not be ascii");

        Self {
            inner: JsStringSliceVariant::U16NonAscii(value),
        }
    }

    pub(crate) fn variant(self) -> JsStringSliceVariant<'a> {
        self.inner
    }

    pub fn len(&self) -> usize {
        match self.variant() {
            JsStringSliceVariant::U8Ascii(s) => s.len(),
            JsStringSliceVariant::U8NonAscii(_, len) => len,
            JsStringSliceVariant::U16NonAscii(s) | JsStringSliceVariant::U16Ascii(s) => s.len(),
        }
    }

    pub fn is_ascii(&self) -> bool {
        matches!(
            self.variant(),
            JsStringSliceVariant::U8Ascii(_) | JsStringSliceVariant::U16Ascii(_)
        )
    }

    /// Trims both leading and trailing space.
    #[inline]
    #[must_use]
    pub fn trim(&self) -> Self {
        self.trim_start().trim_end()
    }

    /// Trims all leading space.
    #[inline]
    #[must_use]
    pub fn trim_start(&self) -> JsStringSlice<'a> {
        match self.variant() {
            JsStringSliceVariant::U8Ascii(s) => {
                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u8_ascii_unchecked(s.trim_start()) }
            }
            JsStringSliceVariant::U8NonAscii(s, _) => JsStringSlice::from(s.trim_start()),
            JsStringSliceVariant::U16Ascii(s) => {
                let value = if let Some(left) = s.iter().copied().position(|r| {
                    !char::from_u32(u32::from(r))
                        .map(is_trimmable_whitespace)
                        .unwrap_or_default()
                }) {
                    &s[left..]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("") };
                };

                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u16_ascii_unchecked(value) }
            }
            JsStringSliceVariant::U16NonAscii(s) => {
                let value = if let Some(left) = s.iter().copied().position(|r| {
                    !char::from_u32(u32::from(r))
                        .map(is_trimmable_whitespace)
                        .unwrap_or_default()
                }) {
                    &s[left..]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("") };
                };

                JsStringSlice::from(value)
            }
        }
    }

    /// Trims all trailing space.
    #[inline]
    #[must_use]
    pub fn trim_end(&self) -> JsStringSlice<'a> {
        match self.variant() {
            JsStringSliceVariant::U8Ascii(s) => {
                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u8_ascii_unchecked(s.trim_end()) }
            }
            JsStringSliceVariant::U8NonAscii(s, _) => JsStringSlice::from(s.trim_end()),
            JsStringSliceVariant::U16Ascii(s) => {
                let value = if let Some(right) = s.iter().copied().rposition(|r| {
                    !char::from_u32(u32::from(r))
                        .map(is_trimmable_whitespace)
                        .unwrap_or_default()
                }) {
                    &s[..=right]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("") };
                };

                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u16_ascii_unchecked(value) }
            }
            JsStringSliceVariant::U16NonAscii(s) => {
                let value = if let Some(right) = s.iter().copied().rposition(|r| {
                    !char::from_u32(u32::from(r))
                        .map(is_trimmable_whitespace)
                        .unwrap_or_default()
                }) {
                    &s[..=right]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("") };
                };

                JsStringSlice::from(value)
            }
        }
    }

    pub fn iter(self) -> crate::string::Iter<'a> {
        crate::string::Iter::new(self)
    }
}

impl<'a> From<&'a JsString> for JsStringSlice<'a> {
    fn from(value: &'a JsString) -> Self {
        Self::from(value.as_str())
    }
}

impl<'a> From<JsStr<'a>> for JsStringSlice<'a> {
    fn from(value: JsStr<'a>) -> Self {
        match value.variant() {
            JsStrVariant::Ascii(s) => {
                // SAFETY: `JsStrVariant::Ascii` always contains ASCII string, so this safe.
                unsafe { Self::u8_ascii_unchecked(s) }
            }
            JsStrVariant::U16(s) => {
                // SAFETY: `JsStrVariant::Ascii` always contains non-ASCII string, so this safe.
                unsafe { Self::u16_non_ascii_unchecked(s) }
            }
        }
    }
}

impl<'a> From<&'a str> for JsStringSlice<'a> {
    fn from(value: &'a str) -> Self {
        if value.is_ascii() {
            // SAFETY: Already checked that it's ASCII, so this is safe.
            return unsafe { Self::u8_ascii_unchecked(value) };
        }

        // SAFETY: Already checked that it's non-ASCII, so this is safe.
        unsafe { Self::u8_non_ascii_unchecked(value) }
    }
}

impl<'a> From<&'a [u16]> for JsStringSlice<'a> {
    fn from(s: &'a [u16]) -> Self {
        if is_ascii(s) {
            // SAFETY: Already checked that it's ASCII, so this is safe.
            return unsafe { Self::u16_ascii_unchecked(s) };
        }

        // SAFETY: Already checked that it's non-ASCII, so this is safe.
        unsafe { Self::u16_non_ascii_unchecked(s) }
    }
}
