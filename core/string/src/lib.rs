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
// Remove when/if https://github.com/rust-lang/rust/issues/95228 stabilizes.
// Right now this allows us to use the stable polyfill from the `sptr` crate, which uses
// the same names from the unstable functions of the `std::ptr` module.
#![allow(unstable_name_collisions)]
#![allow(clippy::module_name_repetitions)]

mod builder;
mod common;
mod display;
mod iter;
mod str;
mod tagged;

#[cfg(test)]
mod tests;

use self::{iter::Windows, str::JsSliceIndex};
use crate::display::{JsStrDisplayEscaped, JsStrDisplayLossy};
use crate::tagged::{Tagged, UnwrappedTagged};
#[doc(inline)]
pub use crate::{
    builder::{CommonJsStringBuilder, Latin1JsStringBuilder, Utf16JsStringBuilder},
    common::StaticJsStrings,
    iter::Iter,
    str::{JsStr, JsStrVariant},
};
use std::fmt::Write;
use std::{
    alloc::{alloc, dealloc, Layout},
    cell::Cell,
    convert::Infallible,
    hash::{Hash, Hasher},
    iter::Peekable,
    mem::ManuallyDrop,
    process::abort,
    ptr::{self, addr_of, addr_of_mut, NonNull},
    str::FromStr,
};

fn alloc_overflow() -> ! {
    panic!("detected overflow during string allocation")
}

/// Helper function to check if a `char` is trimmable.
pub(crate) const fn is_trimmable_whitespace(c: char) -> bool {
    // The rust implementation of `trim` does not regard the same characters whitespace as ecma standard does
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
    // The rust implementation of `trim` does not regard the same characters whitespace as ecma standard does
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

/// Represents a Unicode codepoint within a [`JsString`], which could be a valid
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodePoint::Unicode(c) => f.write_char(*c),
            CodePoint::UnpairedSurrogate(c) => {
                write!(f, "\\u{c:04X}")
            }
        }
    }
}

/// A `usize` contains a flag and the length of Latin1/UTF-16 .
/// ```text
/// ┌────────────────────────────────────┐
/// │ length (usize::BITS - 1) │ flag(1) │
/// └────────────────────────────────────┘
/// ```
/// The latin1/UTF-16 flag is stored in the bottom bit.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
struct TaggedLen(usize);

impl TaggedLen {
    const LATIN1_BITFLAG: usize = 1 << 0;
    const BITFLAG_COUNT: usize = 1;

    const fn new(len: usize, latin1: bool) -> Self {
        Self((len << Self::BITFLAG_COUNT) | (latin1 as usize))
    }

    const fn is_latin1(self) -> bool {
        (self.0 & Self::LATIN1_BITFLAG) != 0
    }

    const fn len(self) -> usize {
        self.0 >> Self::BITFLAG_COUNT
    }
}

/// The raw representation of a [`JsString`] from a string literal.
#[derive(Debug)]
#[repr(C)]
pub struct StaticJsString {
    tagged_len: TaggedLen,
    _zero: usize,
    ptr: *const u8,
}

// SAFETY: This is Sync because reads to `_zero` will always read 0 and
// `ptr` cannot be mutated thanks to the 'static requirement.
unsafe impl Sync for StaticJsString {}

impl StaticJsString {
    /// Create a `StaticJsString` from a static `JsStr`.
    #[must_use]
    pub const fn new(string: JsStr<'static>) -> StaticJsString {
        match string.variant() {
            JsStrVariant::Latin1(l) => StaticJsString {
                tagged_len: TaggedLen::new(l.len(), true),
                _zero: 0,
                ptr: l.as_ptr(),
            },
            JsStrVariant::Utf16(u) => StaticJsString {
                tagged_len: TaggedLen::new(u.len(), false),
                _zero: 0,
                ptr: u.as_ptr().cast(),
            },
        }
    }
}

/// Memory variant to pass `Miri` test.
///
/// If it equals to `0usize`,
/// we mark it read-only, otherwise it is readable and writable
union RefCount {
    read_only: usize,
    read_write: ManuallyDrop<Cell<usize>>,
}

/// The raw representation of a [`JsString`] in the heap.
#[repr(C)]
struct RawJsString {
    tagged_len: TaggedLen,
    refcount: RefCount,
    data: [u8; 0],
}

