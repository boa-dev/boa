//! Boa's **`boa_interner`** is a string interner for compiler performance.
//!
//! # Crate Overview
//!
//! The idea behind using a string interner is that in most of the code, strings such as
//! identifiers and literals are often repeated. This causes extra burden when comparing them and
//! storing them. A string interner stores a unique `usize` symbol for each string, making sure
//! that there are no duplicates. This makes it much easier to compare, since it's just comparing
//! to `usize`, and also it's easier to store, since instead of a heap-allocated string, you only
//! need to store a `usize`. This reduces memory consumption and improves performance in the
//! compiler.
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![allow(
    clippy::redundant_pub_crate,
    // TODO deny once false positive is fixed (https://github.com/rust-lang/rust-clippy/issues/9626).
    clippy::trait_duplication_in_bounds
)]
#![cfg_attr(not(feature = "arbitrary"), no_std)]

extern crate alloc;

mod fixed_string;
mod interned_str;
mod raw;
mod sym;

#[cfg(test)]
mod tests;

use alloc::{borrow::Cow, format, string::String};
use raw::RawInterner;

pub use sym::*;

/// An enumeration of all slice types [`Interner`] can internally store.
///
/// This struct allows us to intern either `UTF-8` or `UTF-16` str references, which are the two
/// encodings [`Interner`] can store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JStrRef<'a> {
    /// A `UTF-8` string reference.
    Utf8(&'a str),

    /// A `UTF-16` string reference.
    Utf16(&'a [u16]),
}

impl<'a> From<&'a str> for JStrRef<'a> {
    fn from(s: &'a str) -> Self {
        JStrRef::Utf8(s)
    }
}

impl<'a> From<&'a [u16]> for JStrRef<'a> {
    fn from(s: &'a [u16]) -> Self {
        JStrRef::Utf16(s)
    }
}

impl<'a, const N: usize> From<&'a [u16; N]> for JStrRef<'a> {
    fn from(s: &'a [u16; N]) -> Self {
        JStrRef::Utf16(s)
    }
}

/// A double reference to an interned string inside [`Interner`].
///
/// [`JSInternedStrRef::utf8`] returns an [`Option`], since not every `UTF-16` string is fully
/// representable as a `UTF-8` string (because of unpaired surrogates). However, every `UTF-8`
/// string is representable as a `UTF-16` string, so `JSInternedStrRef::utf8` returns a
/// [<code>&\[u16\]</code>][core::slice].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JSInternedStrRef<'a, 'b> {
    utf8: Option<&'a str>,
    utf16: &'b [u16],
}

