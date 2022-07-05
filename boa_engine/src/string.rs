#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::missing_safety_doc)]
#![allow(unstable_name_collisions)]

//! A UTF-16–encoded, reference counted, immutable string.
//!
//! This module contains the [`JsString`] type, the [`js_string`] macro and
//! the [`utf16`] macro.
//!
//! The [`js_string`] macro is almost always used when you need to create a new
//! [`JsString`], and the [`utf16`] macro is used for const conversions of
//! string literals to UTF-16.

use crate::{builtins::string::is_trimmable_whitespace, JsBigInt};
use boa_gc::{unsafe_empty_trace, Finalize, Trace};
pub use utf16_lit::utf16;

use rustc_hash::{FxHashMap, FxHasher};
use std::{
    alloc::{alloc, dealloc, Layout},
    borrow::Borrow,
    cell::Cell,
    hash::BuildHasherDefault,
    hash::{Hash, Hasher},
    ops::{Deref, Index},
    ptr::{self, NonNull},
    slice::SliceIndex,
};

/// Utility macro to create a `JsString`.
///
/// # Examples
///
/// You can call the macro without arguments to create an empty [`JsString`]:
///
/// ```
/// use boa_engine::js_string;
/// use boa_engine::string::utf16;
///
/// let empty_str = js_string!();
/// assert!(empty_str.is_empty());
/// ```
///
///
/// You can create a [`JsString`] from a string literal, which completely skips
/// the runtime conversion from [`&str`] to [`&\[u16\]`]:
///
/// ```
/// # use boa_engine::js_string;
/// # use boa_engine::string::utf16;
/// let hw = js_string!("Hello, world!");
/// assert_eq!(&hw, &utf16!("Hello, world!"));
/// ```
///
/// Any [`\[u16\]`] slice is a valid [`JsString`], including unpaired surrogates:
///
/// ```
/// # use boa_engine::js_string;
/// let array = js_string!(&[0xD8AFu16, 0x00A0, 0xD8FF, 0x00F0]);
/// ```
///
/// You can also pass it any number of [`&\[u16\]`] as arguments to create a new
/// [`JsString`] with the concatenation of every slice:
///
/// ```
/// # use boa_engine::js_string;
/// # use boa_engine::string::utf16;
/// const NAME: &[u16]  = &utf16!("human! ");
/// let greeting = js_string!("Hello, ");
/// let msg = js_string!(&greeting, &NAME, &utf16!("Nice to meet you!"));
///
/// assert_eq!(&msg, &utf16!("Hello, human! Nice to meet you!"));
/// ```
#[macro_export]
macro_rules! js_string {
    () => {
        $crate::JsString::default()
    };
    ($s:literal) => {
        $crate::JsString::from(&$crate::string::utf16!($s))
    };
    ($s:expr) => {
        $crate::JsString::from($s)
    };
    ( $x:expr, $y:expr ) => {
        $crate::JsString::concat($x, $y)
    };
    ( $( $s:expr ),+ ) => {
        $crate::JsString::concat_array(&[ $( $s ),+ ])
    };
}

