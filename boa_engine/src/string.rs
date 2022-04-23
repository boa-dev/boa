use crate::builtins::string::is_trimmable_whitespace;
use boa_gc::{unsafe_empty_trace, Finalize, Trace};
use rustc_hash::FxHashSet;
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    borrow::Borrow,
    cell::Cell,
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroUsize,
    ops::Deref,
    ptr::copy_nonoverlapping,
    rc::Rc,
};

/// A fat pointer representing a static str.
struct StaticStr(*const u8, usize);

impl StaticStr {
    /// Build `Self` from a static string slice
    #[inline]
    const fn new(s: &'static str) -> Self {
        Self(s.as_ptr(), s.len())
    }

    /// Returns the raw pointer of the str.
    #[inline]
    const fn ptr(&self) -> *const u8 {
        self.0
    }

    /// Returns the length of the str.
    #[inline]
    const fn len(&self) -> usize {
        self.1
    }
}

const CONSTANTS_ARRAY: [StaticStr; 426] = [
    // Empty string
    StaticStr::new(""),
    // Misc
    StaticStr::new(","),
    StaticStr::new(":"),
    // Generic use
    StaticStr::new("name"),
    StaticStr::new("length"),
    StaticStr::new("arguments"),
    StaticStr::new("prototype"),
    StaticStr::new("constructor"),
    StaticStr::new("return"),
    StaticStr::new("throw"),
    StaticStr::new("global"),
    StaticStr::new("globalThis"),
    // typeof
    StaticStr::new("null"),
    StaticStr::new("undefined"),
    StaticStr::new("number"),
    StaticStr::new("string"),
    StaticStr::new("symbol"),
    StaticStr::new("bigint"),
    StaticStr::new("object"),
    StaticStr::new("function"),
    // Property descriptor
    StaticStr::new("value"),
    StaticStr::new("get"),
    StaticStr::new("set"),
    StaticStr::new("writable"),
    StaticStr::new("enumerable"),
    StaticStr::new("configurable"),
    // Object object
    StaticStr::new("Object"),
    StaticStr::new("assign"),
    StaticStr::new("create"),
    StaticStr::new("toString"),
    StaticStr::new("valueOf"),
    StaticStr::new("is"),
    StaticStr::new("seal"),
    StaticStr::new("isSealed"),
    StaticStr::new("freeze"),
    StaticStr::new("isFrozen"),
    StaticStr::new("preventExtensions"),
    StaticStr::new("isExtensible"),
    StaticStr::new("getOwnPropertyDescriptor"),
    StaticStr::new("getOwnPropertyDescriptors"),
    StaticStr::new("getOwnPropertyNames"),
    StaticStr::new("getOwnPropertySymbols"),
    StaticStr::new("hasOwnProperty"),
    StaticStr::new("propertyIsEnumerable"),
    StaticStr::new("isPrototypeOf"),
    StaticStr::new("setPrototypeOf"),
    StaticStr::new("getPrototypeOf"),
    StaticStr::new("defineProperty"),
    StaticStr::new("defineProperties"),
    StaticStr::new("deleteProperty"),
    StaticStr::new("construct"),
    StaticStr::new("hasOwn"),
    StaticStr::new("ownKeys"),
    StaticStr::new("keys"),
    StaticStr::new("values"),
    StaticStr::new("entries"),
    StaticStr::new("fromEntries"),
    // Function object
    StaticStr::new("Function"),
    StaticStr::new("apply"),
    StaticStr::new("bind"),
    StaticStr::new("call"),
    // Generator object
    StaticStr::new("Generator"),
    StaticStr::new("GeneratorFunction"),
    // Array object
    StaticStr::new("Array"),
    StaticStr::new("at"),
    StaticStr::new("from"),
    StaticStr::new("isArray"),
    StaticStr::new("of"),
    StaticStr::new("get [Symbol.species]"),
    StaticStr::new("copyWithin"),
    StaticStr::new("entries"),
    StaticStr::new("every"),
    StaticStr::new("fill"),
    StaticStr::new("filter"),
    StaticStr::new("find"),
    StaticStr::new("findIndex"),
    StaticStr::new("findLast"),
    StaticStr::new("findLastIndex"),
    StaticStr::new("flat"),
    StaticStr::new("flatMap"),
    StaticStr::new("forEach"),
    StaticStr::new("includes"),
    StaticStr::new("indexOf"),
    StaticStr::new("join"),
    StaticStr::new("map"),
    StaticStr::new("next"),
    StaticStr::new("reduce"),
    StaticStr::new("reduceRight"),
    StaticStr::new("reverse"),
    StaticStr::new("shift"),
    StaticStr::new("slice"),
    StaticStr::new("splice"),
    StaticStr::new("some"),
    StaticStr::new("sort"),
    StaticStr::new("unshift"),
    StaticStr::new("push"),
    StaticStr::new("pop"),
    // String object
    StaticStr::new("String"),
    StaticStr::new("charAt"),
    StaticStr::new("charCodeAt"),
    StaticStr::new("codePointAt"),
    StaticStr::new("concat"),
    StaticStr::new("endsWith"),
    StaticStr::new("fromCharCode"),
    StaticStr::new("fromCodePoint"),
    StaticStr::new("includes"),
    StaticStr::new("indexOf"),
    StaticStr::new("lastIndexOf"),
    StaticStr::new("match"),
    StaticStr::new("matchAll"),
    StaticStr::new("normalize"),
    StaticStr::new("padEnd"),
    StaticStr::new("padStart"),
    StaticStr::new("raw"),
    StaticStr::new("repeat"),
    StaticStr::new("replace"),
    StaticStr::new("replaceAll"),
    StaticStr::new("search"),
    StaticStr::new("slice"),
    StaticStr::new("split"),
    StaticStr::new("startsWith"),
    StaticStr::new("substr"),
    StaticStr::new("substring"),
    StaticStr::new("toLocaleString"),
    StaticStr::new("toLowerCase"),
    StaticStr::new("toUpperCase"),
    StaticStr::new("trim"),
    StaticStr::new("trimEnd"),
    StaticStr::new("trimStart"),
    // Number object
    StaticStr::new("Number"),
    StaticStr::new("Infinity"),
    StaticStr::new("NaN"),
    StaticStr::new("parseInt"),
    StaticStr::new("parseFloat"),
    StaticStr::new("isFinite"),
    StaticStr::new("isNaN"),
    StaticStr::new("parseInt"),
    StaticStr::new("EPSILON"),
    StaticStr::new("MAX_SAFE_INTEGER"),
    StaticStr::new("MIN_SAFE_INTEGER"),
    StaticStr::new("MAX_VALUE"),
    StaticStr::new("MIN_VALUE"),
    StaticStr::new("NEGATIVE_INFINITY"),
    StaticStr::new("POSITIVE_INFINITY"),
    StaticStr::new("isSafeInteger"),
    StaticStr::new("isInteger"),
    StaticStr::new("toExponential"),
    StaticStr::new("toFixed"),
    StaticStr::new("toPrecision"),
    // Boolean object
    StaticStr::new("Boolean"),
    // BigInt object
    StaticStr::new("BigInt"),
    StaticStr::new("asIntN"),
    StaticStr::new("asUintN"),
    // RegExp object
    StaticStr::new("RegExp"),
    StaticStr::new("exec"),
    StaticStr::new("test"),
    StaticStr::new("flags"),
    StaticStr::new("index"),
    StaticStr::new("lastIndex"),
    StaticStr::new("hasIndices"),
    StaticStr::new("ignoreCase"),
    StaticStr::new("multiline"),
    StaticStr::new("dotAll"),
    StaticStr::new("unicode"),
    StaticStr::new("sticky"),
    StaticStr::new("source"),
    // Symbol object
    StaticStr::new("Symbol"),
    StaticStr::new("for"),
    StaticStr::new("keyFor"),
    StaticStr::new("description"),
    StaticStr::new("asyncIterator"),
    StaticStr::new("Symbol.asyncIterator"),
    StaticStr::new("hasInstance"),
    StaticStr::new("Symbol.hasInstance"),
    StaticStr::new("isConcatSpreadable"),
    StaticStr::new("Symbol.isConcatSpreadable"),
    StaticStr::new("species"),
    StaticStr::new("Symbol.species"),
    StaticStr::new("unscopables"),
    StaticStr::new("Symbol.unscopables"),
    StaticStr::new("[Symbol.hasInstance]"),
    StaticStr::new("iterator"),
    StaticStr::new("Symbol.iterator"),
    StaticStr::new("[Symbol.iterator]"),
    StaticStr::new("Symbol.match"),
    StaticStr::new("[Symbol.match]"),
    StaticStr::new("Symbol.matchAll"),
    StaticStr::new("[Symbol.matchAll]"),
    StaticStr::new("Symbol.replace"),
    StaticStr::new("[Symbol.replace]"),
    StaticStr::new("Symbol.search"),
    StaticStr::new("[Symbol.search]"),
    StaticStr::new("Symbol.split"),
    StaticStr::new("[Symbol.split]"),
    StaticStr::new("toStringTag"),
    StaticStr::new("Symbol.toStringTag"),
    StaticStr::new("[Symbol.toStringTag]"),
    StaticStr::new("toPrimitive"),
    StaticStr::new("Symbol.toPrimitive"),
    StaticStr::new("[Symbol.toPrimitive]"),
    // Map object
    StaticStr::new("Map"),
    StaticStr::new("clear"),
    StaticStr::new("delete"),
    StaticStr::new("get"),
    StaticStr::new("has"),
    StaticStr::new("set"),
    StaticStr::new("size"),
    // Set object
    StaticStr::new("Set"),
    StaticStr::new("add"),
    // Reflect object
    StaticStr::new("Reflect"),
    // Proxy object
    StaticStr::new("Proxy"),
    StaticStr::new("revocable"),
    // Error objects
    StaticStr::new("Error"),
    StaticStr::new("AggregateError"),
    StaticStr::new("TypeError"),
    StaticStr::new("RangeError"),
    StaticStr::new("SyntaxError"),
    StaticStr::new("ReferenceError"),
    StaticStr::new("EvalError"),
    StaticStr::new("URIError"),
    StaticStr::new("message"),
    // Date object
    StaticStr::new("Date"),
    StaticStr::new("toJSON"),
    StaticStr::new("getDate"),
    StaticStr::new("getDay"),
    StaticStr::new("getFullYear"),
    StaticStr::new("getHours"),
    StaticStr::new("getMilliseconds"),
    StaticStr::new("getMinutes"),
    StaticStr::new("getMonth"),
    StaticStr::new("getSeconds"),
    StaticStr::new("getTime"),
    StaticStr::new("getYear"),
    StaticStr::new("getTimezoneOffset"),
    StaticStr::new("getUTCDate"),
    StaticStr::new("getUTCDay"),
    StaticStr::new("getUTCFullYear"),
    StaticStr::new("getUTCHours"),
    StaticStr::new("getUTCMilliseconds"),
    StaticStr::new("getUTCMinutes"),
    StaticStr::new("getUTCMonth"),
    StaticStr::new("getUTCSeconds"),
    StaticStr::new("setDate"),
    StaticStr::new("setFullYear"),
    StaticStr::new("setHours"),
    StaticStr::new("setMilliseconds"),
    StaticStr::new("setMinutes"),
    StaticStr::new("setMonth"),
    StaticStr::new("setSeconds"),
    StaticStr::new("setYear"),
    StaticStr::new("setTime"),
    StaticStr::new("setUTCDate"),
    StaticStr::new("setUTCFullYear"),
    StaticStr::new("setUTCHours"),
    StaticStr::new("setUTCMilliseconds"),
    StaticStr::new("setUTCMinutes"),
    StaticStr::new("setUTCMonth"),
    StaticStr::new("setUTCSeconds"),
    StaticStr::new("toDateString"),
    StaticStr::new("toGMTString"),
    StaticStr::new("toISOString"),
    StaticStr::new("toTimeString"),
    StaticStr::new("toUTCString"),
    StaticStr::new("now"),
    StaticStr::new("UTC"),
    // JSON object
    StaticStr::new("JSON"),
    StaticStr::new("parse"),
    StaticStr::new("stringify"),
    // Math object
    StaticStr::new("Math"),
    StaticStr::new("LN10"),
    StaticStr::new("LN2"),
    StaticStr::new("LOG10E"),
    StaticStr::new("LOG2E"),
    StaticStr::new("PI"),
    StaticStr::new("SQRT1_2"),
    StaticStr::new("SQRT2"),
    StaticStr::new("abs"),
    StaticStr::new("acos"),
    StaticStr::new("acosh"),
    StaticStr::new("asin"),
    StaticStr::new("asinh"),
    StaticStr::new("atan"),
    StaticStr::new("atanh"),
    StaticStr::new("atan2"),
    StaticStr::new("cbrt"),
    StaticStr::new("ceil"),
    StaticStr::new("clz32"),
    StaticStr::new("cos"),
    StaticStr::new("cosh"),
    StaticStr::new("exp"),
    StaticStr::new("expm1"),
    StaticStr::new("floor"),
    StaticStr::new("fround"),
    StaticStr::new("hypot"),
    StaticStr::new("imul"),
    StaticStr::new("log"),
    StaticStr::new("log1p"),
    StaticStr::new("log10"),
    StaticStr::new("log2"),
    StaticStr::new("max"),
    StaticStr::new("min"),
    StaticStr::new("pow"),
    StaticStr::new("random"),
    StaticStr::new("round"),
    StaticStr::new("sign"),
    StaticStr::new("sin"),
    StaticStr::new("sinh"),
    StaticStr::new("sqrt"),
    StaticStr::new("tan"),
    StaticStr::new("tanh"),
    StaticStr::new("trunc"),
    // Intl object
    StaticStr::new("Intl"),
    StaticStr::new("DateTimeFormat"),
    StaticStr::new("getCanonicalLocales"),
    // TypedArray object
    StaticStr::new("TypedArray"),
    StaticStr::new("ArrayBuffer"),
    StaticStr::new("Int8Array"),
    StaticStr::new("Uint8Array"),
    StaticStr::new("Uint8ClampedArray"),
    StaticStr::new("Int16Array"),
    StaticStr::new("Uint16Array"),
    StaticStr::new("Int32Array"),
    StaticStr::new("Uint32Array"),
    StaticStr::new("BigInt64Array"),
    StaticStr::new("BigUint64Array"),
    StaticStr::new("Float32Array"),
    StaticStr::new("Float64Array"),
    StaticStr::new("BYTES_PER_ELEMENT"),
    StaticStr::new("buffer"),
    StaticStr::new("byteLength"),
    StaticStr::new("byteOffset"),
    StaticStr::new("isView"),
    StaticStr::new("subarray"),
    // DataView object
    StaticStr::new("DataView"),
    StaticStr::new("getBigInt64"),
    StaticStr::new("getBigUint64"),
    StaticStr::new("getFloat32"),
    StaticStr::new("getFloat64"),
    StaticStr::new("getInt8"),
    StaticStr::new("getInt16"),
    StaticStr::new("getInt32"),
    StaticStr::new("getUint8"),
    StaticStr::new("getUint16"),
    StaticStr::new("getUint32"),
    StaticStr::new("setBigInt64"),
    StaticStr::new("setBigUint64"),
    StaticStr::new("setFloat32"),
    StaticStr::new("setFloat64"),
    StaticStr::new("setInt8"),
    StaticStr::new("setInt16"),
    StaticStr::new("setInt32"),
    StaticStr::new("setUint8"),
    StaticStr::new("setUint16"),
    StaticStr::new("setUint32"),
    // Console object
    StaticStr::new("console"),
    StaticStr::new("assert"),
    StaticStr::new("debug"),
    StaticStr::new("error"),
    StaticStr::new("info"),
    StaticStr::new("trace"),
    StaticStr::new("warn"),
    StaticStr::new("exception"),
    StaticStr::new("count"),
    StaticStr::new("countReset"),
    StaticStr::new("group"),
    StaticStr::new("groupCollapsed"),
    StaticStr::new("groupEnd"),
    StaticStr::new("time"),
    StaticStr::new("timeLog"),
    StaticStr::new("timeEnd"),
    StaticStr::new("dir"),
    StaticStr::new("dirxml"),
    // Minified name
    StaticStr::new("a"),
    StaticStr::new("b"),
    StaticStr::new("c"),
    StaticStr::new("d"),
    StaticStr::new("e"),
    StaticStr::new("f"),
    StaticStr::new("g"),
    StaticStr::new("h"),
    StaticStr::new("i"),
    StaticStr::new("j"),
    StaticStr::new("k"),
    StaticStr::new("l"),
    StaticStr::new("m"),
    StaticStr::new("n"),
    StaticStr::new("o"),
    StaticStr::new("p"),
    StaticStr::new("q"),
    StaticStr::new("r"),
    StaticStr::new("s"),
    StaticStr::new("t"),
    StaticStr::new("u"),
    StaticStr::new("v"),
    StaticStr::new("w"),
    StaticStr::new("x"),
    StaticStr::new("y"),
    StaticStr::new("z"),
    StaticStr::new("A"),
    StaticStr::new("B"),
    StaticStr::new("C"),
    StaticStr::new("D"),
    StaticStr::new("E"),
    StaticStr::new("F"),
    StaticStr::new("G"),
    StaticStr::new("H"),
    StaticStr::new("I"),
    StaticStr::new("J"),
    StaticStr::new("K"),
    StaticStr::new("L"),
    StaticStr::new("M"),
    StaticStr::new("N"),
    StaticStr::new("O"),
    StaticStr::new("P"),
    StaticStr::new("Q"),
    StaticStr::new("R"),
    StaticStr::new("S"),
    StaticStr::new("T"),
    StaticStr::new("U"),
    StaticStr::new("V"),
    StaticStr::new("W"),
    StaticStr::new("X"),
    StaticStr::new("Y"),
    StaticStr::new("Z"),
    StaticStr::new("_"),
    StaticStr::new("$"),
];