impl RawJsString {
    const fn is_latin1(&self) -> bool {
        self.tagged_len.is_latin1()
    }

    const fn len(&self) -> usize {
        self.tagged_len.len()
    }
}

const DATA_OFFSET: usize = size_of::<RawJsString>();

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
#[allow(clippy::module_name_repetitions)]
pub struct JsString {
    ptr: Tagged<RawJsString>,
}

// JsString should always be pointer sized.
static_assertions::assert_eq_size!(JsString, *const ());

impl<'a> From<&'a JsString> for JsStr<'a> {
    #[inline]
    fn from(value: &'a JsString) -> Self {
        value.as_str()
    }
}

impl<'a> IntoIterator for &'a JsString {
    type IntoIter = Iter<'a>;
    type Item = u16;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl JsString {
    /// Create a [`JsString`] from a static js string.
    #[must_use]
    pub const fn from_static_js_string(src: &'static StaticJsString) -> Self {
        let src = ptr::from_ref(src).cast::<RawJsString>();
        JsString {
            // SAFETY:
            // `StaticJsString` has the same memory layout as `RawJsString` for the first 2 fields
            // which means it is safe to use it to represent `RawJsString` as long as we only acccess the first 2 fields,
            // and the static reference indicates that the pointer cast is valid.
            ptr: unsafe { Tagged::from_ptr(src.cast_mut()) },
        }
    }

    /// Create an iterator over the [`JsString`].
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self.as_str())
    }

    /// Create an iterator over overlapping subslices of length size.
    #[inline]
    #[must_use]
    pub fn windows(&self, size: usize) -> Windows<'_> {
        Windows::new(self.as_str(), size)
    }

    /// Obtains the underlying [`&[u16]`][slice] slice of a [`JsString`]
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> JsStr<'_> {
        match self.ptr.unwrap() {
            UnwrappedTagged::Ptr(h) => {
                // SAFETY:
                // - The `RawJsString` type has all the necessary information to reconstruct a valid
                //   slice (length and starting pointer).
                //
                // - We aligned `h.data()` on allocation, and the block is of size `h.len`, so this
                //   should only generate valid reads.
                //
                // - The lifetime of `&Self::Target` is shorter than the lifetime of `self`, as seen
                //   by its signature, so this doesn't outlive `self`.
                //
                // - The `RawJsString` created from string literal has a static reference to the string literal,
                //   making it safe to be dereferenced and used as a static `JsStr`.
                //
                // - `Cell<usize>` is readable as an usize as long as we don't try to mutate the pointed variable,
                //   which means it is safe to read the `refcount` as `read_only` here.
                unsafe {
                    let h = h.as_ptr();
                    if (*h).refcount.read_only == 0 {
                        let h = h.cast::<StaticJsString>();
                        return if (*h).tagged_len.is_latin1() {
                            JsStr::latin1(std::slice::from_raw_parts(
                                (*h).ptr,
                                (*h).tagged_len.len(),
                            ))
                        } else {
                            JsStr::utf16(std::slice::from_raw_parts(
                                (*h).ptr.cast(),
                                (*h).tagged_len.len(),
                            ))
                        };
                    }

                    let len = (*h).len();
                    if (*h).is_latin1() {
                        JsStr::latin1(std::slice::from_raw_parts(addr_of!((*h).data).cast(), len))
                    } else {
                        JsStr::utf16(std::slice::from_raw_parts(addr_of!((*h).data).cast(), len))
                    }
                }
            }
            UnwrappedTagged::Tag(index) => {
                // SAFETY: all static strings are valid indices on `STATIC_JS_STRINGS`, so `get` should always
                // return `Some`.
                unsafe { StaticJsStrings::get(index).unwrap_unchecked() }
            }
        }
    }

    /// Creates a new [`JsString`] from the concatenation of `x` and `y`.
    #[inline]
    #[must_use]
    pub fn concat(x: JsStr<'_>, y: JsStr<'_>) -> Self {
        Self::concat_array(&[x, y])
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

        let ptr = Self::allocate_inner(full_count, latin1_encoding);

        let string = {
            // SAFETY: `allocate_inner` guarantees that `ptr` is a valid pointer.
            let mut data = unsafe { addr_of_mut!((*ptr.as_ptr()).data).cast::<u8>() };
            for &string in strings {
                // SAFETY:
                // The sum of all `count` for each `string` equals `full_count`, and since we're
                // iteratively writing each of them to `data`, `copy_non_overlapping` always stays
                // in-bounds for `count` reads of each string and `full_count` writes to `data`.
                //
                // Each `string` must be properly aligned to be a valid slice, and `data` must be
                // properly aligned by `allocate_inner`.
                //
                // `allocate_inner` must return a valid pointer to newly allocated memory, meaning
                // `ptr` and all `string`s should never overlap.
                unsafe {
                    // NOTE: The aligment is checked when we allocate the array.
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
            Self {
                // SAFETY: We already know it's a valid heap pointer.
                ptr: unsafe { Tagged::from_ptr(ptr.as_ptr()) },
            }
        };

        StaticJsStrings::get_string(&string.as_str()).unwrap_or(string)
    }

    /// Decodes a [`JsString`] into a [`String`], replacing invalid data with its escaped representation
    /// in 4 digit hexadecimal.
    #[inline]
    #[must_use]
    pub fn to_std_string_escaped(&self) -> String {
        self.to_string_escaped()
    }

    /// Decodes a [`JsString`] into a [`String`], replacing invalid data with the
    /// replacement character U+FFFD.
    #[inline]
    #[must_use]
    pub fn to_std_string_lossy(&self) -> String {
        self.code_points()
            .map(|cp| match cp {
                CodePoint::Unicode(c) => c,
                CodePoint::UnpairedSurrogate(_) => '\u{FFFD}',
            })
            .collect()
    }

    /// Decodes a [`JsString`] into a [`String`], returning
    ///
    /// # Errors
    ///
    /// [`FromUtf16Error`][std::string::FromUtf16Error] if it contains any invalid data.
    #[inline]
    pub fn to_std_string(&self) -> Result<String, std::string::FromUtf16Error> {
        match self.as_str().variant() {
            JsStrVariant::Latin1(v) => Ok(v.iter().copied().map(char::from).collect()),
            JsStrVariant::Utf16(v) => String::from_utf16(v),
        }
    }

    /// Decodes a [`JsString`] into an iterator of [`Result<String, u16>`], returning surrogates as
    /// errors.
    #[inline]
    pub fn to_std_string_with_surrogates(&self) -> impl Iterator<Item = Result<String, u16>> + '_ {
        struct WideStringDecoderIterator<I: Iterator> {
            codepoints: Peekable<I>,
        }

        impl<I: Iterator> WideStringDecoderIterator<I> {
            fn new(iterator: I) -> Self {
                Self {
                    codepoints: iterator.peekable(),
                }
            }
        }

        impl<I> Iterator for WideStringDecoderIterator<I>
        where
            I: Iterator<Item = CodePoint>,
        {
            type Item = Result<String, u16>;

            fn next(&mut self) -> Option<Self::Item> {
                let cp = self.codepoints.next()?;
                let char = match cp {
                    CodePoint::Unicode(c) => c,
                    CodePoint::UnpairedSurrogate(surr) => return Some(Err(surr)),
                };

                let mut string = String::from(char);

                loop {
                    let Some(cp) = self.codepoints.peek().and_then(|cp| match cp {
                        CodePoint::Unicode(c) => Some(*c),
                        CodePoint::UnpairedSurrogate(_) => None,
                    }) else {
                        break;
                    };

                    string.push(cp);

                    self.codepoints
                        .next()
                        .expect("should exist by the check above");
                }

                Some(Ok(string))
            }
        }

        WideStringDecoderIterator::new(self.code_points())
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
    pub fn code_points(&self) -> impl Iterator<Item = CodePoint> + Clone + '_ {
        self.as_str().code_points()
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

        match self.as_str().variant() {
            JsStrVariant::Latin1(v) => {
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
            JsStrVariant::Utf16(v) => {
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

    /// Allocates a new [`RawJsString`] with an internal capacity of `str_len` chars.
    ///
    /// # Panics
    ///
    /// Panics if `try_allocate_inner` returns `Err`.
    fn allocate_inner(str_len: usize, latin1: bool) -> NonNull<RawJsString> {
        match Self::try_allocate_inner(str_len, latin1) {
            Ok(v) => v,
            Err(None) => alloc_overflow(),
            Err(Some(layout)) => std::alloc::handle_alloc_error(layout),
        }
    }

    // This is marked as safe because it is always valid to call this function to request any number
    // of `u16`, since this function ought to fail on an OOM error.
    /// Allocates a new [`RawJsString`] with an internal capacity of `str_len` chars.
    ///
    /// # Errors
    ///
    /// Returns `Err(None)` on integer overflows `usize::MAX`.
    /// Returns `Err(Some(Layout))` on allocation error.
    fn try_allocate_inner(
        str_len: usize,
        latin1: bool,
    ) -> Result<NonNull<RawJsString>, Option<Layout>> {
        let (layout, offset) = if latin1 {
            Layout::array::<u8>(str_len)
        } else {
            Layout::array::<u16>(str_len)
        }
        .and_then(|arr| Layout::new::<RawJsString>().extend(arr))
        .map(|(layout, offset)| (layout.pad_to_align(), offset))
        .map_err(|_| None)?;

        debug_assert_eq!(offset, DATA_OFFSET);

        #[allow(clippy::cast_ptr_alignment)]
        // SAFETY:
        // The layout size of `RawJsString` is never zero, since it has to store
        // the length of the string and the reference count.
        let inner = unsafe { alloc(layout).cast::<RawJsString>() };

        // We need to verify that the pointer returned by `alloc` is not null, otherwise
        // we should abort, since an allocation error is pretty unrecoverable for us
        // right now.
        let inner = NonNull::new(inner).ok_or(Some(layout))?;

        // SAFETY:
        // `NonNull` verified for us that the pointer returned by `alloc` is valid,
        // meaning we can write to its pointed memory.
        unsafe {
            // Write the first part, the `RawJsString`.
            inner.as_ptr().write(RawJsString {
                tagged_len: TaggedLen::new(str_len, latin1),
                refcount: RefCount {
                    read_write: ManuallyDrop::new(Cell::new(1)),
                },
                data: [0; 0],
            });
        }

        debug_assert!({
            let inner = inner.as_ptr();
            // SAFETY:
            // - `inner` must be a valid pointer, since it comes from a `NonNull`,
            // meaning we can safely dereference it to `RawJsString`.
            // - `offset` should point us to the beginning of the array,
            // and since we requested an `RawJsString` layout with a trailing
            // `[u16; str_len]`, the memory of the array must be in the `usize`
            // range for the allocation to succeed.
            unsafe {
                ptr::eq(
                    inner.cast::<u8>().add(offset).cast(),
                    (*inner).data.as_mut_ptr(),
                )
            }
        });

        Ok(inner)
    }

    /// Creates a new [`JsString`] from `data`, without checking if the string is in the interner.
    fn from_slice_skip_interning(string: JsStr<'_>) -> Self {
        let count = string.len();
        let ptr = Self::allocate_inner(count, string.is_latin1());

        // SAFETY: `allocate_inner` guarantees that `ptr` is a valid pointer.
        let data = unsafe { addr_of_mut!((*ptr.as_ptr()).data).cast::<u8>() };

        // SAFETY:
        // - We read `count = data.len()` elements from `data`, which is within the bounds of the slice.
        // - `allocate_inner` must allocate at least `count` elements, which allows us to safely
        //   write at least `count` elements.
        // - `allocate_inner` should already take care of the alignment of `ptr`, and `data` must be
        //   aligned to be a valid slice.
        // - `allocate_inner` must return a valid pointer to newly allocated memory, meaning `ptr`
        //   and `data` should never overlap.
        unsafe {
            // NOTE: The aligment is checked when we allocate the array.
            #[allow(clippy::cast_ptr_alignment)]
            match string.variant() {
                JsStrVariant::Latin1(s) => {
                    ptr::copy_nonoverlapping(s.as_ptr(), data.cast::<u8>(), count);
                }
                JsStrVariant::Utf16(s) => {
                    ptr::copy_nonoverlapping(s.as_ptr(), data.cast::<u16>(), count);
                }
            }
        }
        Self {
            // SAFETY: `allocate_inner` guarantees `ptr` is a valid heap pointer.
            ptr: Tagged::from_non_null(ptr),
        }
    }

    /// Creates a new [`JsString`] from `data`.
    fn from_slice(string: JsStr<'_>) -> Self {
        if let Some(s) = StaticJsStrings::get_string(&string) {
            return s;
        }
        Self::from_slice_skip_interning(string)
    }

    /// Get the length of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    /// Return true if the [`JsString`] is emtpy.
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
        match self.as_str().variant() {
            JsStrVariant::Latin1(v) => v.contains(&element),
            JsStrVariant::Utf16(v) => v.contains(&u16::from(element)),
        }
    }

    /// Trim whitespace from the start and end of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn trim(&self) -> JsStr<'_> {
        self.as_str().trim()
    }

    /// Trim whitespace from the start of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn trim_start(&self) -> JsStr<'_> {
        self.as_str().trim_start()
    }

    /// Trim whitespace from the end of the [`JsString`].
    #[inline]
    #[must_use]
    pub fn trim_end(&self) -> JsStr<'_> {
        self.as_str().trim_end()
    }

    /// Check if the [`JsString`] is static.
    #[inline]
    #[must_use]
    pub fn is_static(&self) -> bool {
        self.refcount().is_none()
    }

    /// Get the element a the given index, [`None`] otherwise.
    #[inline]
    #[must_use]
    pub fn get<'a, I>(&'a self, index: I) -> Option<I::Value>
    where
        I: JsSliceIndex<'a>,
    {
        I::get(self.as_str(), index)
    }

    /// Returns an element or subslice depending on the type of index, without doing bounds check.
    ///
    /// # Safety
    ///
    /// Caller must ensure the index is not out of bounds
    #[inline]
    #[must_use]
    pub unsafe fn get_unchecked<'a, I>(&'a self, index: I) -> I::Value
    where
        I: JsSliceIndex<'a>,
    {
        // SAFETY: Caller must ensure the index is not out of bounds
        unsafe { I::get_unchecked(self.as_str(), index) }
    }

    /// Get the element a the given index.
    ///
    /// # Panics
    ///
    /// If the index is out of bounds.
    #[inline]
    #[must_use]
    pub fn get_expect<'a, I>(&'a self, index: I) -> I::Value
    where
        I: JsSliceIndex<'a>,
    {
        self.get(index).expect("Index out of bounds")
    }

    /// Gets the number of `JsString`s which point to this allocation.
    #[inline]
    #[must_use]
    pub fn refcount(&self) -> Option<usize> {
        match self.ptr.unwrap() {
            UnwrappedTagged::Ptr(inner) => {
                // SAFETY:
                // `NonNull` and the constructions of `JsString` guarantee that `inner` is always valid.
                // And `Cell<usize>` is readable as an usize as long as we don't try to mutate the pointed variable,
                // which means it is safe to read the `refcount` as `read_only` here.
                let rc = unsafe { (*inner.as_ptr()).refcount.read_only };
                if rc == 0 {
                    None
                } else {
                    Some(rc)
                }
            }
            UnwrappedTagged::Tag(_inner) => None,
        }
    }

    /// Gets a displayable escaped string. This may be faster and has fewer
    /// allocations than `format!("{}", str.to_string_escaped())` when
    /// displaying.
    #[inline]
    #[must_use]
    pub fn display_escaped(&self) -> JsStrDisplayEscaped<'_> {
        JsStrDisplayEscaped::from(self.as_str())
    }

    /// Gets a displayable lossy string. This may be faster and has fewer
    /// allocations than `format!("{}", str.to_string_lossy())` when displaying.
    #[inline]
    #[must_use]
    pub fn display_lossy(&self) -> JsStrDisplayLossy<'_> {
        JsStrDisplayLossy::from(self.as_str())
    }
}

