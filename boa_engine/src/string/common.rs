use std::hash::BuildHasherDefault;

use crate::tagged::Tagged;

use super::JsString;
use boa_macros::utf16;
use paste::paste;
use rustc_hash::{FxHashMap, FxHasher};

macro_rules! well_known_statics {
    ( $( $(#[$attr:meta])* ($name:ident, $string:literal) ),+$(,)? ) => {
        $(
            paste!{
                #[doc = "Gets the static `JsString` for `\"" $string "\"`."]
                #[allow(unused)]
                pub(crate) const $name: JsString = JsString {
                    ptr: Tagged::from_tag(
                        Self::find_index(utf16!($string)),
                    ),
                };
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

    // Some consts are only used on certain features, which triggers the unused lint.
    well_known_statics! {
        (EMPTY_STRING, ""),
        // Symbols
        (SYMBOL_ASYNC_ITERATOR, "Symbol.asyncIterator"),
        (SYMBOL_HAS_INSTANCE, "Symbol.hasInstance"),
        (SYMBOL_IS_CONCAT_SPREADABLE, "Symbol.isConcatSpreadable"),
        (SYMBOL_ITERATOR, "Symbol.iterator"),
        (SYMBOL_MATCH, "Symbol.match"),
        (SYMBOL_MATCH_ALL, "Symbol.matchAll"),
        (SYMBOL_REPLACE, "Symbol.replace"),
        (SYMBOL_SEARCH, "Symbol.search"),
        (SYMBOL_SPECIES, "Symbol.species"),
        (SYMBOL_SPLIT, "Symbol.split"),
        (SYMBOL_TO_PRIMITIVE, "Symbol.toPrimitive"),
        (SYMBOL_TO_STRING_TAG, "Symbol.toStringTag"),
        (SYMBOL_UNSCOPABLES, "Symbol.unscopables"),
        (FN_SYMBOL_ASYNC_ITERATOR, "[Symbol.asyncIterator]"),
        (FN_SYMBOL_HAS_INSTANCE, "[Symbol.hasInstance]"),
        (FN_SYMBOL_IS_CONCAT_SPREADABLE, "[Symbol.isConcatSpreadable]"),
        (FN_SYMBOL_ITERATOR, "[Symbol.iterator]"),
        (FN_SYMBOL_MATCH, "[Symbol.match]"),
        (FN_SYMBOL_MATCH_ALL, "[Symbol.matchAll]"),
        (FN_SYMBOL_REPLACE, "[Symbol.replace]"),
        (FN_SYMBOL_SEARCH, "[Symbol.search]"),
        (FN_SYMBOL_SPECIES, "[Symbol.species]"),
        (FN_SYMBOL_SPLIT, "[Symbol.split]"),
        (FN_SYMBOL_TO_PRIMITIVE, "[Symbol.toPrimitive]"),
        (FN_SYMBOL_TO_STRING_TAG, "[Symbol.toStringTag]"),
        (FN_SYMBOL_UNSCOPABLES, "[Symbol.unscopables]"),
        // Builtins
        (ARRAY, "Array"),
        (ARRAY_BUFFER, "ArrayBuffer"),
        (ASYNC_FUNCTION, "AsyncFunction"),
        (ASYNC_GENERATOR, "AsyncGenerator"),
        (ASYNC_GENERATOR_FUNCTION, "AsyncGeneratorFunction"),
        (BIG_INT, "BigInt"),
        (BOOLEAN, "Boolean"),
        (DATA_VIEW, "DataView"),
        (DATE, "Date"),
        (ERROR, "Error"),
        (AGGREGATE_ERROR, "AggregateError"),
        (EVAL_ERROR, "EvalError"),
        (RANGE_ERROR, "RangeError"),
        (REFERENCE_ERROR, "ReferenceError"),
        (SYNTAX_ERROR, "SyntaxError"),
        (TYPE_ERROR, "TypeError"),
        (URI_ERROR, "URIError"),
        (ESCAPE, "escape"),
        (UNESCAPE, "unescape"),
        (EVAL, "eval"),
        (FUNCTION, "Function"),
        (GENERATOR, "Generator"),
        (GENERATOR_FUNCTION, "GeneratorFunction"),
        (INTL, "Intl"),
        (COLLATOR, "Collator"),
        (LIST_FORMAT, "ListFormat"),
        (LOCALE, "Locale"),
        (PLURAL_RULES, "PluralRules"),
        (SEGMENTER, "Segmenter"),
        (DATE_TIME_FORMAT, "DateTimeFormat"),
        (JSON, "JSON"),
        (MAP, "Map"),
        (MATH, "Math"),
        (NUMBER, "Number"),
        (IS_FINITE, "isFinite"),
        (IS_NAN, "isNaN"),
        (PARSE_INT, "parseInt"),
        (PARSE_FLOAT, "parseFloat"),
        (OBJECT, "Object"),
        (PROMISE, "Promise"),
        (PROXY, "Proxy"),
        (REFLECT, "Reflect"),
        (REG_EXP, "RegExp"),
        (SET, "Set"),
        (STRING, "String"),
        (SYMBOL, "Symbol"),
        (TYPED_ARRAY, "TypedArray"),
        (INT8_ARRAY, "Int8Array"),
        (UINT8_ARRAY, "Uint8Array"),
        (UINT8_CLAMPED_ARRAY, "Uint8ClampedArray"),
        (INT16_ARRAY, "Int16Array"),
        (UINT16_ARRAY, "Uint16Array"),
        (INT32_ARRAY, "Int32Array"),
        (UINT32_ARRAY, "Uint32Array"),
        (BIG_INT64_ARRAY, "BigInt64Array"),
        (BIG_UINT64_ARRAY, "BigUint64Array"),
        (FLOAT32_ARRAY, "Float32Array"),
        (FLOAT64_ARRAY, "Float64Array"),
        (ENCODE_URI, "encodeURI"),
        (ENCODE_URI_COMPONENT, "encodeURIComponent"),
        (DECODE_URI, "decodeURI"),
        (DECODE_URI_COMPONENT, "decodeURIComponent"),
        (WEAK_REF, "WeakRef"),
        (WEAK_MAP, "WeakMap"),
        (WEAK_SET, "WeakSet"),
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

/// Array of raw static strings that aren't reference counted.
const RAW_STATICS: &[&[u16]] = &[
    utf16!(""),
    // Well known symbols
    utf16!("Symbol.asyncIterator"),
    utf16!("[Symbol.asyncIterator]"),
    utf16!("Symbol.hasInstance"),
    utf16!("[Symbol.hasInstance]"),
    utf16!("Symbol.isConcatSpreadable"),
    utf16!("[Symbol.isConcatSpreadable]"),
    utf16!("Symbol.iterator"),
    utf16!("[Symbol.iterator]"),
    utf16!("Symbol.match"),
    utf16!("[Symbol.match]"),
    utf16!("Symbol.matchAll"),
    utf16!("[Symbol.matchAll]"),
    utf16!("Symbol.replace"),
    utf16!("[Symbol.replace]"),
    utf16!("Symbol.search"),
    utf16!("[Symbol.search]"),
    utf16!("Symbol.species"),
    utf16!("[Symbol.species]"),
    utf16!("Symbol.split"),
    utf16!("[Symbol.split]"),
    utf16!("Symbol.toPrimitive"),
    utf16!("[Symbol.toPrimitive]"),
    utf16!("Symbol.toStringTag"),
    utf16!("[Symbol.toStringTag]"),
    utf16!("Symbol.unscopables"),
    utf16!("[Symbol.unscopables]"),
    // Well known builtins
    utf16!("Array"),
    utf16!("ArrayBuffer"),
    utf16!("AsyncFunction"),
    utf16!("AsyncGenerator"),
    utf16!("AsyncGeneratorFunction"),
    utf16!("BigInt"),
    utf16!("Boolean"),
    utf16!("DataView"),
    utf16!("Date"),
    utf16!("Error"),
    utf16!("AggregateError"),
    utf16!("EvalError"),
    utf16!("RangeError"),
    utf16!("ReferenceError"),
    utf16!("SyntaxError"),
    utf16!("TypeError"),
    utf16!("URIError"),
    utf16!("escape"),
    utf16!("unescape"),
    utf16!("eval"),
    utf16!("Function"),
    utf16!("Generator"),
    utf16!("GeneratorFunction"),
    utf16!("Intl"),
    utf16!("Collator"),
    utf16!("ListFormat"),
    utf16!("Locale"),
    utf16!("PluralRules"),
    utf16!("Segmenter"),
    utf16!("DateTimeFormat"),
    utf16!("JSON"),
    utf16!("Map"),
    utf16!("Math"),
    utf16!("Number"),
    utf16!("isFinite"),
    utf16!("isNaN"),
    utf16!("parseInt"),
    utf16!("parseFloat"),
    utf16!("Object"),
    utf16!("Promise"),
    utf16!("Proxy"),
    utf16!("Reflect"),
    utf16!("RegExp"),
    utf16!("Set"),
    utf16!("String"),
    utf16!("Symbol"),
    utf16!("TypedArray"),
    utf16!("Int8Array"),
    utf16!("Uint8Array"),
    utf16!("Uint8ClampedArray"),
    utf16!("Int16Array"),
    utf16!("Uint16Array"),
    utf16!("Int32Array"),
    utf16!("Uint32Array"),
    utf16!("BigInt64Array"),
    utf16!("BigUint64Array"),
    utf16!("Float32Array"),
    utf16!("Float64Array"),
    utf16!("encodeURI"),
    utf16!("encodeURIComponent"),
    utf16!("decodeURI"),
    utf16!("decodeURIComponent"),
    utf16!("WeakRef"),
    utf16!("WeakMap"),
    utf16!("WeakSet"),
    // Misc
    utf16!(","),
    utf16!(":"),
    // Generic use
    utf16!("name"),
    utf16!("length"),
    utf16!("arguments"),
    utf16!("prototype"),
    utf16!("constructor"),
    utf16!("return"),
    utf16!("throw"),
    utf16!("global"),
    utf16!("globalThis"),
    // typeof
    utf16!("null"),
    utf16!("undefined"),
    utf16!("number"),
    utf16!("string"),
    utf16!("symbol"),
    utf16!("bigint"),
    utf16!("object"),
    utf16!("function"),
    // Property descriptor
    utf16!("value"),
    utf16!("get"),
    utf16!("set"),
    utf16!("writable"),
    utf16!("enumerable"),
    utf16!("configurable"),
    // Object object
    utf16!("Object"),
    utf16!("assign"),
    utf16!("create"),
    utf16!("toString"),
    utf16!("valueOf"),
    utf16!("is"),
    utf16!("seal"),
    utf16!("isSealed"),
    utf16!("freeze"),
    utf16!("isFrozen"),
    utf16!("isExtensible"),
    utf16!("hasOwnProperty"),
    utf16!("isPrototypeOf"),
    utf16!("setPrototypeOf"),
    utf16!("getPrototypeOf"),
    utf16!("defineProperty"),
    utf16!("defineProperties"),
    utf16!("deleteProperty"),
    utf16!("construct"),
    utf16!("hasOwn"),
    utf16!("ownKeys"),
    utf16!("keys"),
    utf16!("values"),
    utf16!("entries"),
    utf16!("fromEntries"),
    // Function object
    utf16!("apply"),
    utf16!("bind"),
    utf16!("call"),
    // Array object
    utf16!("at"),
    utf16!("from"),
    utf16!("isArray"),
    utf16!("of"),
    utf16!("copyWithin"),
    utf16!("every"),
    utf16!("fill"),
    utf16!("filter"),
    utf16!("find"),
    utf16!("findIndex"),
    utf16!("findLast"),
    utf16!("findLastIndex"),
    utf16!("flat"),
    utf16!("flatMap"),
    utf16!("forEach"),
    utf16!("includes"),
    utf16!("indexOf"),
    utf16!("join"),
    utf16!("map"),
    utf16!("next"),
    utf16!("reduce"),
    utf16!("reduceRight"),
    utf16!("reverse"),
    utf16!("shift"),
    utf16!("slice"),
    utf16!("splice"),
    utf16!("some"),
    utf16!("sort"),
    utf16!("unshift"),
    utf16!("push"),
    utf16!("pop"),
    // String object
    utf16!("charAt"),
    utf16!("charCodeAt"),
    utf16!("codePointAt"),
    utf16!("concat"),
    utf16!("endsWith"),
    utf16!("fromCharCode"),
    utf16!("fromCodePoint"),
    utf16!("lastIndexOf"),
    utf16!("match"),
    utf16!("matchAll"),
    utf16!("normalize"),
    utf16!("padEnd"),
    utf16!("padStart"),
    utf16!("raw"),
    utf16!("repeat"),
    utf16!("replace"),
    utf16!("replaceAll"),
    utf16!("search"),
    utf16!("split"),
    utf16!("startsWith"),
    utf16!("substr"),
    utf16!("substring"),
    utf16!("toLocaleString"),
    utf16!("toLowerCase"),
    utf16!("toUpperCase"),
    utf16!("trim"),
    utf16!("trimEnd"),
    utf16!("trimStart"),
    // Number object
    utf16!("Infinity"),
    utf16!("NaN"),
    utf16!("EPSILON"),
    utf16!("MAX_SAFE_INTEGER"),
    utf16!("MIN_SAFE_INTEGER"),
    utf16!("MAX_VALUE"),
    utf16!("MIN_VALUE"),
    utf16!("isSafeInteger"),
    utf16!("isInteger"),
    utf16!("toExponential"),
    utf16!("toFixed"),
    utf16!("toPrecision"),
    // BigInt object
    utf16!("asIntN"),
    utf16!("asUintN"),
    // RegExp object
    utf16!("exec"),
    utf16!("test"),
    utf16!("flags"),
    utf16!("index"),
    utf16!("lastIndex"),
    utf16!("hasIndices"),
    utf16!("ignoreCase"),
    utf16!("multiline"),
    utf16!("dotAll"),
    utf16!("unicode"),
    utf16!("sticky"),
    utf16!("source"),
    utf16!("get hasIndices"),
    utf16!("get global"),
    utf16!("get ignoreCase"),
    utf16!("get multiline"),
    utf16!("get dotAll"),
    utf16!("get unicode"),
    utf16!("get sticky"),
    utf16!("get flags"),
    utf16!("get source"),
    // Symbol object
    utf16!("for"),
    utf16!("keyFor"),
    utf16!("description"),
    utf16!("asyncIterator"),
    utf16!("hasInstance"),
    utf16!("species"),
    utf16!("unscopables"),
    utf16!("iterator"),
    utf16!("toStringTag"),
    utf16!("toPrimitive"),
    utf16!("get description"),
    // Map object
    utf16!("clear"),
    utf16!("delete"),
    utf16!("has"),
    utf16!("size"),
    // Set object
    utf16!("add"),
    // Reflect object
    // Proxy object
    utf16!("revocable"),
    // Error objects
    utf16!("message"),
    // Date object
    utf16!("toJSON"),
    utf16!("getDate"),
    utf16!("getDay"),
    utf16!("getFullYear"),
    utf16!("getHours"),
    utf16!("getMilliseconds"),
    utf16!("getMinutes"),
    utf16!("getMonth"),
    utf16!("getSeconds"),
    utf16!("getTime"),
    utf16!("getYear"),
    utf16!("getUTCDate"),
    utf16!("getUTCDay"),
    utf16!("getUTCFullYear"),
    utf16!("getUTCHours"),
    utf16!("getUTCMinutes"),
    utf16!("getUTCMonth"),
    utf16!("getUTCSeconds"),
    utf16!("setDate"),
    utf16!("setFullYear"),
    utf16!("setHours"),
    utf16!("setMilliseconds"),
    utf16!("setMinutes"),
    utf16!("setMonth"),
    utf16!("setSeconds"),
    utf16!("setYear"),
    utf16!("setTime"),
    utf16!("setUTCDate"),
    utf16!("setUTCFullYear"),
    utf16!("setUTCHours"),
    utf16!("setUTCMinutes"),
    utf16!("setUTCMonth"),
    utf16!("setUTCSeconds"),
    utf16!("toDateString"),
    utf16!("toGMTString"),
    utf16!("toISOString"),
    utf16!("toTimeString"),
    utf16!("toUTCString"),
    utf16!("now"),
    utf16!("UTC"),
    // JSON object
    utf16!("parse"),
    utf16!("stringify"),
    // Iterator object
    utf16!("Array Iterator"),
    utf16!("Set Iterator"),
    utf16!("String Iterator"),
    utf16!("Map Iterator"),
    utf16!("For In Iterator"),
    // Math object
    utf16!("LN10"),
    utf16!("LN2"),
    utf16!("LOG10E"),
    utf16!("LOG2E"),
    utf16!("PI"),
    utf16!("SQRT1_2"),
    utf16!("SQRT2"),
    utf16!("abs"),
    utf16!("acos"),
    utf16!("acosh"),
    utf16!("asin"),
    utf16!("asinh"),
    utf16!("atan"),
    utf16!("atanh"),
    utf16!("atan2"),
    utf16!("cbrt"),
    utf16!("ceil"),
    utf16!("clz32"),
    utf16!("cos"),
    utf16!("cosh"),
    utf16!("exp"),
    utf16!("expm1"),
    utf16!("floor"),
    utf16!("fround"),
    utf16!("hypot"),
    utf16!("imul"),
    utf16!("log"),
    utf16!("log1p"),
    utf16!("log10"),
    utf16!("log2"),
    utf16!("max"),
    utf16!("min"),
    utf16!("pow"),
    utf16!("random"),
    utf16!("round"),
    utf16!("sign"),
    utf16!("sin"),
    utf16!("sinh"),
    utf16!("sqrt"),
    utf16!("tan"),
    utf16!("tanh"),
    utf16!("trunc"),
    // TypedArray object
    utf16!("buffer"),
    utf16!("byteLength"),
    utf16!("byteOffset"),
    utf16!("isView"),
    utf16!("subarray"),
    utf16!("get byteLength"),
    utf16!("get buffer"),
    utf16!("get byteOffset"),
    utf16!("get size"),
    utf16!("get length"),
    // DataView object
    utf16!("getBigInt64"),
    utf16!("getBigUint64"),
    utf16!("getFloat32"),
    utf16!("getFloat64"),
    utf16!("getInt8"),
    utf16!("getInt16"),
    utf16!("getInt32"),
    utf16!("getUint8"),
    utf16!("getUint16"),
    utf16!("getUint32"),
    utf16!("setBigInt64"),
    utf16!("setBigUint64"),
    utf16!("setFloat32"),
    utf16!("setFloat64"),
    utf16!("setInt8"),
    utf16!("setInt16"),
    utf16!("setInt32"),
    utf16!("setUint8"),
    utf16!("setUint16"),
    utf16!("setUint32"),
    // Console object
    utf16!("console"),
    utf16!("assert"),
    utf16!("debug"),
    utf16!("error"),
    utf16!("info"),
    utf16!("trace"),
    utf16!("warn"),
    utf16!("exception"),
    utf16!("count"),
    utf16!("countReset"),
    utf16!("group"),
    utf16!("groupCollapsed"),
    utf16!("groupEnd"),
    utf16!("time"),
    utf16!("timeLog"),
    utf16!("timeEnd"),
    utf16!("dir"),
    utf16!("dirxml"),
    // Minified name
    utf16!("a"),
    utf16!("b"),
    utf16!("c"),
    utf16!("d"),
    utf16!("e"),
    utf16!("f"),
    utf16!("g"),
    utf16!("h"),
    utf16!("i"),
    utf16!("j"),
    utf16!("k"),
    utf16!("l"),
    utf16!("m"),
    utf16!("n"),
    utf16!("o"),
    utf16!("p"),
    utf16!("q"),
    utf16!("r"),
    utf16!("s"),
    utf16!("t"),
    utf16!("u"),
    utf16!("v"),
    utf16!("w"),
    utf16!("x"),
    utf16!("y"),
    utf16!("z"),
    utf16!("A"),
    utf16!("B"),
    utf16!("C"),
    utf16!("D"),
    utf16!("E"),
    utf16!("F"),
    utf16!("G"),
    utf16!("H"),
    utf16!("I"),
    utf16!("J"),
    utf16!("K"),
    utf16!("L"),
    utf16!("M"),
    utf16!("N"),
    utf16!("O"),
    utf16!("P"),
    utf16!("Q"),
    utf16!("R"),
    utf16!("S"),
    utf16!("T"),
    utf16!("U"),
    utf16!("V"),
    utf16!("W"),
    utf16!("X"),
    utf16!("Y"),
    utf16!("Z"),
    utf16!("_"),
    utf16!("$"),
];
