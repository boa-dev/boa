//! A Latin1 or UTF-16 encoded, reference counted, immutable string.
//!
//! This module contains the [`JsString`] type, the [`js_string`][crate::js_string] macro and the
//! [`js_str`][crate::js_str] macro.
//!
//! The [`js_string`][crate::js_string] macro is used when you need to create a new [`JsString`],
//! and the [`js_str`][crate::js_str] macro is used for const conversions of string literals to [`JsStr`].

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

pub mod common;
mod iter;
mod str;

use self::{common::StaticJsStrings, iter::Windows, str::JsSliceIndex};
#[doc(inline)]
pub use crate::string::{
    iter::Iter,
    str::{JsStr, JsStrVariant},
};
use crate::{
    builtins::string::is_trimmable_whitespace,
    tagged::{Tagged, UnwrappedTagged},
};
use boa_gc::{Finalize, Trace};
use std::{
    alloc::{alloc, dealloc, Layout},
    cell::Cell,
    convert::Infallible,
    hash::{Hash, Hasher},
    iter::Peekable,
    process::abort,
    ptr::{self, addr_of, addr_of_mut, NonNull},
    str::FromStr,
};

fn alloc_overflow() -> ! {
    panic!("detected overflow during string allocation")
}

/// Utility macro to create a [`JsString`].
///
/// # Examples
///
/// You can call the macro without arguments to create an empty `JsString`:
///
/// ```
/// use boa_engine::js_string;
///
/// let empty_str = js_string!();
/// assert!(empty_str.is_empty());
/// ```
///
///
/// You can create a `JsString` from a string literal, which completely skips the runtime
/// conversion from [`&str`] to <code>[&\[u16\]][slice]</code>:
///
/// ```
/// # use boa_engine::js_string;
/// let hw = js_string!("Hello, world!");
/// assert_eq!(&hw, "Hello, world!");
/// ```
///
/// Any `&[u16]` slice is a valid `JsString`, including unpaired surrogates:
///
/// ```
/// # use boa_engine::js_string;
/// let array = js_string!(&[0xD8AFu16, 0x00A0, 0xD8FF, 0x00F0]);
/// ```
///
/// You can also pass it any number of `&[u16]` as arguments to create a new `JsString` with
/// the concatenation of every slice:
///
/// ```
/// # use boa_engine::{js_string, js_str, JsStr};
/// const NAME: JsStr<'_> = js_str!("human! ");
/// let greeting = js_string!("Hello, ");
/// let msg = js_string!(&greeting, NAME, js_str!("Nice to meet you!"));
///
/// assert_eq!(&msg, "Hello, human! Nice to meet you!");
/// ```
#[macro_export]
#[allow(clippy::module_name_repetitions)]
macro_rules! js_string {
    () => {
        $crate::string::JsString::default()
    };
    ($s:literal) => {
        $crate::string::JsString::from($crate::js_str!($s))
    };
    ($s:expr) => {
        $crate::string::JsString::from($s)
    };
    ( $x:expr, $y:expr ) => {
        $crate::string::JsString::concat($crate::string::JsStr::from($x), $crate::string::JsStr::from($y))
    };
    ( $( $s:expr ),+ ) => {
        $crate::string::JsString::concat_array(&[ $( $crate::string::JsStr::from($s) ),+ ])
    };
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
    #[must_use]
    pub const fn code_unit_count(self) -> usize {
        match self {
            Self::Unicode(c) => c.len_utf16(),
            Self::UnpairedSurrogate(_) => 1,
        }
    }

    /// Convert the code point to its [`u32`] representation.
    #[must_use]
    pub fn as_u32(self) -> u32 {
        match self {
            Self::Unicode(c) => u32::from(c),
            Self::UnpairedSurrogate(surr) => u32::from(surr),
        }
    }

    /// If the code point represents a valid 'Unicode scalar value', returns its [`char`]
    /// representation, otherwise returns [`None`] on unpaired surrogates.
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

/// The raw representation of a [`JsString`] in the heap.
#[repr(C)]
struct RawJsString {
    /// Contains the flags and Latin1/UTF-16 length.
    ///
    /// The latin1 flag is stored in the bottom bit.
    flags_and_len: usize,

    /// The number of references to the string.
    ///
    /// When this reaches `0` the string is deallocated.
    refcount: Cell<usize>,

    /// An empty array which is used to get the offset of string data.
    data: [u16; 0],
}

impl RawJsString {
    const LATIN1_BITFLAG: usize = 1 << 0;
    const BITFLAG_COUNT: usize = 1;

    const fn is_latin1(&self) -> bool {
        (self.flags_and_len & Self::LATIN1_BITFLAG) != 0
    }

    const fn len(&self) -> usize {
        self.flags_and_len >> Self::BITFLAG_COUNT
    }

    const fn encode_flags_and_len(len: usize, latin1: bool) -> usize {
        (len << Self::BITFLAG_COUNT) | (latin1 as usize)
    }
}

const DATA_OFFSET: usize = std::mem::size_of::<RawJsString>();

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
#[derive(Trace, Finalize)]
// Safety: `JsString` does not contain any objects which needs to be traced, so this is safe.
#[boa_gc(unsafe_empty_trace)]
#[allow(clippy::module_name_repetitions)]
pub struct JsString {
    ptr: Tagged<RawJsString>,
}

// JsString should always be pointer sized.
static_assertions::assert_eq_size!(JsString, *const ());

impl<'a> From<&'a JsString> for JsStr<'a> {
    fn from(value: &'a JsString) -> Self {
        value.as_str()
    }
}