const MAX_CONSTANT_STRING_LENGTH: usize = {
    let mut max = 0;
    let mut i = 0;
    while i < CONSTANTS_ARRAY.len() {
        let len = CONSTANTS_ARRAY[i].len();
        if len > max {
            max = len;
        }
        i += 1;
    }
    max
};

unsafe fn try_alloc(layout: Layout) -> *mut u8 {
    let ptr = alloc(layout);
    if ptr.is_null() {
        handle_alloc_error(layout);
    }
    ptr
}

thread_local! {
    static CONSTANTS: FxHashSet<JsString> = {
        let mut constants = FxHashSet::default();

        for s in CONSTANTS_ARRAY.iter() {
            let s = JsString::new_static(s);
            constants.insert(s);
        }

        constants
    };
}

/// The inner representation of a [`JsString`].
#[repr(C)]
struct Inner {
    /// The utf8 length, the number of bytes.
    len: usize,

    /// The number of references to the string.
    ///
    /// When this reaches `0` the string is deallocated.
    refcount: Cell<usize>,

    /// An empty array which is used to get the offset of string data.
    data: [u8; 0],
}

impl Inner {
    /// Create a new `Inner` from `&str`.
    #[inline]
    fn new(s: &str) -> NonZeroUsize {
        // We get the layout of the `Inner` type and we extend by the size
        // of the string array.
        let inner_layout = Layout::new::<Self>();
        let (layout, offset) = inner_layout
            .extend(Layout::array::<u8>(s.len()).expect("failed to create memory layout"))
            .expect("failed to extend memory layout");

        let inner = unsafe {
            let inner = try_alloc(layout).cast::<Self>();

            // Write the first part, the Inner.
            inner.write(Self {
                len: s.len(),
                refcount: Cell::new(1),
                data: [0; 0],
            });

            // Get offset into the string data.
            let data = (*inner).data.as_mut_ptr();

            debug_assert!(std::ptr::eq(inner.cast::<u8>().add(offset), data));

            // Copy string data into data offset.
            copy_nonoverlapping(s.as_ptr(), data, s.len());

            inner
        };

        // Safety: We already know it's not null, so this is safe.
        unsafe { NonZeroUsize::new_unchecked(inner as usize) }
    }