const COMMON_STRINGS: &[&[u16]] = &[
    // Empty string
    utf16!("").as_slice(),
    // &Misc
    &utf16!(","),
    &utf16!(":"),
    // Generic use
    &utf16!("name"),
    &utf16!("length"),
    &utf16!("arguments"),
    &utf16!("prototype"),
    &utf16!("constructor"),
    &utf16!("return"),
    &utf16!("throw"),
    &utf16!("global"),
    &utf16!("globalThis"),
    // typeof
    &utf16!("null"),
    &utf16!("undefined"),
    &utf16!("number"),
    &utf16!("string"),
    &utf16!("symbol"),
    &utf16!("bigint"),
    &utf16!("object"),
    &utf16!("function"),
    // Property descriptor
    &utf16!("value"),
    &utf16!("get"),
    &utf16!("set"),
    &utf16!("writable"),
    &utf16!("enumerable"),
    &utf16!("configurable"),
    // Object object
    &utf16!("Object"),
    &utf16!("assign"),
    &utf16!("create"),
    &utf16!("toString"),
    &utf16!("valueOf"),
    &utf16!("is"),
    &utf16!("seal"),
    &utf16!("isSealed"),
    &utf16!("freeze"),
    &utf16!("isFrozen"),
    &utf16!("isExtensible"),
    &utf16!("hasOwnProperty"),
    &utf16!("isPrototypeOf"),
    &utf16!("setPrototypeOf"),
    &utf16!("getPrototypeOf"),
    &utf16!("defineProperty"),
    &utf16!("defineProperties"),
    &utf16!("deleteProperty"),
    &utf16!("construct"),
    &utf16!("hasOwn"),
    &utf16!("ownKeys"),
    &utf16!("keys"),
    &utf16!("values"),
    &utf16!("entries"),
    &utf16!("fromEntries"),
    // Function object
    &utf16!("Function"),
    &utf16!("apply"),
    &utf16!("bind"),
    &utf16!("call"),
    // Generator object
    &utf16!("Generator"),
    // Array object
    &utf16!("Array"),
    &utf16!("at"),
    &utf16!("from"),
    &utf16!("isArray"),
    &utf16!("of"),
    &utf16!("copyWithin"),
    &utf16!("entries"),
    &utf16!("every"),
    &utf16!("fill"),
    &utf16!("filter"),
    &utf16!("find"),
    &utf16!("findIndex"),
    &utf16!("findLast"),
    &utf16!("findLastIndex"),
    &utf16!("flat"),
    &utf16!("flatMap"),
    &utf16!("forEach"),
    &utf16!("includes"),
    &utf16!("indexOf"),
    &utf16!("join"),
    &utf16!("map"),
    &utf16!("next"),
    &utf16!("reduce"),
    &utf16!("reduceRight"),
    &utf16!("reverse"),
    &utf16!("shift"),
    &utf16!("slice"),
    &utf16!("splice"),
    &utf16!("some"),
    &utf16!("sort"),
    &utf16!("unshift"),
    &utf16!("push"),
    &utf16!("pop"),
    // String object
    &utf16!("String"),
    &utf16!("charAt"),
    &utf16!("charCodeAt"),
    &utf16!("codePointAt"),
    &utf16!("concat"),
    &utf16!("endsWith"),
    &utf16!("fromCharCode"),
    &utf16!("fromCodePoint"),
    &utf16!("includes"),
    &utf16!("indexOf"),
    &utf16!("lastIndexOf"),
    &utf16!("match"),
    &utf16!("matchAll"),
    &utf16!("normalize"),
    &utf16!("padEnd"),
    &utf16!("padStart"),
    &utf16!("raw"),
    &utf16!("repeat"),
    &utf16!("replace"),
    &utf16!("replaceAll"),
    &utf16!("search"),
    &utf16!("slice"),
    &utf16!("split"),
    &utf16!("startsWith"),
    &utf16!("substr"),
    &utf16!("substring"),
    &utf16!("toLocaleString"),
    &utf16!("toLowerCase"),
    &utf16!("toUpperCase"),
    &utf16!("trim"),
    &utf16!("trimEnd"),
    &utf16!("trimStart"),
    // Number object
    &utf16!("Number"),
    &utf16!("Infinity"),
    &utf16!("NaN"),
    &utf16!("parseInt"),
    &utf16!("parseFloat"),
    &utf16!("isFinite"),
    &utf16!("isNaN"),
    &utf16!("parseInt"),
    &utf16!("EPSILON"),
    &utf16!("MAX_SAFE_INTEGER"),
    &utf16!("MIN_SAFE_INTEGER"),
    &utf16!("MAX_VALUE"),
    &utf16!("MIN_VALUE"),
    &utf16!("isSafeInteger"),
    &utf16!("isInteger"),
    &utf16!("toExponential"),
    &utf16!("toFixed"),
    &utf16!("toPrecision"),
    // Boolean object
    &utf16!("Boolean"),
    // BigInt object
    &utf16!("BigInt"),
    &utf16!("asIntN"),
    &utf16!("asUintN"),
    // RegExp object
    &utf16!("RegExp"),
    &utf16!("exec"),
    &utf16!("test"),
    &utf16!("flags"),
    &utf16!("index"),
    &utf16!("lastIndex"),
    &utf16!("hasIndices"),
    &utf16!("ignoreCase"),
    &utf16!("multiline"),
    &utf16!("dotAll"),
    &utf16!("unicode"),
    &utf16!("sticky"),
    &utf16!("source"),
    &utf16!("get hasIndices"),
    &utf16!("get global"),
    &utf16!("get ignoreCase"),
    &utf16!("get multiline"),
    &utf16!("get dotAll"),
    &utf16!("get unicode"),
    &utf16!("get sticky"),
    &utf16!("get flags"),
    &utf16!("get source"),
    // Symbol object
    &utf16!("Symbol"),
    &utf16!("for"),
    &utf16!("keyFor"),
    &utf16!("description"),
    &utf16!("asyncIterator"),
    &utf16!("hasInstance"),
    &utf16!("species"),
    &utf16!("Symbol.species"),
    &utf16!("unscopables"),
    &utf16!("iterator"),
    &utf16!("Symbol.iterator"),
    &utf16!("Symbol.match"),
    &utf16!("[Symbol.match]"),
    &utf16!("Symbol.matchAll"),
    &utf16!("Symbol.replace"),
    &utf16!("[Symbol.replace]"),
    &utf16!("Symbol.search"),
    &utf16!("[Symbol.search]"),
    &utf16!("Symbol.split"),
    &utf16!("[Symbol.split]"),
    &utf16!("toStringTag"),
    &utf16!("toPrimitive"),
    &utf16!("get description"),
    // Map object
    &utf16!("Map"),
    &utf16!("clear"),
    &utf16!("delete"),
    &utf16!("has"),
    &utf16!("size"),
    // Set object
    &utf16!("Set"),
    &utf16!("add"),
    // Reflect object
    &utf16!("Reflect"),
    // Proxy object
    &utf16!("Proxy"),
    &utf16!("revocable"),
    // Error objects
    &utf16!("Error"),
    &utf16!("AggregateError"),
    &utf16!("TypeError"),
    &utf16!("RangeError"),
    &utf16!("SyntaxError"),
    &utf16!("ReferenceError"),
    &utf16!("EvalError"),
    &utf16!("ThrowTypeError"),
    &utf16!("URIError"),
    &utf16!("message"),
    // Date object
    &utf16!("Date"),
    &utf16!("toJSON"),
    &utf16!("getDate"),
    &utf16!("getDay"),
    &utf16!("getFullYear"),
    &utf16!("getHours"),
    &utf16!("getMilliseconds"),
    &utf16!("getMinutes"),
    &utf16!("getMonth"),
    &utf16!("getSeconds"),
    &utf16!("getTime"),
    &utf16!("getYear"),
    &utf16!("getUTCDate"),
    &utf16!("getUTCDay"),
    &utf16!("getUTCFullYear"),
    &utf16!("getUTCHours"),
    &utf16!("getUTCMinutes"),
    &utf16!("getUTCMonth"),
    &utf16!("getUTCSeconds"),
    &utf16!("setDate"),
    &utf16!("setFullYear"),
    &utf16!("setHours"),
    &utf16!("setMilliseconds"),
    &utf16!("setMinutes"),
    &utf16!("setMonth"),
    &utf16!("setSeconds"),
    &utf16!("setYear"),
    &utf16!("setTime"),
    &utf16!("setUTCDate"),
    &utf16!("setUTCFullYear"),
    &utf16!("setUTCHours"),
    &utf16!("setUTCMinutes"),
    &utf16!("setUTCMonth"),
    &utf16!("setUTCSeconds"),
    &utf16!("toDateString"),
    &utf16!("toGMTString"),
    &utf16!("toISOString"),
    &utf16!("toTimeString"),
    &utf16!("toUTCString"),
    &utf16!("now"),
    &utf16!("UTC"),
    // JSON object
    &utf16!("JSON"),
    &utf16!("parse"),
    &utf16!("stringify"),
    // Iterator object
    &utf16!("Array Iterator"),
    &utf16!("Set Iterator"),
    &utf16!("String Iterator"),
    &utf16!("Map Iterator"),
    &utf16!("For In Iterator"),
    // Math object
    &utf16!("Math"),
    &utf16!("LN10"),
    &utf16!("LN2"),
    &utf16!("LOG10E"),
    &utf16!("LOG2E"),
    &utf16!("PI"),
    &utf16!("SQRT1_2"),
    &utf16!("SQRT2"),
    &utf16!("abs"),
    &utf16!("acos"),
    &utf16!("acosh"),
    &utf16!("asin"),
    &utf16!("asinh"),
    &utf16!("atan"),
    &utf16!("atanh"),
    &utf16!("atan2"),
    &utf16!("cbrt"),
    &utf16!("ceil"),
    &utf16!("clz32"),
    &utf16!("cos"),
    &utf16!("cosh"),
    &utf16!("exp"),
    &utf16!("expm1"),
    &utf16!("floor"),
    &utf16!("fround"),
    &utf16!("hypot"),
    &utf16!("imul"),
    &utf16!("log"),
    &utf16!("log1p"),
    &utf16!("log10"),
    &utf16!("log2"),
    &utf16!("max"),
    &utf16!("min"),
    &utf16!("pow"),
    &utf16!("random"),
    &utf16!("round"),
    &utf16!("sign"),
    &utf16!("sin"),
    &utf16!("sinh"),
    &utf16!("sqrt"),
    &utf16!("tan"),
    &utf16!("tanh"),
    &utf16!("trunc"),
    // Intl object
    &utf16!("Intl"),
    &utf16!("DateTimeFormat"),
    // TypedArray object
    &utf16!("TypedArray"),
    &utf16!("ArrayBuffer"),
    &utf16!("Int8Array"),
    &utf16!("Uint8Array"),
    &utf16!("Int16Array"),
    &utf16!("Uint16Array"),
    &utf16!("Int32Array"),
    &utf16!("Uint32Array"),
    &utf16!("BigInt64Array"),
    &utf16!("BigUint64Array"),
    &utf16!("Float32Array"),
    &utf16!("Float64Array"),
    &utf16!("buffer"),
    &utf16!("byteLength"),
    &utf16!("byteOffset"),
    &utf16!("isView"),
    &utf16!("subarray"),
    &utf16!("get byteLength"),
    &utf16!("get buffer"),
    &utf16!("get byteOffset"),
    &utf16!("get size"),
    &utf16!("get length"),
    // DataView object
    &utf16!("DataView"),
    &utf16!("getBigInt64"),
    &utf16!("getBigUint64"),
    &utf16!("getFloat32"),
    &utf16!("getFloat64"),
    &utf16!("getInt8"),
    &utf16!("getInt16"),
    &utf16!("getInt32"),
    &utf16!("getUint8"),
    &utf16!("getUint16"),
    &utf16!("getUint32"),
    &utf16!("setBigInt64"),
    &utf16!("setBigUint64"),
    &utf16!("setFloat32"),
    &utf16!("setFloat64"),
    &utf16!("setInt8"),
    &utf16!("setInt16"),
    &utf16!("setInt32"),
    &utf16!("setUint8"),
    &utf16!("setUint16"),
    &utf16!("setUint32"),
    // Console object
    &utf16!("console"),
    &utf16!("assert"),
    &utf16!("debug"),
    &utf16!("error"),
    &utf16!("info"),
    &utf16!("trace"),
    &utf16!("warn"),
    &utf16!("exception"),
    &utf16!("count"),
    &utf16!("countReset"),
    &utf16!("group"),
    &utf16!("groupCollapsed"),
    &utf16!("groupEnd"),
    &utf16!("time"),
    &utf16!("timeLog"),
    &utf16!("timeEnd"),
    &utf16!("dir"),
    &utf16!("dirxml"),
    // Minified name
    &utf16!("a"),
    &utf16!("b"),
    &utf16!("c"),
    &utf16!("d"),
    &utf16!("e"),
    &utf16!("f"),
    &utf16!("g"),
    &utf16!("h"),
    &utf16!("i"),
    &utf16!("j"),
    &utf16!("k"),
    &utf16!("l"),
    &utf16!("m"),
    &utf16!("n"),
    &utf16!("o"),
    &utf16!("p"),
    &utf16!("q"),
    &utf16!("r"),
    &utf16!("s"),
    &utf16!("t"),
    &utf16!("u"),
    &utf16!("v"),
    &utf16!("w"),
    &utf16!("x"),
    &utf16!("y"),
    &utf16!("z"),
    &utf16!("A"),
    &utf16!("B"),
    &utf16!("C"),
    &utf16!("D"),
    &utf16!("E"),
    &utf16!("F"),
    &utf16!("G"),
    &utf16!("H"),
    &utf16!("I"),
    &utf16!("J"),
    &utf16!("K"),
    &utf16!("L"),
    &utf16!("M"),
    &utf16!("N"),
    &utf16!("O"),
    &utf16!("P"),
    &utf16!("Q"),
    &utf16!("R"),
    &utf16!("S"),
    &utf16!("T"),
    &utf16!("U"),
    &utf16!("V"),
    &utf16!("W"),
    &utf16!("X"),
    &utf16!("Y"),
    &utf16!("Z"),
    &utf16!("_"),
    &utf16!("$"),
];