impl<'a> IntoIterator for &'a JsString {
    type IntoIter = Iter<'a>;

    type Item = u16;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl JsString {
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
    #[must_use]
    pub fn as_str(&self) -> JsStr<'_> {
        match self.ptr.unwrap() {
            UnwrappedTagged::Ptr(h) => {
                // SAFETY:
                // - The `RawJsString` type has all the necessary information to reconstruct a valid
                //   slice (length and starting pointer).
                //
                // - We aligned `h.data` on allocation, and the block is of size `h.len`, so this
                //   should only generate valid reads.
                //
                // - The lifetime of `&Self::Target` is shorter than the lifetime of `self`, as seen
                //   by its signature, so this doesn't outlive `self`.
                unsafe {
                    let h = h.as_ptr();

                    if (*h).is_latin1() {
                        JsStr::latin1(std::slice::from_raw_parts(
                            addr_of!((*h).data).cast(),
                            (*h).len(),
                        ))
                    } else {
                        JsStr::utf16(std::slice::from_raw_parts(
                            addr_of!((*h).data).cast(),
                            (*h).len(),
                        ))
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
    #[must_use]
    pub fn concat(x: JsStr<'_>, y: JsStr<'_>) -> Self {
        Self::concat_array(&[x, y])
    }

    /// Creates a new [`JsString`] from the concatenation of every element of
    /// `strings`.
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
                // Safety: We already know it's a valid heap pointer.
                ptr: unsafe { Tagged::from_ptr(ptr.as_ptr()) },
            }
        };

        StaticJsStrings::get_string(&string.as_str()).unwrap_or(string)
    }

    /// Decodes a [`JsString`] into a [`String`], replacing invalid data with its escaped representation
    /// in 4 digit hexadecimal.
    #[must_use]
    pub fn to_std_string_escaped(&self) -> String {
        self.to_string_escaped()
    }

    /// Decodes a [`JsString`] into a [`String`], returning
    ///
    /// # Errors
    ///
    /// [`FromUtf16Error`][std::string::FromUtf16Error] if it contains any invalid data.
    pub fn to_std_string(&self) -> Result<String, std::string::FromUtf16Error> {
        match self.as_str().variant() {
            JsStrVariant::Latin1(v) => Ok(v.iter().copied().map(char::from).collect()),
            JsStrVariant::Utf16(v) => String::from_utf16(v),
        }
    }

    /// Decodes a [`JsString`] into an iterator of [`Result<String, u16>`], returning surrogates as
    /// errors.
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

        js_string!(&text[..])
    }

    /// Gets an iterator of all the Unicode codepoints of a [`JsString`].
    pub fn code_points(&self) -> impl Iterator<Item = CodePoint> + Clone + '_ {
        char::decode_utf16(self.iter()).map(|res| match res {
            Ok(c) => CodePoint::Unicode(c),
            Err(e) => CodePoint::UnpairedSurrogate(e.unpaired_surrogate()),
        })
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

        fast_float::parse(string).unwrap_or(f64::NAN)
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
                flags_and_len: RawJsString::encode_flags_and_len(str_len, latin1),
                refcount: Cell::new(1),
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
            // Safety: `allocate_inner` guarantees `ptr` is a valid heap pointer.
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
        match self.ptr.unwrap() {
            UnwrappedTagged::Ptr(h) => {
                // SAFETY:
                // - The `RawJsString` type has all the necessary information to reconstruct a valid
                //   slice (length and starting pointer).
                //
                // - We aligned `h.data` on allocation, and the block is of size `h.len`, so this
                //   should only generate valid reads.
                //
                // - The lifetime of `&Self::Target` is shorter than the lifetime of `self`, as seen
                //   by its signature, so this doesn't outlive `self`.
                unsafe {
                    let h = h.as_ptr();
                    (*h).len()
                }
            }
            UnwrappedTagged::Tag(index) => {
                // SAFETY: all static strings are valid indices on `STATIC_JS_STRINGS`, so `get` should always
                // return `Some`.
                unsafe { StaticJsStrings::get(index).unwrap_unchecked().len() }
            }
        }
    }

