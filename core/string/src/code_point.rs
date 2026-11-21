use std::fmt::Write;

/// Represents a Unicode codepoint within a [`crate::JsString`], which could be a valid
/// '[Unicode scalar value]', or an unpaired surrogate.
///
/// [Unicode scalar value]: https://www.unicode.org/glossary/#unicode_scalar_value
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodePoint {
    /// A valid Unicode scalar value.
    Unicode(char),

    /// An unpaired surrogate.
    UnpairedSurrogate(u16),
}

impl CodePoint {
    /// Get the number of UTF-16 code units needed to encode this code point.
    #[inline]
    #[must_use]
    pub const fn code_unit_count(self) -> usize {
        match self {
            Self::Unicode(c) => c.len_utf16(),
            Self::UnpairedSurrogate(_) => 1,
        }
    }

    /// Convert the code point to its [`u32`] representation.
    #[inline]
    #[must_use]
    pub fn as_u32(self) -> u32 {
        match self {
            Self::Unicode(c) => u32::from(c),
            Self::UnpairedSurrogate(surr) => u32::from(surr),
        }
    }

    /// If the code point represents a valid 'Unicode scalar value', returns its [`char`]
    /// representation, otherwise returns [`None`] on unpaired surrogates.
    #[inline]
    #[must_use]
    pub const fn as_char(self) -> Option<char> {
        match self {
            Self::Unicode(c) => Some(c),
            Self::UnpairedSurrogate(_) => None,
        }
    }

    /// Encodes this code point as UTF-16 into the provided u16 buffer, and then returns the subslice
    /// of the buffer that contains the encoded character.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is not large enough. A buffer of length 2 is large enough to encode any
    /// code point.
    #[inline]
    #[must_use]
    pub fn encode_utf16(self, dst: &mut [u16]) -> &mut [u16] {
        match self {
            Self::Unicode(c) => c.encode_utf16(dst),
            Self::UnpairedSurrogate(surr) => {
                dst[0] = surr;
                &mut dst[0..=0]
            }
        }
    }
}

impl std::fmt::Display for CodePoint {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodePoint::Unicode(c) => f.write_char(*c),
            CodePoint::UnpairedSurrogate(c) => {
                write!(f, "\\u{c:04X}")
            }
        }
    }
}

impl From<char> for CodePoint {
    fn from(value: char) -> Self {
        Self::Unicode(value)
    }
}

impl From<u16> for CodePoint {
    fn from(value: u16) -> Self {
        char::from_u32(u32::from(value))
            .map_or_else(|| CodePoint::UnpairedSurrogate(value), CodePoint::Unicode)
    }
}
