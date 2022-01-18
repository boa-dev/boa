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
#![deny(
    clippy::all,
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
)]
#![warn(clippy::perf, clippy::single_match_else, clippy::dbg_macro)]
#![allow(
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions,
    clippy::let_unit_value,
    rustdoc::missing_doc_code_examples
)]

#[cfg(test)]
mod tests;

use std::{fmt::Display, num::NonZeroUsize};

use gc::{unsafe_empty_trace, Finalize, Trace};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use string_interner::{backend::BucketBackend, StringInterner, Symbol};

/// Backend of the string interner.
type Backend = BucketBackend<Sym>;

/// The string interner for Boa.
///
/// This is a type alias that makes it easier to reference it in the code.
#[derive(Debug)]
pub struct Interner {
    inner: StringInterner<Backend>,
}

impl Interner {
    /// Creates a new StringInterner with the given initial capacity.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: StringInterner::with_capacity(cap),
        }
    }

    /// Returns the number of strings interned by the interner.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the string interner has no interned strings.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the symbol for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
    pub fn get<T>(&self, string: T) -> Option<Sym>
    where
        T: AsRef<str>,
    {
        let string = string.as_ref();
        Self::get_static(string).or_else(|| self.inner.get(string))
    }

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Panics
    /// If the interner already interns the maximum number of strings possible by the chosen symbol type.
    pub fn get_or_intern<T>(&mut self, string: T) -> Sym
    where
        T: AsRef<str>,
    {
        let string = string.as_ref();
        Self::get_static(string).unwrap_or_else(|| self.inner.get_or_intern(string))
    }

    /// Interns the given `'static` string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Note
    ///
    /// This is more efficient than [`StringInterner::get_or_intern`] since it might
    /// avoid some memory allocations if the backends supports this.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    pub fn get_or_intern_static(&mut self, string: &'static str) -> Sym {
        Self::get_static(string).unwrap_or_else(|| self.inner.get_or_intern_static(string))
    }

    /// Shrink backend capacity to fit the interned strings exactly.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    /// Returns the string for the given symbol if any.
    #[inline]
    pub fn resolve(&self, symbol: Sym) -> Option<&str> {
        let index = symbol.as_raw().get();
        if index <= Self::STATIC_STRINGS.len() {
            Some(Self::STATIC_STRINGS[index - 1])
        } else {
            self.inner.resolve(symbol)
        }
    }

    /// Gets the symbol of the static string if one of them
    fn get_static(string: &str) -> Option<Sym> {
        Self::STATIC_STRINGS
            .into_iter()
            .enumerate()
            .find(|&(_i, s)| string == s)
            .map(|(i, _str)| {
                let raw = NonZeroUsize::new(i.wrapping_add(1)).expect("static array too big");
                Sym::from_raw(raw)
            })
    }
}

impl<T> FromIterator<T> for Interner
where
    T: AsRef<str>,
{
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            inner: StringInterner::from_iter(iter),
        }
    }
}

impl<T> Extend<T> for Interner
where
    T: AsRef<str>,
{
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.inner.extend(iter)
    }
}

impl<'a> IntoIterator for &'a Interner {
    type Item = (Sym, &'a str);
    type IntoIter = <&'a Backend as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self {
            inner: StringInterner::new(),
        }
    }
}

/// The string symbol type for Boa.
///
/// This symbol type is internally a `NonZeroUsize`, which makes it pointer-width in size and it's
/// optimized so that it can occupy 1 pointer width even in an `Option` type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Finalize)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Sym {
    value: NonZeroUsize,
}

impl Sym {
    /// Padding for the symbol internal value.
    const PADDING: usize = Interner::STATIC_STRINGS.len() + 1;

    /// Symbol for the empty string (`""`).
    pub const EMPTY_STRING: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(1)) };

    /// Symbol for the `"arguments"` string.
    pub const ARGUMENTS: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(2)) };

    /// Symbol for the `"await"` string.
    pub const AWAIT: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(3)) };

    /// Symbol for the `"yield"` string.
    pub const YIELD: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(4)) };

    /// Symbol for the `"eval"` string.
    pub const EVAL: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(5)) };

    /// Symbol for the `"default"` string.
    pub const DEFAULT: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(6)) };

    /// Symbol for the `"null"` string.
    pub const NULL: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(7)) };

    /// Symbol for the `"RegExp"` string.
    pub const REGEXP: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(8)) };

    /// Symbol for the `"get"` string.
    pub const GET: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(9)) };

    /// Symbol for the `"set"` string.
    pub const SET: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(10)) };

    /// Symbol for the `"<main>"` string.
    pub const MAIN: Self = unsafe { Self::from_raw(NonZeroUsize::new_unchecked(11)) };

    /// Creates a `Sym` from a raw `NonZeroUsize`.
    const fn from_raw(value: NonZeroUsize) -> Self {
        Self { value }
    }

    /// Retrieves the raw `NonZeroUsize` for this symbol.`
    const fn as_raw(self) -> NonZeroUsize {
        self.value
    }
}

impl Symbol for Sym {
    #[inline]
    fn try_from_usize(index: usize) -> Option<Self> {
        index
            .checked_add(Self::PADDING)
            .and_then(NonZeroUsize::new)
            .map(|value| Self { value })
    }

    #[inline]
    fn to_usize(self) -> usize {
        self.value.get() - Self::PADDING
    }
}

// Safe because `Sym` implements `Copy`.
unsafe impl Trace for Sym {
    unsafe_empty_trace!();
}

/// Converts a given element to a string using an interner.
pub trait ToInternedString {
    /// Converts a given element to a string using an interner.
    fn to_interned_string(&self, interner: &Interner) -> String;
}

impl<T> ToInternedString for T
where
    T: Display,
{
    fn to_interned_string(&self, _interner: &Interner) -> String {
        format!("{}", self)
    }
}

impl Interner {
    /// List of commonly used static strings.
    ///
    /// Make sure that any string added as a `Sym` constant is also added here.
    const STATIC_STRINGS: [&'static str; 11] = [
        "",
        "arguments",
        "await",
        "yield",
        "eval",
        "default",
        "null",
        "RegExp",
        "get",
        "set",
        "<main>",
    ];
}
