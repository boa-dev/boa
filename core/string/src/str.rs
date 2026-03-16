use super::iter::{CodePointsIter, Windows};
use crate::{CodePoint, Iter, display::JsStrDisplayLossy, is_trimmable_whitespace};
use std::{
    hash::{Hash, Hasher},
    slice::SliceIndex,
};

/// A view into a rope string.
#[derive(Debug, Clone, Copy)]
pub struct RopeSlice<'a> {
    pub header: &'a crate::vtable::JsStringHeader,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl RopeSlice<'_> {
    /// Get the length of the slice.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }
}

/// This is equivalent to Rust's `&str`.
#[derive(Clone, Copy)]
pub enum JsStr<'a> {
    /// Latin1 string representation.
    Latin1(&'a [u8]),

    /// U16 string representation.
    Utf16(&'a [u16]),

    /// A view into a rope string.
    Rope(RopeSlice<'a>),
}

// SAFETY: JsStr<'_> has only immutable references to Sync types, so this is safe.
unsafe impl Sync for JsStr<'_> {}

// SAFETY: It's read-only, sending this reference to another thread doesn't
//         risk data races (there’s no mutation happening), so this is safe.
unsafe impl Send for JsStr<'_> {}

impl<'a> JsStr<'a> {
    /// This represents an empty string.
    pub const EMPTY: Self = Self::Latin1("".as_bytes());

    /// Creates a [`JsStr`] from codepoints that can fit in a `u8`.
    #[inline]
    #[must_use]
    pub const fn latin1(value: &'a [u8]) -> Self {
        Self::Latin1(value)
    }

    /// Creates a [`JsStr`] from utf16 encoded string.
    #[inline]
    #[must_use]
    pub const fn utf16(value: &'a [u16]) -> Self {
        Self::Utf16(value)
    }

    /// Creates a [`JsStr`] from a rope slice.
    #[inline]
    #[must_use]
    pub const fn rope(slice: RopeSlice<'a>) -> Self {
        Self::Rope(slice)
    }

    /// Return the inner variant of the [`JsStr`].
    #[inline]
    #[must_use]
    pub const fn variant(self) -> Self {
        self
    }

    /// Get the length of the [`JsStr`].
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            Self::Latin1(data) => data.len(),
            Self::Utf16(data) => data.len(),
            Self::Rope(rope) => rope.len(),
        }
    }

    /// Check if the [`JsStr`] is latin1 encoded.
    #[inline]
    #[must_use]
    pub const fn is_latin1(&self) -> bool {
        matches!(self, Self::Latin1(_))
    }

    /// Returns [`u8`] slice if the [`JsStr`] is latin1 encoded, otherwise [`None`].
    #[inline]
    #[must_use]
    pub const fn as_latin1(&self) -> Option<&[u8]> {
        match self {
            Self::Latin1(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the same string slice but with a static reference, removing any
    /// lifetime limits.
    ///
    /// # Safety
    /// The caller is responsible to ensure the lifetime of this slice.
    #[inline]
    #[must_use]
    pub unsafe fn as_static(self) -> JsStr<'static> {
        match self {
            Self::Latin1(v) => {
                // SAFETY: Caller is responsible for ensuring the lifetime of this slice.
                let static_v: &'static [u8] =
                    unsafe { std::slice::from_raw_parts(v.as_ptr(), v.len()) };
                JsStr::<'static>::Latin1(static_v)
            }
            Self::Utf16(v) => {
                // SAFETY: Caller is responsible for ensuring the lifetime of this slice.
                let static_v: &'static [u16] =
                    unsafe { std::slice::from_raw_parts(v.as_ptr(), v.len()) };
                JsStr::<'static>::Utf16(static_v)
            }
            Self::Rope(r) => {
                JsStr::<'static>::Rope(RopeSlice {
                    // SAFETY: The `JsStr` is a view into a string that is guaranteed to stay alive for 'static
                    // if the caller ensures the lifetime requirements.
                    header: unsafe { &*std::ptr::from_ref(r.header) },
                    start: r.start,
                    end: r.end,
                })
            }
        }
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

    /// Returns an element or subslice depending on the type of index, otherwise [`None`].
    #[inline]
    #[must_use]
    pub fn get<I>(self, index: I) -> Option<I::Value>
    where
        I: JsSliceIndex<'a>,
    {
        JsSliceIndex::get(self, index)
    }

    /// Get the element at the given index.
    ///
    /// # Panics
    ///
    /// If the index is out of bounds.
    #[inline]
    #[must_use]
    pub fn get_expect<I>(&self, index: I) -> I::Value
    where
        I: JsSliceIndex<'a>,
    {
        self.get(index).expect("Index out of bounds")
    }

    /// Returns an element or subslice depending on the type of index, without doing bounds check.
    ///
    /// # Safety
    ///
    /// Caller must ensure the index is not out of bounds
    #[inline]
    #[must_use]
    pub unsafe fn get_unchecked<I>(self, index: I) -> I::Value
    where
        I: JsSliceIndex<'a>,
    {
        // SAFETY: Caller must ensure the index is not out of bounds
        unsafe { JsSliceIndex::get_unchecked(self, index) }
    }

    /// Convert the [`JsStr`] into a [`Vec<U16>`].
    #[inline]
    #[must_use]
    pub fn to_vec(&self) -> Vec<u16> {
        match self {
            Self::Latin1(v) => v.iter().copied().map(u16::from).collect(),
            Self::Utf16(v) => v.to_vec(),
            Self::Rope(_) => self.iter().collect(),
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

    /// Abstract operation `StringIndexOf ( string, searchValue, fromIndex )`
    ///
    /// Note: Instead of returning an isize with `-1` as the "not found" value, we make use of the
    /// type system and return <code>[Option]\<usize\></code> with [`None`] as the "not found" value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringindexof
    #[inline]
    #[must_use]
    pub fn index_of(&self, search_value: JsStr<'_>, from_index: usize) -> Option<usize> {
        // 1. Assert: Type(string) is String.
        // 2. Assert: Type(searchValue) is String.
        // 3. Assert: fromIndex is a non-negative integer.

        // 4. Let len be the length of string.
        let len = self.len();

        // 5. If searchValue is the empty String and fromIndex ≤ len, return fromIndex.
        if search_value.is_empty() {
            return if from_index <= len {
                Some(from_index)
            } else {
                None
            };
        }

        // 6. Let searchLen be the length of searchValue.
        // 7. For each integer i starting with fromIndex such that i ≤ len - searchLen, in ascending order, do
        // a. Let candidate be the substring of string from i to i + searchLen.
        // b. If candidate is the same sequence of code units as searchValue, return i.
        // 8. Return -1.
        self.windows(search_value.len())
            .skip(from_index)
            .position(|s| s == search_value)
            .map(|i| i + from_index)
    }

    /// Abstract operation `CodePointAt( string, position )`.
    ///
    /// The abstract operation `CodePointAt` takes arguments `string` (a String) and `position` (a
    /// non-negative integer) and returns a Record with fields `[[CodePoint]]` (a code point),
    /// `[[CodeUnitCount]]` (a positive integer), and `[[IsUnpairedSurrogate]]` (a Boolean). It
    /// interprets string as a sequence of UTF-16 encoded code points, as described in 6.1.4, and reads
    /// from it a single code point starting with the code unit at index `position`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-codepointat
    ///
    /// # Panics
    ///
    /// If `position` is smaller than size of string.
    #[inline]
    #[must_use]
    pub fn code_point_at(&self, position: usize) -> CodePoint {
        // 1. Let size be the length of string.
        let size = self.len();

        // 2. Assert: position ≥ 0 and position < size.
        // position >= 0 ensured by position: usize
        assert!(position < size);

        match self {
            Self::Latin1(v) => {
                let code_point = v.get(position).expect("Already checked the size");
                CodePoint::Unicode(*code_point as char)
            }
            // 3. Let first be the code unit at index position within string.
            // 4. Let cp be the code point whose numeric value is that of first.
            // 5. If first is not a leading surrogate or trailing surrogate, then
            // a. Return the Record { [[CodePoint]]: cp, [[CodeUnitCount]]: 1, [[IsUnpairedSurrogate]]: false }.
            // 6. If first is a trailing surrogate or position + 1 = size, then
            // a. Return the Record { [[CodePoint]]: cp, [[CodeUnitCount]]: 1, [[IsUnpairedSurrogate]]: true }.
            // 7. Let second be the code unit at index position + 1 within string.
            // 8. If second is not a trailing surrogate, then
            // a. Return the Record { [[CodePoint]]: cp, [[CodeUnitCount]]: 1, [[IsUnpairedSurrogate]]: true }.
            // 9. Set cp to ! UTF16SurrogatePairToCodePoint(first, second).
            Self::Utf16(v) => {
                // We can skip the checks and instead use the `char::decode_utf16` function to take care of that for us.
                let code_point = v
                    .get(position..=position + 1)
                    .unwrap_or(&v[position..=position]);

                match char::decode_utf16(code_point.iter().copied())
                    .next()
                    .expect("code_point always has a value")
                {
                    Ok(c) => CodePoint::Unicode(c),
                    Err(e) => CodePoint::UnpairedSurrogate(e.unpaired_surrogate()),
                }
            }
            Self::Rope(_) => self
                .code_points()
                .nth(position)
                .expect("Already checked the size"),
        }
    }

    /// Abstract operation `StringToNumber ( str )`
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringtonumber
    #[inline]
    #[must_use]
    pub fn to_number(&self) -> f64 {
        // 1. Let text be ! StringToCodePoints(str).
        // 2. Let literal be ParseText(text, StringNumericLiteral).
        let Ok(string) = self.to_std_string() else {
            // 3. If literal is a List of errors, return NaN.
            return f64::NAN;
        };
        // 4. Return StringNumericValue of literal.
        let string = string.trim_matches(is_trimmable_whitespace);
        match string {
            "" => return 0.0,
            "-Infinity" => return f64::NEG_INFINITY,
            "Infinity" | "+Infinity" => return f64::INFINITY,
            _ => {}
        }

        let mut s = string.bytes();
        let base = match (s.next(), s.next()) {
            (Some(b'0'), Some(b'b' | b'B')) => Some(2),
            (Some(b'0'), Some(b'o' | b'O')) => Some(8),
            (Some(b'0'), Some(b'x' | b'X')) => Some(16),
            // Make sure that no further variants of "infinity" are parsed.
            (Some(b'i' | b'I'), _) => {
                return f64::NAN;
            }
            _ => None,
        };

        // Parse numbers that begin with `0b`, `0o` and `0x`.
        if let Some(base) = base {
            let string = &string[2..];
            if string.is_empty() {
                return f64::NAN;
            }

            // Fast path
            if let Ok(value) = u32::from_str_radix(string, base) {
                return f64::from(value);
            }

            // Slow path
            let mut value: f64 = 0.0;
            for c in s {
                if let Some(digit) = char::from(c).to_digit(base) {
                    value = value.mul_add(f64::from(base), f64::from(digit));
                } else {
                    return f64::NAN;
                }
            }
            return value;
        }

        fast_float2::parse(string).unwrap_or(f64::NAN)
    }

    /// Gets an iterator of all the Unicode codepoints of a [`JsStr`].
    #[inline]
    #[must_use]
    pub fn code_points(&self) -> CodePointsIter<'a> {
        CodePointsIter::new(*self)
    }

    /// Checks if the [`JsStr`] contains a byte.
    #[inline]
    #[must_use]
    pub fn contains(&self, element: u8) -> bool {
        match self {
            Self::Latin1(v) => v.contains(&element),
            Self::Utf16(v) => v.contains(&u16::from(element)),
            Self::Rope(_) => self.iter().any(|u| u == u16::from(element)),
        }
    }

    /// Gets an iterator of all the Unicode codepoints of a [`JsStr`], replacing
    /// unpaired surrogates with the replacement character. This is faster than
    /// using [`Self::code_points`].
    #[inline]
    pub fn code_points_lossy(self) -> impl Iterator<Item = char> + 'a {
        char::decode_utf16(self.iter()).map(|res| res.unwrap_or('\u{FFFD}'))
    }

    /// Decodes a [`JsStr`] into a [`String`], returning an error if it contains any invalid data.
    ///
    /// # Errors
    ///
    /// [`FromUtf16Error`][std::string::FromUtf16Error] if it contains any invalid data.
    #[inline]
    pub fn to_std_string(&self) -> Result<String, std::string::FromUtf16Error> {
        match self {
            Self::Latin1(v) => Ok(v.iter().copied().map(char::from).collect()),
            Self::Utf16(v) => String::from_utf16(v),
            Self::Rope(_) => String::from_utf16(&self.to_vec()),
        }
    }

    /// Decodes a [`JsStr`] into a [`String`], replacing invalid data with the
    /// replacement character U+FFFD.
    #[inline]
    #[must_use]
    pub fn to_std_string_lossy(&self) -> String {
        self.display_lossy().to_string()
    }

    /// Gets a displayable lossy string.
    ///
    /// This may be faster and has fewer
    /// allocations than `format!("{}", str.to_string_lossy())` when displaying.
    #[inline]
    #[must_use]
    pub fn display_lossy(&self) -> JsStrDisplayLossy<'a> {
        JsStrDisplayLossy::from(*self)
    }
}