const MAX_COMMON_STRING_LENGTH: usize = {
    let mut max = 0;
    let mut i = 0;
    while i < COMMON_STRINGS.len() {
        let len = COMMON_STRINGS[i].len();
        if len > max {
            max = len;
        }
        i += 1;
    }
    max
};

thread_local! {
    static COMMON_STRINGS_CACHE: FxHashMap<&'static [u16], JsString> = {
        let mut constants = FxHashMap::with_capacity_and_hasher(
            COMMON_STRINGS.len(),
            BuildHasherDefault::<FxHasher>::default(),
        );

        for (idx, &s) in COMMON_STRINGS.iter().enumerate() {
            // Safety:
            // As we're just building a cache of `JsString` indices
            // to access the stored `COMMON_STRINGS`, this
            // cannot generate invalid `TaggedInner`s, since `idx` is always
            // a valid index in `COMMON_STRINGS`.
            let v = unsafe {
                JsString {
                    ptr: TaggedJsString::new_static(idx),
                }
            };
            constants.insert(s, v);
        }

        constants
    };
}

/// Represents a Unicode codepoint within a [`JsString`], which could be a valid
/// '[Unicode scalar value]', or an unpaired surrogate.
///
/// [Unicode scalar value]: https://www.unicode.org/glossary/#unicode_scalar_value
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CodePoint {
    Unicode(char),
    UnpairedSurrogate(u16),
}

