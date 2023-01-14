use crate::tagged::Tagged;

use super::JsString;
use boa_macros::{static_strings, utf16};

macro_rules! well_known_symbols {
    ( $( $(#[$attr:meta])* ($name:ident, $string:literal) ),+$(,)? ) => {
        $(
            $(#[$attr])* pub(crate) const fn $name(&'static self) -> JsString {
                JsString {
                    ptr: Tagged::from_tag(
                        self.find_index(utf16!($string)),
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
pub(crate) struct StaticJsStrings {
    strings: &'static [&'static [u16]],
    max_length: usize,
}

impl StaticJsStrings {
    const fn find_index(&'static self, candidate: &[u16]) -> usize {
        let mut i = 0;
        while i < self.strings.len() {
            let s = self.strings[i];
            if const_eq(s, candidate) {
                return i;
            }
            i += 1;
        }
        panic!("couldn't find the required string on the common string array");
    }

    /// Get the `JsString` corresponding to `string`, or `None` if the string
    /// doesn't exist inside the static array.
    pub(crate) fn get_string(&'static self, string: &[u16]) -> Option<JsString> {
        if string.len() > self.max_length {
            return None;
        }

        let index = self.strings.binary_search(&string).ok()?;

        Some(JsString {
            ptr: Tagged::from_tag(index),
        })
    }

    /// Get the `&[u16]` slice corresponding to the provided index, or `None` if the index
    /// provided exceeds the size of the static array.
    pub(crate) fn get(&'static self, index: usize) -> Option<&'static [u16]> {
        self.strings.get(index).copied()
    }

    /// Gets the empty string (`""`) `JsString`.
    pub(crate) const fn empty_string(&'static self) -> JsString {
        JsString {
            ptr: Tagged::from_tag(self.find_index(&[])),
        }
    }

    well_known_symbols! {
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

pub(crate) static STATIC_JS_STRINGS: StaticJsStrings = StaticJsStrings {
    strings: RAW_STATICS,
    max_length: {
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
    },
};

static RAW_STATICS: &[&[u16]] = &static_strings! {
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
    "isExtensible",
    "hasOwnProperty",
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
    // Array object
    "Array",
    "at",
    "from",
    "isArray",
    "of",
    "copyWithin",
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
    "EPSILON",
    "MAX_SAFE_INTEGER",
    "MIN_SAFE_INTEGER",
    "MAX_VALUE",
    "MIN_VALUE",
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
    "get hasIndices",
    "get global",
    "get ignoreCase",
    "get multiline",
    "get dotAll",
    "get unicode",
    "get sticky",
    "get flags",
    "get source",
    // Symbol object
    "Symbol",
    "for",
    "keyFor",
    "description",
    "asyncIterator",
    "hasInstance",
    "species",
    "unscopables",
    "iterator",
    "toStringTag",
    "toPrimitive",
    "get description",
    // Map object
    "Map",
    "clear",
    "delete",
    "has",
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
    "ThrowTypeError",
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
    "getUTCDate",
    "getUTCDay",
    "getUTCFullYear",
    "getUTCHours",
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
    // Iterator object
    "Array Iterator",
    "Set Iterator",
    "String Iterator",
    "Map Iterator",
    "For In Iterator",
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
    // TypedArray object
    "TypedArray",
    "ArrayBuffer",
    "Int8Array",
    "Uint8Array",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "BigInt64Array",
    "BigUint64Array",
    "Float32Array",
    "Float64Array",
    "buffer",
    "byteLength",
    "byteOffset",
    "isView",
    "subarray",
    "get byteLength",
    "get buffer",
    "get byteOffset",
    "get size",
    "get length",
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
    // Well known symbols
    "Symbol.asyncIterator",
    "[Symbol.asyncIterator]",

    "Symbol.hasInstance",
    "[Symbol.hasInstance]",

    "Symbol.isConcatSpreadable",
    "[Symbol.isConcatSpreadable]",

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

    "Symbol.species",
    "[Symbol.species]",

    "Symbol.split",
    "[Symbol.split]",

    "Symbol.toPrimitive",
    "[Symbol.toPrimitive]",

    "Symbol.toStringTag",
    "[Symbol.toStringTag]",

    "Symbol.unscopables",
    "[Symbol.unscopables]",
};

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
