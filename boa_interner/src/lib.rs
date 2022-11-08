//! String interner for Boa.
//!
//! The idea behind using a string interner is that in most of the code, strings such as
//! identifiers and literals are often repeated. This causes extra burden when comparing them and
//! storing them. A string interner stores a unique `usize` symbol for each string, making sure
//! that there are no duplicates. This makes it much easier to compare, since it's just comparing
//! to `usize`, and also it's easier to store, since instead of a heap-allocated string, you only
//! need to store a `usize`. This reduces memory consumption and improves performance in the
//! compiler.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(
    clippy::perf,
    clippy::single_match_else,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::struct_excessive_bools,
    clippy::doc_markdown,
    clippy::semicolon_if_nothing_returned,
    clippy::pedantic
)]
#![deny(
    clippy::all,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::use_self,
    clippy::unnested_or_patterns,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_unwrap_or,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    // rustdoc,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    unsafe_op_in_unsafe_fn
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_ptr_alignment,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions,
    clippy::let_unit_value,
    // TODO deny once false positive is fixed (https://github.com/rust-lang/rust-clippy/issues/9626).
    clippy::trait_duplication_in_bounds,
)]

extern crate static_assertions as sa;

mod fixed_string;
mod interned_str;
mod raw;
mod sym;
#[cfg(test)]
mod tests;

use std::borrow::Cow;

use raw::RawInterner;
pub use sym::*;

/// An enumeration of all slice types [`Interner`] can internally store.
///
/// This struct allows us to intern either `UTF-8` or `UTF-16` str references, which are the two
/// encodings [`Interner`] can store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JStrRef<'a> {
    Utf8(&'a str),
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
/// [<code>&\[u16\]</code>][std::slice].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JSInternedStrRef<'a, 'b> {
    utf8: Option<&'a str>,
    utf16: &'b [u16],
}

impl<'a, 'b> JSInternedStrRef<'a, 'b> {
    /// Returns the inner reference to the interned string in `UTF-8` encoding.
    /// if the string is not representable in `UTF-8`, returns [`None`]
    pub fn utf8(&self) -> Option<&'a str> {
        self.utf8
    }

    /// Returns the inner reference to the interned string in `UTF-16` encoding.
    pub fn utf16(&self) -> &'b [u16] {
        self.utf16
    }

    /// Joins the result of both possible strings into a common type.
    ///
    /// If `self` is representable by a `UTF-8` string and the `prioritize_utf8` argument is set,
    /// it will prioritize calling `f`, and will only call `g` if `self` is only representable by a
    /// `UTF-16` string. Otherwise, it will directly call `g`.
    pub fn join<F, G, T>(self, f: F, g: G, prioritize_utf8: bool) -> T
    where
        F: FnOnce(&'a str) -> T,
        G: FnOnce(&'b [u16]) -> T,
    {
        if prioritize_utf8 {
            if let Some(str) = self.utf8 {
                return f(str);
            }
        }
        g(self.utf16)
    }

    /// Same as [`join`][`JSInternedStrRef::join`], but where you can pass an additional context.
    ///
    /// Useful when you have a `&mut Context` context that cannot be borrowed by both closures at
    /// the same time.
    pub fn join_with_context<C, F, G, T>(self, f: F, g: G, ctx: C, prioritize_utf8: bool) -> T
    where
        F: FnOnce(&'a str, C) -> T,
        G: FnOnce(&'b [u16], C) -> T,
    {
        if prioritize_utf8 {
            if let Some(str) = self.utf8 {
                return f(str, ctx);
            }
        }
        g(self.utf16, ctx)
    }

    /// Converts both string types into a common type `C`.
    ///
    /// If `self` is representable by a `UTF-8` string and the `prioritize_utf8` argument is set, it
    /// will prioritize converting its `UTF-8` representation first, and will only convert its
    /// `UTF-16` representation if it is only representable by a `UTF-16` string. Otherwise, it will
    /// directly convert its `UTF-16` representation.
    pub fn into_common<C>(self, prioritize_utf8: bool) -> C
    where
        C: From<&'a str> + From<&'b [u16]>,
    {
        self.join(Into::into, Into::into, prioritize_utf8)
    }
}

impl<'a, 'b> std::fmt::Display for JSInternedStrRef<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.join_with_context(
            std::fmt::Display::fmt,
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
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`Interner`] with the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            utf8_interner: RawInterner::with_capacity(capacity),
            utf16_interner: RawInterner::with_capacity(capacity),
        }
    }

    /// Returns the number of strings interned by the interner.
    #[inline]
    pub fn len(&self) -> usize {
        // `utf16_interner.len()` == `utf8_interner.len()`,
        // so we can use any of them.
        COMMON_STRINGS_UTF8.len() + self.utf16_interner.len()
    }

    /// Returns `true` if the [`Interner`] contains no interned strings.
    #[inline]
    pub fn is_empty(&self) -> bool {
        COMMON_STRINGS_UTF8.is_empty() && self.utf16_interner.is_empty()
    }

    /// Returns the symbol for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
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

            // We need a way to check for the strings that can be interned by `utf16_interner` but
            // not by `utf8_interner` (since there are some UTF-16 strings with surrogates that are
            // not representable in UTF-8), so we use the sentinel value `""` as a marker indicating
            // that the `Sym` corresponding to that string is only available in `utf16_interner`.
            //
            // We don't need to worry about matches with `""` inside `get`, because
            // `COMMON_STRINGS_UTF8` filters all the empty strings before interning.
            let index = if let Some(utf8) = utf8 {
                self.utf8_interner.intern(utf8.as_bytes())
            } else {
                self.utf8_interner.intern_static(b"")
            };

            let utf16_index = self.utf16_interner.intern(&utf16);

            // Just to check everything is okay
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
    pub fn get_or_intern_static(&mut self, utf8: &'static str, utf16: &'static [u16]) -> Sym {
        // Uses the utf8 because it's quicker to check inside `COMMON_STRINGS_UTF8`
        // (which is a perfect hash set) than to check inside `COMMON_STRINGS_UTF16`
        // (which is a lazy static hash set).
        self.get(utf8).unwrap_or_else(|| {
            let index = self.utf8_interner.intern(utf8.as_bytes());
            let utf16_index = self.utf16_interner.intern(utf16);

            // Just to check everything is okay
            debug_assert_eq!(index, utf16_index);

            index
                .checked_add(1 + COMMON_STRINGS_UTF8.len())
                .and_then(Sym::new)
                .expect("Cannot intern new string: integer overflow")
        })
    }

    /// Returns the string for the given symbol if any.
    #[inline]
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
                std::str::from_utf8_unchecked(
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
    #[inline]
    pub fn resolve_expect(&self, symbol: Sym) -> JSInternedStrRef<'_, '_> {
        self.resolve(symbol).expect("string disappeared")
    }

    /// Gets the symbol of the common string if one of them
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