impl CodePoint {
    /// Get the number of UTF-16 code units needed to encode
    /// this code point.
    pub(crate) fn code_unit_count(self) -> usize {
        match self {
            Self::Unicode(c) => c.len_utf16(),
            Self::UnpairedSurrogate(_) => 1,
        }
    }

    /// Convert the code point to its [`u32`] representation.
    pub(crate) fn as_u32(self) -> u32 {
        match self {
            Self::Unicode(c) => u32::from(c),
            Self::UnpairedSurrogate(surr) => u32::from(surr),
        }
    }

    /// If the code point represents a valid 'Unicode scalar value', returns
    /// its [`char`] representation, otherwise returns [`None`] on unpaired
    /// surrogates.
    pub(crate) fn as_char(self) -> Option<char> {
        match self {
            Self::Unicode(c) => Some(c),
            Self::UnpairedSurrogate(_) => None,
        }
    }

    /// Encodes this code point as UTF-16 into the provided u16 buffer, and then
    /// returns the subslice of the buffer that contains the encoded character.
    ///
    /// # Panics
    ///
    /// Panics if the buffer is not large enough. A buffer of length 2 is large
    /// enough to encode any code point.
    pub(crate) fn encode_utf16(self, dst: &mut [u16]) -> &mut [u16] {
        match self {
            CodePoint::Unicode(c) => c.encode_utf16(dst),
            CodePoint::UnpairedSurrogate(surr) => {
                dst[0] = surr;
                &mut dst[0..=0]
            }
        }
    }
}

/// The raw representation of a [`JsString`] in the heap.
#[repr(C)]
struct RawJsString {
    /// The UTF-16 length.
    len: usize,

    /// The number of references to the string.
    ///
    /// When this reaches `0` the string is deallocated.
    refcount: Cell<usize>,

    /// An empty array which is used to get the offset of string data.
    data: [u16; 0],
}

// Safety: `JsString` does not contain any objects which needs to be traced,
// so this is safe.
unsafe impl Trace for JsString {
    unsafe_empty_trace!();
}

/// This struct uses a technique called tagged pointer to benefit from the fact that newly
/// allocated pointers are always word aligned on 64-bits platforms, making it impossible
/// to have a LSB equal to 1. More details about this technique on the article of Wikipedia
/// about [tagged pointers][tagged_wp].
///
/// # Representation
///
/// If the LSB of the internal [`NonNull<RawJsString>`] is set (1), then the pointer address represents
/// an index value for [`COMMON_STRINGS`], where the remaining MSBs store the index.
/// Otherwise, the whole pointer represents the address of a heap allocated [`RawJsString`].
///
/// It uses [`NonNull`], which guarantees that [`TaggedJsString`] (and subsequently [`JsString`])
/// can use the "null pointer optimization" to optimize the size of [`Option<TaggedJsString>`].
///
/// # Provenance
///
/// This struct stores a [`NonNull<RawJsString>`] instead of a [`NonZeroUsize`][std::num::NonZeroUsize]
/// in order to preserve the provenance of our valid heap pointers.
/// On the other hand, all index values are just casted to invalid pointers,
/// because we don't need to preserve the provenance of [`usize`] indices.
///
/// [tagged_wp]: https://en.wikipedia.org/wiki/Tagged_pointer
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct TaggedJsString(NonNull<RawJsString>);