impl Hash for JsStr<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.content_hash());
    }
}

impl JsStr<'_> {
    /// Computes the hash of the string content.
    #[inline]
    #[must_use]
    pub fn content_hash(&self) -> u64 {
        let mut h = rustc_hash::FxHasher::default();
        match *self {
            Self::Latin1(s) => {
                h.write_usize(s.len());
                for elem in s {
                    h.write_u16(u16::from(*elem));
                }
            }
            Self::Utf16(s) => {
                h.write_usize(s.len());
                for elem in s {
                    h.write_u16(*elem);
                }
            }
            Self::Rope(_) => {
                h.write_usize(self.len());
                for elem in self.iter() {
                    h.write_u16(elem);
                }
            }
        }
        h.finish()
    }
}

impl Ord for JsStr<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Latin1(x), Self::Latin1(y)) => x.cmp(y),
            (Self::Utf16(x), Self::Utf16(y)) => x.cmp(y),
            _ => self.iter().cmp(other.iter()),
        }
    }
}

impl Eq for JsStr<'_> {}

impl PartialEq for JsStr<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Latin1(lhs), Self::Latin1(rhs)) => return lhs == rhs,
            (Self::Utf16(lhs), Self::Utf16(rhs)) => return lhs == rhs,
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

impl PartialEq<str> for JsStr<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        match *self {
            Self::Latin1(v) => v == other.as_bytes(),
            Self::Utf16(v) => other.encode_utf16().zip(v).all(|(a, b)| a == *b),
            Self::Rope(_) => other.encode_utf16().zip(self.iter()).all(|(a, b)| a == b),
        }
    }
}

