use crate::{builtins::string::is_trimmable_whitespace, string::Iter};
use boa_interner::JStrRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsStrVariant<'a> {
    Ascii(&'a str),
    U16(&'a [u16]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsStr<'a> {
    inner: JsStrVariant<'a>,
}

impl<'a> JsStr<'a> {
    #[inline]
    #[must_use]
    pub(crate) unsafe fn ascii_unchecked(value: &'a str) -> Self {
        debug_assert!(value.is_ascii());

        Self {
            inner: JsStrVariant::Ascii(value),
        }
    }

    #[inline]
    #[must_use]
    pub(crate) unsafe fn u16_unchecked(value: &'a [u16]) -> Self {
        // debug_assert!(value.is_ascii());

        Self {
            inner: JsStrVariant::U16(value),
        }
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        match self.inner {
            JsStrVariant::Ascii(v) => v.len(),
            JsStrVariant::U16(v) => v.len(),
        }
    }

    #[inline]
    #[must_use]
    pub fn variant(self) -> JsStrVariant<'a> {
        self.inner
    }

    #[inline]
    #[must_use]
    pub fn is_ascii(&self) -> bool {
        matches!(self.inner, JsStrVariant::Ascii(_))
    }

    #[inline]
    #[must_use]
    pub fn as_ascii(&self) -> Option<&str> {
        if let JsStrVariant::Ascii(slice) = self.inner {
            return Some(slice);
        }

        None
    }

    /// TODO: doc
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(*self)
    }

    pub(crate) fn as_str_ref(&self) -> JStrRef<'_> {
        match self.inner {
            JsStrVariant::Ascii(s) => JStrRef::Utf8(s),
            JsStrVariant::U16(s) => JStrRef::Utf16(s),
        }
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    pub fn trim_start(&self) -> Self {
        match self.variant() {
            JsStrVariant::Ascii(s) => {
                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStr::ascii_unchecked(s.trim_start()) }
            }
            JsStrVariant::U16(s) => {
                let value = if let Some(left) = s.iter().copied().position(|r| {
                    !char::from_u32(u32::from(r))
                        .map(is_trimmable_whitespace)
                        .unwrap_or_default()
                }) {
                    &s[left..]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStr::ascii_unchecked("") };
                };

                // TODO: If we have a string that has ascii non-white space characters,
                //       and a leading non-ascii white space, that is trimmed making this ascii.
                //
                // SAFETY:
                unsafe { JsStr::u16_unchecked(value) }
            }
        }
    }

    /// Trims all trailing space.
    #[inline]
    #[must_use]
    pub fn trim_end(&self) -> Self {
        match self.variant() {
            JsStrVariant::Ascii(s) => {
                // SAFETY: Calling `trim_end()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStr::ascii_unchecked(s.trim_end()) }
            }
            JsStrVariant::U16(s) => {
                let value = if let Some(right) = s.iter().copied().rposition(|r| {
                    !char::from_u32(u32::from(r))
                        .map(is_trimmable_whitespace)
                        .unwrap_or_default()
                }) {
                    &s[..=right]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStr::ascii_unchecked("") };
                };

                // TODO: If we have a string that has ascii non-white space characters,
                //       and a trailing non-ascii white space, that is trimmed making this ascii.
                //
                // SAFETY:
                unsafe { JsStr::u16_unchecked(value) }
            }
        }
    }
}