impl TaggedJsString {
    /// Creates a new [`TaggedJsString`] from a pointer to a valid [`RawJsString`].
    ///
    /// # Safety
    ///
    /// `inner` must point to a valid instance of [`RawJsString`], which should be
    /// deallocated only by [`JsString`].
    #[inline]
    unsafe fn new_heap(inner: NonNull<RawJsString>) -> Self {
        Self(inner)
    }

    /// Creates a new static [`TaggedJsString`] from the index of an element inside
    /// [`COMMON_STRINGS`].
    ///
    /// # Safety
    ///
    /// `idx` must be a valid index on [`COMMON_STRINGS`].
    #[inline]
    const unsafe fn new_static(idx: usize) -> Self {
        // SAFETY:
        // The operation `(idx << 1) | 1` sets the least significant
        // bit to 1, meaning any pointer (valid or invalid) created using
        // this address cannot be null.
        unsafe { Self(NonNull::new_unchecked(sptr::invalid_mut((idx << 1) | 1))) }
    }

    /// Checks if [`TaggedJsString`] contains an index for [`COMMON_STRINGS`].
    #[inline]
    fn is_static(self) -> bool {
        (self.0.as_ptr() as usize) & 1 == 1
    }

    /// Returns a reference to a string stored on the heap,
    /// without checking if its internal pointer is valid.
    ///
    /// # Safety
    ///
    /// `self` must be a heap allocated [`RawJsString`].
    #[inline]
    const unsafe fn get_heap_unchecked(self) -> NonNull<RawJsString> {
        self.0
    }

    /// Returns the string inside [`COMMON_STRINGS`] corresponding to the
    /// index inside [`TaggedJsString`], without checking its validity.
    ///
    /// # Safety
    ///
    /// `self` must not be a pointer to a heap allocated [`RawJsString`], and it
    /// must be a valid index inside [`COMMON_STRINGS`].
    #[inline]
    unsafe fn get_static_unchecked(self) -> &'static [u16] {
        // SAFETY:
        // The caller must ensure `self` is a valid index inside
        // `COMMON_STRINGS`.
        unsafe { COMMON_STRINGS.get_unchecked((self.0.as_ptr() as usize) >> 1) }
    }
}
/// A UTF-16–encoded, reference counted, immutable string.
///
/// This is pretty similar to a <code>[Rc][std::rc::Rc]\<[\[u16\]][std::slice]\></code>,
/// but without the length metadata associated with the [`Rc`][std::rc::Rc] fat pointer.
/// Instead, the length of every string
/// is stored on the heap, along with its reference counter and its data.
///
/// We define some commonly used string constants in an interner. For these
/// strings, we don't allocate memory on the heap to reduce the overhead of
/// memory allocation and reference counting.
///
/// # Deref
///
/// [`JsString`] implements <code>[Deref]<Target = [\[u16\]][std::slice]></code>, inheriting
/// all of [`\[u16\]`][std::slice]'s methods.
#[derive(Finalize)]
pub struct JsString {
    ptr: TaggedJsString,
}

