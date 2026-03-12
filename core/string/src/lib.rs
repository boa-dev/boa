//! A Latin1 or UTF-16 encoded, reference counted, immutable string.

// Required per unsafe code standards to ensure every unsafe usage is properly documented.
// - `unsafe_op_in_unsafe_fn` will be warn-by-default in edition 2024:
//   https://github.com/rust-lang/rust/issues/71668#issuecomment-1189396860
// - `undocumented_unsafe_blocks` and `missing_safety_doc` requires a `Safety:` section in the
//   comment or doc of the unsafe block or function, respectively.
#![deny(
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc
)]
#![allow(clippy::module_name_repetitions)]

mod builder;
mod code_point;
mod common;
mod display;
mod iter;
mod str;
mod r#type;
pub(crate) mod vtable;

#[cfg(test)]
mod tests;

use self::iter::Windows;
use crate::display::{JsStrDisplayEscaped, JsStrDisplayLossy, JsStringDebugInfo};
use crate::iter::CodePointsIter;
use crate::r#type::{Latin1, Utf16};
pub use crate::vtable::StaticString;
pub(crate) use crate::vtable::{RawJsString, RopeString, SequenceString, SliceString};
#[doc(inline)]
pub use crate::{
    builder::{CommonJsStringBuilder, Latin1JsStringBuilder, Utf16JsStringBuilder},
    code_point::CodePoint,
    common::StaticJsStrings,
    iter::Iter,
    str::{JsStr, JsStrVariant},
};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::{borrow::Cow, mem::ManuallyDrop};
use std::{
    convert::Infallible,
    hash::{Hash, Hasher},
    ptr::{self},
    str::FromStr,
};
use vtable::JsStringVTable;

fn alloc_overflow() -> ! {
    panic!("detected overflow during string allocation")
}

/// Helper function to check if a `char` is trimmable.
pub(crate) const fn is_trimmable_whitespace(c: char) -> bool {
    // The rust implementation of `trim` does not regard the same characters whitespace as
    // ecma standard does.
    //
    // Rust uses \p{White_Space} by default, which also includes:
    // `\u{0085}' (next line)
    // And does not include:
    // '\u{FEFF}' (zero width non-breaking space)
    // Explicit whitespace: https://tc39.es/ecma262/#sec-white-space
    matches!(
        c,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{FEFF}' |
    // Unicode Space_Separator category
    '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' |
    // Line terminators: https://tc39.es/ecma262/#sec-line-terminators
    '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}'
    )
}

/// Helper function to check if a `u8` latin1 character is trimmable.
pub(crate) const fn is_trimmable_whitespace_latin1(c: u8) -> bool {
    // The rust implementation of `trim` does not regard the same characters whitespace as
    // ecma standard does.
    //
    // Rust uses \p{White_Space} by default, which also includes:
    // `\u{0085}' (next line)
    // And does not include:
    // '\u{FEFF}' (zero width non-breaking space)
    // Explicit whitespace: https://tc39.es/ecma262/#sec-white-space
    matches!(
        c,
        0x09 | 0x0B | 0x0C | 0x20 | 0xA0 |
        // Line terminators: https://tc39.es/ecma262/#sec-line-terminators
        0x0A | 0x0D
    )
}

/// Strings can be represented internally by multiple kinds. This is used to identify
/// the storage kind of string.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum JsStringKind {
    /// A sequential memory slice of Latin1 bytes. See [`SequenceString`].
    Latin1Sequence = 0,

    /// A sequential memory slice of UTF-16 code units. See [`SequenceString`].
    Utf16Sequence = 1,

    /// A slice of an existing string. See [`SliceString`].
    Slice = 2,

    /// A static string that is valid for `'static` lifetime.
    Static = 3,

    /// A rope string that is a tree of other strings. See [`RopeString`].
    Rope = 4,
}

/// A Latin1 or UTF-16–encoded, reference counted, immutable string.
///
/// This is pretty similar to a <code>[Rc][std::rc::Rc]\<[\[u16\]][slice]\></code>, but without the
/// length metadata associated with the `Rc` fat pointer. Instead, the length of every string is
/// stored on the heap, along with its reference counter and its data.
///
/// The string can be latin1 (stored as a byte for space efficiency) or U16 encoding.
///
/// We define some commonly used string constants in an interner. For these strings, we don't allocate
/// memory on the heap to reduce the overhead of memory allocation and reference counting.
///
/// # Internal representation
///
/// The `ptr` field always points to a structure whose first field is a `JsStringVTable`.
/// This enables uniform vtable dispatch for all string operations without branching.
///
/// Because we ensure this invariant at every construction, we can directly point to this
/// type to allow for better optimization (and simpler code).
#[allow(clippy::module_name_repetitions)]
pub struct JsString {
    /// Pointer to the string data. Always points to a `RawJsString` header.
    pub(crate) ptr: NonNull<RawJsString>,
}

