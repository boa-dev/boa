use crate::{is_trimmable_whitespace, is_trimmable_whitespace_latin1, Iter};
use std::{
    hash::{Hash, Hasher},
    slice::SliceIndex,
};

use super::iter::Windows;

// Modified port of <https://doc.rust-lang.org/std/primitive.slice.html#method.trim_ascii_start>
#[inline]
pub(crate) const fn trim_latin1_start(mut bytes: &[u8]) -> &[u8] {
    // Note: A pattern matching based approach (instead of indexing) allows
    // making the function const.
    while let [first, rest @ ..] = bytes {
        if is_trimmable_whitespace_latin1(*first) {
            bytes = rest;
        } else {
            break;
        }
    }
    bytes
}

// Modified port of <https://doc.rust-lang.org/std/primitive.slice.html#method.trim_ascii_end>
#[inline]
pub(crate) const fn trim_latin1_end(mut bytes: &[u8]) -> &[u8] {
    // Note: A pattern matching based approach (instead of indexing) allows
    // making the function const.
    while let [rest @ .., last] = bytes {
        if is_trimmable_whitespace_latin1(*last) {
            bytes = rest;
        } else {
            break;
        }
    }
    bytes
}

/// Inner representation of a [`JsStr`].
#[derive(Debug, Clone, Copy)]
pub enum JsStrVariant<'a> {
    /// Latin1 string representation.
    Latin1(&'a [u8]),

    /// U16 string representation.
    Utf16(&'a [u16]),
}

/// This is equivalent to Rust's `&str`.
#[derive(Debug, Clone, Copy)]
pub struct JsStr<'a> {
    inner: JsStrVariant<'a>,
}

impl<'a> JsStr<'a> {
    /// This represents an empty string.
    pub const EMPTY: Self = Self::latin1("".as_bytes());

    /// Creates a [`JsStr`] from codepoints that can fit in a `u8`.
    #[inline]
    #[must_use]
    pub const fn latin1(value: &'a [u8]) -> Self {
        Self {
            inner: JsStrVariant::Latin1(value),
        }
    }

    /// Creates a [`JsStr`] from utf16 encoded string.
    #[inline]
    #[must_use]
    pub const fn utf16(value: &'a [u16]) -> Self {
        Self {
            inner: JsStrVariant::Utf16(value),
        }
    }

    /// Get the length of the [`JsStr`].
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        match self.inner {
            JsStrVariant::Latin1(v) => v.len(),
            JsStrVariant::Utf16(v) => v.len(),
        }
    }

    /// Return the inner [`JsStrVariant`] varient of the [`JsStr`].
    #[inline]
    #[must_use]
    pub fn variant(self) -> JsStrVariant<'a> {
        self.inner
    }

    /// Check if the [`JsStr`] is latin1 encoded.
    #[inline]
    #[must_use]
    pub fn is_latin1(&self) -> bool {
        matches!(self.inner, JsStrVariant::Latin1(_))
    }

    /// Returns [`u8`] slice if the [`JsStr`] is latin1 encoded, otherwise [`None`].
    #[inline]
    #[must_use]
    pub const fn as_latin1(&self) -> Option<&[u8]> {
        if let JsStrVariant::Latin1(slice) = self.inner {
            return Some(slice);
        }

        None
    }

    /// Iterate over the codepoints of the string.
    #[inline]
    #[must_use]
    pub fn iter(self) -> Iter<'a> {
        Iter::new(self)
    }

    /// Iterate over the codepoints of the string.
    #[inline]
    #[must_use]
    pub fn windows(self, size: usize) -> Windows<'a> {
        Windows::new(self, size)
    }

    /// Check if the [`JsStr`] is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Trims both leading and trailing space.
    #[inline]
    #[must_use]
    pub fn trim(self) -> JsStr<'a> {
        self.trim_start().trim_end()
    }

    /// Trims all leading space.
    #[inline]
    #[must_use]
    pub fn trim_start(self) -> Self {
        match self.variant() {
            JsStrVariant::Latin1(s) => Self::latin1(trim_latin1_start(s)),
            JsStrVariant::Utf16(s) => {
                let value = if let Some(left) = s.iter().copied().position(|r| {
                    !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)
                }) {
                    &s[left..]
                } else {
                    return Self::EMPTY;
                };

                JsStr {
                    inner: JsStrVariant::Utf16(value),
                }
            }
        }
    }

    /// Trims all trailing space.
    #[inline]
    #[must_use]
    pub fn trim_end(self) -> Self {
        match self.variant() {
            JsStrVariant::Latin1(s) => Self::latin1(trim_latin1_end(s)),
            JsStrVariant::Utf16(s) => {
                let value = if let Some(right) = s.iter().copied().rposition(|r| {
                    !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)
                }) {
                    &s[..=right]
                } else {
                    return Self::EMPTY;
                };

                JsStr {
                    inner: JsStrVariant::Utf16(value),
                }
            }
        }
    }

    /// Returns an element or subslice depending on the type of index, otherwise [`None`].
    #[inline]
    #[must_use]
    pub fn get<I>(self, index: I) -> Option<I::Value>
    where
        I: JsSliceIndex<'a>,
    {
        I::get(self, index)
    }

    /// Returns an element or subslice depending on the type of index, without doing bounds check.
    #[inline]
    #[must_use]
    pub unsafe fn get_unchecked<I>(self, index: I) -> I::Value
    where
        I: JsSliceIndex<'a>,
    {
        unsafe { I::get_unchecked(self, index) }
    }

    /// Convert the [`JsStr`] into a [`Vec<U16>`].
    #[inline]
    #[must_use]
    pub fn to_vec(&self) -> Vec<u16> {
        match self.variant() {
            JsStrVariant::Latin1(v) => v.iter().copied().map(u16::from).collect(),
            JsStrVariant::Utf16(v) => v.to_vec(),
        }
    }

    /// Returns true if needle is a prefix of the [`JsStr`].
    #[inline]
    #[must_use]
    // We check the size, so this should never panic.
    #[allow(clippy::missing_panics_doc)]
    pub fn starts_with(&self, needle: JsStr<'_>) -> bool {
        let n = needle.len();
        self.len() >= n && needle == self.get(..n).expect("already checked size")
    }
    /// Returns `true` if `needle` is a suffix of the [`JsStr`].
    #[inline]
    #[must_use]
    // We check the size, so this should never panic.
    #[allow(clippy::missing_panics_doc)]
    pub fn ends_with(&self, needle: JsStr<'_>) -> bool {
        let (m, n) = (self.len(), needle.len());
        m >= n && needle == self.get(m - n..).expect("already checked size")
    }
}