/// Enum representing either a reference to a heap allocated [`RawJsString`]
/// or a static reference to a [`\[u16\]`][std::slice] inside [`COMMON_STRINGS`].
enum JsStringPtrKind<'a> {
    // A string allocated on the heap.
    Heap(&'a mut RawJsString),
    // A static string slice.
    Static(&'static [u16]),
}

impl JsString {
    /// Returns the inner pointer data, unwrapping its tagged data
    /// if the pointer contains a static index for [`COMMON_STRINGS`].
    #[inline]
    fn ptr(&self) -> JsStringPtrKind<'_> {
        // Check the first bit to 1.
        if self.ptr.is_static() {
            // Safety: We already checked.
            JsStringPtrKind::Static(unsafe { self.ptr.get_static_unchecked() })
        } else {
            // Safety: We already checked.
            JsStringPtrKind::Heap(unsafe { self.ptr.get_heap_unchecked().as_mut() })
        }
    }

    // This is marked as safe because it is always valid to call this function to request
    // any number of `u16`, since this function ought to fail on an OOM error.
    /// Allocates a new [`RawJsString`] with an internal capacity of `str_len` chars.
    fn allocate_inner(str_len: usize) -> NonNull<RawJsString> {
        // We get the layout of the `Inner` type and we extend by the size
        // of the string array.
        let (layout, offset) = Layout::array::<u16>(str_len)
            .and_then(|arr| Layout::new::<RawJsString>().extend(arr))
            .map(|(layout, offset)| (layout.pad_to_align(), offset))
            .expect("failed to create memory layout");

        // SAFETY:
        // - The layout size of `Inner` is never zero, since it has to store
        // the length of the string and the reference count.
        let inner = unsafe { alloc(layout).cast::<RawJsString>() };

        // We need to verify that the pointer returned by `alloc` is not null, otherwise
        // we should abort, since an allocation error is pretty unrecoverable for us
        // right now.
        let inner = NonNull::new(inner).unwrap_or_else(|| std::alloc::handle_alloc_error(layout));

        // SAFETY:
        // `NonNull` verified for us that the pointer returned by `alloc` is valid,
        // meaning we can write to its pointed memory.
        unsafe {
            // Write the first part, the Inner.
            inner.as_ptr().write(RawJsString {
                len: str_len,
                refcount: Cell::new(1),
                data: [0; 0],
            });
        }

        debug_assert!({
            let inner = inner.as_ptr();
            // SAFETY:
            // - `inner` must be a valid pointer, since it comes from a `NonNull`,
            // meaning we can safely dereference it to `Inner`.
            // - `offset` should point us to the beginning of the array,
            // and since we requested an `Inner` layout with a trailing
            // `[u16; str_len]`, the memory of the array must be in the `usize`
            // range for the allocation to succeed.
            unsafe {
                let data = (*inner).data.as_ptr();
                ptr::eq(inner.cast::<u8>().add(offset).cast(), data)
            }
        });

        inner
    }

    /// Creates a new [`JsString`] from `data`, without checking if the string is
    /// in the interner.
    fn from_slice_skip_interning(data: &[u16]) -> Self {
        let count = data.len();
        let ptr = Self::allocate_inner(count);
        // SAFETY:
        // - We read `count = data.len()` elements from `data`, which is within the bounds
        //   of the slice.
        // - `allocate_inner` must allocate at least `count` elements, which
        //   allows us to safely write at least `count` elements.
        // - `allocate_inner` should already take care of the alignment of `ptr`,
        //   and `data` must be aligned to be a valid slice.
        // - `allocate_inner` must return a valid pointer to newly allocated memory,
        //    meaning `ptr` and `data` should never overlap.
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), (*ptr.as_ptr()).data.as_mut_ptr(), count);
        }
        Self {
            // Safety: We already know it's a valid heap pointer.
            ptr: unsafe { TaggedJsString::new_heap(ptr) },
        }
    }

    /// Obtains the underlying [`&[u16]`][std::slice] slice of a [`JsString`]
    pub fn as_slice(&self) -> &[u16] {
        self
    }

    /// Creates a new [`JsString`] from the concatenation of `x` and `y`.
    pub fn concat(x: &[u16], y: &[u16]) -> Self {
        Self::concat_array(&[x, y])
    }

    /// Creates a new [`JsString`] from the concatenation of every element of
    /// `strings`.
    pub fn concat_array(strings: &[&[u16]]) -> Self {
        let full_count = strings.iter().fold(0, |len, s| len + s.len());

        let ptr = Self::allocate_inner(full_count);

        let string = {
            // SAFETY:
            // `ptr` being a `NonNull` ensures that a dereference of its underlying
            // pointer is always valid.
            let mut data = unsafe { (*ptr.as_ptr()).data.as_mut_ptr() };
            for string in strings {
                let count = string.len();
                // SAFETY:
                // The sum of all `count` for each `string` equals `full_count`,
                // and since we're iteratively writing each of them to `data`,
                // `copy_non_overlapping` always stays in-bounds for `count` reads
                // of each string and `full_count` writes to `data`.
                //
                // Each `string` must be properly aligned to be a valid
                // slice, and `data` must be properly aligned by `allocate_inner`.
                //
                // `allocate_inner` must return a valid pointer to newly allocated memory,
                // meaning `ptr` and all `string`s should never overlap.
                unsafe {
                    ptr::copy_nonoverlapping(string.as_ptr(), data, count);
                    data = data.add(count);
                }
            }
            Self {
                // Safety: We already know it's a valid heap pointer.
                ptr: unsafe { TaggedJsString::new_heap(ptr) },
            }
        };

        if string.len() <= MAX_COMMON_STRING_LENGTH {
            if let Some(constant) = COMMON_STRINGS_CACHE.with(|c| c.get(&string[..]).cloned()) {
                return constant;
            }
        }

        string
    }

    /// Decodes a [`JsString`] into a [`String`], replacing invalid data with
    /// its escaped representation in 4 digit hexadecimal.
    pub fn to_std_string_escaped(&self) -> String {
        self.to_string_escaped()
    }

    /// Decodes a [`JsString`] into a [`String`], returning
    /// [`FromUtf16Error`][std::string::FromUtf16Error] if it contains any invalid data.
    pub fn to_std_string(&self) -> Result<String, std::string::FromUtf16Error> {
        String::from_utf16(self)
    }

    /// Gets an iterator of all the Unicode codepoints of a [`JsString`].
    pub(crate) fn code_points(&self) -> impl Iterator<Item = CodePoint> + '_ {
        char::decode_utf16(self.iter().copied()).map(|res| match res {
            Ok(c) => CodePoint::Unicode(c),
            Err(e) => CodePoint::UnpairedSurrogate(e.unpaired_surrogate()),
        })
    }

    /// Abstract operation `StringIndexOf ( string, searchValue, fromIndex )`
    ///
    /// Note: Instead of returning an isize with `-1` as the "not found" value,
    /// we make use of the type system and return <code>[Option]\<usize\></code>
    /// with [`None`] as the "not found" value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringindexof
    pub(crate) fn index_of(&self, search_value: &[u16], from_index: usize) -> Option<usize> {
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
    pub(crate) fn code_point_at(&self, position: usize) -> CodePoint {
        // 1. Let size be the length of string.
        let size = self.len();

        // 2. Assert: position ≥ 0 and position < size.
        // position >= 0 ensured by position: usize
        assert!(position < size);

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

        // We can skip the checks and instead use the `char::decode_utf16` function to take care of that for us.
        let code_point = self
            .get(position..=position + 1)
            .unwrap_or(&self[position..=position]);

        match char::decode_utf16(code_point.iter().copied())
            .next()
            .expect("code_point always has a value")
        {
            Ok(c) => CodePoint::Unicode(c),
            Err(e) => CodePoint::UnpairedSurrogate(e.unpaired_surrogate()),
        }
    }

    /// Abstract operation `StringToNumber ( str )`
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringtonumber
    #[allow(clippy::question_mark)]
    pub(crate) fn to_number(&self) -> f64 {
        // 1. Let text be ! StringToCodePoints(str).
        // 2. Let literal be ParseText(text, StringNumericLiteral).
        let string = if let Ok(string) = self.to_std_string() {
            string
        } else {
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
            let mut value = 0.0;
            for c in s {
                if let Some(digit) = char::from(c).to_digit(base) {
                    value = value * f64::from(base) + f64::from(digit);
                } else {
                    return f64::NAN;
                }
            }
            return value;
        }

        match string {
            // Handle special cases so `fast_float` does not return infinity.
            "inf" | "+inf" | "-inf" => f64::NAN,
            string => fast_float::parse(string).unwrap_or(f64::NAN),
        }
    }

    /// Abstract operation `StringToBigInt ( str )`
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringtobigint
    pub(crate) fn to_big_int(&self) -> Option<JsBigInt> {
        // 1. Let text be ! StringToCodePoints(str).
        // 2. Let literal be ParseText(text, StringIntegerLiteral).
        // 3. If literal is a List of errors, return undefined.
        // 4. Let mv be the MV of literal.
        // 5. Assert: mv is an integer.
        // 6. Return ℤ(mv).
        JsBigInt::from_string(self.to_std_string().ok().as_ref()?)
    }
}