// `JsString` should always be thin-pointer sized.
static_assertions::assert_eq_size!(JsString, *const ());

impl<'a> From<&'a JsString> for JsStr<'a> {
    #[inline]
    fn from(value: &'a JsString) -> Self {
        value.as_str()
    }
}

impl<'a> IntoIterator for &'a JsString {
    type Item = u16;
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl JsString {
    /// Create an iterator over the [`JsString`].
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        self.as_str().iter()
    }

    /// Create an iterator over overlapping subslices of length size.
    #[inline]
    #[must_use]
    pub fn windows(&self, size: usize) -> Windows<'_> {
        self.as_str().windows(size)
    }

    /// Decodes a [`JsString`] into a [`String`], replacing invalid data with its escaped representation
    /// in 4 digit hexadecimal.
    #[inline]
    #[must_use]
    pub fn to_std_string_escaped(&self) -> String {
        self.display_escaped().to_string()
    }

    /// Decodes a [`JsString`] into a [`String`], replacing invalid data with the
    /// replacement character U+FFFD.
    #[inline]
    #[must_use]
    pub fn to_std_string_lossy(&self) -> String {
        self.display_lossy().to_string()
    }

    /// Decodes a [`JsString`] into a [`String`], returning an error if the string contains unpaired
    /// surrogates.
    ///
    /// # Errors
    ///
    /// [`FromUtf16Error`][std::string::FromUtf16Error] if it contains any invalid data.
    #[inline]
    pub fn to_std_string(&self) -> Result<String, std::string::FromUtf16Error> {
        self.as_str().to_std_string()
    }

    /// Decodes a [`JsString`] into an iterator of [`Result<String, u16>`], returning surrogates as
    /// errors.
    #[inline]
    #[allow(clippy::missing_panics_doc)]
    pub fn to_std_string_with_surrogates(&self) -> impl Iterator<Item = Result<String, u16>> + '_ {
        let mut iter = self.code_points().peekable();

        std::iter::from_fn(move || {
            let cp = iter.next()?;
            let char = match cp {
                CodePoint::Unicode(c) => c,
                CodePoint::UnpairedSurrogate(surr) => return Some(Err(surr)),
            };

            let mut string = String::from(char);

            loop {
                let Some(cp) = iter.peek().and_then(|cp| match cp {
                    CodePoint::Unicode(c) => Some(*c),
                    CodePoint::UnpairedSurrogate(_) => None,
                }) else {
                    break;
                };

                string.push(cp);

                iter.next().expect("should exist by the check above");
            }

            Some(Ok(string))
        })
    }

    /// Maps the valid segments of an UTF16 string and leaves the unpaired surrogates unchanged.
    #[inline]
    #[must_use]
    pub fn map_valid_segments<F>(&self, mut f: F) -> Self
    where
        F: FnMut(String) -> String,
    {
        let mut text = Vec::new();

        for part in self.to_std_string_with_surrogates() {
            match part {
                Ok(string) => text.extend(f(string).encode_utf16()),
                Err(surr) => text.push(surr),
            }
        }

        Self::from(&text[..])
    }

    /// Gets an iterator of all the Unicode codepoints of a [`JsString`].
    #[inline]
    #[must_use]
    pub fn code_points(&self) -> CodePointsIter<'_> {
        // SAFETY: The `vtable().code_points` function is guaranteed to be a valid function pointer
        // for the specific string type, and `self.ptr` is a valid `NonNull<RawJsString>` pointer.
        (self.vtable().code_points)(self.ptr)
    }

    /// Get the variant of this string.
    #[inline]
    #[must_use]
    pub fn variant(&self) -> JsStrVariant<'_> {
        // SAFETY: The pointer `self.ptr` is always valid and points to a `RawJsString` header.
        let header = unsafe { self.ptr.as_ref() };
        match header.kind {
            JsStringKind::Latin1Sequence => {
                // SAFETY: `header.kind` is `Latin1Sequence`, so `self.ptr` can be safely cast to `SequenceString<Latin1>`.
                let seq: &SequenceString<Latin1> = unsafe { self.ptr.cast().as_ref() };
                // SAFETY: `seq.data()` is a valid pointer to the Latin1 data, and `header.len` is the correct length.
                JsStrVariant::Latin1(unsafe { std::slice::from_raw_parts(seq.data(), header.len) })
            }
            JsStringKind::Utf16Sequence => {
                // SAFETY: `header.kind` is `Utf16Sequence`, so `self.ptr` can be safely cast to `SequenceString<Utf16>`.
                let seq: &SequenceString<Utf16> = unsafe { self.ptr.cast().as_ref() };
                // SAFETY: `seq.data()` is a valid pointer to the UTF-16 data, and `header.len` is the correct length.
                JsStrVariant::Utf16(unsafe {
                    std::slice::from_raw_parts(seq.data().cast(), header.len)
                })
            }
            // For Static, Slice, and Rope, the `as_str()` method handles the variant conversion.
            // This avoids redundant logic and ensures consistency.
            JsStringKind::Static | JsStringKind::Slice | JsStringKind::Rope => {
                self.as_str().variant()
            }
        }
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
        self.as_str().index_of(search_value, from_index)
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
        self.as_str().code_point_at(position)
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
        self.as_str().to_number()
    }

    /// Get the length of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        // SAFETY: The pointer `self.ptr` is always valid and points to a `RawJsString` header.
        unsafe { self.ptr.as_ref().len }
    }

    /// Return true if the [`JsString`] is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert the [`JsString`] into a [`Vec<U16>`].
    #[inline]
    #[must_use]
    pub fn to_vec(&self) -> Vec<u16> {
        self.as_str().to_vec()
    }

    /// Check if the [`JsString`] contains a byte.
    #[inline]
    #[must_use]
    pub fn contains(&self, element: u8) -> bool {
        self.as_str().contains(element)
    }

    /// Trim whitespace from the start and end of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn trim(&self) -> JsString {
        // Calculate both bounds directly to avoid intermediate allocations.
        let (start, end) = match self.variant() {
            JsStrVariant::Latin1(v) => {
                let Some(start) = v.iter().position(|c| !is_trimmable_whitespace_latin1(*c)) else {
                    return StaticJsStrings::EMPTY_STRING;
                };
                let end = v
                    .iter()
                    .rposition(|c| !is_trimmable_whitespace_latin1(*c))
                    .unwrap_or(start);
                (start, end)
            }
            JsStrVariant::Utf16(v) => {
                let Some(start) = v.iter().copied().position(|r| {
                    !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)
                }) else {
                    return StaticJsStrings::EMPTY_STRING;
                };
                let end = v
                    .iter()
                    .copied()
                    .rposition(|r| {
                        !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)
                    })
                    .unwrap_or(start);
                (start, end)
            }
        };

        // SAFETY: `start` and `end` are calculated from valid indices within the string,
        // ensuring `start <= end` and `end <= self.len()`.
        unsafe { Self::slice_unchecked(self, start, end + 1) }
    }

    /// Trim whitespace from the start of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn trim_start(&self) -> JsString {
        let Some(start) = (match self.variant() {
            JsStrVariant::Latin1(v) => v.iter().position(|c| !is_trimmable_whitespace_latin1(*c)),
            JsStrVariant::Utf16(v) => v
                .iter()
                .copied()
                .position(|r| !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)),
        }) else {
            return StaticJsStrings::EMPTY_STRING;
        };

        // SAFETY: `start` is a valid index within the string, ensuring `start <= self.len()`.
        unsafe { Self::slice_unchecked(self, start, self.len()) }
    }

    /// Trim whitespace from the end of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn trim_end(&self) -> JsString {
        let Some(end) = (match self.variant() {
            JsStrVariant::Latin1(v) => v.iter().rposition(|c| !is_trimmable_whitespace_latin1(*c)),
            JsStrVariant::Utf16(v) => v
                .iter()
                .copied()
                .rposition(|r| !char::from_u32(u32::from(r)).is_some_and(is_trimmable_whitespace)),
        }) else {
            return StaticJsStrings::EMPTY_STRING;
        };

        // SAFETY: `end` is a valid index within the string, ensuring `end + 1 <= self.len()`.
        unsafe { Self::slice_unchecked(self, 0, end + 1) }
    }

    /// Returns true if needle is a prefix of the [`JsStr`].
    #[inline]
    #[must_use]
    // We check the size, so this should never panic.
    #[allow(clippy::missing_panics_doc)]
    pub fn starts_with(&self, needle: JsStr<'_>) -> bool {
        self.as_str().starts_with(needle)
    }

    /// Returns `true` if `needle` is a suffix of the [`JsStr`].
    #[inline]
    #[must_use]
    // We check the size, so this should never panic.
    #[allow(clippy::missing_panics_doc)]
    pub fn ends_with(&self, needle: JsStr<'_>) -> bool {
        self.as_str().ends_with(needle)
    }

    /// Get the element at the given index, or [`None`] if the index is out of range.
    #[inline]
    #[must_use]
    pub fn get<I>(&self, index: I) -> Option<JsString>
    where
        I: JsStringSliceIndex,
    {
        index.get(self)
    }

    /// Get the element at the given index, or panic.
    ///
    /// # Panics
    /// If the index returns `None`, this will panic.
    #[inline]
    #[must_use]
    pub fn get_expect<I>(&self, index: I) -> JsString
    where
        I: JsStringSliceIndex,
    {
        index.get(self).expect("Unexpected get()")
    }

    /// Gets a displayable escaped string. This may be faster and has fewer
    /// allocations than `format!("{}", str.to_string_escaped())` when
    /// displaying.
    #[inline]
    #[must_use]
    pub fn display_escaped(&self) -> JsStrDisplayEscaped<'_> {
        JsStrDisplayEscaped::from(self)
    }

    /// Gets a displayable lossy string. This may be faster and has fewer
    /// allocations than `format!("{}", str.to_string_lossy())` when displaying.
    #[inline]
    #[must_use]
    pub fn display_lossy(&self) -> JsStrDisplayLossy<'_> {
        self.as_str().display_lossy()
    }

    /// Get a debug displayable info and metadata for this string.
    #[inline]
    #[must_use]
    pub fn debug_info(&self) -> JsStringDebugInfo<'_> {
        self.into()
    }

    /// Consumes the [`JsString`], returning the internal pointer.
    ///
    /// To avoid a memory leak the pointer must be converted back to a `JsString` using
    /// [`JsString::from_raw`].
    #[must_use]
    pub fn into_raw(self) -> NonNull<RawJsString> {
        ManuallyDrop::new(self).ptr.cast()
    }

    /// Constructs a `JsString` from the internal pointer.
    ///
    /// The raw pointer must have been previously returned by a call to
    /// [`JsString::into_raw`].
    ///
    /// # Safety
    ///
    /// This function is unsafe because improper use may lead to memory unsafety,
    /// even if the returned `JsString` is never accessed.
    #[inline]
    #[must_use]
    pub const unsafe fn from_raw(ptr: NonNull<RawJsString>) -> Self {
        Self { ptr }
    }
}