impl<'a, 'b> JSInternedStrRef<'a, 'b> {
    /// Returns the inner reference to the interned string in `UTF-8` encoding.
    /// if the string is not representable in `UTF-8`, returns [`None`]
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let interned = interner.resolve_expect(sym);
    /// assert_eq!(interned.utf8(), Some("hello"));
    /// ```
    #[inline]
    #[must_use]
    pub const fn utf8(&self) -> Option<&'a str> {
        self.utf8
    }

    /// Returns the inner reference to the interned string in `UTF-16` encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let interned = interner.resolve_expect(sym);
    /// let utf16: Vec<u16> = "hello".encode_utf16().collect();
    /// assert_eq!(interned.utf16(), utf16.as_slice());
    /// ```
    #[inline]
    #[must_use]
    pub const fn utf16(&self) -> &'b [u16] {
        self.utf16
    }

    /// Joins the result of both possible strings into a common type.
    ///
    /// If `self` is representable by a `UTF-8` string and the `prioritize_utf8` argument is set,
    /// it will prioritize calling `f`, and will only call `g` if `self` is only representable by a
    /// `UTF-16` string. Otherwise, it will directly call `g`.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let interned = interner.resolve_expect(sym);
    /// let result = interned.join(
    ///     |utf8| utf8.to_uppercase(),
    ///     |utf16| String::from_utf16_lossy(utf16).to_uppercase(),
    ///     true,
    /// );
    /// assert_eq!(result, "HELLO");
    /// ```
    pub fn join<F, G, T>(self, f: F, g: G, prioritize_utf8: bool) -> T
    where
        F: FnOnce(&'a str) -> T,
        G: FnOnce(&'b [u16]) -> T,
    {
        if prioritize_utf8 && let Some(str) = self.utf8 {
            return f(str);
        }
        g(self.utf16)
    }

    /// Same as [`join`][`JSInternedStrRef::join`], but where you can pass an additional context.
    ///
    /// Useful when you have a `&mut Context` context that cannot be borrowed by both closures at
    /// the same time.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let interned = interner.resolve_expect(sym);
    /// let mut output = String::new();
    /// interned.join_with_context(
    ///     |utf8, buf: &mut String| buf.push_str(&utf8.to_uppercase()),
    ///     |utf16, buf: &mut String| buf.push_str(&String::from_utf16_lossy(utf16).to_uppercase()),
    ///     &mut output,
    ///     true,
    /// );
    /// assert_eq!(output, "HELLO");
    /// ```
    pub fn join_with_context<C, F, G, T>(self, f: F, g: G, ctx: C, prioritize_utf8: bool) -> T
    where
        F: FnOnce(&'a str, C) -> T,
        G: FnOnce(&'b [u16], C) -> T,
    {
        if prioritize_utf8 && let Some(str) = self.utf8 {
            return f(str, ctx);
        }
        g(self.utf16, ctx)
    }

    /// Converts both string types into a common type `C`.
    ///
    /// If `self` is representable by a `UTF-8` string and the `prioritize_utf8` argument is set, it
    /// will prioritize converting its `UTF-8` representation first, and will only convert its
    /// `UTF-16` representation if it is only representable by a `UTF-16` string. Otherwise, it will
    /// directly convert its `UTF-16` representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// enum JsString<'a> {
    ///     Utf8(&'a str),
    ///     Utf16(&'a [u16]),
    /// }
    ///
    /// impl<'a> From<&'a str> for JsString<'a> {
    ///     fn from(s: &'a str) -> Self {
    ///         JsString::Utf8(s)
    ///     }
    /// }
    ///
    /// impl<'a> From<&'a [u16]> for JsString<'a> {
    ///     fn from(s: &'a [u16]) -> Self {
    ///         JsString::Utf16(s)
    ///     }
    /// }
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let interned = interner.resolve_expect(sym);
    /// let result: JsString<'_> = interned.into_common(true);
    /// assert!(matches!(result, JsString::Utf8("hello")));
    /// ```
    pub fn into_common<C>(self, prioritize_utf8: bool) -> C
    where
        C: From<&'a str> + From<&'b [u16]>,
    {
        self.join(Into::into, Into::into, prioritize_utf8)
    }
}

impl core::fmt::Display for JSInternedStrRef<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.join_with_context(
            core::fmt::Display::fmt,
            |js, f| {
                char::decode_utf16(js.iter().copied())
                    .map(|r| match r {
                        Ok(c) => String::from(c),
                        Err(e) => format!("\\u{:04X}", e.unpaired_surrogate()),
                    })
                    .collect::<String>()
                    .fmt(f)
            },
            f,
            true,
        )
    }
}

/// The string interner for Boa.
#[derive(Debug, Default)]
pub struct Interner {
    utf8_interner: RawInterner<u8>,
    utf16_interner: RawInterner<u16>,
}

impl Interner {
    /// Creates a new [`Interner`].
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// assert!(interner.resolve(sym).is_some());
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`Interner`] with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::with_capacity(10);
    /// let sym = interner.get_or_intern("hello");
    /// assert!(interner.resolve(sym).is_some());
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            utf8_interner: RawInterner::with_capacity(capacity),
            utf16_interner: RawInterner::with_capacity(capacity),
        }
    }

    /// Returns the number of strings interned by the interner.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let initial_len = interner.len();
    /// interner.get_or_intern("hello");
    /// assert_eq!(interner.len(), initial_len + 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        COMMON_STRINGS_UTF8.len() + self.utf16_interner.len()
    }

    /// Returns `true` if the [`Interner`] contains no interned strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let interner = Interner::new();
    /// assert!(!interner.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        COMMON_STRINGS_UTF8.is_empty() && self.utf16_interner.is_empty()
    }

    /// Returns the symbol for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// assert!(interner.get("hello").is_none());
    /// interner.get_or_intern("hello");
    /// assert!(interner.get("hello").is_some());
    /// ```
    pub fn get<'a, T>(&self, string: T) -> Option<Sym>
    where
        T: Into<JStrRef<'a>>,
    {
        let string = string.into();
        Self::get_common(string).or_else(|| {
            let index = match string {
                JStrRef::Utf8(s) => self.utf8_interner.get(s.as_bytes()),
                JStrRef::Utf16(s) => self.utf16_interner.get(s),
            };
            // SAFETY:
            // `get_or_intern/get_or_intern_static` already have checks to avoid returning indices
            // that could cause overflows, meaning the indices returned by
            // `idx + 1 + COMMON_STRINGS_UTF8.len()` cannot cause overflows.
            unsafe { index.map(|i| Sym::new_unchecked(i + 1 + COMMON_STRINGS_UTF8.len())) }
        })
    }

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible by the chosen symbol type.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym1 = interner.get_or_intern("hello");
    /// let sym2 = interner.get_or_intern("hello");
    /// assert_eq!(sym1, sym2);
    /// let sym3 = interner.get_or_intern("world");
    /// assert_ne!(sym1, sym3);
    /// ```
    pub fn get_or_intern<'a, T>(&mut self, string: T) -> Sym
    where
        T: Into<JStrRef<'a>>,
    {
        let string = string.into();
        self.get(string).unwrap_or_else(|| {
            let (utf8, utf16) = match string {
                JStrRef::Utf8(s) => (
                    Some(Cow::Borrowed(s)),
                    Cow::Owned(s.encode_utf16().collect()),
                ),
                JStrRef::Utf16(s) => (String::from_utf16(s).ok().map(Cow::Owned), Cow::Borrowed(s)),
            };

            let index = if let Some(utf8) = utf8 {
                self.utf8_interner.intern(utf8.as_bytes())
            } else {
                self.utf8_interner.intern_static(b"")
            };

            let utf16_index = self.utf16_interner.intern(&utf16);

            assert_eq!(index, utf16_index);

            index
                .checked_add(1 + COMMON_STRINGS_UTF8.len())
                .and_then(Sym::new)
                .expect("Cannot intern new string: integer overflow")
        })
    }

    /// Interns the given `'static` string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Note
    ///
    /// This is more efficient than [`Interner::get_or_intern`], since it avoids allocating space
    /// for one `string` inside the [`Interner`], with the disadvantage that you need to provide
    /// both the `UTF-8` and the `UTF-16` representation of the string.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible by the chosen symbol type.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// static HELLO_UTF16: &[u16] = &[0x68, 0x65, 0x6C, 0x6C, 0x6F];
    ///
    /// let mut interner = Interner::new();
    /// let sym1 = interner.get_or_intern_static("hello", HELLO_UTF16);
    /// let sym2 = interner.get_or_intern("hello");
    /// assert_eq!(sym1, sym2);
    /// ```
    pub fn get_or_intern_static(&mut self, utf8: &'static str, utf16: &'static [u16]) -> Sym {
        self.get(utf8).unwrap_or_else(|| {
            let index = self.utf8_interner.intern(utf8.as_bytes());
            let utf16_index = self.utf16_interner.intern(utf16);

            debug_assert_eq!(index, utf16_index);

            index
                .checked_add(1 + COMMON_STRINGS_UTF8.len())
                .and_then(Sym::new)
                .expect("Cannot intern new string: integer overflow")
        })
    }

    /// Returns the string for the given symbol if any.
    ///
    /// # Panics
    ///
    /// Panics if the size of both statics is not equal or the interners do
    /// not have the same size
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let resolved = interner.resolve(sym);
    /// assert!(resolved.is_some());
    /// assert_eq!(resolved.unwrap().utf8(), Some("hello"));
    /// ```
    #[must_use]
    pub fn resolve(&self, symbol: Sym) -> Option<JSInternedStrRef<'_, '_>> {
        let index = symbol.get() - 1;

        if let Some(utf8) = COMMON_STRINGS_UTF8.index(index).copied() {
            let utf16 = COMMON_STRINGS_UTF16
                .get_index(index)
                .copied()
                .expect("The sizes of both statics must be equal");
            return Some(JSInternedStrRef {
                utf8: Some(utf8),
                utf16,
            });
        }

        let index = index - COMMON_STRINGS_UTF8.len();

        if let Some(utf16) = self.utf16_interner.index(index) {
            let index = index - (self.utf16_interner.len() - self.utf8_interner.len());
            // SAFETY:
            // We only manipulate valid UTF-8 `str`s and convert them to `[u8]` for convenience,
            // so converting back to a `str` is safe.
            let utf8 = unsafe {
                core::str::from_utf8_unchecked(
                    self.utf8_interner
                        .index(index)
                        .expect("both interners must have the same size"),
                )
            };
            return Some(JSInternedStrRef {
                utf8: if utf8.is_empty() { None } else { Some(utf8) },
                utf16,
            });
        }

        None
    }

    /// Returns the string for the given symbol.
    ///
    /// # Panics
    ///
    /// If the interner cannot resolve the given symbol.
    ///
    /// # Examples
    ///
    /// ```
    /// use boa_interner::Interner;
    ///
    /// let mut interner = Interner::new();
    /// let sym = interner.get_or_intern("hello");
    /// let resolved = interner.resolve_expect(sym);
    /// assert_eq!(resolved.utf8(), Some("hello"));
    /// ```
    #[inline]
    #[must_use]
    pub fn resolve_expect(&self, symbol: Sym) -> JSInternedStrRef<'_, '_> {
        self.resolve(symbol).expect("string disappeared")
    }

    fn get_common(string: JStrRef<'_>) -> Option<Sym> {
        match string {
            JStrRef::Utf8(s) => COMMON_STRINGS_UTF8.get_index(s).map(|idx| {
                // SAFETY: `idx >= 0`, since it's an `usize`, and `idx + 1 > 0`.
                // In this case, we don't need to worry about overflows because we have a static
                // assertion in place checking that `COMMON_STRINGS.len() < usize::MAX`.
                unsafe { Sym::new_unchecked(idx + 1) }
            }),
            JStrRef::Utf16(s) => COMMON_STRINGS_UTF16.get_index_of(&s).map(|idx| {
                // SAFETY: `idx >= 0`, since it's an `usize`, and `idx + 1 > 0`.
                // In this case, we don't need to worry about overflows because we have a static
                // assertion in place checking that `COMMON_STRINGS.len() < usize::MAX`.
                unsafe { Sym::new_unchecked(idx + 1) }
            }),
        }
    }
}

/// Implements the display formatting with indentation.
pub trait ToIndentedString {
    /// Converts the element to a string using an interner, with the given indentation.
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String;
}

/// Converts a given element to a string using an interner.
pub trait ToInternedString {
    /// Converts a given element to a string using an interner.
    fn to_interned_string(&self, interner: &Interner) -> String;
}

impl<T> ToInternedString for T
where
    T: ToIndentedString,
{
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}