impl AsRef<[u16]> for JsString {
    fn as_ref(&self) -> &[u16] {
        self
    }
}

impl Borrow<[u16]> for JsString {
    fn borrow(&self) -> &[u16] {
        self
    }
}

impl Clone for JsString {
    #[inline]
    fn clone(&self) -> Self {
        if let JsStringPtrKind::Heap(inner) = self.ptr() {
            inner.refcount.set(inner.refcount.get() + 1);
        }
        Self { ptr: self.ptr }
    }
}

impl Default for JsString {
    #[inline]
    fn default() -> Self {
        sa::const_assert!(!COMMON_STRINGS.is_empty());
        // Safety:
        // `COMMON_STRINGS` must not be empty for this to be safe.
        // The static assertion above verifies this.
        unsafe {
            Self {
                ptr: TaggedJsString::new_static(0),
            }
        }
    }
}

impl Drop for JsString {
    #[inline]
    fn drop(&mut self) {
        if let JsStringPtrKind::Heap(inner) = self.ptr() {
            inner.refcount.set(inner.refcount.get() - 1);
            if inner.refcount.get() == 0 {
                // Safety:
                // If refcount is 0 and we call drop, that means this is the last `JsString`
                // which points to this memory allocation, so deallocating it is safe.
                unsafe {
                    ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
                        inner.data.as_mut_ptr(),
                        inner.len,
                    ));
                    dealloc((inner as *mut RawJsString).cast(), Layout::for_value(inner));
                }
            }
        }
    }
}

impl std::fmt::Debug for JsString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::char::decode_utf16(self.as_slice().to_owned())
            .map(|r| {
                r.map_or_else(
                    |err| format!("<0x{:04x}>", err.unpaired_surrogate()),
                    String::from,
                )
            })
            .collect::<String>()
            .fmt(f)
    }
}

impl Deref for JsString {
    type Target = [u16];

    fn deref(&self) -> &Self::Target {
        match self.ptr() {
            JsStringPtrKind::Heap(h) => {
                // SAFETY:
                // - The `Inner` type has all the necessary information
                // to reconstruct a valid slice (length and starting pointer).
                //
                // - We aligned `h.data` on allocation, and the
                // block is of size `h.len`, so this should only generate
                // valid reads.
                //
                // - The lifetime of `&Self::Target` is shorter
                // than the lifetime of `self`, as seen by its signature,
                // so this doesn't outlive `self`.
                unsafe { std::slice::from_raw_parts(h.data.as_ptr(), h.len) }
            }
            JsStringPtrKind::Static(s) => s,
        }
    }
}

impl Eq for JsString {}

impl From<&[u16]> for JsString {
    fn from(s: &[u16]) -> Self {
        if s.len() <= MAX_COMMON_STRING_LENGTH {
            if let Some(constant) = COMMON_STRINGS_CACHE.with(|c| c.get(s).cloned()) {
                return constant;
            }
        }
        Self::from_slice_skip_interning(s)
    }
}

impl From<Vec<u16>> for JsString {
    fn from(vec: Vec<u16>) -> Self {
        JsString::from(&vec[..])
    }
}

impl From<&str> for JsString {
    #[inline]
    fn from(s: &str) -> Self {
        let s = s.encode_utf16().collect::<Vec<_>>();

        Self::from(&s[..])
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
    fn hash<H: Hasher>(&self, state: &mut H) {
        self[..].hash(state);
    }
}

impl<I: SliceIndex<[u16]>> Index<I> for JsString {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl Ord for JsString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self[..].cmp(other)
    }
}

impl PartialEq for JsString {
    fn eq(&self, other: &Self) -> bool {
        if self.ptr == other.ptr {
            return true;
        }

        self[..] == other[..]
    }
}

impl PartialEq<JsString> for [u16] {
    fn eq(&self, other: &JsString) -> bool {
        self == &**other
    }
}

impl<const N: usize> PartialEq<JsString> for [u16; N] {
    fn eq(&self, other: &JsString) -> bool {
        self[..] == *other
    }
}

impl PartialEq<[u16]> for JsString {
    fn eq(&self, other: &[u16]) -> bool {
        &**self == other
    }
}

impl<const N: usize> PartialEq<[u16; N]> for JsString {
    fn eq(&self, other: &[u16; N]) -> bool {
        *self == other[..]
    }
}

