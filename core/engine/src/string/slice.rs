use super::{is_ascii, JsStr, JsStrVariant};
use crate::{
    builtins::string::is_trimmable_whitespace,
    string::{Iter, JsString},
};

#[derive(Debug, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum JsStringSliceVariant<'a> {
    U8Ascii(&'a [u8]),
    U8NonAscii(&'a str, usize),
    U16Ascii(&'a [u16]),
    U16NonAscii(&'a [u16]),
}

/// TODO: doc
#[derive(Debug, Clone, Copy)]
pub struct JsStringSlice<'a> {
    inner: JsStringSliceVariant<'a>,
}

impl<'a> JsStringSlice<'a> {
    pub(crate) unsafe fn u8_ascii_unchecked(value: &'a [u8]) -> Self {
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

    /// Get the length of the [`JsStringSlice`].
    #[must_use]
    pub fn len(&self) -> usize {
        match self.variant() {
            JsStringSliceVariant::U8Ascii(s) => s.len(),
            JsStringSliceVariant::U8NonAscii(_, len) => len,
            JsStringSliceVariant::U16NonAscii(s) | JsStringSliceVariant::U16Ascii(s) => s.len(),
        }
    }

    /// Check if [`JsStringSlice`] is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if the [`JsStringSlice`] is ascii range.
    #[must_use]
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
                // Safety: A JsStringSlice's Ascii field must always contain valid ascii, so this is safe.
                let s = unsafe { std::str::from_utf8_unchecked(s) };

                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u8_ascii_unchecked(s.trim_start().as_bytes()) }
            }
            JsStringSliceVariant::U8NonAscii(s, _) => JsStringSlice::from(s.trim_start()),
            JsStringSliceVariant::U16Ascii(s) => {
                let value = if let Some(left) = s.iter().copied().position(|r| {
                    !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)
                }) {
                    &s[left..]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("".as_bytes()) };
                };

                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u16_ascii_unchecked(value) }
            }
            JsStringSliceVariant::U16NonAscii(s) => {
                let value = if let Some(left) = s.iter().copied().position(|r| {
                    !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)
                }) {
                    &s[left..]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("".as_bytes()) };
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
                // Safety: A JsStringSlice's Ascii field must always contain valid ascii, so this is safe.
                let s = unsafe { std::str::from_utf8_unchecked(s) };

                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u8_ascii_unchecked(s.trim_end().as_bytes()) }
            }
            JsStringSliceVariant::U8NonAscii(s, _) => JsStringSlice::from(s.trim_end()),
            JsStringSliceVariant::U16Ascii(s) => {
                let value = if let Some(right) = s.iter().copied().rposition(|r| {
                    !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)
                }) {
                    &s[..=right]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("".as_bytes()) };
                };

                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u16_ascii_unchecked(value) }
            }
            JsStringSliceVariant::U16NonAscii(s) => {
                let value = if let Some(right) = s.iter().copied().rposition(|r| {
                    !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)
                }) {
                    &s[..=right]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("".as_bytes()) };
                };

                JsStringSlice::from(value)
            }
        }
    }

    /// Return an iterator over the [`JsStringSlice`].
    #[must_use]
    pub fn iter(self) -> Iter<'a> {
        Iter::new(self)
    }
}

impl<'a> From<&'a JsString> for JsStringSlice<'a> {
    #[inline]
    fn from(value: &'a JsString) -> Self {
        Self::from(value.as_str())
    }
}

impl<'a> From<JsStr<'a>> for JsStringSlice<'a> {
    #[inline]
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
    #[inline]
    fn from(value: &'a str) -> Self {
        if value.is_ascii() {
            // SAFETY: Already checked that it's ASCII, so this is safe.
            return unsafe { Self::u8_ascii_unchecked(value.as_bytes()) };
        }

        // SAFETY: Already checked that it's non-ASCII, so this is safe.
        unsafe { Self::u8_non_ascii_unchecked(value) }
    }
}

impl<'a> From<&'a [u16]> for JsStringSlice<'a> {
    #[inline]
    fn from(s: &'a [u16]) -> Self {
        if is_ascii(s) {
            // SAFETY: Already checked that it's ASCII, so this is safe.
            return unsafe { Self::u16_ascii_unchecked(s) };
        }

        // SAFETY: Already checked that it's non-ASCII, so this is safe.
        unsafe { Self::u16_non_ascii_unchecked(s) }
    }
}