    /// Concatenate array of strings.
    #[inline]
    fn concat_array(strings: &[&str]) -> NonZeroUsize {
        let mut total_string_size = 0;
        for string in strings {
            total_string_size += string.len();
        }

        // We get the layout of the `Inner` type and we extend by the size
        // of the string array.
        let inner_layout = Layout::new::<Self>();
        let (layout, offset) = inner_layout
            .extend(Layout::array::<u8>(total_string_size).expect("failed to create memory layout"))
            .expect("failed to extend memory layout");

        let inner = unsafe {
            let inner = try_alloc(layout).cast::<Self>();

            // Write the first part, the Inner.
            inner.write(Self {
                len: total_string_size,
                refcount: Cell::new(1),
                data: [0; 0],
            });

            // Get offset into the string data.
            let data = (*inner).data.as_mut_ptr();

            debug_assert!(std::ptr::eq(inner.cast::<u8>().add(offset), data));

            // Copy the two string data into data offset.
            let mut offset = 0;
            for string in strings {
                copy_nonoverlapping(string.as_ptr(), data.add(offset), string.len());
                offset += string.len();
            }

            inner
        };

        // Safety: We already know it's not null, so this is safe.
        unsafe { NonZeroUsize::new_unchecked(inner as usize) }
    }

    /// Deallocate inner type with string data.
    #[inline]
    unsafe fn dealloc(x: *mut Self) {
        let len = (*x).len;

        let inner_layout = Layout::new::<Self>();
        let (layout, _offset) = inner_layout
            .extend(Layout::array::<u8>(len).expect("failed to create memory layout"))
            .expect("failed to extend memory layout");

        dealloc(x.cast::<_>(), layout);
    }
}

