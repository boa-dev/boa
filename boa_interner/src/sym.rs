use boa_macros::utf16;
use indexmap::IndexSet;
use once_cell::sync::Lazy;
use std::num::NonZeroUsize;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The string symbol type for Boa.
///
/// This symbol type is internally a `NonZeroUsize`, which makes it pointer-width in size and it's
/// optimized so that it can occupy 1 pointer width even in an `Option` type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[allow(clippy::unsafe_derive_deserialize)]
pub struct Sym {
    value: NonZeroUsize,
}

impl Sym {
    /// Symbol for the empty string (`""`).
    pub const EMPTY_STRING: Self = unsafe { Self::new_unchecked(1) };

    /// Symbol for the `"arguments"` string.
    pub const ARGUMENTS: Self = unsafe { Self::new_unchecked(2) };

    /// Symbol for the `"await"` string.
    pub const AWAIT: Self = unsafe { Self::new_unchecked(3) };

    /// Symbol for the `"yield"` string.
    pub const YIELD: Self = unsafe { Self::new_unchecked(4) };

    /// Symbol for the `"eval"` string.
    pub const EVAL: Self = unsafe { Self::new_unchecked(5) };

    /// Symbol for the `"default"` string.
    pub const DEFAULT: Self = unsafe { Self::new_unchecked(6) };

    /// Symbol for the `"null"` string.
    pub const NULL: Self = unsafe { Self::new_unchecked(7) };

    /// Symbol for the `"RegExp"` string.
    pub const REGEXP: Self = unsafe { Self::new_unchecked(8) };

    /// Symbol for the `"get"` string.
    pub const GET: Self = unsafe { Self::new_unchecked(9) };

    /// Symbol for the `"set"` string.
    pub const SET: Self = unsafe { Self::new_unchecked(10) };

    /// Symbol for the `"<main>"` string.
    pub const MAIN: Self = unsafe { Self::new_unchecked(11) };

    /// Symbol for the `"raw"` string.
    pub const RAW: Self = unsafe { Self::new_unchecked(12) };

    /// Symbol for the `"static"` string.
    pub const STATIC: Self = unsafe { Self::new_unchecked(13) };

    /// Symbol for the `"prototype"` string.
    pub const PROTOTYPE: Self = unsafe { Self::new_unchecked(14) };

    /// Symbol for the `"constructor"` string.
    pub const CONSTRUCTOR: Self = unsafe { Self::new_unchecked(15) };

    /// Symbol for the `"implements"` string.
    pub const IMPLEMENTS: Self = unsafe { Self::new_unchecked(16) };

    /// Symbol for the `"interface"` string.
    pub const INTERFACE: Self = unsafe { Self::new_unchecked(17) };

    /// Symbol for the `"let"` string.
    pub const LET: Self = unsafe { Self::new_unchecked(18) };

    /// Symbol for the `"package"` string.
    pub const PACKAGE: Self = unsafe { Self::new_unchecked(19) };

    /// Symbol for the `"private"` string.
    pub const PRIVATE: Self = unsafe { Self::new_unchecked(20) };

    /// Symbol for the `"protected"` string.
    pub const PROTECTED: Self = unsafe { Self::new_unchecked(21) };

    /// Symbol for the `"public"` string.
    pub const PUBLIC: Self = unsafe { Self::new_unchecked(22) };

    /// Symbol for the `"anonymous"` string.
    pub const ANONYMOUS: Self = unsafe { Self::new_unchecked(23) };

    /// Symbol for the `"true"` string.
    pub const TRUE: Self = unsafe { Self::new_unchecked(24) };

    /// Symbol for the `"false"` string.
    pub const FALSE: Self = unsafe { Self::new_unchecked(25) };

    /// Symbol for the `"async"` string.
    pub const ASYNC: Self = unsafe { Self::new_unchecked(26) };

    /// Symbol for the `"of"` string.
    pub const OF: Self = unsafe { Self::new_unchecked(27) };

    /// Symbol for the `"target"` string.
    pub const TARGET: Self = unsafe { Self::new_unchecked(28) };

    /// Symbol for the `"__proto__"` string.
    pub const __PROTO__: Self = unsafe { Self::new_unchecked(29) };

    /// Creates a new [`Sym`] from the provided `value`, or returns `None` if `index` is zero.
    #[inline]
    pub(super) fn new(value: usize) -> Option<Self> {
        NonZeroUsize::new(value).map(|value| Self { value })
    }

    /// Creates a new [`Sym`] from the provided `value`, without checking if `value` is not zero
    ///
    /// # Safety
    ///
    /// `value` must not be zero.
    #[inline]
    pub(super) const unsafe fn new_unchecked(value: usize) -> Self {
        Self {
            value:
            // SAFETY: The caller must ensure the invariants of the function.
            unsafe {
                NonZeroUsize::new_unchecked(value)
            },
        }
    }

    /// Returns the internal value of the [`Sym`]
    #[inline]
    #[must_use]
    pub const fn get(self) -> usize {
        self.value.get()
    }
}

macro_rules! create_static_strings {
    ( $( $s:literal ),+$(,)? ) => {
        /// Ordered set of commonly used static `UTF-8` strings.
        ///
        /// # Note
        ///
        /// `COMMON_STRINGS_UTF8`, `COMMON_STRINGS_UTF16` and the constants
        /// defined in [`Sym`] must always be in sync.
        pub(super) static COMMON_STRINGS_UTF8: phf::OrderedSet<&'static str> = {
            const COMMON_STRINGS: phf::OrderedSet<&'static str> = phf::phf_ordered_set! {
                $( $s ),+
            };
            // A `COMMON_STRINGS` of size `usize::MAX` would cause an overflow on our `Interner`
            sa::const_assert!(COMMON_STRINGS.len() < usize::MAX);
            COMMON_STRINGS
        };

        /// Ordered set of commonly used static `UTF-16` strings.
        ///
        /// # Note
        ///
        /// `COMMON_STRINGS_UTF8`, `COMMON_STRINGS_UTF16` and the constants
        /// defined in [`Sym`] must always be in sync.
        // FIXME: use phf when const expressions are allowed. https://github.com/rust-phf/rust-phf/issues/188
        pub(super) static COMMON_STRINGS_UTF16: Lazy<IndexSet<&'static [u16]>> = Lazy::new(|| {
            let mut set = IndexSet::with_capacity(COMMON_STRINGS_UTF8.len());
            $( set.insert(utf16!($s)); )+
            set
        });
    };
}

create_static_strings! {
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
    "raw",
    "static",
    "prototype",
    "constructor",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "anonymous",
    "true",
    "false",
    "async",
    "of",
    "target",
    "__proto__",
}
