use std::hash::BuildHasherDefault;

use crate::tagged::Tagged;

use super::JsString;
use boa_macros::utf16;
use rustc_hash::{FxHashMap, FxHasher};

use boa_builtins::RAW_STATICS;

macro_rules! well_known_statics {
    ( $( $(#[$attr:meta])* ($name:ident, $string:literal) ),+$(,)? ) => {
        $(
            $(#[$attr])* pub(crate) const fn $name() -> JsString {
                JsString {
                    ptr: Tagged::from_tag(
                        Self::find_index(utf16!($string)),
                    ),
                }
            }
        )+
    };
}

/// List of commonly used strings in Javascript code.
///
/// Any strings defined here are used as a static [`JsString`] instead of allocating on the heap.
#[derive(Debug)]
pub(crate) struct StaticJsStrings;

impl StaticJsStrings {
    // useful to search at compile time a certain string in the array
    const fn find_index(candidate: &[u16]) -> usize {
        const fn const_eq(lhs: &[u16], rhs: &[u16]) -> bool {
            if lhs.len() != rhs.len() {
                return false;
            }

            let mut i = 0;
            while i < lhs.len() {
                if lhs[i] != rhs[i] {
                    return false;
                }
                i += 1;
            }
            true
        }
        let mut i = 0;
        while i < RAW_STATICS.len() {
            let s = RAW_STATICS[i];
            if const_eq(s, candidate) {
                return i;
            }
            i += 1;
        }
        panic!("couldn't find the required string on the common string array");
    }

    /// Gets the `JsString` corresponding to `string`, or `None` if the string
    /// doesn't exist inside the static array.
    pub(crate) fn get_string(string: &[u16]) -> Option<JsString> {
        if string.len() > MAX_STATIC_LENGTH {
            return None;
        }

        let index = RAW_STATICS_CACHE.with(|map| map.get(string).copied())?;

        Some(JsString {
            ptr: Tagged::from_tag(index),
        })
    }

    /// Gets the `&[u16]` slice corresponding to the provided index, or `None` if the index
    /// provided exceeds the size of the static array.
    pub(crate) fn get(index: usize) -> Option<&'static [u16]> {
        RAW_STATICS.get(index).copied()
    }

    well_known_statics! {
        /// Gets the empty string (`""`) `JsString`.
        (empty_string, ""),
        /// Gets the static `JsString` for `"Symbol.asyncIterator"`.
        (symbol_async_iterator, "Symbol.asyncIterator"),
        /// Gets the static `JsString` for `"Symbol.hasInstance"`.
        (symbol_has_instance, "Symbol.hasInstance"),
        /// Gets the static `JsString` for `"Symbol.isConcatSpreadable"`.
        (symbol_is_concat_spreadable, "Symbol.isConcatSpreadable"),
        /// Gets the static `JsString` for `"Symbol.iterator"`.
        (symbol_iterator, "Symbol.iterator"),
        /// Gets the static `JsString` for `"Symbol.match"`.
        (symbol_match, "Symbol.match"),
        /// Gets the static `JsString` for `"Symbol.matchAll"`.
        (symbol_match_all, "Symbol.matchAll"),
        /// Gets the static `JsString` for `"Symbol.replace"`.
        (symbol_replace, "Symbol.replace"),
        /// Gets the static `JsString` for `"Symbol.search"`.
        (symbol_search, "Symbol.search"),
        /// Gets the static `JsString` for `"Symbol.species"`.
        (symbol_species, "Symbol.species"),
        /// Gets the static `JsString` for `"Symbol.split"`.
        (symbol_split, "Symbol.split"),
        /// Gets the static `JsString` for `"Symbol.toPrimitive"`.
        (symbol_to_primitive, "Symbol.toPrimitive"),
        /// Gets the static `JsString` for `"Symbol.toStringTag"`.
        (symbol_to_string_tag, "Symbol.toStringTag"),
        /// Gets the static `JsString` for `"Symbol.unscopables"`.
        (symbol_unscopables, "Symbol.unscopables"),
    }
}

static MAX_STATIC_LENGTH: usize = {
    let mut max = 0;
    let mut i = 0;
    while i < RAW_STATICS.len() {
        let len = RAW_STATICS[i].len();
        if len > max {
            max = len;
        }
        i += 1;
    }
    max
};

thread_local! {
    /// Map from a string inside [`RAW_STATICS`] to its corresponding static index on `RAW_STATICS`.
    static RAW_STATICS_CACHE: FxHashMap<&'static [u16], usize> = {
        let mut constants = FxHashMap::with_capacity_and_hasher(
            RAW_STATICS.len(),
            BuildHasherDefault::<FxHasher>::default(),
        );

        for (idx, &s) in RAW_STATICS.iter().enumerate() {
            constants.insert(s, idx);
        }

        constants
    };
}