/// This represents a JavaScript primitive string.
///
/// This is similar to `Rc<str>`. But unlike `Rc<str>` which stores the length
/// on the stack and a pointer to the data (this is also known as fat pointers).
/// The `JsString` length and data is stored on the heap. and just an non-null
/// pointer is kept, so its size is the size of a pointer.
#[derive(Finalize)]
pub struct JsString {
    /// This represents a raw pointer. It maybe a [`StaticStr`], or a [`Inner`].
    inner: NonZeroUsize,
    _marker: PhantomData<Rc<str>>,
}

impl Default for JsString {
    #[inline]
    fn default() -> Self {
        Self::new_static(&CONSTANTS_ARRAY[0])
    }
}

/// Data stored in [`JsString`].
enum InnerKind<'a> {
    // A string allocated on the heap.
    Heap(&'a Inner),
    // A static string slice.
    Static(&'static str),
}

impl JsString {
    /// Create a new JavaScript string from [`StaticStr`].
    #[inline]
    fn new_static(s: &StaticStr) -> Self {
        Self {
            // Safety: We already know it's not null, so this is safe.
            // Set the first bit to 1, indicating that it is static.
            inner: unsafe { NonZeroUsize::new_unchecked((s as *const _ as usize) | 1) },
            _marker: PhantomData,
        }
    }

    /// Create an empty string, same as calling default.
    #[inline]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a new JavaScript string.
    #[inline]
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        let s = s.as_ref();

        if s.len() <= MAX_CONSTANT_STRING_LENGTH {
            if let Some(constant) = CONSTANTS.with(|c| c.get(s).cloned()) {
                return constant;
            }
        }

        Self {
            inner: Inner::new(s),
            _marker: PhantomData,
        }
    }

    /// Concatenate two string.
    pub fn concat<T, U>(x: T, y: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let x = x.as_ref();
        let y = y.as_ref();

        let this = Self {
            inner: Inner::concat_array(&[x, y]),
            _marker: PhantomData,
        };

        if this.len() <= MAX_CONSTANT_STRING_LENGTH {
            if let Some(constant) = CONSTANTS.with(|c| c.get(&this).cloned()) {
                return constant;
            }
        }

        this
    }

    /// Concatenate array of string.
    pub fn concat_array(strings: &[&str]) -> Self {
        let this = Self {
            inner: Inner::concat_array(strings),
            _marker: PhantomData,
        };

        if this.len() <= MAX_CONSTANT_STRING_LENGTH {
            if let Some(constant) = CONSTANTS.with(|c| c.get(&this).cloned()) {
                return constant;
            }
        }

        this
    }

    /// Return the inner representation.
    #[inline]
    fn inner<'a>(&'a self) -> InnerKind<'a> {
        let ptr = self.inner.get();
        // Check the first bit to 1.
        match ptr & 1 {
            1 => unsafe {
                let ptr = &*((ptr & !1) as *const StaticStr);
                let slice = std::slice::from_raw_parts(ptr.ptr(), ptr.len());
                InnerKind::Static(std::str::from_utf8_unchecked(slice))
            },
            _ => InnerKind::Heap(unsafe { &*(ptr as *const Inner) }),
        }
    }

    /// Return the JavaScript string as a rust `&str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self.inner() {
            InnerKind::Heap(inner) => unsafe {
                let slice = std::slice::from_raw_parts(inner.data.as_ptr(), inner.len);
                std::str::from_utf8_unchecked(slice)
            },
            InnerKind::Static(inner) => inner,
        }
    }

    /// Gets the number of `JsString`s which point to this allocation.
    #[inline]
    pub fn refcount(this: &Self) -> usize {
        match this.inner() {
            InnerKind::Heap(inner) => inner.refcount.get(),
            InnerKind::Static(_inner) => 0,
        }
    }

    /// Returns `true` if the two `JsString`s point to the same allocation (in a vein similar to [`ptr::eq`]).
    ///
    /// [`ptr::eq`]: std::ptr::eq
    #[inline]
    pub fn ptr_eq(x: &Self, y: &Self) -> bool {
        x.inner == y.inner
    }

    /// `6.1.4.1 StringIndexOf ( string, searchValue, fromIndex )`
    ///
    /// Note: Instead of returning an isize with `-1` as the "not found" value,
    /// We make use of the type system and return Option<usize> with None as the "not found" value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringindexof
    pub(crate) fn index_of(&self, search_value: &Self, from_index: usize) -> Option<usize> {
        // 1. Assert: Type(string) is String.
        // 2. Assert: Type(searchValue) is String.
        // 3. Assert: fromIndex is a non-negative integer.

        // 4. Let len be the length of string.
        let len = self.encode_utf16().count();

        // 5. If searchValue is the empty String and fromIndex ≤ len, return fromIndex.
        if search_value.is_empty() && from_index <= len {
            return Some(from_index);
        }

        // 6. Let searchLen be the length of searchValue.
        let search_len = search_value.encode_utf16().count();

        // 7. For each integer i starting with fromIndex such that i ≤ len - searchLen, in ascending order, do
        for i in from_index..=len {
            if i as isize > (len as isize - search_len as isize) {
                break;
            }

            // a. Let candidate be the substring of string from i to i + searchLen.
            let candidate = String::from_utf16_lossy(
                &self
                    .encode_utf16()
                    .skip(i)
                    .take(search_len)
                    .collect::<Vec<u16>>(),
            );

            // b. If candidate is the same sequence of code units as searchValue, return i.
            if candidate == search_value.as_str() {
                return Some(i);
            }
        }

        // 8. Return -1.
        None
    }

    pub(crate) fn string_to_number(&self) -> f64 {
        let string = self.trim_matches(is_trimmable_whitespace);

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
}