// `&JsStr<'static>` must always be aligned so it can be tagged.
static_assertions::const_assert!(align_of::<*const JsStr<'static>>() >= 2);

/// Dealing with inner types.
impl JsString {
    /// Check if this is a static string.
    #[inline]
    #[must_use]
    pub fn is_static(&self) -> bool {
        self.kind() == JsStringKind::Static
    }

    /// Get the vtable for this string.
    #[inline]
    #[must_use]
    const fn vtable(&self) -> &JsStringVTable {
        // SAFETY: The pointer `self.ptr` is always valid and points to a `RawJsString` header.
        unsafe { self.ptr.as_ref().vtable }
    }

    /// Create a [`JsString`] from a [`StaticString`] instance. This is assumed that the
    /// static string referenced is available for the duration of the `JsString` instance
    /// returned.
    #[inline]
    #[must_use]
    pub const fn from_static(str: &'static StaticString) -> Self {
        // SAFETY: `str` is a reference to a `StaticString`, which is guaranteed to have a valid `header` field.
        // The address of `str.header` is valid and non-null, and casting to `*mut RawJsString` is safe
        // because `RawJsString` is the common header for all string types.
        Self {
            // SAFETY: The address of `str.header` is guaranteed to be non-null.
            ptr: unsafe { NonNull::new_unchecked((&raw const str.header).cast_mut()) },
        }
    }

    /// Create a [`JsString`] from an existing `JsString` and start, end
    /// range. `end` is 1 past the last character (or `== data.len()`
    /// for the last character).
    ///
    /// # Safety
    /// It is the responsibility of the caller to ensure:
    ///   - `start` <= `end`. If `start` == `end`, the string is empty.
    ///   - `end` <= `data.len()`.
    #[inline]
    #[must_use]
    pub unsafe fn slice_unchecked(data: &JsString, start: usize, end: usize) -> Self {
        // Safety: The caller guarantees `start <= end` and `end <= data.len()`.
        // `SliceString::new` creates a valid slice, and `Box::leak` correctly manages memory.
        let slice = Box::new(unsafe { SliceString::new(data, start, end) });

        Self {
            ptr: NonNull::from(Box::leak(slice)).cast(),
        }
    }

    /// Casts the string to an inner type.
    ///
    /// # Safety
    /// The caller must ensure that the string is of the correct kind.
    #[inline]
    #[must_use]
    pub(crate) unsafe fn as_inner<T>(&self) -> &T {
        // SAFETY: The caller must ensure that the string is of the correct kind,
        // allowing the `self.ptr` to be safely cast to `NonNull<T>` and then dereferenced.
        unsafe { self.ptr.cast().as_ref() }
    }

    /// Get the depth of the rope (0 if not a rope).
    #[inline]
    #[must_use]
    pub fn depth(&self) -> u8 {
        if self.kind() == JsStringKind::Rope {
            // SAFETY: `self.kind()` is checked to be `Rope`, so `self.as_inner::<RopeString>()` is safe.
            unsafe { self.as_inner::<RopeString>().depth() }
        } else {
            0
        }
    }

    /// Create a [`JsString`] from an existing `JsString` and start, end
    /// range. Returns None if the start/end is invalid.
    #[inline]
    #[must_use]
    pub fn slice(&self, p1: usize, mut p2: usize) -> JsString {
        if p2 > self.len() {
            p2 = self.len();
        }
        if p1 >= p2 {
            StaticJsStrings::EMPTY_STRING
        } else {
            // SAFETY: The conditions `p1 <= p2` and `p2 <= self.len()` are ensured by the `if` blocks above.
            unsafe { Self::slice_unchecked(self, p1, p2) }
        }
    }

    /// Get the kind of this string (for debugging/introspection).
    #[inline]
    #[must_use]
    pub(crate) fn kind(&self) -> JsStringKind {
        // SAFETY: The pointer `self.ptr` is always valid and points to a `RawJsString` header.
        unsafe { self.ptr.as_ref().kind }
    }

    /// Returns the string as a [`JsStr`].
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> JsStr<'static> {
        // SAFETY: The pointer `self.ptr` is always valid and points to a `RawJsString` header.
        let header = unsafe { self.ptr.as_ref() };
        // FAST PATH: Devirtualize common kinds.
        match header.kind {
            JsStringKind::Latin1Sequence => {
                // SAFETY: `header.kind` is `Latin1Sequence`, so `self.ptr` can be safely cast to `SequenceString<Latin1>`.
                let seq: &SequenceString<Latin1> = unsafe { self.ptr.cast().as_ref() };
                // SAFETY: `seq.data()` returns a valid pointer to the Latin1 data, and `header.len` is the correct length.
                unsafe { JsStr::latin1(std::slice::from_raw_parts(seq.data(), header.len)) }
            }
            JsStringKind::Utf16Sequence => {
                // SAFETY: `header.kind` is `Utf16Sequence`, so `self.ptr` can be safely cast to `SequenceString<Utf16>`.
                let seq: &SequenceString<Utf16> = unsafe { self.ptr.cast().as_ref() };
                // SAFETY: `seq.data()` returns a valid pointer to the UTF-16 data, and `header.len` is the correct length.
                unsafe { JsStr::utf16(std::slice::from_raw_parts(seq.data().cast(), header.len)) }
            }
            JsStringKind::Static => {
                // SAFETY: `header.kind` is `Static`, so `self.ptr` can be safely cast to `StaticString`.
                let s: &StaticString = unsafe { self.ptr.cast().as_ref() };
                s.str
            }
            JsStringKind::Slice => {
                // SAFETY: `header.kind` is `Slice`, so `self.ptr` can be safely cast to `SliceString`.
                let s: &SliceString = unsafe { self.ptr.cast().as_ref() };
                s.inner
            }
            JsStringKind::Rope => {
                // SAFETY: The `vtable.as_str` function is guaranteed to be a valid function pointer
                // for the specific string type, and `self.ptr` is a valid `NonNull<RawJsString>` pointer.
                (header.vtable.as_str)(self.ptr)
            }
        }
    }

    /// Returns the code unit at `index`.
    #[inline]
    #[must_use]
    pub fn code_unit_at(&self, index: usize) -> Option<u16> {
        // SAFETY: The pointer `self.ptr` is always valid and points to a `RawJsString` header.
        let header = unsafe { self.ptr.as_ref() };
        if index >= header.len {
            return None;
        }
        // FAST PATH: Devirtualize common kinds.
        match header.kind {
            JsStringKind::Latin1Sequence => {
                // SAFETY: `header.kind` is `Latin1Sequence`, so `self.ptr` can be safely cast to `SequenceString<Latin1>`.
                let seq: &SequenceString<Latin1> = unsafe { self.ptr.cast().as_ref() };
                // SAFETY: `seq.data()` returns a valid pointer to the Latin1 data.
                // `index` is checked to be within `header.len`, so `add(index)` is in bounds.
                Some(u16::from(unsafe { *seq.data().add(index) }))
            }
            JsStringKind::Utf16Sequence => {
                // SAFETY: `header.kind` is `Utf16Sequence`, so `self.ptr` can be safely cast to `SequenceString<Utf16>`.
                let seq: &SequenceString<Utf16> = unsafe { self.ptr.cast().as_ref() };
                // SAFETY: `seq.data()` returns a valid pointer to the UTF-16 data.
                // `index` is checked to be within `header.len`, so `add(index)` is in bounds.
                // The pointer is aligned because `RawJsString` has an alignment of 8 and a size that is a multiple of 2.
                #[allow(clippy::cast_ptr_alignment)]
                Some(unsafe { *seq.data().cast::<u16>().add(index) })
            }
            JsStringKind::Static | JsStringKind::Slice => {
                // All these have a direct JsStr representation.
                self.as_str().get(index)
            }
            JsStringKind::Rope => (header.vtable.code_unit_at)(self.ptr, index),
        }
    }

    /// Returns a reference to the reference count cell, if it exists.
    #[inline]
    fn refcount_cell(&self) -> Option<&AtomicUsize> {
        // SAFETY: The pointer is always valid.
        let header = unsafe { self.ptr.as_ref() };
        if header.kind == JsStringKind::Static {
            None
        } else {
            // SAFETY: Alignment and size match, and we checked it's not static.
            Some(unsafe { &*(&raw const header.refcount).cast::<AtomicUsize>() })
        }
    }
}

