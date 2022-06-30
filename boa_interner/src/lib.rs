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
    rustdoc::missing_doc_code_examples
)]

extern crate static_assertions as sa;

mod fixed_string;
mod interned_str;
mod sym;
#[cfg(test)]
mod tests;

use fixed_string::FixedString;
pub use sym::*;

use std::fmt::{Debug, Display};

use interned_str::InternedStr;
use rustc_hash::FxHashMap;

/// The string interner for Boa.
#[derive(Debug, Default)]
pub struct Interner {
    // COMMENT FOR DEVS:
    // This interner works on the assumption that
    // `head` won't ever be reallocated, since this could invalidate
    // some of our stored pointers inside `spans`.
    // This means that any operation on `head` and `full` should be carefully
    // reviewed to not cause Undefined Behaviour.
    // `get_or_intern` has a more thorough explanation on this.
    //
    // Also, if you want to implement `shrink_to_fit` (and friends),
    // please check out https://github.com/Robbepop/string-interner/pull/47 first.
    // This doesn't implement that method, since implementing it increases
    // our memory footprint.
    symbols: FxHashMap<InternedStr, Sym>,
    spans: Vec<InternedStr>,
    head: FixedString,
    full: Vec<FixedString>,
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
            symbols: FxHashMap::default(),
            spans: Vec::with_capacity(capacity),
            head: FixedString::new(capacity),
            full: Vec::new(),
        }
    }

    /// Returns the number of strings interned by the interner.
    #[inline]
    pub fn len(&self) -> usize {
        COMMON_STRINGS.len() + self.spans.len()
    }

    /// Returns `true` if the [`Interner`] contains no interned strings.
    #[inline]
    pub fn is_empty(&self) -> bool {
        COMMON_STRINGS.is_empty() && self.spans.is_empty()
    }

    /// Returns the symbol for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
    pub fn get<T>(&self, string: T) -> Option<Sym>
    where
        T: AsRef<str>,
    {
        let string = string.as_ref();
        Self::get_common(string).or_else(|| self.symbols.get(string).copied())
    }

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible by the chosen symbol type.
    pub fn get_or_intern<T>(&mut self, string: T) -> Sym
    where
        T: AsRef<str>,
    {
        let string = string.as_ref();
        if let Some(sym) = self.get(string) {
            return sym;
        }

        // SAFETY:
        //
        // Firstly, this interner works on the assumption that the allocated
        // memory by `head` won't ever be moved from its position on the heap,
        // which is an important point to understand why manipulating it like
        // this is safe.
        //
        // `String` (which is simply a `Vec<u8>` with additional invariants)
        // is essentially a pointer to heap memory that can be moved without
        // any problems, since copying a pointer cannot invalidate the memory
        // that it points to.
        //
        // However, `String` CAN be invalidated when pushing, extending or
        // shrinking it, since all those operations reallocate on the heap.
        //
        // To prevent that, we HAVE to ensure the capacity will succeed without
        // having to reallocate, and the only way to do that without invalidating
        // any other alive `InternedStr` is to create a brand new `head` with
        // enough capacity and push the old `head` to `full` to keep it alive
        // throughout the lifetime of the whole `Interner`.
        //
        // `FixedString` encapsulates this by only allowing checked `push`es
        // to the internal string, but we still have to ensure the memory
        // of `head` is not deallocated until the whole `Interner` deallocates,
        // which we can do by moving it inside the `Interner` itself, specifically
        // on the `full` vector, where every other old `head` also lives.
        let interned_str = unsafe {
            self.head.push(string).unwrap_or_else(|| {
                let new_cap =
                    (usize::max(self.head.capacity(), string.len()) + 1).next_power_of_two();
                let new_head = FixedString::new(new_cap);
                let old_head = std::mem::replace(&mut self.head, new_head);

                // If the user creates an `Interner`
                // with `Interner::with_capacity(BIG_NUMBER)` and
                // the first interned string's length is bigger than `BIG_NUMBER`,
                // `self.full.push(old_head)` would push a big, empty string of
                // allocated size `BIG_NUMBER` into `full`.
                // This prevents that case.
                if !old_head.is_empty() {
                    self.full.push(old_head);
                }
                self.head.push_unchecked(string)
            })
        };

        // SAFETY: We are obtaining a pointer to the internal memory of
        // `head`, which is alive through the whole life of `Interner`, so
        // this is safe.
        unsafe { self.generate_symbol(interned_str) }
    }

    /// Interns the given `'static` string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Note
    ///
    /// This is more efficient than [`Interner::get_or_intern`], since it
    /// avoids storing `string` inside the [`Interner`].
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    pub fn get_or_intern_static(&mut self, string: &'static str) -> Sym {
        self.get(string).unwrap_or_else(|| {
            // SAFETY: a static `str` is always alive, so its pointer
            // should therefore always be valid.
            unsafe { self.generate_symbol(InternedStr::new(string.into())) }
        })
    }

    /// Returns the string for the given symbol if any.
    #[inline]
    pub fn resolve(&self, symbol: Sym) -> Option<&str> {
        let index = symbol.get() - 1;

        COMMON_STRINGS.index(index).copied().or_else(|| {
            self.spans.get(index - COMMON_STRINGS.len()).map(|ptr|
                // SAFETY: We always ensure the stored `InternedStr`s always
                // reference memory inside `head` and `full`
                unsafe {ptr.as_str()})
        })
    }

    /// Returns the string for the given symbol.
    ///
    /// # Panics
    ///
    /// If the interner cannot resolve the given symbol.
    #[inline]
    pub fn resolve_expect(&self, symbol: Sym) -> &str {
        self.resolve(symbol).expect("string disappeared")
    }

    /// Gets the symbol of the common string if one of them
    fn get_common(string: &str) -> Option<Sym> {
        COMMON_STRINGS.get_index(string).map(|idx|
            // SAFETY: `idx >= 0`, since it's an `usize`, and `idx + 1 > 0`.
            // In this case, we don't need to worry about overflows
            // because we have a static assertion in place checking that
            // `COMMON_STRINGS.len() < usize::MAX`.
            unsafe {
                Sym::new_unchecked(idx + 1)
            })
    }

    /// Generates a new symbol for the provided [`str`] pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure `string` points to a valid
    /// memory inside `head` and that it won't be invalidated
    /// by allocations and deallocations.
    unsafe fn generate_symbol(&mut self, string: InternedStr) -> Sym {
        let next = Sym::new(self.len() + 1).expect("cannot get interner symbol: integer overflow");
        self.spans.push(string.clone());
        self.symbols.insert(string, next);
        next
    }
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
        self.to_string()
    }
}