// Safety: [`JsString`] does not contain any objects which recquire trace,
// so this is safe.
unsafe impl Trace for JsString {
    unsafe_empty_trace!();
}

impl Clone for JsString {
    #[inline]
    fn clone(&self) -> Self {
        if let InnerKind::Heap(inner) = self.inner() {
            inner.refcount.set(inner.refcount.get() + 1);
        }
        Self {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl Drop for JsString {
    #[inline]
    fn drop(&mut self) {
        if let InnerKind::Heap(inner) = self.inner() {
            if inner.refcount.get() == 1 {
                // Safety: If refcount is 1 and we call drop, that means this is the last
                // JsString which points to this memory allocation, so deallocating it is safe.
                unsafe {
                    Inner::dealloc(self.inner.get() as *mut _);
                }
            } else {
                inner.refcount.set(inner.refcount.get() - 1);
            }
        }
    }
}

impl std::fmt::Debug for JsString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::fmt::Display for JsString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<&str> for JsString {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<Box<str>> for JsString {
    #[inline]
    fn from(s: Box<str>) -> Self {
        Self::new(s)
    }
}

impl From<String> for JsString {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for JsString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for JsString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for JsString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl PartialEq<Self> for JsString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // If they point at the same memory allocation, then they are equal.
        if Self::ptr_eq(self, other) {
            return true;
        }

        self.as_str() == other.as_str()
    }
}

impl Eq for JsString {}

impl Hash for JsString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for JsString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for JsString {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other)
    }
}

impl PartialEq<str> for JsString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<JsString> for str {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<&str> for JsString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<JsString> for &str {
    #[inline]
    fn eq(&self, other: &JsString) -> bool {
        *self == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::JsString;
    use std::mem::size_of;

    #[test]
    fn empty() {
        let _empty = JsString::new("");
    }

    #[test]
    fn pointer_size() {
        assert_eq!(size_of::<JsString>(), size_of::<*const u8>());
        assert_eq!(size_of::<Option<JsString>>(), size_of::<*const u8>());
    }

    #[test]
    fn refcount() {
        let x = JsString::new("Hello wrold");
        assert_eq!(JsString::refcount(&x), 1);

        {
            let y = x.clone();
            assert_eq!(JsString::refcount(&x), 2);
            assert_eq!(JsString::refcount(&y), 2);

            {
                let z = y.clone();
                assert_eq!(JsString::refcount(&x), 3);
                assert_eq!(JsString::refcount(&y), 3);
                assert_eq!(JsString::refcount(&z), 3);
            }

            assert_eq!(JsString::refcount(&x), 2);
            assert_eq!(JsString::refcount(&y), 2);
        }

        assert_eq!(JsString::refcount(&x), 1);
    }

    #[test]
    fn ptr_eq() {
        let x = JsString::new("Hello");
        let y = x.clone();

        assert!(JsString::ptr_eq(&x, &y));

        let z = JsString::new("Hello");
        assert!(!JsString::ptr_eq(&x, &z));
        assert!(!JsString::ptr_eq(&y, &z));
    }

    #[test]
    fn as_str() {
        let s = "Hello";
        let x = JsString::new(s);

        assert_eq!(x.as_str(), s);
    }

    #[test]
    fn hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let s = "Hello, world!";
        let x = JsString::new(s);

        assert_eq!(x.as_str(), s);

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let s_hash = hasher.finish();
        let mut hasher = DefaultHasher::new();
        x.hash(&mut hasher);
        let x_hash = hasher.finish();

        assert_eq!(s_hash, x_hash);
    }

    #[test]
    fn concat() {
        let x = JsString::new("hello");
        let y = ", ";
        let z = JsString::new("world");
        let w = String::from("!");

        let xy = JsString::concat(x, y);
        assert_eq!(xy, "hello, ");
        assert_eq!(JsString::refcount(&xy), 1);

        let xyz = JsString::concat(xy, z);
        assert_eq!(xyz, "hello, world");
        assert_eq!(JsString::refcount(&xyz), 1);

        let xyzw = JsString::concat(xyz, w);
        assert_eq!(xyzw, "hello, world!");
        assert_eq!(JsString::refcount(&xyzw), 1);
    }
}