impl JsString {
    /// Creates a new [`JsString`] from the concatenation of `x` and `y`.
    #[inline]
    #[must_use]
    pub fn concat(x: &Self, y: &Self) -> Self {
        Self::concat_array_strings(&[x.clone(), y.clone()])
    }

    /// Creates a new [`JsString`] from the concatenation of two slices `x` and `y`.
    #[inline]
    #[must_use]
    pub fn concat_slices(x: JsStr<'_>, y: JsStr<'_>) -> Self {
        Self::concat_array(&[x, y])
    }

    /// Creates a new [`JsString`] from the concatenation of every element of
    /// `strings`.
    ///
    /// This will use a rope representation if the concatenation is large enough to
    /// warrant it.
    #[inline]
    #[must_use]
    pub fn concat_array_strings(strings: &[Self]) -> Self {
        if strings.is_empty() {
            return StaticJsStrings::EMPTY_STRING;
        }
        if strings.len() == 1 {
            return strings[0].clone();
        }

        let full_count: usize = strings.iter().map(Self::len).sum();

        // Hybrid Strategy: Use ropes for large concatenations.
        if full_count > 1024 {
            return Self::concat_strings_balanced(strings);
        }

        let slices: Vec<_> = strings.iter().map(Self::as_str).collect();
        Self::concat_array(&slices)
    }