impl PartialOrd for JsString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self[..].partial_cmp(other)
    }
}

/// Utility trait that adds trimming functionality to every `UTF-16` string.
pub(crate) trait Utf16Trim {
    /// Trims both leading and trailing space from `self`.
    fn trim(&self) -> &Self {
        self.trim_start().trim_end()
    }

    /// Trims all leading space from `self`.
    fn trim_start(&self) -> &Self;

    /// Trims all trailing space from `self`.
    fn trim_end(&self) -> &Self;
}

impl Utf16Trim for [u16] {
    fn trim_start(&self) -> &Self {
        if let Some(left) = self.iter().copied().position(|r| {
            !char::from_u32(u32::from(r))
                .map(is_trimmable_whitespace)
                .unwrap_or_default()
        }) {
            &self[left..]
        } else {
            &[]
        }
    }
    fn trim_end(&self) -> &Self {
        if let Some(right) = self.iter().copied().rposition(|r| {
            !char::from_u32(u32::from(r))
                .map(is_trimmable_whitespace)
                .unwrap_or_default()
        }) {
            &self[..=right]
        } else {
            &[]
        }
    }
}

/// Utility trait that adds a `UTF-16` escaped representation to every
/// [`[u16]`][std::slice].
pub(crate) trait ToStringEscaped {
    /// Decodes `self` as an `UTF-16` encoded string,
    /// escaping any unpaired surrogates by its codepoint value.
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
#[cfg(test)]
mod tests {
    use super::utf16;
    use super::{JsString, JsStringPtrKind};
    use std::mem::size_of;

    impl JsString {
        /// Gets the number of `JsString`s which point to this allocation.
        #[inline]
        pub fn refcount(this: &Self) -> Option<usize> {
            match this.ptr() {
                JsStringPtrKind::Heap(inner) => Some(inner.refcount.get()),
                JsStringPtrKind::Static(_inner) => None,
            }
        }
    }

    #[test]
    fn empty() {
        let s = js_string!();
        assert_eq!(*s, "".encode_utf16().collect::<Vec<u16>>());
    }

    #[test]
    fn pointer_size() {
        assert_eq!(size_of::<JsString>(), size_of::<*const ()>());
        assert_eq!(size_of::<Option<JsString>>(), size_of::<*const ()>());
    }

    #[test]
    fn refcount() {
        let x = js_string!("Hello world");
        assert_eq!(JsString::refcount(&x), Some(1));

        {
            let y = x.clone();
            assert_eq!(JsString::refcount(&x), Some(2));
            assert_eq!(JsString::refcount(&y), Some(2));

            {
                let z = y.clone();
                assert_eq!(JsString::refcount(&x), Some(3));
                assert_eq!(JsString::refcount(&y), Some(3));
                assert_eq!(JsString::refcount(&z), Some(3));
            }

            assert_eq!(JsString::refcount(&x), Some(2));
            assert_eq!(JsString::refcount(&y), Some(2));
        }

        assert_eq!(JsString::refcount(&x), Some(1));
    }

    #[test]
    fn static_refcount() {
        let x = js_string!();
        assert_eq!(JsString::refcount(&x), None);

        {
            let y = x.clone();
            assert_eq!(JsString::refcount(&x), None);
            assert_eq!(JsString::refcount(&y), None);
        };

        assert_eq!(JsString::refcount(&x), None);
    }

    #[test]
    fn ptr_eq() {
        let x = js_string!("Hello");
        let y = x.clone();

        assert!(!x.ptr.is_static());

        assert_eq!(x.ptr, y.ptr);

        let z = js_string!("Hello");
        assert_ne!(x.ptr, z.ptr);
        assert_ne!(y.ptr, z.ptr);
    }

    #[test]
    fn static_ptr_eq() {
        let x = js_string!();
        let y = x.clone();

        assert!(x.ptr.is_static());

        assert_eq!(x.ptr, y.ptr);

        let z = js_string!();
        assert_eq!(x.ptr, z.ptr);
        assert_eq!(y.ptr, z.ptr);
    }

    #[test]
    fn as_str() {
        const HELLO: &str = "Hello";
        let x = js_string!(HELLO);

        assert_eq!(*x, HELLO.encode_utf16().collect::<Vec<u16>>());
    }

    #[test]
    fn hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        const HELLOWORLD: &[u16] = &utf16!("Hello World!");
        let x = js_string!(HELLOWORLD);

        assert_eq!(&*x, HELLOWORLD);

        let mut hasher = DefaultHasher::new();
        HELLOWORLD.hash(&mut hasher);
        let s_hash = hasher.finish();

        let mut hasher = DefaultHasher::new();
        x.hash(&mut hasher);
        let x_hash = hasher.finish();

        assert_eq!(s_hash, x_hash);
    }

    #[test]
    fn concat() {
        const Y: &[u16] = &utf16!(", ");
        const W: &[u16] = &utf16!("!");

        let x = js_string!("hello");
        let z = js_string!("world");

        let xy = js_string!(&x, Y);
        assert_eq!(xy, utf16!("hello, "));
        assert_eq!(JsString::refcount(&xy), Some(1));

        let xyz = js_string!(&xy, &z);
        assert_eq!(xyz, utf16!("hello, world"));
        assert_eq!(JsString::refcount(&xyz), Some(1));

        let xyzw = js_string!(&xyz, W);
        assert_eq!(xyzw, utf16!("hello, world!"));
        assert_eq!(JsString::refcount(&xyzw), Some(1));
    }
}
