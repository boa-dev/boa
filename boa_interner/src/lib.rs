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
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,

    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    missing_docs,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,

    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,

    // clippy allowed by default
    clippy::dbg_macro,

    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
)]
#![allow(
    clippy::redundant_pub_crate,
    // TODO deny once false positive is fixed (https://github.com/rust-lang/rust-clippy/issues/9626).
    clippy::trait_duplication_in_bounds
)]
#![cfg_attr(not(feature = "arbitrary"), no_std)]

extern crate alloc;

mod sym;

#[cfg(test)]
mod tests;

use alloc::{format, string::String};
use boa_types::string::{
    common::{StaticJsStrings, RAW_STATICS},
    JsString, JsStringSlice,
};

use alloc::vec::Vec;
use core::hash::BuildHasherDefault;
use hashbrown::{hash_map::Entry, HashMap};
use rustc_hash::FxHasher;
pub use sym::*;

pub use boa_types::js_string;

type Map<T, U> = HashMap<T, U, BuildHasherDefault<FxHasher>>;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JSInternedStrRef {
    inner: JsString,
}

impl core::ops::Deref for JSInternedStrRef {
    type Target = JsString;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<JsString> for JSInternedStrRef {
    fn from(value: JsString) -> Self {
        Self { inner: value }
    }
}

impl From<JSInternedStrRef> for JsString {
    fn from(value: JSInternedStrRef) -> Self {
        value.inner
    }
}

impl core::fmt::Display for JSInternedStrRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        char::decode_utf16(self.inner.iter())
            .map(|r| match r {
                Ok(c) => String::from(c),
                Err(e) => format!("\\u{:04X}", e.unpaired_surrogate()),
            })
            .collect::<String>()
            .fmt(f)
    }
}

/// The string interner for Boa.
#[derive(Debug, Default)]
pub struct Interner {
    symbol_cache: Map<JsString, usize>,
    full: Vec<JsString>,
}

impl Interner {
    /// Creates a new [`Interner`].
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`Interner`] with the specified capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(_capacity: usize) -> Self {
        Self {
            symbol_cache: Map::default(),
            full: Vec::new(),
        }
    }

    /// Returns the number of strings interned by the interner.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.full.len()
    }

    /// Returns `true` if the [`Interner`] contains no interned strings.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.full.is_empty()
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
        T: Into<JsStringSlice<'a>>,
    {
        let string = JsString::from(string.into());
        if let Some(index) = string.as_static() {
            return Sym::new(index + 1).expect("should not be zero");
        }
        let next_index = self.full.len();
        match self.symbol_cache.entry(string.clone()) {
            Entry::Occupied(entry) => {
                return Sym::new(*entry.get() + 1 + RAW_STATICS.len()).expect("should not be zero");
            }
            Entry::Vacant(entry) => {
                entry.insert(next_index);
            }
        }

        self.full.push(string);

        Sym::new(next_index + 1 + RAW_STATICS.len()).expect("should not be zero")
    }

    /// Returns the string for the given symbol if any.
    #[must_use]
    pub fn resolve(&self, sym: Sym) -> Option<JSInternedStrRef> {
        let index = sym.get() - 1;

        if index < RAW_STATICS.len() {
            return StaticJsStrings::get_string(StaticJsStrings::get(index)?).map(Into::into);
        }

        let index = index - RAW_STATICS.len();

        self.full.get(index).cloned().map(Into::into)
    }

    /// Returns the string for the given symbol.
    ///
    /// # Panics
    ///
    /// If the interner cannot resolve the given symbol.
    #[inline]
    #[must_use]
    pub fn resolve_expect(&self, symbol: Sym) -> JSInternedStrRef {
        self.resolve(symbol).expect("string disappeared")
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