    /// Internal helper to build a balanced rope tree from a slice of strings.
    fn concat_strings_balanced(strings: &[Self]) -> Self {
        match strings.len() {
            0 => StaticJsStrings::EMPTY_STRING,
            1 => strings[0].clone(),
            2 => RopeString::create(strings[0].clone(), strings[1].clone()),
            _ => {
                let mid = strings.len() / 2;
                let left = Self::concat_strings_balanced(&strings[..mid]);
                let right = Self::concat_strings_balanced(&strings[mid..]);
                RopeString::create(left, right)
            }
        }
    }

    /// Creates a new [`JsString`] from the concatenation of every element of
    /// `strings`.
    #[inline]
    #[must_use]
    pub fn concat_array(strings: &[JsStr<'_>]) -> Self {
        let mut latin1_encoding = true;
        let mut full_count = 0usize;
        for string in strings {
            let Some(sum) = full_count.checked_add(string.len()) else {
                alloc_overflow()
            };
            if !string.is_latin1() {
                latin1_encoding = false;
            }
            full_count = sum;
        }

        let (ptr, data_offset) = if latin1_encoding {
            let p = SequenceString::<Latin1>::allocate(full_count);
            (p.cast::<u8>(), size_of::<SequenceString<Latin1>>())
        } else {
            let p = SequenceString::<Utf16>::allocate(full_count);
            (p.cast::<u8>(), size_of::<SequenceString<Utf16>>())
        };

        let string = {
            // SAFETY: `allocate_*_seq` guarantees that `ptr` is a valid pointer to a sequence string.
            let mut data = unsafe {
                let seq_ptr = ptr.as_ptr();
                seq_ptr.add(data_offset)
            };
            for &string in strings {
                // SAFETY:
                // The sum of all `count` for each `string` equals `full_count`, and since we're
                // iteratively writing each of them to `data`, `copy_non_overlapping` always stays
                // in-bounds for `count` reads of each string and `full_count` writes to `data`.
                //
                // Each `string` must be properly aligned to be a valid slice, and `data` must be
                // properly aligned by `allocate_seq`.
                //
                // `allocate_seq` must return a valid pointer to newly allocated memory, meaning
                // `ptr` and all `string`s should never overlap.
                unsafe {
                    // NOTE: The alignment is checked when we allocate the array.
                    #[allow(clippy::cast_ptr_alignment)]
                    match (latin1_encoding, string.variant()) {
                        (true, JsStrVariant::Latin1(s)) => {
                            let count = s.len();
                            ptr::copy_nonoverlapping(s.as_ptr(), data.cast::<u8>(), count);
                            data = data.cast::<u8>().add(count).cast::<u8>();
                        }
                        (false, JsStrVariant::Latin1(s)) => {
                            let count = s.len();
                            for (i, byte) in s.iter().enumerate() {
                                *data.cast::<u16>().add(i) = u16::from(*byte);
                            }
                            data = data.cast::<u16>().add(count).cast::<u8>();
                        }
                        (false, JsStrVariant::Utf16(s)) => {
                            let count = s.len();
                            ptr::copy_nonoverlapping(s.as_ptr(), data.cast::<u16>(), count);
                            data = data.cast::<u16>().add(count).cast::<u8>();
                        }
                        (true, JsStrVariant::Utf16(_)) => {
                            unreachable!("Already checked that it's latin1 encoding")
                        }
                    }
                }
            }

            Self { ptr: ptr.cast() }
        };

        StaticJsStrings::get_string(&string.as_str()).unwrap_or(string)
    }

    /// Creates a new [`JsString`] from `data`, without checking if the string is in the interner.
    fn from_slice_skip_interning(string: JsStr<'_>) -> Self {
        let count = string.len();

        // SAFETY:
        // - We read `count = data.len()` elements from `data`, which is within the bounds of the slice.
        // - `allocate_*_seq` must allocate at least `count` elements, which allows us to safely
        //   write at least `count` elements.
        // - `allocate_*_seq` should already take care of the alignment of `ptr`, and `data` must be
        //   aligned to be a valid slice.
        // - `allocate_*_seq` must return a valid pointer to newly allocated memory, meaning `ptr`
        //   and `data` should never overlap.
        unsafe {
            // NOTE: The alignment is checked when we allocate the array.
            #[allow(clippy::cast_ptr_alignment)]
            match string.variant() {
                JsStrVariant::Latin1(s) => {
                    let ptr = SequenceString::<Latin1>::allocate(count);
                    let data = (&raw mut (*ptr.as_ptr()).data)
                        .cast::<<Latin1 as r#type::StringType>::Byte>();
                    ptr::copy_nonoverlapping(s.as_ptr(), data, count);
                    Self { ptr: ptr.cast() }
                }
                JsStrVariant::Utf16(s) => {
                    let ptr = SequenceString::<Utf16>::allocate(count);
                    let data = (&raw mut (*ptr.as_ptr()).data)
                        .cast::<<Utf16 as r#type::StringType>::Byte>();
                    ptr::copy_nonoverlapping(s.as_ptr(), data, count);
                    Self { ptr: ptr.cast() }
                }
            }
        }
    }

    /// Creates a new [`JsString`] from `data`.
    fn from_js_str(string: JsStr<'_>) -> Self {
        if let Some(s) = StaticJsStrings::get_string(&string) {
            return s;
        }
        Self::from_slice_skip_interning(string)
    }

    /// Gets the number of `JsString`s which point to this allocation.
    #[inline]
    #[must_use]
    pub fn refcount(&self) -> Option<usize> {
        self.refcount_cell().map(|rc| rc.load(Ordering::Relaxed))
    }
}

impl Clone for JsString {
    #[inline]
    fn clone(&self) -> Self {
        if let Some(refcount) = self.refcount_cell() {
            refcount.fetch_add(1, Ordering::Relaxed);
        }
        Self { ptr: self.ptr }
    }
}

impl Default for JsString {
    #[inline]
    fn default() -> Self {
        StaticJsStrings::EMPTY_STRING
    }
}

impl Drop for JsString {
    #[inline]
    fn drop(&mut self) {
        if let Some(refcount) = self.refcount_cell() {
            let val = refcount.load(Ordering::Relaxed);
            if val > 1 {
                refcount.store(val - 1, Ordering::Relaxed);
                return;
            }
        }
        // SAFETY: The pointer is always valid and this is the last reference.
        let vtable = unsafe { self.ptr.as_ref().vtable };
        (vtable.dealloc)(self.ptr);
    }
}

impl std::fmt::Debug for JsString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("JsString")
            .field(&self.display_escaped().to_string())
            .finish()
    }
}

