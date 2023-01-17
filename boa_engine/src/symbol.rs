//! Boa's implementation of ECMAScript's global `Symbol` object.
//!
//! The data type symbol is a primitive data type.
//! The `Symbol()` function returns a value of type symbol, has static properties that expose
//! several members of built-in objects, has static methods that expose the global symbol registry,
//! and resembles a built-in object class, but is incomplete as a constructor because it does not
//! support the syntax "`new Symbol()`".
//!
//! Every symbol value returned from `Symbol()` is unique.
//!
//! More information:
//! - [MDN documentation][mdn]
//! - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-symbol-value
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol

#![deny(
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc
)]

use crate::{
    js_string,
    string::{common::STATIC_JS_STRINGS, utf16},
    tagged::{Tagged, UnwrappedTagged},
    JsString,
};
use boa_gc::{empty_trace, Finalize, Trace};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use std::{
    hash::{Hash, Hasher},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

/// Reserved number of symbols.
///
/// This is where the well known symbol live
/// and internal engine symbols.
const RESERVED_SYMBOL_HASHES: u64 = 127;

fn get_id() -> Option<u64> {
    // Symbol hash.
    //
    // For now this is an incremented u64 number.
    static SYMBOL_HASH_COUNT: AtomicU64 = AtomicU64::new(RESERVED_SYMBOL_HASHES + 1);

    SYMBOL_HASH_COUNT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .ok()
}

/// List of well known symbols.
#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum WellKnown {
    AsyncIterator,
    HasInstance,
    IsConcatSpreadable,
    Iterator,
    Match,
    MatchAll,
    Replace,
    Search,
    Species,
    Split,
    ToPrimitive,
    ToStringTag,
    Unscopables,
}

impl WellKnown {
    const fn description(self) -> JsString {
        match self {
            WellKnown::AsyncIterator => STATIC_JS_STRINGS.symbol_async_iterator(),
            WellKnown::HasInstance => STATIC_JS_STRINGS.symbol_has_instance(),
            WellKnown::IsConcatSpreadable => STATIC_JS_STRINGS.symbol_is_concat_spreadable(),
            WellKnown::Iterator => STATIC_JS_STRINGS.symbol_iterator(),
            WellKnown::Match => STATIC_JS_STRINGS.symbol_match(),
            WellKnown::MatchAll => STATIC_JS_STRINGS.symbol_match_all(),
            WellKnown::Replace => STATIC_JS_STRINGS.symbol_replace(),
            WellKnown::Search => STATIC_JS_STRINGS.symbol_search(),
            WellKnown::Species => STATIC_JS_STRINGS.symbol_species(),
            WellKnown::Split => STATIC_JS_STRINGS.symbol_split(),
            WellKnown::ToPrimitive => STATIC_JS_STRINGS.symbol_to_primitive(),
            WellKnown::ToStringTag => STATIC_JS_STRINGS.symbol_to_string_tag(),
            WellKnown::Unscopables => STATIC_JS_STRINGS.symbol_unscopables(),
        }
    }

    const fn hash(self) -> u64 {
        self as u64
    }

    const fn tag(self) -> usize {
        self as usize
    }

    fn from_tag(tag: usize) -> Option<Self> {
        Self::try_from_primitive(u8::try_from(tag).ok()?).ok()
    }
}

/// The inner representation of a JavaScript symbol.
#[derive(Debug, Clone)]
struct Inner {
    hash: u64,
    description: Option<JsString>,
}

/// This represents a JavaScript symbol primitive.
pub struct JsSymbol {
    repr: Tagged<Inner>,
}

// SAFETY: `JsSymbol` uses `Arc` to do the reference counting, making this type thread-safe.
unsafe impl Send for JsSymbol {}
// SAFETY: `JsSymbol` uses `Arc` to do the reference counting, making this type thread-safe.
unsafe impl Sync for JsSymbol {}

impl Finalize for JsSymbol {}

// Safety: JsSymbol does not contain any objects which needs to be traced,
// so this is safe.
unsafe impl Trace for JsSymbol {
    empty_trace!();
}