    /// Return true if the [`JsString`] is emtpy.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert the [`JsString`] into a [`Vec<U16>`].
    #[must_use]
    pub fn to_vec(&self) -> Vec<u16> {
        self.as_str().to_vec()
    }

    /// Check if the [`JsString`] contains a byte.
    #[must_use]
    pub fn contains(&self, element: u8) -> bool {
        match self.as_str().variant() {
            JsStrVariant::Latin1(v) => v.contains(&element),
            JsStrVariant::Utf16(v) => v.contains(&u16::from(element)),
        }
    }

    /// Trim whitespace from the start and end of the [`JsString`].
    #[must_use]
    pub fn trim(&self) -> JsStr<'_> {
        self.as_str().trim()
    }

    /// Trim whitespace from the start of the [`JsString`].
    #[must_use]
    pub fn trim_start(&self) -> JsStr<'_> {
        self.as_str().trim_start()
    }

    /// Trim whitespace from the end of the [`JsString`].
    #[must_use]
    pub fn trim_end(&self) -> JsStr<'_> {
        self.as_str().trim_end()
    }

    /// Check if the [`JsString`] is static.
    #[must_use]
    pub fn is_static(&self) -> bool {
        self.ptr.is_tagged()
    }

    /// Get the element a the given index, [`None`] otherwise.
    #[must_use]
    pub fn get<'a, I>(&'a self, index: I) -> Option<I::Value>
    where
        I: JsSliceIndex<'a>,
    {
        I::get(self.as_str(), index)
    }

    /// Get the element a the given index.
    ///
    /// # Panics
    ///
    /// If the index is out of bounds.
    #[must_use]
    pub fn get_expect<'a, I>(&'a self, index: I) -> I::Value
    where
        I: JsSliceIndex<'a>,
    {
        self.get(index).expect("Index out of bounds")
    }
}