impl Eq for JsString {}

macro_rules! impl_from_number_for_js_string {
    ($($module: ident => $($ty:ty),+)+) => {
        $(
            $(
                impl From<$ty> for JsString {
                    #[inline]
                    fn from(value: $ty) -> Self {
                        JsString::from_slice_skip_interning(JsStr::latin1(
                            $module::Buffer::new().format(value).as_bytes(),
                        ))
                    }
                }
            )+
        )+
    };
}

impl_from_number_for_js_string!(
    itoa => i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, isize, usize
    ryu_js => f32, f64
);

impl From<&[u16]> for JsString {
    #[inline]
    fn from(s: &[u16]) -> Self {
        JsString::from_js_str(JsStr::utf16(s))
    }
}

impl From<&str> for JsString {
    #[inline]
    fn from(s: &str) -> Self {
        if s.is_ascii() {
            let js_str = JsStr::latin1(s.as_bytes());
            return StaticJsStrings::get_string(&js_str)
                .unwrap_or_else(|| JsString::from_slice_skip_interning(js_str));
        }
        // Non-ASCII but still Latin1-encodable (U+0080..=U+00FF): chars map 1-to-1 to u8.
        if s.chars().all(|c| c as u32 <= 0xFF) {
            let bytes: Vec<u8> = s.chars().map(|c| c as u8).collect();
            let js_str = JsStr::latin1(&bytes);
            return StaticJsStrings::get_string(&js_str)
                .unwrap_or_else(|| JsString::from_slice_skip_interning(js_str));
        }
        let s = s.encode_utf16().collect::<Vec<_>>();
        JsString::from_slice_skip_interning(JsStr::utf16(&s[..]))
    }
}