macro_rules! well_known_symbols {
    ( $( $(#[$attr:meta])* ($name:ident, $variant:path) ),+$(,)? ) => {
        $(
            $(#[$attr])* pub(crate) const fn $name() -> JsSymbol {
                JsSymbol {
                    repr: Tagged::from_tag($variant.tag()),
                }
            }
        )+
    };
}

impl JsSymbol {
    /// Creates a new symbol.
    ///
    /// Returns `None` if the maximum number of possible symbols has been reached (`u64::MAX`).
    #[inline]
    #[must_use]
    pub fn new(description: Option<JsString>) -> Option<Self> {
        let hash = get_id()?;
        let arc = Arc::new(Inner { hash, description });

        Some(Self {
            // SAFETY: Pointers returned by `Arc::into_raw` must be non-null.
            repr: unsafe { Tagged::from_ptr(Arc::into_raw(arc).cast_mut()) },
        })
    }

    /// Returns the `Symbol`s description.
    #[inline]
    #[must_use]
    pub fn description(&self) -> Option<JsString> {
        match self.repr.unwrap() {
            UnwrappedTagged::Ptr(ptr) => {
                // SAFETY: `ptr` comes from `Arc`, which ensures the validity of the pointer
                // as long as we corrently call `Arc::from_raw` on `Drop`.
                unsafe { ptr.as_ref().description.clone() }
            }
            UnwrappedTagged::Tag(tag) => {
                // SAFETY: All tagged reprs always come from `WellKnown` itself, making
                // this operation always safe.
                let wk = unsafe { WellKnown::from_tag(tag).unwrap_unchecked() };
                Some(wk.description())
            }
        }
    }

    /// Returns the `Symbol`s hash.
    ///
    /// The hash is guaranteed to be unique.
    #[inline]
    #[must_use]
    pub fn hash(&self) -> u64 {
        match self.repr.unwrap() {
            UnwrappedTagged::Ptr(ptr) => {
                // SAFETY: `ptr` comes from `Arc`, which ensures the validity of the pointer
                // as long as we correctly call `Arc::from_raw` on `Drop`.
                unsafe { ptr.as_ref().hash }
            }
            UnwrappedTagged::Tag(tag) => {
                // SAFETY: All tagged reprs always come from `WellKnown` itself, making
                // this operation always safe.
                unsafe { WellKnown::from_tag(tag).unwrap_unchecked().hash() }
            }
        }
    }

    /// Abstract operation `SymbolDescriptiveString ( sym )`
    ///
    /// More info:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-symboldescriptivestring
    #[must_use]
    pub fn descriptive_string(&self) -> JsString {
        self.description().as_ref().map_or_else(
            || js_string!("Symbol()"),
            |desc| js_string!(utf16!("Symbol("), desc, utf16!(")")),
        )
    }

    well_known_symbols! {
        /// Gets the static `JsSymbol` for `"Symbol.asyncIterator"`.
        (async_iterator, WellKnown::AsyncIterator),
        /// Gets the static `JsSymbol` for `"Symbol.hasInstance"`.
        (has_instance, WellKnown::HasInstance),
        /// Gets the static `JsSymbol` for `"Symbol.isConcatSpreadable"`.
        (is_concat_spreadable, WellKnown::IsConcatSpreadable),
        /// Gets the static `JsSymbol` for `"Symbol.iterator"`.
        (iterator, WellKnown::Iterator),
        /// Gets the static `JsSymbol` for `"Symbol.match"`.
        (r#match, WellKnown::Match),
        /// Gets the static `JsSymbol` for `"Symbol.matchAll"`.
        (match_all, WellKnown::MatchAll),
        /// Gets the static `JsSymbol` for `"Symbol.replace"`.
        (replace, WellKnown::Replace),
        /// Gets the static `JsSymbol` for `"Symbol.search"`.
        (search, WellKnown::Search),
        /// Gets the static `JsSymbol` for `"Symbol.species"`.
        (species, WellKnown::Species),
        /// Gets the static `JsSymbol` for `"Symbol.split"`.
        (split, WellKnown::Split),
        /// Gets the static `JsSymbol` for `"Symbol.toPrimitive"`.
        (to_primitive, WellKnown::ToPrimitive),
        /// Gets the static `JsSymbol` for `"Symbol.toStringTag"`.
        (to_string_tag, WellKnown::ToStringTag),
        /// Gets the static `JsSymbol` for `"Symbol.unscopables"`.
        (unscopables, WellKnown::Unscopables),
    }
}

impl Clone for JsSymbol {
    fn clone(&self) -> Self {
        if let UnwrappedTagged::Ptr(ptr) = self.repr.unwrap() {
            // SAFETY: the pointer returned by `self.repr` must be a valid pointer
            // that came from an `Arc::into_raw` call.
            unsafe {
                let arc = Arc::from_raw(ptr.as_ptr().cast_const());
                // Don't need the Arc since `self` is already a copyable pointer, just need to
                // trigger the `clone` impl.
                std::mem::forget(arc.clone());
                std::mem::forget(arc);
            }
        }
        Self { repr: self.repr }
    }
}

impl Drop for JsSymbol {
    fn drop(&mut self) {
        if let UnwrappedTagged::Ptr(ptr) = self.repr.unwrap() {
            // SAFETY: the pointer returned by `self.repr` must be a valid pointer
            // that came from an `Arc::into_raw` call.
            unsafe { drop(Arc::from_raw(ptr.as_ptr().cast_const())) }
        }
    }
}

impl std::fmt::Debug for JsSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsSymbol")
            .field("hash", &self.hash())
            .field("description", &self.description())
            .finish()
    }
}

impl std::fmt::Display for JsSymbol {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.description() {
            Some(desc) => write!(f, "Symbol({})", desc.to_std_string_escaped()),
            None => write!(f, "Symbol()"),
        }
    }
}

impl Eq for JsSymbol {}

impl PartialEq for JsSymbol {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

impl PartialOrd for JsSymbol {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.hash().partial_cmp(&other.hash())
    }
}

impl Ord for JsSymbol {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash().cmp(&other.hash())
    }
}

impl Hash for JsSymbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash().hash(state);
    }
}