impl PartialEq<&str> for JsStr<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self == *other
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

impl std::fmt::Debug for JsStr<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsStr").field("len", &self.len()).finish()
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
        match value {
            JsStr::Latin1(v) => v.get(index).copied().map(u16::from),
            JsStr::Utf16(v) => v.get(index).copied(),
            JsStr::Rope(_) => {
                if index < value.len() {
                    Some(value.iter().nth(index).expect("Already checked the size"))
                } else {
                    None
                }
            }
        }
    }

    /// # Safety
    ///
    /// Caller must ensure the index is not out of bounds
    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        // SAFETY: Caller must ensure the index is not out of bounds.
        unsafe {
            match value {
                JsStr::Latin1(v) => u16::from(*v.get_unchecked(index)),
                JsStr::Utf16(v) => *v.get_unchecked(index),
                JsStr::Rope(_) => value.iter().nth(index).expect("Already checked the size"),
            }
        }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::Range<usize> {
    type Value = JsStr<'a>;

    #[inline]
    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        match value {
            JsStr::Latin1(v) => v.get(index).map(JsStr::latin1),
            JsStr::Utf16(v) => v.get(index).map(JsStr::utf16),
            JsStr::Rope(r) => {
                if index.start <= index.end && index.end <= r.len() {
                    Some(JsStr::Rope(RopeSlice {
                        header: r.header,
                        start: r.start + index.start,
                        end: r.start + index.end,
                    }))
                } else {
                    None
                }
            }
        }
    }

    /// # Safety
    ///
    /// Caller must ensure the index is not out of bounds
    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        match value {
            // SAFETY: Caller must ensure the index is not out of bounds.
            JsStr::Latin1(v) => unsafe { JsStr::latin1(v.get_unchecked(index)) },
            // SAFETY: Caller must ensure the index is not out of bounds.
            JsStr::Utf16(v) => unsafe { JsStr::utf16(v.get_unchecked(index)) },
            JsStr::Rope(r) => JsStr::Rope(RopeSlice {
                header: r.header,
                start: r.start + index.start,
                end: r.start + index.end,
            }),
        }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::RangeInclusive<usize> {
    type Value = JsStr<'a>;

    #[inline]
    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        if *index.end() == usize::MAX {
            return None;
        }
        JsSliceIndex::get(value, *index.start()..*index.end() + 1)
    }

    /// # Safety
    ///
    /// Caller must ensure the index is not out of bounds
    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        // Safety: Caller must ensure the index is not out of bounds
        unsafe { JsSliceIndex::get_unchecked(value, *index.start()..*index.end() + 1) }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::RangeFrom<usize> {
    type Value = JsStr<'a>;

    #[inline]
    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        JsSliceIndex::get(value, index.start..value.len())
    }

    /// # Safety
    ///
    /// Caller must ensure the index is not out of bounds
    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        // Safety: Caller must ensure the index is not out of bounds
        unsafe { JsSliceIndex::get_unchecked(value, index.start..value.len()) }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::RangeTo<usize> {
    type Value = JsStr<'a>;

    #[inline]
    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        JsSliceIndex::get(value, 0..index.end)
    }

    /// # Safety
    ///
    /// Caller must ensure the index is not out of bounds
    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, index: Self) -> Self::Value {
        // Safety: Caller must ensure the index is not out of bounds
        unsafe { JsSliceIndex::get_unchecked(value, 0..index.end) }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::RangeFull {
    type Value = JsStr<'a>;

    #[inline]
    fn get(value: JsStr<'a>, _index: Self) -> Option<Self::Value> {
        Some(value)
    }

    /// # Safety
    ///
    /// Caller must ensure the index is not out of bounds
    #[inline]
    unsafe fn get_unchecked(value: JsStr<'a>, _index: Self) -> Self::Value {
        value
    }
}