impl From<JsStr<'_>> for JsString {
    #[inline]
    fn from(value: JsStr<'_>) -> Self {
        StaticJsStrings::get_string(&value)
            .unwrap_or_else(|| JsString::from_slice_skip_interning(value))
    }
}

impl From<&JsString> for JsString {
    #[inline]
    fn from(value: &JsString) -> Self {
        value.clone()
    }
}

impl From<&[JsString]> for JsString {
    #[inline]
    fn from(value: &[JsString]) -> Self {
        Self::concat_array_strings(value)
    }
}

impl<const N: usize> From<&[JsString; N]> for JsString {
    #[inline]
    fn from(value: &[JsString; N]) -> Self {
        Self::concat_array_strings(value)
    }
}

impl From<String> for JsString {
    #[inline]
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl<'a> From<Cow<'a, str>> for JsString {
    #[inline]
    fn from(s: Cow<'a, str>) -> Self {
        match s {
            Cow::Borrowed(s) => s.into(),
            Cow::Owned(s) => s.into(),
        }
    }
}

impl<const N: usize> From<&[u16; N]> for JsString {
    #[inline]
    fn from(s: &[u16; N]) -> Self {
        Self::from(&s[..])
    }
}

impl Hash for JsString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // SAFETY: The pointer is always valid.
        let header = unsafe { self.ptr.as_ref() };
        let hash_ptr = (&raw const header.hash).cast::<AtomicU64>();
        // SAFETY: Alignment and size match. we only mutate if hash == 0.
        let mut hash = unsafe { (*hash_ptr).load(Ordering::Relaxed) };
        if hash == 0 {
            hash = self.as_str().content_hash();
            if hash == 0 {
                hash = 1;
            }
            if header.kind != JsStringKind::Static {
                // SAFETY: Not a static string.
                unsafe { (*hash_ptr).store(hash, Ordering::Relaxed) };
            }
        }
        state.write_u64(hash);
    }
}