impl Hash for JsStr<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // NOTE: The hash function has been inlined to ensure that a hash of latin1 and U16
        // encoded strings remains the same if they have the same characters
        match self.variant() {
            JsStrVariant::Latin1(s) => {
                state.write_usize(s.len());
                for elem in s {
                    state.write_u16(u16::from(*elem));
                }
            }
            JsStrVariant::Utf16(s) => {
                state.write_usize(s.len());
                for elem in s {
                    state.write_u16(*elem);
                }
            }
        }
    }
}

impl Ord for JsStr<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.variant(), other.variant()) {
            (JsStrVariant::Latin1(x), JsStrVariant::Latin1(y)) => x.cmp(y),
            (JsStrVariant::Utf16(x), JsStrVariant::Utf16(y)) => x.cmp(y),
            _ => self.iter().cmp(other.iter()),
        }
    }
}

impl Eq for JsStr<'_> {}

impl PartialEq for JsStr<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self.variant(), other.variant()) {
            (JsStrVariant::Latin1(lhs), JsStrVariant::Latin1(rhs)) => return lhs == rhs,
            (JsStrVariant::Utf16(lhs), JsStrVariant::Utf16(rhs)) => return lhs == rhs,
            _ => {}
        }
        if self.len() != other.len() {
            return false;
        }
        for (x, y) in self.iter().zip(other.iter()) {
            if x != y {
                return false;
            }
        }
        true
    }
}

impl<'a> PartialEq<JsStr<'a>> for [u16] {
    #[inline]
    fn eq(&self, other: &JsStr<'a>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (x, y) in self.iter().copied().zip(other.iter()) {
            if x != y {
                return false;
            }
        }
        true
    }
}

pub trait JsSliceIndex<'a>: SliceIndex<[u8]> + SliceIndex<[u16]> {
    type Value;

    fn get(_: JsStr<'a>, index: Self) -> Option<Self::Value>;

    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value;
}

impl<'a> JsSliceIndex<'a> for usize {
    type Value = u16;

    #[inline]
    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        match value.variant() {
            JsStrVariant::Latin1(v) => v.get(index).copied().map(u16::from),
            JsStrVariant::Utf16(v) => v.get(index).copied(),
        }
    }

    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        unsafe {
            match value.variant() {
                JsStrVariant::Latin1(v) => u16::from(*v.get_unchecked(index)),
                JsStrVariant::Utf16(v) => *v.get_unchecked(index),
            }
        }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::Range<usize> {
    type Value = JsStr<'a>;

    #[inline]
    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        match value.variant() {
            JsStrVariant::Latin1(v) => v.get(index).map(JsStr::latin1),
            JsStrVariant::Utf16(v) => v.get(index).map(JsStr::utf16),
        }
    }

    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        unsafe {
            match value.variant() {
                JsStrVariant::Latin1(v) => JsStr::latin1(v.get_unchecked(index)),
                JsStrVariant::Utf16(v) => JsStr::utf16(v.get_unchecked(index)),
            }
        }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::RangeInclusive<usize> {
    type Value = JsStr<'a>;

    #[inline]
    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        match value.variant() {
            JsStrVariant::Latin1(v) => v.get(index).map(JsStr::latin1),
            JsStrVariant::Utf16(v) => v.get(index).map(JsStr::utf16),
        }
    }

    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        unsafe {
            match value.variant() {
                JsStrVariant::Latin1(v) => JsStr::latin1(v.get_unchecked(index)),
                JsStrVariant::Utf16(v) => JsStr::utf16(v.get_unchecked(index)),
            }
        }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::RangeFrom<usize> {
    type Value = JsStr<'a>;

    #[inline]
    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        match value.variant() {
            JsStrVariant::Latin1(v) => v.get(index).map(JsStr::latin1),
            JsStrVariant::Utf16(v) => v.get(index).map(JsStr::utf16),
        }
    }

    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        unsafe {
            match value.variant() {
                JsStrVariant::Latin1(v) => JsStr::latin1(v.get_unchecked(index)),
                JsStrVariant::Utf16(v) => JsStr::utf16(v.get_unchecked(index)),
            }
        }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::RangeTo<usize> {
    type Value = JsStr<'a>;

    #[inline]
    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        match value.variant() {
            JsStrVariant::Latin1(v) => v.get(index).map(JsStr::latin1),
            JsStrVariant::Utf16(v) => v.get(index).map(JsStr::utf16),
        }
    }

    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        unsafe {
            match value.variant() {
                JsStrVariant::Latin1(v) => JsStr::latin1(v.get_unchecked(index)),
                JsStrVariant::Utf16(v) => JsStr::utf16(v.get_unchecked(index)),
            }
        }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::RangeFull {
    type Value = JsStr<'a>;

    #[inline]
    fn get(value: JsStr<'a>, _index: Self) -> Option<Self::Value> {
        Some(value)
    }

    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        value
    }
}