impl Clone for JsString {
    #[inline]
    fn clone(&self) -> Self {
        if let UnwrappedTagged::Ptr(inner) = self.ptr.unwrap() {
            // SAFETY:
            // `NonNull` and the constructions of `JsString` guarantee that `inner` is always valid.
            // And `Cell<usize>` is readable as an usize as long as we don't try to mutate the pointed variable,
            // which means it is safe to read the `refcount` as `read_only` here.
            let rc = unsafe { (*inner.as_ptr()).refcount.read_only };
            if rc == 0 {
                // pointee is a static string
                return Self { ptr: self.ptr };
            }
            // SAFETY: `NonNull` and the constructions of `JsString` guarantee that `inner` is always valid.
            let inner = unsafe { inner.as_ref() };

            let strong = rc.wrapping_add(1);
            if strong == 0 {
                abort()
            }
            // SAFETY:
            // This has been checked aboved to ensure it is a `read_write` variant,
            // which means it is safe to write the `refcount` as `read_write` here.
            unsafe {
                inner.refcount.read_write.set(strong);
            }
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
        if let UnwrappedTagged::Ptr(raw) = self.ptr.unwrap() {
            // See https://doc.rust-lang.org/src/alloc/sync.rs.html#1672 for details.

            // SAFETY:
            // `NonNull` and the constructions of `JsString` guarantees that `raw` is always valid.
            // And `Cell<usize>` is readable as an usize as long as we don't try to mutate the pointed variable,
            // which means it is safe to read the `refcount` as `read_only` here.
            let refcount = unsafe { (*raw.as_ptr()).refcount.read_only };
            if refcount == 0 {
                // Just a static string. No need to drop.
                return;
            }

            // SAFETY: `NonNull` and the constructions of `JsString` guarantees that `raw` is always valid.
            let inner = unsafe { raw.as_ref() };

            // SAFETY:
            // This has been checked aboved to ensure it is a `read_write` variant,
            // which means it is safe to write the `refcount` as `read_write` here.
            unsafe {
                inner.refcount.read_write.set(refcount - 1);
                if inner.refcount.read_write.get() != 0 {
                    return;
                }
            }

            // SAFETY:
            // All the checks for the validity of the layout have already been made on `alloc_inner`,
            // so we can skip the unwrap.
            let layout = unsafe {
                if inner.is_latin1() {
                    Layout::for_value(inner)
                        .extend(Layout::array::<u8>(inner.len()).unwrap_unchecked())
                        .unwrap_unchecked()
                        .0
                        .pad_to_align()
                } else {
                    Layout::for_value(inner)
                        .extend(Layout::array::<u16>(inner.len()).unwrap_unchecked())
                        .unwrap_unchecked()
                        .0
                        .pad_to_align()
                }
            };

            // SAFETY:
            // If refcount is 0 and we call drop, that means this is the last `JsString` which
            // points to this memory allocation, so deallocating it is safe.
            unsafe {
                dealloc(raw.as_ptr().cast(), layout);
            }
        }
    }
}

impl ToStringEscaped for JsString {
    #[inline]
    fn to_string_escaped(&self) -> String {
        format!("{}", self.display_escaped())
    }
}

impl std::fmt::Debug for JsString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_std_string_escaped().fmt(f)
    }
}