impl PartialOrd for JsStr<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JsString {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(&other.as_str())
    }
}

impl PartialEq for JsString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.ptr == other.ptr {
            return true;
        }
        if self.len() != other.len() {
            return false;
        }

        self.as_str() == other.as_str()
    }
}

impl PartialEq<JsString> for [u16] {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
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

impl<const N: usize> PartialEq<JsString> for [u16; N] {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        self[..] == *other
    }
}

impl PartialEq<[u16]> for JsString {
    #[inline]
    fn eq(&self, other: &[u16]) -> bool {
        other == self
    }
}

impl<const N: usize> PartialEq<[u16; N]> for JsString {
    #[inline]
    fn eq(&self, other: &[u16; N]) -> bool {
        *self == other[..]
    }
}

impl PartialEq<str> for JsString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for JsString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<JsString> for str {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        other == self
    }
}

impl PartialEq<JsStr<'_>> for JsString {
    #[inline]
    fn eq(&self, other: &JsStr<'_>) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<JsString> for JsStr<'_> {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        other == self
    }
}

impl PartialOrd for JsString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for JsString {
    type Err = Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

/// Similar to [`std::ops::RangeBounds`] but custom implemented for getting direct indices.
// TODO: remove [`str::JsSliceIndex`] and rename this when `JsStr` is no more.
pub trait JsStringSliceIndex {
    /// Get the substring (or `None` if outside the string).
    fn get(self, str: &JsString) -> Option<JsString>;
}

macro_rules! impl_js_string_slice_index {
    ($($type:ty),+ $(,)?) => {
        $(
        impl JsStringSliceIndex for $type {
            fn get(self, str: &JsString) -> Option<JsString> {
                let start = match std::ops::RangeBounds::<usize>::start_bound(&self) {
                    std::ops::Bound::Included(start) => *start,
                    std::ops::Bound::Excluded(start) => *start + 1,
                    std::ops::Bound::Unbounded => 0,
                };

                let end = match std::ops::RangeBounds::<usize>::end_bound(&self) {
                    std::ops::Bound::Included(end) => *end + 1,
                    std::ops::Bound::Excluded(end) => *end,
                    std::ops::Bound::Unbounded => str.len(),
                };

                if end > str.len() || start > end {
                    None
                } else {
                    // SAFETY: we just checked the indices.
                    Some(unsafe { JsString::slice_unchecked(str, start, end) })
                }
            }
        }
        )+
    };
}

impl_js_string_slice_index!(
    std::ops::Range<usize>,
    std::ops::RangeInclusive<usize>,
    std::ops::RangeTo<usize>,
    std::ops::RangeToInclusive<usize>,
    std::ops::RangeFrom<usize>,
    std::ops::RangeFull,
);