impl Clone for JsString {
    #[inline]
    fn clone(&self) -> Self {
        if let UnwrappedTagged::Ptr(inner) = self.ptr.unwrap() {
            // SAFETY: The reference count of `JsString` guarantees that `raw` is always valid.
            let inner = unsafe { inner.as_ref() };
            let strong = inner.refcount.get().wrapping_add(1);
            if strong == 0 {
                abort()
            }
            inner.refcount.set(strong);
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
    fn drop(&mut self) {
        if let UnwrappedTagged::Ptr(raw) = self.ptr.unwrap() {
            // See https://doc.rust-lang.org/src/alloc/sync.rs.html#1672 for details.

            // SAFETY: The reference count of `JsString` guarantees that `raw` is always valid.
            let inner = unsafe { raw.as_ref() };
            inner.refcount.set(inner.refcount.get() - 1);
            if inner.refcount.get() != 0 {
                return;
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

            // Safety:
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
        match self.as_str().variant() {
            JsStrVariant::Latin1(v) => v.iter().copied().map(char::from).collect(),
            JsStrVariant::Utf16(v) => v.to_string_escaped(),
        }
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
    fn from(value: JsStr<'_>) -> Self {
        StaticJsStrings::get_string(&value)
            .unwrap_or_else(|| JsString::from_slice_skip_interning(value))
    }
}

impl From<&[JsString]> for JsString {
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
    fn from(s: &[u16; N]) -> Self {
        Self::from(&s[..])
    }
}

impl Hash for JsString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for JsStr<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JsString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(&other.as_str())
    }
}

impl PartialEq for JsString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<JsString> for [u16] {
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
    fn eq(&self, other: &JsString) -> bool {
        self[..] == *other
    }
}

impl PartialEq<[u16]> for JsString {
    fn eq(&self, other: &[u16]) -> bool {
        other == self
    }
}

impl<const N: usize> PartialEq<[u16; N]> for JsString {
    fn eq(&self, other: &[u16; N]) -> bool {
        *self == other[..]
    }
}

impl PartialEq<str> for JsString {
    fn eq(&self, other: &str) -> bool {
        let utf16 = self.code_points();
        let mut utf8 = other.chars();

        for lhs in utf16 {
            if let Some(rhs) = utf8.next() {
                match lhs {
                    CodePoint::Unicode(lhs) if lhs == rhs => continue,
                    _ => return false,
                }
            }
            return false;
        }
        utf8.next().is_none()
    }
}

impl PartialEq<JsString> for str {
    fn eq(&self, other: &JsString) -> bool {
        other == self
    }
}

impl PartialEq<JsStr<'_>> for JsString {
    fn eq(&self, other: &JsStr<'_>) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<JsString> for JsStr<'_> {
    fn eq(&self, other: &JsString) -> bool {
        other == self
    }
}

impl PartialOrd for JsString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for JsString {
    type Err = Infallible;

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
    fn to_string_escaped(&self) -> String {
        char::decode_utf16(self.iter().copied())
            .map(|r| match r {
                Ok(c) => String::from(c),
                Err(e) => format!("\\u{:04X}", e.unpaired_surrogate()),
            })
            .collect()
    }
}

#[allow(clippy::redundant_clone)]
#[cfg(test)]
mod tests {
    use std::hash::{BuildHasher, BuildHasherDefault, Hash};

    use crate::{string::common::StaticJsStrings, tagged::UnwrappedTagged, JsStr};

    use super::JsString;
    use boa_macros::{js_str, utf16};
    use rustc_hash::FxHasher;

    impl JsString {
        /// Gets the number of `JsString`s which point to this allocation.
        fn refcount(&self) -> Option<usize> {
            match self.ptr.unwrap() {
                UnwrappedTagged::Ptr(inner) => {
                    // SAFETY: The reference count of `JsString` guarantees that `inner` is always valid.
                    let inner = unsafe { inner.as_ref() };
                    Some(inner.refcount.get())
                }
                UnwrappedTagged::Tag(_inner) => None,
            }
        }
    }

    fn hash_value<T: Hash>(value: &T) -> u64 {
        BuildHasherDefault::<FxHasher>::default().hash_one(value)
    }

    #[test]
    fn empty() {
        let s = js_string!();
        assert_eq!(&s, utf16!(""));
    }

    #[test]
    fn refcount() {
        let x = js_string!("Hello world");
        assert_eq!(x.refcount(), Some(1));

        {
            let y = x.clone();
            assert_eq!(x.refcount(), Some(2));
            assert_eq!(y.refcount(), Some(2));

            {
                let z = y.clone();
                assert_eq!(x.refcount(), Some(3));
                assert_eq!(y.refcount(), Some(3));
                assert_eq!(z.refcount(), Some(3));
            }

            assert_eq!(x.refcount(), Some(2));
            assert_eq!(y.refcount(), Some(2));
        }

        assert_eq!(x.refcount(), Some(1));
    }

    #[test]
    fn static_refcount() {
        let x = js_string!();
        assert_eq!(x.refcount(), None);

        {
            let y = x.clone();
            assert_eq!(x.refcount(), None);
            assert_eq!(y.refcount(), None);
        };

        assert_eq!(x.refcount(), None);
    }

    #[test]
    fn ptr_eq() {
        let x = js_string!("Hello");
        let y = x.clone();

        assert!(!x.ptr.is_tagged());

        assert_eq!(x.ptr.addr(), y.ptr.addr());

        let z = js_string!("Hello");
        assert_ne!(x.ptr.addr(), z.ptr.addr());
        assert_ne!(y.ptr.addr(), z.ptr.addr());
    }

    #[test]
    fn static_ptr_eq() {
        let x = js_string!();
        let y = x.clone();

        assert!(x.ptr.is_tagged());

        assert_eq!(x.ptr.addr(), y.ptr.addr());

        let z = js_string!();
        assert_eq!(x.ptr.addr(), z.ptr.addr());
        assert_eq!(y.ptr.addr(), z.ptr.addr());
    }

    #[test]
    fn as_str() {
        const HELLO: &[u16] = utf16!("Hello");
        let x = js_string!(HELLO);

        assert_eq!(&x, HELLO);
    }

    #[test]
    fn hash() {
        const HELLOWORLD: JsStr<'_> = js_str!("Hello World!");
        let x = js_string!(HELLOWORLD);

        assert_eq!(x.as_str(), HELLOWORLD);

        assert!(HELLOWORLD.is_latin1());
        assert!(x.as_str().is_latin1());

        let s_hash = hash_value(&HELLOWORLD);
        let x_hash = hash_value(&x);

        assert_eq!(s_hash, x_hash);
    }

    #[test]
    fn concat() {
        const Y: &[u16] = utf16!(", ");
        const W: &[u16] = utf16!("!");

        let x = js_string!("hello");
        let z = js_string!("world");

        let xy = js_string!(&x, &JsString::from(Y));
        assert_eq!(&xy, utf16!("hello, "));
        assert_eq!(xy.refcount(), Some(1));

        let xyz = js_string!(&xy, &z);
        assert_eq!(&xyz, utf16!("hello, world"));
        assert_eq!(xyz.refcount(), Some(1));

        let xyzw = js_string!(&xyz, &JsString::from(W));
        assert_eq!(&xyzw, utf16!("hello, world!"));
        assert_eq!(xyzw.refcount(), Some(1));
    }

    #[test]
    fn trim_start_non_ascii_to_ascii() {
        let s = "\u{2029}abc";
        let x = js_string!(s);

        let y = js_string!(x.trim_start());

        assert_eq!(&y, s.trim_start());
    }

    #[test]
    fn conversion_to_known_static_js_string() {
        const JS_STR_U8: &JsStr<'_> = &js_str!("length");
        const JS_STR_U16: &JsStr<'_> = &JsStr::utf16(utf16!("length"));

        assert!(JS_STR_U8.is_latin1());
        assert!(!JS_STR_U16.is_latin1());

        assert_eq!(JS_STR_U8, JS_STR_U8);
        assert_eq!(JS_STR_U16, JS_STR_U16);

        assert_eq!(JS_STR_U8, JS_STR_U16);
        assert_eq!(JS_STR_U16, JS_STR_U8);

        assert_eq!(hash_value(JS_STR_U8), hash_value(JS_STR_U16));

        let string = StaticJsStrings::get_string(JS_STR_U8);

        assert!(string.is_some());
        assert!(string.unwrap().as_str().is_latin1());

        let string = StaticJsStrings::get_string(JS_STR_U16);

        assert!(string.is_some());
        assert!(string.unwrap().as_str().is_latin1());
    }
}