impl Eq for JsString {}

impl From<&[u16]> for JsString {
    #[inline]
    fn from(s: &[u16]) -> Self {
        JsString::from_slice(JsStr::utf16(s))
    }
}

impl From<&str> for JsString {
    #[inline]
    fn from(s: &str) -> Self {
        // TODO: Check for latin1 encoding
        if s.is_ascii() {
            let js_str = JsStr::latin1(s.as_bytes());
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

impl From<&[JsString]> for JsString {
    #[inline]
    fn from(value: &[JsString]) -> Self {
        Self::concat_array(
            &value
                .iter()
                .map(Self::as_str)
                .map(Into::into)
                .collect::<Vec<_>>()[..],
        )
    }
}
impl From<String> for JsString {
    #[inline]
    fn from(s: String) -> Self {
        Self::from(s.as_str())
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
        self.as_str().hash(state);
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

/// Utility trait that adds a `UTF-16` escaped representation to every [`[u16]`][slice].
pub(crate) trait ToStringEscaped {
    /// Decodes `self` as an `UTF-16` encoded string, escaping any unpaired surrogates by its
    /// codepoint value.
    fn to_string_escaped(&self) -> String;
}

impl ToStringEscaped for [u16] {
    #[inline]
    fn to_string_escaped(&self) -> String {
        JsString::from(self).to_string_escaped()
    }
}
