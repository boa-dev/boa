use crate::builtins::string::is_trimmable_whitespace;
use boa_gc::{unsafe_empty_trace, Finalize, Trace};
use rustc_hash::{FxHashSet, FxHasher};
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    borrow::Borrow,
    cell::Cell,
    hash::BuildHasherDefault,
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroUsize,
    ops::Deref,
    ptr::copy_nonoverlapping,
    rc::Rc,
};

const CONSTANTS_ARRAY: [&str; 426] = [
    // Empty string
    "",
    // Misc
    ",",
    ":",
    // Generic use
    "name",
    "length",
    "arguments",
    "prototype",
    "constructor",
    "return",
    "throw",
    "global",
    "globalThis",
    // typeof
    "null",
    "undefined",
    "number",
    "string",
    "symbol",
    "bigint",
    "object",
    "function",
    // Property descriptor
    "value",
    "get",
    "set",
    "writable",
    "enumerable",
    "configurable",
    // Object object
    "Object",
    "assign",
    "create",
    "toString",
    "valueOf",
    "is",
    "seal",
    "isSealed",
    "freeze",
    "isFrozen",
    "preventExtensions",
    "isExtensible",
    "getOwnPropertyDescriptor",
    "getOwnPropertyDescriptors",
    "getOwnPropertyNames",
    "getOwnPropertySymbols",
    "hasOwnProperty",
    "propertyIsEnumerable",
    "isPrototypeOf",
    "setPrototypeOf",
    "getPrototypeOf",
    "defineProperty",
    "defineProperties",
    "deleteProperty",
    "construct",
    "hasOwn",
    "ownKeys",
    "keys",
    "values",
    "entries",
    "fromEntries",
    // Function object
    "Function",
    "apply",
    "bind",
    "call",
    // Generator object
    "Generator",
    "GeneratorFunction",
    // Array object
    "Array",
    "at",
    "from",
    "isArray",
    "of",
    "get [Symbol.species]",
    "copyWithin",
    "entries",
    "every",
    "fill",
    "filter",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "flat",
    "flatMap",
    "forEach",
    "includes",
    "indexOf",
    "join",
    "map",
    "next",
    "reduce",
    "reduceRight",
    "reverse",
    "shift",
    "slice",
    "splice",
    "some",
    "sort",
    "unshift",
    "push",
    "pop",
    // String object
    "String",
    "charAt",
    "charCodeAt",
    "codePointAt",
    "concat",
    "endsWith",
    "fromCharCode",
    "fromCodePoint",
    "includes",
    "indexOf",
    "lastIndexOf",
    "match",
    "matchAll",
    "normalize",
    "padEnd",
    "padStart",
    "raw",
    "repeat",
    "replace",
    "replaceAll",
    "search",
    "slice",
    "split",
    "startsWith",
    "substr",
    "substring",
    "toLocaleString",
    "toLowerCase",
    "toUpperCase",
    "trim",
    "trimEnd",
    "trimStart",
    // Number object
    "Number",
    "Infinity",
    "NaN",
    "parseInt",
    "parseFloat",
    "isFinite",
    "isNaN",
    "parseInt",
    "EPSILON",
    "MAX_SAFE_INTEGER",
    "MIN_SAFE_INTEGER",
    "MAX_VALUE",
    "MIN_VALUE",
    "NEGATIVE_INFINITY",
    "POSITIVE_INFINITY",
    "isSafeInteger",
    "isInteger",
    "toExponential",
    "toFixed",
    "toPrecision",
    // Boolean object
    "Boolean",
    // BigInt object
    "BigInt",
    "asIntN",
    "asUintN",
    // RegExp object
    "RegExp",
    "exec",
    "test",
    "flags",
    "index",
    "lastIndex",
    "hasIndices",
    "ignoreCase",
    "multiline",
    "dotAll",
    "unicode",
    "sticky",
    "source",
    // Symbol object
    "Symbol",
    "for",
    "keyFor",
    "description",
    "asyncIterator",
    "Symbol.asyncIterator",
    "hasInstance",
    "Symbol.hasInstance",
    "isConcatSpreadable",
    "Symbol.isConcatSpreadable",
    "species",
    "Symbol.species",
    "unscopables",
    "Symbol.unscopables",
    "[Symbol.hasInstance]",
    "iterator",
    "Symbol.iterator",
    "[Symbol.iterator]",
    "Symbol.match",
    "[Symbol.match]",
    "Symbol.matchAll",
    "[Symbol.matchAll]",
    "Symbol.replace",
    "[Symbol.replace]",
    "Symbol.search",
    "[Symbol.search]",
    "Symbol.split",
    "[Symbol.split]",
    "toStringTag",
    "Symbol.toStringTag",
    "[Symbol.toStringTag]",
    "toPrimitive",
    "Symbol.toPrimitive",
    "[Symbol.toPrimitive]",
    // Map object
    "Map",
    "clear",
    "delete",
    "get",
    "has",
    "set",
    "size",
    // Set object
    "Set",
    "add",
    // Reflect object
    "Reflect",
    // Proxy object
    "Proxy",
    "revocable",
    // Error objects
    "Error",
    "AggregateError",
    "TypeError",
    "RangeError",
    "SyntaxError",
    "ReferenceError",
    "EvalError",
    "URIError",
    "message",
    // Date object
    "Date",
    "toJSON",
    "getDate",
    "getDay",
    "getFullYear",
    "getHours",
    "getMilliseconds",
    "getMinutes",
    "getMonth",
    "getSeconds",
    "getTime",
    "getYear",
    "getTimezoneOffset",
    "getUTCDate",
    "getUTCDay",
    "getUTCFullYear",
    "getUTCHours",
    "getUTCMilliseconds",
    "getUTCMinutes",
    "getUTCMonth",
    "getUTCSeconds",
    "setDate",
    "setFullYear",
    "setHours",
    "setMilliseconds",
    "setMinutes",
    "setMonth",
    "setSeconds",
    "setYear",
    "setTime",
    "setUTCDate",
    "setUTCFullYear",
    "setUTCHours",
    "setUTCMilliseconds",
    "setUTCMinutes",
    "setUTCMonth",
    "setUTCSeconds",
    "toDateString",
    "toGMTString",
    "toISOString",
    "toTimeString",
    "toUTCString",
    "now",
    "UTC",
    // JSON object
    "JSON",
    "parse",
    "stringify",
    // Math object
    "Math",
    "LN10",
    "LN2",
    "LOG10E",
    "LOG2E",
    "PI",
    "SQRT1_2",
    "SQRT2",
    "abs",
    "acos",
    "acosh",
    "asin",
    "asinh",
    "atan",
    "atanh",
    "atan2",
    "cbrt",
    "ceil",
    "clz32",
    "cos",
    "cosh",
    "exp",
    "expm1",
    "floor",
    "fround",
    "hypot",
    "imul",
    "log",
    "log1p",
    "log10",
    "log2",
    "max",
    "min",
    "pow",
    "random",
    "round",
    "sign",
    "sin",
    "sinh",
    "sqrt",
    "tan",
    "tanh",
    "trunc",
    // Intl object
    "Intl",
    "DateTimeFormat",
    "getCanonicalLocales",
    // TypedArray object
    "TypedArray",
    "ArrayBuffer",
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "BigInt64Array",
    "BigUint64Array",
    "Float32Array",
    "Float64Array",
    "BYTES_PER_ELEMENT",
    "buffer",
    "byteLength",
    "byteOffset",
    "isView",
    "subarray",
    // DataView object
    "DataView",
    "getBigInt64",
    "getBigUint64",
    "getFloat32",
    "getFloat64",
    "getInt8",
    "getInt16",
    "getInt32",
    "getUint8",
    "getUint16",
    "getUint32",
    "setBigInt64",
    "setBigUint64",
    "setFloat32",
    "setFloat64",
    "setInt8",
    "setInt16",
    "setInt32",
    "setUint8",
    "setUint16",
    "setUint32",
    // Console object
    "console",
    "assert",
    "debug",
    "error",
    "info",
    "trace",
    "warn",
    "exception",
    "count",
    "countReset",
    "group",
    "groupCollapsed",
    "groupEnd",
    "time",
    "timeLog",
    "timeEnd",
    "dir",
    "dirxml",
    // Minified name
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
    "I",
    "J",
    "K",
    "L",
    "M",
    "N",
    "O",
    "P",
    "Q",
    "R",
    "S",
    "T",
    "U",
    "V",
    "W",
    "X",
    "Y",
    "Z",
    "_",
    "$",
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
        let len = CONSTANTS_ARRAY.len();
        let mut constants = FxHashSet::with_capacity_and_hasher(len, BuildHasherDefault::<FxHasher>::default());

        for idx in 0..len {
            let s = JsString::new_static(idx);
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
    fn new(s: &str) -> *mut Self {
        // We get the layout of the `Inner` type and we extend by the size
        // of the string array.
        let inner_layout = Layout::new::<Self>();
        let (layout, offset) = inner_layout
            .extend(Layout::array::<u8>(s.len()).expect("failed to create memory layout"))
            .expect("failed to extend memory layout");

        unsafe {
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
        }
    }

    /// Concatenate array of strings.
    #[inline]
    fn concat_array(strings: &[&str]) -> *mut Self {
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

        unsafe {
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
        }
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
    inner: Flag,
    _marker: PhantomData<Rc<str>>,
}

/// It maybe an index of [`CONSTANTS_ARRAY`], or a raw pointer of [`Inner`].
/// Use the first bit as the flag.
/// Detail: <https://en.wikipedia.org/wiki/Tagged_pointer>
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Flag(NonZeroUsize);

impl Flag {
    #[inline]
    fn new_heap(inner: *mut Inner) -> Self {
        // Safety: We already know it's not null, so this is safe.
        Self(unsafe { NonZeroUsize::new_unchecked(inner as usize) })
    }

    /// Set the first bit to 1, indicating that it is static. Store the index
    /// in 1..63 bits.
    #[inline]
    const fn new_static(idx: usize) -> Self {
        // Safety: We already know it's not null, so this is safe.
        Self(unsafe { NonZeroUsize::new_unchecked((idx << 1) | 1) })
    }

    /// Check if the first bit is 1.
    #[inline]
    const fn is_static(self) -> bool {
        self.0.get() & 1 == 1
    }

    /// # Safety
    ///
    /// It maybe a static string.
    #[inline]
    const unsafe fn get_heap_unchecked(self) -> *mut Inner {
        self.0.get() as *mut _
    }

    /// # Safety
    ///
    /// It maybe a string allocated on the heap.
    #[inline]
    const unsafe fn get_static_unchecked(self) -> &'static str {
        // shift right to get the index.
        CONSTANTS_ARRAY[self.0.get() >> 1]
    }
}

impl Default for JsString {
    #[inline]
    fn default() -> Self {
        Self::new_static(0)
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
    /// Create a new JavaScript string from an index of [`CONSTANTS_ARRAY`].
    #[inline]
    fn new_static(idx: usize) -> Self {
        Self {
            inner: Flag::new_static(idx),
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
            inner: Flag::new_heap(Inner::new(s)),
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
            inner: Flag::new_heap(Inner::concat_array(&[x, y])),
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
            inner: Flag::new_heap(Inner::concat_array(strings)),
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
    fn inner(&self) -> InnerKind<'_> {
        // Check the first bit to 1.
        if self.inner.is_static() {
            // Safety: We already checked.
            InnerKind::Static(unsafe { self.inner.get_static_unchecked() })
        } else {
            // Safety: We already checked.
            InnerKind::Heap(unsafe { &*self.inner.get_heap_unchecked() })
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
                    Inner::dealloc(self.inner.get_heap_unchecked());
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
    fn static_refcount() {
        let x = JsString::new("");
        assert_eq!(JsString::refcount(&x), 0);

        let idx = {
            let y = x.clone();
            assert_eq!(JsString::refcount(&x), 0);
            assert_eq!(JsString::refcount(&y), 0);
            y.inner
        };

        assert_eq!(x.inner, idx);
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
    fn static_ptr_eq() {
        let x = JsString::new("");
        let y = x.clone();

        assert!(JsString::ptr_eq(&x, &y));

        let z = JsString::new("");
        assert!(JsString::ptr_eq(&x, &z));
        assert!(JsString::ptr_eq(&y, &z));
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
