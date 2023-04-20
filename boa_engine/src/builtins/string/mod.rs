//! Boa's implementation of ECMAScript's global `String` object.
//!
//! The `String` global object is a constructor for strings or a sequence of characters.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-string-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String

use crate::{
    builtins::{Array, BuiltInObject, Number, RegExp},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData},
    property::{Attribute, PropertyDescriptor},
    realm::Realm,
    string::utf16,
    string::{CodePoint, Utf16Trim},
    symbol::JsSymbol,
    value::IntegerOrInfinity,
    Context, JsArgs, JsResult, JsString, JsValue,
};
use boa_profiler::Profiler;
use icu_normalizer::{ComposingNormalizer, DecomposingNormalizer};
use std::cmp::{max, min};

use super::{BuiltInBuilder, BuiltInConstructor, IntrinsicObject};

mod string_iterator;
pub(crate) use string_iterator::StringIterator;

/// The set of normalizers required for the `String.prototype.normalize` function.
#[derive(Debug)]
pub(crate) struct StringNormalizers {
    pub(crate) nfc: ComposingNormalizer,
    pub(crate) nfkc: ComposingNormalizer,
    pub(crate) nfd: DecomposingNormalizer,
    pub(crate) nfkd: DecomposingNormalizer,
}

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Placement {
    Start,
    End,
}

/// Helper function to check if a `char` is trimmable.
pub(crate) const fn is_trimmable_whitespace(c: char) -> bool {
    // The rust implementation of `trim` does not regard the same characters whitespace as ecma standard does
    //
    // Rust uses \p{White_Space} by default, which also includes:
    // `\u{0085}' (next line)
    // And does not include:
    // '\u{FEFF}' (zero width non-breaking space)
    // Explicit whitespace: https://tc39.es/ecma262/#sec-white-space
    matches!(
        c,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{FEFF}' |
    // Unicode Space_Separator category
    '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' |
    // Line terminators: https://tc39.es/ecma262/#sec-line-terminators
    '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}'
    )
}

/// JavaScript `String` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct String;

impl IntrinsicObject for String {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let symbol_iterator = JsSymbol::iterator();

        let trim_start = BuiltInBuilder::new(realm)
            .callable(Self::trim_start)
            .length(0)
            .name("trimStart")
            .build();

        let trim_end = BuiltInBuilder::new(realm)
            .callable(Self::trim_end)
            .length(0)
            .name("trimEnd")
            .build();

        #[cfg(feature = "annex-b")]
        let trim_left = trim_start.clone();

        #[cfg(feature = "annex-b")]
        let trim_right = trim_end.clone();

        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT;
        let builder = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(utf16!("length"), 0, attribute)
            .property(
                utf16!("trimStart"),
                trim_start,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                utf16!("trimEnd"),
                trim_end,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_method(Self::raw, "raw", 1)
            .static_method(Self::from_char_code, "fromCharCode", 1)
            .static_method(Self::from_code_point, "fromCodePoint", 1)
            .method(Self::char_at, "charAt", 1)
            .method(Self::char_code_at, "charCodeAt", 1)
            .method(Self::code_point_at, "codePointAt", 1)
            .method(Self::to_string, "toString", 0)
            .method(Self::concat, "concat", 1)
            .method(Self::repeat, "repeat", 1)
            .method(Self::slice, "slice", 2)
            .method(Self::starts_with, "startsWith", 1)
            .method(Self::ends_with, "endsWith", 1)
            .method(Self::includes, "includes", 1)
            .method(Self::index_of, "indexOf", 1)
            .method(Self::last_index_of, "lastIndexOf", 1)
            .method(Self::locale_compare, "localeCompare", 1)
            .method(Self::r#match, "match", 1)
            .method(Self::normalize, "normalize", 0)
            .method(Self::pad_end, "padEnd", 1)
            .method(Self::pad_start, "padStart", 1)
            .method(Self::trim, "trim", 0)
            .method(Self::to_case::<false>, "toLowerCase", 0)
            .method(Self::to_case::<true>, "toUpperCase", 0)
            .method(Self::to_locale_case::<false>, "toLocaleLowerCase", 0)
            .method(Self::to_locale_case::<true>, "toLocaleUpperCase", 0)
            .method(Self::substring, "substring", 2)
            .method(Self::split, "split", 2)
            .method(Self::value_of, "valueOf", 0)
            .method(Self::match_all, "matchAll", 1)
            .method(Self::replace, "replace", 2)
            .method(Self::replace_all, "replaceAll", 2)
            .method(Self::iterator, (symbol_iterator, "[Symbol.iterator]"), 0)
            .method(Self::search, "search", 1)
            .method(Self::at, "at", 1);

        #[cfg(feature = "annex-b")]
        {
            builder
                .property(
                    utf16!("trimLeft"),
                    trim_left,
                    Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
                )
                .property(
                    utf16!("trimRight"),
                    trim_right,
                    Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
                )
                .method(Self::substr, "substr", 2)
                .method(Self::anchor, "anchor", 1)
                .method(Self::big, "big", 0)
                .method(Self::blink, "blink", 0)
                .method(Self::bold, "bold", 0)
                .method(Self::fixed, "fixed", 0)
                .method(Self::fontcolor, "fontcolor", 1)
                .method(Self::fontsize, "fontsize", 1)
                .method(Self::italics, "italics", 0)
                .method(Self::link, "link", 1)
                .method(Self::small, "small", 0)
                .method(Self::strike, "strike", 0)
                .method(Self::sub, "sub", 0)
                .method(Self::sup, "sup", 0)
                .build();
        }

        #[cfg(not(feature = "annex-b"))]
        builder.build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for String {
    const NAME: &'static str = "String";
}

impl BuiltInConstructor for String {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::string;

    /// Constructor `String( value )`
    ///
    /// <https://tc39.es/ecma262/#sec-string-constructor-string-value>
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        let string = match args.get(0) {
            // 2. Else,
            // a. If NewTarget is undefined and Type(value) is Symbol, return SymbolDescriptiveString(value).
            Some(JsValue::Symbol(ref sym)) if new_target.is_undefined() => {
                return Ok(sym.descriptive_string().into())
            }
            // b. Let s be ? ToString(value).
            Some(value) => value.to_string(context)?,
            // 1. If value is not present, let s be the empty String.
            None => js_string!(),
        };

        // 3. If NewTarget is undefined, return s.
        if new_target.is_undefined() {
            return Ok(string.into());
        }

        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::string, context)?;
        // 4. Return ! StringCreate(s, ? GetPrototypeFromConstructor(NewTarget, "%String.prototype%")).
        Ok(Self::string_create(string, prototype, context).into())
    }
}

impl String {
    /// JavaScript strings must be between `0` and less than positive `Infinity` and cannot be a negative number.
    /// The range of allowed values can be described like this: `[0, +∞)`.
    ///
    /// The resulting string can also not be larger than the maximum string size,
    /// which can differ in JavaScript engines. In Boa it is `2^32 - 1`
    pub(crate) const MAX_STRING_LENGTH: usize = u32::MAX as usize;

    /// Abstract function `StringCreate( value, prototype )`.
    ///
    /// Call this function if you want to create a `String` exotic object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringcreate
    fn string_create(value: JsString, prototype: JsObject, context: &mut Context<'_>) -> JsObject {
        // 7. Let length be the number of code unit elements in value.
        let len = value.len();

        // 1. Let S be ! MakeBasicObject(« [[Prototype]], [[Extensible]], [[StringData]] »).
        // 2. Set S.[[Prototype]] to prototype.
        // 3. Set S.[[StringData]] to value.
        // 4. Set S.[[GetOwnProperty]] as specified in 10.4.3.1.
        // 5. Set S.[[DefineOwnProperty]] as specified in 10.4.3.2.
        // 6. Set S.[[OwnPropertyKeys]] as specified in 10.4.3.3.
        let s = JsObject::from_proto_and_data(prototype, ObjectData::string(value));

        // 8. Perform ! DefinePropertyOrThrow(S, "length", PropertyDescriptor { [[Value]]: 𝔽(length),
        // [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }).
        s.define_property_or_throw(
            utf16!("length"),
            PropertyDescriptor::builder()
                .value(len)
                .writable(false)
                .enumerable(false)
                .configurable(false),
            context,
        )
        .expect("length definition for a new string must not fail");

        // 9. Return S.
        s
    }

    /// Abstract operation `thisStringValue( value )`
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#thisstringvalue
    fn this_string_value(this: &JsValue) -> JsResult<JsString> {
        // 1. If Type(value) is String, return value.
        this.as_string()
            .cloned()
            // 2. If Type(value) is Object and value has a [[StringData]] internal slot, then
            //     a. Let s be value.[[StringData]].
            //     b. Assert: Type(s) is String.
            //     c. Return s.
            .or_else(|| this.as_object().and_then(|obj| obj.borrow().as_string()))
            // 3. Throw a TypeError exception.
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a string")
                    .into()
            })
    }

    /// `String.fromCodePoint(num1[, ...[, numN]])`
    ///
    /// The static `String.fromCodePoint()` method returns a string created by using the specified sequence of code points.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.fromcodepoint
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fromCodePoint
    pub(crate) fn from_code_point(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let result be the empty String.
        let mut result = Vec::with_capacity(args.len());

        let mut buf = [0; 2];

        // 2. For each element next of codePoints, do
        for arg in args.iter() {
            // a. Let nextCP be ? ToNumber(next).
            let nextcp = arg.to_number(context)?;

            // b. If ! IsIntegralNumber(nextCP) is false, throw a RangeError exception.
            if !Number::is_float_integer(nextcp) {
                return Err(JsNativeError::range()
                    .with_message(format!("codepoint `{nextcp}` is not an integer"))
                    .into());
            }

            // c. If ℝ(nextCP) < 0 or ℝ(nextCP) > 0x10FFFF, throw a RangeError exception.
            if nextcp < 0.0 || nextcp > f64::from(0x0010_FFFF) {
                return Err(JsNativeError::range()
                    .with_message(format!("codepoint `{nextcp}` outside of Unicode range"))
                    .into());
            }

            // SAFETY:
            // - `nextcp` is not NaN (by the call to `is_float_integer`).
            // - `nextcp` is not infinite (by the call to `is_float_integer`).
            // - `nextcp` is in the u32 range (by the check above).
            let nextcp = unsafe { nextcp.to_int_unchecked::<u32>() };

            // d. Set result to the string-concatenation of result and ! UTF16EncodeCodePoint(ℝ(nextCP)).
            result.extend_from_slice(match u16::try_from(nextcp) {
                Ok(ref cp) => std::slice::from_ref(cp),
                Err(_) => char::from_u32(nextcp)
                    .expect("u32 is in range and cannot be a surrogate by the conversion above")
                    .encode_utf16(&mut buf),
            });
        }

        // 3. Assert: If codePoints is empty, then result is the empty String.
        // 4. Return result.
        Ok(js_string!(&result[..]).into())
    }

    /// `String.raw( template, ...substitutions )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.raw
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/raw
    pub(crate) fn raw(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let substitutions = args.get(1..).unwrap_or_default();

        // 1. Let numberOfSubstitutions be the number of elements in substitutions.
        let number_of_substitutions = substitutions.len() as u64;

        // 2. Let cooked be ? ToObject(template).
        let cooked = args.get_or_undefined(0).to_object(context)?;

        // 3. Let raw be ? ToObject(? Get(cooked, "raw")).
        let raw = cooked.get(utf16!("raw"), context)?.to_object(context)?;

        // 4. Let literalSegments be ? LengthOfArrayLike(raw).
        let literal_segments = raw.length_of_array_like(context)?;

        // 5. If literalSegments ≤ 0, return the empty String.
        // This is not <= because a `usize` is always positive.
        if literal_segments == 0 {
            return Ok(js_string!().into());
        }

        // 6. Let stringElements be a new empty List.
        let mut string_elements = vec![];

        // 7. Let nextIndex be 0.
        let mut next_index = 0;
        // 8. Repeat,
        loop {
            // a. Let nextKey be ! ToString(𝔽(nextIndex)).
            let next_key = next_index;

            // b. Let nextSeg be ? ToString(? Get(raw, nextKey)).
            let next_seg = raw.get(next_key, context)?.to_string(context)?;

            // c. Append the code unit elements of nextSeg to the end of stringElements.
            string_elements.extend(next_seg.iter().copied());

            // d. If nextIndex + 1 = literalSegments, then
            if next_index + 1 == literal_segments {
                // i. Return the String value whose code units are the elements in the List stringElements.
                //    If stringElements has no elements, the empty String is returned.
                return Ok(js_string!(string_elements).into());
            }

            // e. If nextIndex < numberOfSubstitutions, let next be substitutions[nextIndex].
            let next = if next_index < number_of_substitutions {
                substitutions.get_or_undefined(next_index as usize).clone()

            // f. Else, let next be the empty String.
            } else {
                js_string!().into()
            };

            // g. Let nextSub be ? ToString(next).
            let next_sub = next.to_string(context)?;

            // h. Append the code unit elements of nextSub to the end of stringElements.
            string_elements.extend(next_sub.iter().copied());

            // i. Set nextIndex to nextIndex + 1.
            next_index += 1;
        }
    }

    /// `String.fromCharCode(...codeUnits)`
    ///
    /// Construct a `String` from one or more code points (as numbers).
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/multipage/text-processing.html#sec-string.fromcharcode
    pub(crate) fn from_char_code(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let result be the empty String.
        let mut result = Vec::new();

        // 2. For each element next of codeUnits, do
        for next in args {
            // a. Let nextCU be the code unit whose numeric value is ℝ(? ToUint16(next)).
            let next_cu = next.to_uint16(context)?;

            // b. Set result to the string-concatenation of result and nextCU.
            result.push(next_cu);
        }

        // 3. Return result.
        Ok(js_string!(result).into())
    }

    /// `String.prototype.toString ( )`
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.tostring
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Return ? thisStringValue(this value).
        Ok(Self::this_string_value(this)?.into())
    }

    /// `String.prototype.charAt( index )`
    ///
    /// The `String` object's `charAt()` method returns a new string consisting of the single UTF-16 code unit located at the specified offset into the string.
    ///
    /// Characters in a string are indexed from left to right. The index of the first character is `0`,
    /// and the index of the last character—in a string called `stringName`—is `stringName.length - 1`.
    /// If the `index` you supply is out of this range, JavaScript returns an empty string.
    ///
    /// If no index is provided to `charAt()`, the default is `0`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.charat
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/charAt
    pub(crate) fn char_at(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        // 3. Let position be ? ToIntegerOrInfinity(pos).
        let position = args.get_or_undefined(0).to_integer_or_infinity(context)?;

        match position {
            // 4. Let size be the length of S.
            // 6. Return the substring of S from position to position + 1.
            IntegerOrInfinity::Integer(i) if i >= 0 && i < string.len() as i64 => {
                let i = i as usize;
                Ok(js_string!(&string[i..=i]).into())
            }
            // 5. If position < 0 or position ≥ size, return the empty String.
            _ => Ok(js_string!().into()),
        }
    }

    /// `String.prototype.at ( index )`
    ///
    /// This String object's at() method returns a String consisting of the single UTF-16 code unit located at the specified position.
    /// Returns undefined if the given index cannot be found.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/proposal-relative-indexing-method/#sec-string.prototype.at
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/at
    pub(crate) fn at(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let s = this.to_string(context)?;

        // 3. Let len be the length of S.
        let len = s.len() as i64;

        // 4. Let relativeIndex be ? ToIntegerOrInfinity(index).
        let relative_index = args.get_or_undefined(0).to_integer_or_infinity(context)?;
        let k = match relative_index {
            // 5. If relativeIndex ≥ 0, then
            // a. Let k be relativeIndex.
            IntegerOrInfinity::Integer(i) if i >= 0 && i < len => i as usize,
            // 6. Else,
            // a. Let k be len + relativeIndex.
            IntegerOrInfinity::Integer(i) if i < 0 && (-i) <= len => (len + i) as usize,
            // 7. If k < 0 or k ≥ len, return undefined.
            _ => return Ok(JsValue::undefined()),
        };

        // 8. Return the substring of S from k to k + 1.
        Ok(js_string!(&s[k..=k]).into())
    }

    /// `String.prototype.codePointAt( index )`
    ///
    /// The `codePointAt()` method returns an integer between `0` to `1114111` (`0x10FFFF`) representing the UTF-16 code unit at the given index.
    ///
    /// If no UTF-16 surrogate pair begins at the index, the code point at the index is returned.
    ///
    /// `codePointAt()` returns `undefined` if the given index is less than `0`, or if it is equal to or greater than the `length` of the string.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.codepointat
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/codePointAt
    pub(crate) fn code_point_at(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        // 3. Let position be ? ToIntegerOrInfinity(pos).
        let position = args.get_or_undefined(0).to_integer_or_infinity(context)?;

        match position {
            // 4. Let size be the length of S.
            IntegerOrInfinity::Integer(i) if i >= 0 && i < string.len() as i64 => {
                // 6. Let cp be ! CodePointAt(S, position).
                // 7. Return 𝔽(cp.[[CodePoint]]).
                Ok(string
                    .code_point_at(usize::try_from(i).expect("already checked that i >= 0"))
                    .as_u32()
                    .into())
            }
            // 5. If position < 0 or position ≥ size, return undefined.
            _ => Ok(JsValue::undefined()),
        }
    }

    /// `String.prototype.charCodeAt( index )`
    ///
    /// The `charCodeAt()` method returns an integer between `0` and `65535` representing the UTF-16 code unit at the given index.
    ///
    /// Unicode code points range from `0` to `1114111` (`0x10FFFF`). The first 128 Unicode code points are a direct match of the ASCII character encoding.
    ///
    /// `charCodeAt()` returns `NaN` if the given index is less than `0`, or if it is equal to or greater than the `length` of the string.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.charcodeat
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/charCodeAt
    pub(crate) fn char_code_at(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        // 3. Let position be ? ToIntegerOrInfinity(pos).
        let position = args.get_or_undefined(0).to_integer_or_infinity(context)?;

        match position {
            // 4. Let size be the length of S.
            IntegerOrInfinity::Integer(i) if i >= 0 && i < string.len() as i64 => {
                // 6. Return the Number value for the numeric value of the code unit at index position within the String S.
                Ok(u32::from(string[i as usize]).into())
            }
            // 5. If position < 0 or position ≥ size, return NaN.
            _ => Ok(JsValue::nan()),
        }
    }

    /// `String.prototype.concat( str1[, ...strN] )`
    ///
    /// The `concat()` method concatenates the string arguments to the calling string and returns a new string.
    ///
    /// Changes to the original string or the returned string don't affect the other.
    ///
    /// If the arguments are not of the type string, they are converted to string values before concatenating.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.concat
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/concat
    pub(crate) fn concat(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let mut string = this.to_string(context)?;

        // 3. Let R be S.
        // 4. For each element next of args, do
        for arg in args {
            // a. Let nextString be ? ToString(next).
            // b. Set R to the string-concatenation of R and nextString.
            string = js_string!(&string, &arg.to_string(context)?);
        }

        // 5. Return R.
        Ok(JsValue::new(string))
    }

    /// `String.prototype.repeat( count )`
    ///
    /// The `repeat()` method constructs and returns a new string which contains the specified number of
    /// copies of the string on which it was called, concatenated together.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.repeat
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/repeat
    pub(crate) fn repeat(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        let len = string.len();

        // 3. Let n be ? ToIntegerOrInfinity(count).
        match args.get_or_undefined(0).to_integer_or_infinity(context)? {
            IntegerOrInfinity::Integer(n)
                if n > 0 && (n as usize) * len <= Self::MAX_STRING_LENGTH =>
            {
                if string.is_empty() {
                    return Ok(js_string!().into());
                }
                let n = n as usize;
                let mut result = Vec::with_capacity(n * len);

                std::iter::repeat(&string[..])
                    .take(n)
                    .for_each(|s| result.extend_from_slice(s));

                // 6. Return the String value that is made from n copies of S appended together.
                Ok(js_string!(result).into())
            }
            // 5. If n is 0, return the empty String.
            IntegerOrInfinity::Integer(n) if n == 0 => Ok(js_string!().into()),
            // 4. If n < 0 or n is +∞, throw a RangeError exception.
            _ => Err(JsNativeError::range()
                .with_message(
                    "repeat count must be a positive finite number \
                        that doesn't overflow the maximum string length (2^32 - 1)",
                )
                .into()),
        }
    }

    /// `String.prototype.slice( beginIndex [, endIndex] )`
    ///
    /// The `slice()` method extracts a section of a string and returns it as a new string, without modifying the original string.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.slice
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/slice
    pub(crate) fn slice(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        // 3. Let len be the length of S.
        let len = string.len() as i64;

        // 4. Let intStart be ? ToIntegerOrInfinity(start).
        let from = match args.get_or_undefined(0).to_integer_or_infinity(context)? {
            // 6. Else if intStart < 0, let from be max(len + intStart, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => max(len + i, 0),

            // 7. Else, let from be min(intStart, len).
            IntegerOrInfinity::Integer(i) => min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,

            // 5. If intStart is -∞, let from be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
        } as usize;

        // 8. If end is undefined, let intEnd be len; else let intEnd be ? ToIntegerOrInfinity(end).
        let to = match args
            .get(1)
            .filter(|end| !end.is_undefined())
            .map(|end| end.to_integer_or_infinity(context))
            .transpose()?
            .unwrap_or(IntegerOrInfinity::Integer(len))
        {
            // 10. Else if intEnd < 0, let to be max(len + intEnd, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => max(len + i, 0),

            // 11. Else, let to be min(intEnd, len).
            IntegerOrInfinity::Integer(i) => min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,

            // 9. If intEnd is -∞, let to be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
        } as usize;

        // 12. If from ≥ to, return the empty String.
        if from >= to {
            Ok(js_string!().into())
        } else {
            // 13. Return the substring of S from from to to.
            Ok(js_string!(&string[from..to]).into())
        }
    }

    /// `String.prototype.startWith( searchString[, position] )`
    ///
    /// The `startsWith()` method determines whether a string begins with the characters of a specified string, returning `true` or `false` as appropriate.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.startswith
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/startsWith
    pub(crate) fn starts_with(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        let search_string = args.get_or_undefined(0);

        // 3. Let isRegExp be ? IsRegExp(searchString).
        // 4. If isRegExp is true, throw a TypeError exception.
        if is_reg_exp(search_string, context)? {
            return Err(JsNativeError::typ().with_message(
                "First argument to String.prototype.startsWith must not be a regular expression",
            ).into());
        }

        // 5. Let searchStr be ? ToString(searchString).
        let search_string = search_string.to_string(context)?;

        // 6. Let len be the length of S.
        let len = string.len() as i64;

        // 7. If position is undefined, let pos be 0; else let pos be ? ToIntegerOrInfinity(position).
        let pos = match args.get_or_undefined(1) {
            &JsValue::Undefined => IntegerOrInfinity::Integer(0),
            position => position.to_integer_or_infinity(context)?,
        };

        // 8. Let start be the result of clamping pos between 0 and len.
        let start = pos.clamp_finite(0, len) as usize;

        // 9. Let searchLength be the length of searchStr.
        let search_length = search_string.len();

        // 10. If searchLength = 0, return true.
        if search_length == 0 {
            return Ok(JsValue::new(true));
        }

        // 11. Let end be start + searchLength.
        let end = start + search_length;

        // 12. If end > len, return false.
        if end > len as usize {
            Ok(JsValue::new(false))
        } else {
            // 13. Let substring be the substring of S from start to end.
            // 14. Return ! SameValueNonNumeric(substring, searchStr).
            // `SameValueNonNumeric` forwards to `==`, so directly check
            // equality to avoid converting to `JsValue`
            Ok(JsValue::new(search_string == string[start..end]))
        }
    }

    /// `String.prototype.endsWith( searchString[, length] )`
    ///
    /// The `endsWith()` method determines whether a string ends with the characters of a specified string, returning `true` or `false` as appropriate.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.endswith
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/endsWith
    pub(crate) fn ends_with(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        let search_str = match args.get_or_undefined(0) {
            // 3. Let isRegExp be ? IsRegExp(searchString).
            // 4. If isRegExp is true, throw a TypeError exception.
            search_string if is_reg_exp(search_string, context)? => {
                return Err(JsNativeError::typ().with_message(
                    "First argument to String.prototype.endsWith must not be a regular expression",
                ).into());
            }
            // 5. Let searchStr be ? ToString(searchString).
            search_string => search_string.to_string(context)?,
        };

        // 6. Let len be the length of S.
        let len = string.len() as i64;

        // 7. If endPosition is undefined, let pos be len; else let pos be ? ToIntegerOrInfinity(endPosition).
        let end = match args.get_or_undefined(1) {
            end_position if end_position.is_undefined() => IntegerOrInfinity::Integer(len),
            end_position => end_position.to_integer_or_infinity(context)?,
        };

        // 8. Let end be the result of clamping pos between 0 and len.
        let end = end.clamp_finite(0, len) as usize;

        // 9. Let searchLength be the length of searchStr.
        let search_length = search_str.len();

        // 10. If searchLength = 0, return true.
        if search_length == 0 {
            return Ok(true.into());
        }

        // 11. Let start be end - searchLength.
        if let Some(start) = end.checked_sub(search_length) {
            // 13. Let substring be the substring of S from start to end.
            // 14. Return ! SameValueNonNumeric(substring, searchStr).
            // `SameValueNonNumeric` forwards to `==`, so directly check
            // equality to avoid converting to `JsValue`
            Ok(JsValue::new(search_str == string[start..end]))
        } else {
            // 12. If start < 0, return false.
            Ok(false.into())
        }
    }

    /// `String.prototype.includes( searchString[, position] )`
    ///
    /// The `includes()` method determines whether one string may be found within another string, returning `true` or `false` as appropriate.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.includes
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/includes
    pub(crate) fn includes(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        let search_str = match args.get_or_undefined(0) {
            // 3. Let isRegExp be ? IsRegExp(searchString).
            search_string if is_reg_exp(search_string, context)? => {
                return Err(JsNativeError::typ().with_message(
                    // 4. If isRegExp is true, throw a TypeError exception.
                    "First argument to String.prototype.includes must not be a regular expression",
                ).into());
            }
            // 5. Let searchStr be ? ToString(searchString).
            search_string => search_string.to_string(context)?,
        };

        // 6. Let pos be ? ToIntegerOrInfinity(position).
        // 7. Assert: If position is undefined, then pos is 0.
        let pos = args.get_or_undefined(1).to_integer_or_infinity(context)?;

        // 8. Let len be the length of S.
        // 9. Let start be the result of clamping pos between 0 and len.
        let start = pos.clamp_finite(0, string.len() as i64) as usize;

        // 10. Let index be ! StringIndexOf(S, searchStr, start).
        // 11. If index is not -1, return true.
        // 12. Return false.
        Ok(string.index_of(&search_str, start).is_some().into())
    }

    /// `String.prototype.replace( regexp|substr, newSubstr|function )`
    ///
    /// The `replace()` method returns a new string with some or all matches of a `pattern` replaced by a `replacement`.
    ///
    /// The `pattern` can be a string or a `RegExp`, and the `replacement` can be a string or a function to be called for each match.
    /// If `pattern` is a string, only the first occurrence will be replaced.
    ///
    /// The original string is left unchanged.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.replace
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replace
    pub(crate) fn replace(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // Helper enum.
        enum CallableOrString<'a> {
            FunctionalReplace(&'a JsObject),
            ReplaceValue(JsString),
        }

        // 1. Let O be ? RequireObjectCoercible(this value).
        let o = this.require_object_coercible()?;

        let search_value = args.get_or_undefined(0);
        let replace_value = args.get_or_undefined(1);

        // 2. If searchValue is neither undefined nor null, then
        if !search_value.is_null_or_undefined() {
            // a. Let replacer be ? GetMethod(searchValue, @@replace).
            let replacer = search_value.get_method(JsSymbol::replace(), context)?;

            // b. If replacer is not undefined, then
            if let Some(replacer) = replacer {
                // i. Return ? Call(replacer, searchValue, « O, replaceValue »).
                return replacer.call(search_value, &[o.clone(), replace_value.clone()], context);
            }
        }

        // 3. Let string be ? ToString(O).
        let string = o.to_string(context)?;

        // 4. Let searchString be ? ToString(searchValue).
        let search_string = search_value.to_string(context)?;

        // 5. Let functionalReplace be IsCallable(replaceValue).
        let functional_replace = replace_value.as_callable();

        // 6. If functionalReplace is false, then
        let replace_value = if let Some(callable) = functional_replace {
            CallableOrString::FunctionalReplace(callable)
        } else {
            // a. Set replaceValue to ? ToString(replaceValue).
            CallableOrString::ReplaceValue(replace_value.to_string(context)?)
        };

        // 7. Let searchLength be the length of searchString.
        let search_length = search_string.len();

        // 8. Let position be ! StringIndexOf(string, searchString, 0).
        // 9. If position is -1, return string.
        let Some(position) = string.index_of(&search_string, 0) else {
            return Ok(string.into());
        };

        // 10. Let preserved be the substring of string from 0 to position.
        let preserved = &string[..position];

        let replacement = match replace_value {
            // 11. If functionalReplace is true, then
            CallableOrString::FunctionalReplace(replace_fn) => {
                // a. Let replacement be ? ToString(? Call(replaceValue, undefined, « searchString, 𝔽(position), string »)).
                replace_fn
                    .call(
                        &JsValue::undefined(),
                        &[search_string.into(), position.into(), string.clone().into()],
                        context,
                    )?
                    .to_string(context)?
            }
            // 12. Else,
            CallableOrString::ReplaceValue(replace_value) => {
                // a. Assert: Type(replaceValue) is String.
                // b. Let captures be a new empty List.
                let captures = Vec::new();

                // c. Let replacement be ! GetSubstitution(searchString, string, position, captures, undefined, replaceValue).
                get_substitution(
                    &search_string,
                    &string,
                    position,
                    &captures,
                    &JsValue::undefined(),
                    &replace_value,
                    context,
                )?
            }
        };

        // 13. Return the string-concatenation of preserved, replacement, and the substring of string from position + searchLength.
        Ok(js_string!(preserved, &replacement, &string[position + search_length..]).into())
    }

    /// `22.1.3.18 String.prototype.replaceAll ( searchValue, replaceValue )`
    ///
    /// The replaceAll() method returns a new string with all matches of a pattern replaced by a
    /// replacement.
    ///
    /// The pattern can be a string or a `RegExp`, and the replacement can be a string or a
    /// function to be called for each match.
    ///
    /// The original string is left unchanged.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.replaceall
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replace
    pub(crate) fn replace_all(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let o = this.require_object_coercible()?;

        let search_value = args.get_or_undefined(0);
        let replace_value = args.get_or_undefined(1);

        // 2. If searchValue is neither undefined nor null, then
        if !search_value.is_null_or_undefined() {
            // a. Let isRegExp be ? IsRegExp(searchValue).
            if let Some(obj) = search_value.as_object() {
                // b. If isRegExp is true, then
                if is_reg_exp_object(obj, context)? {
                    // i. Let flags be ? Get(searchValue, "flags").
                    let flags = obj.get(utf16!("flags"), context)?;

                    // ii. Perform ? RequireObjectCoercible(flags).
                    flags.require_object_coercible()?;

                    // iii. If ? ToString(flags) does not contain "g", throw a TypeError exception.
                    if !flags.to_string(context)?.contains(&u16::from(b'g')) {
                        return Err(JsNativeError::typ().with_message(
                            "String.prototype.replaceAll called with a non-global RegExp argument",
                        ).into());
                    }
                }
            }

            // c. Let replacer be ? GetMethod(searchValue, @@replace).
            let replacer = search_value.get_method(JsSymbol::replace(), context)?;

            // d. If replacer is not undefined, then
            if let Some(replacer) = replacer {
                // i. Return ? Call(replacer, searchValue, « O, replaceValue »).
                return replacer.call(search_value, &[o.clone(), replace_value.clone()], context);
            }
        }

        // 3. Let string be ? ToString(O).
        let string = o.to_string(context)?;

        // 4. Let searchString be ? ToString(searchValue).
        let search_string = search_value.to_string(context)?;

        // 5. Let functionalReplace be IsCallable(replaceValue).
        let replace = if let Some(f) = replace_value.as_callable() {
            Ok(f)
        } else {
            // 6. If functionalReplace is false, then
            // a. Set replaceValue to ? ToString(replaceValue).
            Err(replace_value.to_string(context)?)
        };

        // 7. Let searchLength be the length of searchString.
        let search_length = search_string.len();

        // 8. Let advanceBy be max(1, searchLength).
        let advance_by = max(1, search_length);

        // 9. Let matchPositions be a new empty List.
        let mut match_positions = Vec::new();

        // 10. Let position be ! StringIndexOf(string, searchString, 0).
        let mut position = string.index_of(&search_string, 0);

        // 11. Repeat, while position is not -1,
        while let Some(p) = position {
            // a. Append position to the end of matchPositions.
            match_positions.push(p);

            // b. Set position to ! StringIndexOf(string, searchString, position + advanceBy).
            position = string.index_of(&search_string, p + advance_by);
        }

        // 12. Let endOfLastMatch be 0.
        let mut end_of_last_match = 0;

        // 13. Let result be the empty String.
        let mut result = Vec::with_capacity(string.len());

        // 14. For each element p of matchPositions, do
        for p in match_positions {
            // a. Let preserved be the substring of string from endOfLastMatch to p.
            let preserved = &string[end_of_last_match..p];

            // c. Else,
            let replacement = match replace {
                // b. If functionalReplace is true, then
                Ok(replace_fn) => {
                    // i. Let replacement be ? ToString(? Call(replaceValue, undefined, « searchString, 𝔽(p), string »)).
                    replace_fn
                        .call(
                            &JsValue::undefined(),
                            &[
                                search_string.clone().into(),
                                p.into(),
                                string.clone().into(),
                            ],
                            context,
                        )?
                        .to_string(context)?
                }
                // i. Assert: Type(replaceValue) is String.
                // ii. Let captures be a new empty List.
                // iii. Let replacement be ! GetSubstitution(searchString, string, p, captures, undefined, replaceValue).
                Err(ref replace_str) => get_substitution(
                    &search_string,
                    &string,
                    p,
                    &[],
                    &JsValue::undefined(),
                    replace_str,
                    context,
                )
                .expect("GetSubstitution should never fail here."),
            };

            // d. Set result to the string-concatenation of result, preserved, and replacement.
            result.extend_from_slice(preserved);
            result.extend_from_slice(&replacement);

            // e. Set endOfLastMatch to p + searchLength.
            end_of_last_match = p + search_length;
        }

        // 15. If endOfLastMatch < the length of string, then
        if end_of_last_match < string.len() {
            // a. Set result to the string-concatenation of result and the substring of string from endOfLastMatch.
            result.extend_from_slice(&string[end_of_last_match..]);
        }

        // 16. Return result.
        Ok(js_string!(result).into())
    }

    /// `String.prototype.indexOf( searchValue[, fromIndex] )`
    ///
    /// The `indexOf()` method returns the index within the calling `String` object of the first occurrence
    /// of the specified value, starting the search at `fromIndex`.
    ///
    /// Returns `-1` if the value is not found.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.indexof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/indexOf
    pub(crate) fn index_of(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        // 3. Let searchStr be ? ToString(searchString).
        let search_str = args.get_or_undefined(0).to_string(context)?;

        // 4. Let pos be ? ToIntegerOrInfinity(position).
        // 5. Assert: If position is undefined, then pos is 0.
        let pos = args.get_or_undefined(1).to_integer_or_infinity(context)?;

        // 6. Let len be the length of S.
        let len = string.len() as i64;

        // 7. Let start be the result of clamping pos between 0 and len.
        let start = pos.clamp_finite(0, len) as usize;

        // 8. Return 𝔽(! StringIndexOf(S, searchStr, start)).
        Ok(string
            .index_of(&search_str, start)
            .map_or(-1, |i| i as i64)
            .into())
    }

    /// `String.prototype.lastIndexOf( searchValue[, fromIndex] )`
    ///
    /// The `lastIndexOf()` method returns the index within the calling `String` object of the last occurrence
    /// of the specified value, searching backwards from `fromIndex`.
    ///
    /// Returns `-1` if the value is not found.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.lastindexof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/lastIndexOf
    pub(crate) fn last_index_of(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        // 3. Let searchStr be ? ToString(searchString).
        let search_str = args.get_or_undefined(0).to_string(context)?;

        // 4. Let numPos be ? ToNumber(position).
        // 5. Assert: If position is undefined, then numPos is NaN.
        let num_pos = args.get_or_undefined(1).to_number(context)?;

        // 6. If numPos is NaN, let pos be +∞; otherwise, let pos be ! ToIntegerOrInfinity(numPos).
        let pos = if num_pos.is_nan() {
            IntegerOrInfinity::PositiveInfinity
        } else {
            JsValue::new(num_pos)
                .to_integer_or_infinity(context)
                .expect("Already called `to_number so this must not fail.")
        };

        // 7. Let len be the length of S.
        let len = string.len();
        // 8. Let start be the result of clamping pos between 0 and len.
        let start = pos.clamp_finite(0, len as i64) as usize;

        // 9. If searchStr is the empty String, return 𝔽(start).
        if search_str.is_empty() {
            return Ok(JsValue::new(start));
        }

        // 10. Let searchLen be the length of searchStr.
        let search_len = search_str.len();

        if let Some(end) = len.checked_sub(search_len) {
            // 11. For each non-negative integer i starting with start such that i ≤ len - searchLen, in descending order, do
            for i in (0..=min(start, end)).rev() {
                // a. Let candidate be the substring of S from i to i + searchLen.
                let candidate = &string[i..i + search_len];

                // b. If candidate is the same sequence of code units as searchStr, return 𝔽(i).
                if candidate == &search_str {
                    return Ok(i.into());
                }
            }
        }

        // 12. Return -1𝔽.
        Ok(JsValue::new(-1))
    }

    /// [`String.prototype.localeCompare ( that [ , locales [ , options ] ] )`][spec]
    ///
    /// Returns a number indicating whether a reference string comes before, or after, or is the
    /// same as the given string in sort order.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sup-String.prototype.localeCompare
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/localeCompare
    pub(crate) fn locale_compare(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let o = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let s = o.to_string(context)?;

        // 3. Let thatValue be ? ToString(that).
        let that_value = args.get_or_undefined(0).to_string(context)?;

        let cmp = {
            #[cfg(feature = "intl")]
            {
                // 4. Let collator be ? Construct(%Collator%, « locales, options »).
                let collator = crate::builtins::intl::collator::Collator::constructor(
                    &context
                        .intrinsics()
                        .constructors()
                        .collator()
                        .constructor()
                        .into(),
                    args.get(1..).unwrap_or_default(),
                    context,
                )?;

                let collator = collator
                    .as_object()
                    .map(JsObject::borrow)
                    .expect("constructor must return a JsObject");
                let collator = collator
                    .as_collator()
                    .expect("constructor must return a `Collator` object")
                    .collator();

                collator.compare_utf16(&s, &that_value) as i8
            }

            // Default to common comparison if the user doesn't have `Intl` enabled.
            #[cfg(not(feature = "intl"))]
            {
                s.cmp(&that_value) as i8
            }
        };

        // 5. Return CompareStrings(collator, S, thatValue).
        Ok(cmp.into())
    }

    /// `String.prototype.match( regexp )`
    ///
    /// The `match()` method retrieves the result of matching a **string** against a [`regular expression`][regex].
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.match
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/match
    /// [regex]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_Expressions
    pub(crate) fn r#match(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let o = this.require_object_coercible()?;

        // 2. If regexp is neither undefined nor null, then
        let regexp = args.get_or_undefined(0);
        if !regexp.is_null_or_undefined() {
            // a. Let matcher be ? GetMethod(regexp, @@match).
            let matcher = regexp.get_method(JsSymbol::r#match(), context)?;
            // b. If matcher is not undefined, then
            if let Some(matcher) = matcher {
                // i. Return ? Call(matcher, regexp, « O »).
                return matcher.call(regexp, &[o.clone()], context);
            }
        }

        // 3. Let S be ? ToString(O).
        let s = o.to_string(context)?;

        // 4. Let rx be ? RegExpCreate(regexp, undefined).
        let rx = RegExp::create(regexp, &JsValue::Undefined, context)?;

        // 5. Return ? Invoke(rx, @@match, « S »).
        rx.invoke(JsSymbol::r#match(), &[JsValue::new(s)], context)
    }

    /// Abstract operation `StringPad ( O, maxLength, fillString, placement )`.
    ///
    /// Performs the actual string padding for padStart/End.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringpad
    pub(crate) fn string_pad(
        object: &JsValue,
        max_length: &JsValue,
        fill_string: &JsValue,
        placement: Placement,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be ? ToString(O).
        let string = object.to_string(context)?;

        // 2. Let intMaxLength be ℝ(? ToLength(maxLength)).
        let int_max_length = max_length.to_length(context)?;

        // 3. Let stringLength be the length of S.
        let string_length = string.len() as u64;

        // 4. If intMaxLength ≤ stringLength, return S.
        if int_max_length <= string_length {
            return Ok(string.into());
        }

        // 5. If fillString is undefined, let filler be the String value consisting solely of the code unit 0x0020 (SPACE).
        let filler = if fill_string.is_undefined() {
            js_string!("\u{0020}")
        } else {
            // 6. Else, let filler be ? ToString(fillString).
            fill_string.to_string(context)?
        };

        // 7. If filler is the empty String, return S.
        if filler.is_empty() {
            return Ok(string.into());
        }

        // 8. Let fillLen be intMaxLength - stringLength.
        let fill_len = int_max_length - string_length;
        let filler_len = filler.len() as u64;

        // 9. Let truncatedStringFiller be the String value consisting of repeated
        // concatenations of filler truncated to length fillLen.
        let repetitions = {
            let q = fill_len / filler_len;
            let r = fill_len % filler_len;
            if r == 0 {
                q
            } else {
                q + 1
            }
        };

        let truncated_string_filler = filler.repeat(repetitions as usize);
        let truncated_string_filler = &truncated_string_filler[..fill_len as usize];

        // 10. If placement is start, return the string-concatenation of truncatedStringFiller and S.
        if placement == Placement::Start {
            Ok(js_string!(truncated_string_filler, &string).into())
        } else {
            // 11. Else, return the string-concatenation of S and truncatedStringFiller.
            Ok(js_string!(&string, truncated_string_filler).into())
        }
    }

    /// `String.prototype.padEnd( targetLength[, padString] )`
    ///
    /// The `padEnd()` method pads the current string with a given string (repeated, if needed) so that the resulting string reaches a given length.
    ///
    /// The padding is applied from the end of the current string.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.padend
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/padEnd
    pub(crate) fn pad_end(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        let max_length = args.get_or_undefined(0);
        let fill_string = args.get_or_undefined(1);

        // 2. Return ? StringPad(O, maxLength, fillString, end).
        Self::string_pad(this, max_length, fill_string, Placement::End, context)
    }

    /// `String.prototype.padStart( targetLength [, padString] )`
    ///
    /// The `padStart()` method pads the current string with another string (multiple times, if needed) until the resulting string reaches the given length.
    ///
    /// The padding is applied from the start of the current string.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.padstart
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/padStart
    pub(crate) fn pad_start(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        let max_length = args.get_or_undefined(0);
        let fill_string = args.get_or_undefined(1);

        // 2. Return ? StringPad(O, maxLength, fillString, end).
        Self::string_pad(this, max_length, fill_string, Placement::Start, context)
    }

    /// String.prototype.trim()
    ///
    /// The `trim()` method removes whitespace from both ends of a string.
    ///
    /// Whitespace in this context is all the whitespace characters (space, tab, no-break space, etc.) and all the line terminator characters (LF, CR, etc.).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.trim
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trim
    pub(crate) fn trim(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Return ? TrimString(S, start+end).
        let object = this.require_object_coercible()?;
        let string = object.to_string(context)?;
        Ok(js_string!(string.trim()).into())
    }

    /// `String.prototype.trimStart()`
    ///
    /// The `trimStart()` method removes whitespace from the beginning of a string.
    ///
    /// Whitespace in this context is all the whitespace characters (space, tab, no-break space, etc.) and all the line terminator characters (LF, CR, etc.).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.trimstart
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trimStart
    pub(crate) fn trim_start(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Return ? TrimString(S, start).
        let object = this.require_object_coercible()?;
        let string = object.to_string(context)?;
        Ok(js_string!(string.trim_start()).into())
    }

    /// String.prototype.trimEnd()
    ///
    /// The `trimEnd()` method removes whitespace from the end of a string.
    ///
    /// Whitespace in this context is all the whitespace characters (space, tab, no-break space, etc.) and all the line terminator characters (LF, CR, etc.).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.trimend
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/trimEnd
    pub(crate) fn trim_end(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        // 2. Return ? TrimString(S, end).
        let object = this.require_object_coercible()?;
        let string = object.to_string(context)?;
        Ok(js_string!(string.trim_end()).into())
    }

    /// [`String.prototype.toUpperCase()`][upper] and [`String.prototype.toLowerCase()`][lower]
    ///
    /// The case methods return the calling string value converted to uppercase or lowercase.
    ///
    /// The value will be **converted** to a string if it isn't one.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [upper]: https://tc39.es/ecma262/#sec-string.prototype.toUppercase
    /// [lower]: https://tc39.es/ecma262/#sec-string.prototype.toLowercase
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/toUpperCase
    pub(crate) fn to_case<const UPPER: bool>(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        // 3. Let sText be ! StringToCodePoints(S).
        // 4. Let upperText be the result of toUppercase(sText), according to
        // the Unicode Default Case Conversion algorithm.
        let text = string.map_valid_segments(|s| {
            if UPPER {
                s.to_uppercase()
            } else {
                s.to_lowercase()
            }
        });

        // 5. Let L be ! CodePointsToString(upperText).
        // 6. Return L.
        Ok(js_string!(text).into())
    }

    /// [`String.prototype.toLocaleLowerCase ( [ locales ] )`][lower] and
    /// [`String.prototype.toLocaleUpperCase ( [ locales ] )`][upper]
    ///
    /// [lower]: https://tc39.es/ecma402/#sup-string.prototype.tolocalelowercase
    /// [upper]: https://tc39.es/ecma402/#sup-string.prototype.tolocaleuppercase
    pub(crate) fn to_locale_case<const UPPER: bool>(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        #[cfg(feature = "intl")]
        {
            use super::intl::locale::{
                best_available_locale, canonicalize_locale_list, default_locale,
            };
            use icu_casemapping::{provider::CaseMappingV1Marker, CaseMapping};
            use icu_locid::LanguageIdentifier;

            // 1. Let O be ? RequireObjectCoercible(this value).
            let this = this.require_object_coercible()?;

            // 2. Let S be ? ToString(O).
            let string = this.to_string(context)?;

            // 3. Return ? TransformCase(S, locales, lower).

            //  TransformCase ( S, locales, targetCase )
            // https://tc39.es/ecma402/#sec-transform-case

            // 1. Let requestedLocales be ? CanonicalizeLocaleList(locales).
            // 2. If requestedLocales is not an empty List, then
            //     a. Let requestedLocale be requestedLocales[0].
            let lang = canonicalize_locale_list(args.get_or_undefined(0), context)?
                .into_iter()
                .next()
                // 3. Else,
                //     a. Let requestedLocale be ! DefaultLocale().
                .unwrap_or_else(|| default_locale(context.icu().locale_canonicalizer()))
                .id;
            // 4. Let noExtensionsLocale be the String value that is requestedLocale with any Unicode locale extension sequences (6.2.1) removed.
            // 5. Let availableLocales be a List with language tags that includes the languages for which the Unicode Character Database contains language sensitive case mappings. Implementations may add additional language tags if they support case mapping for additional locales.
            // 6. Let locale be ! BestAvailableLocale(availableLocales, noExtensionsLocale).
            // 7. If locale is undefined, set locale to "und".
            let lang =
                best_available_locale::<CaseMappingV1Marker>(lang, &context.icu().provider())
                    .unwrap_or(LanguageIdentifier::UND);

            let casemapper =
                CaseMapping::try_new_with_locale(&context.icu().provider(), &lang.into())
                    .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;

            // 8. Let codePoints be StringToCodePoints(S).
            let result = string.map_valid_segments(|segment| {
                if UPPER {
                    // 10. Else,
                    //     a. Assert: targetCase is upper.
                    //     b. Let newCodePoints be a List whose elements are the result of an uppercase transformation of codePoints according to an implementation-derived algorithm using locale or the Unicode Default Case Conversion algorithm.
                    casemapper.to_full_uppercase(&segment)
                } else {
                    // 9. If targetCase is lower, then
                    //     a. Let newCodePoints be a List whose elements are the result of a lowercase transformation of codePoints according to an implementation-derived algorithm using locale or the Unicode Default Case Conversion algorithm.
                    casemapper.to_full_lowercase(&segment)
                }
            });

            // 11. Return CodePointsToString(newCodePoints).
            Ok(result.into())
        }

        #[cfg(not(feature = "intl"))]
        {
            Self::to_case::<UPPER>(this, args, context)
        }
    }

    /// `String.prototype.substring( indexStart[, indexEnd] )`
    ///
    /// The `substring()` method returns the part of the `string` between the start and end indexes, or to the end of the string.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.substring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substring
    pub(crate) fn substring(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let string = this.to_string(context)?;

        // 3. Let len be the length of S.
        let len = string.len() as i64;

        // 4. Let intStart be ? ToIntegerOrInfinity(start).
        let int_start = args.get_or_undefined(0).to_integer_or_infinity(context)?;

        // 5. If end is undefined, let intEnd be len; else let intEnd be ? ToIntegerOrInfinity(end).
        let int_end = match args.get_or_undefined(1) {
            &JsValue::Undefined => IntegerOrInfinity::Integer(len),
            end => end.to_integer_or_infinity(context)?,
        };

        // 6. Let finalStart be the result of clamping intStart between 0 and len.
        let final_start = int_start.clamp_finite(0, len) as usize;

        // 7. Let finalEnd be the result of clamping intEnd between 0 and len.
        let final_end = int_end.clamp_finite(0, len) as usize;

        // 8. Let from be min(finalStart, finalEnd).
        let from = min(final_start, final_end);

        // 9. Let to be max(finalStart, finalEnd).
        let to = max(final_start, final_end);

        // 10. Return the substring of S from from to to.
        Ok(js_string!(&string[from..to]).into())
    }

    /// `String.prototype.split ( separator, limit )`
    ///
    /// The split() method divides a String into an ordered list of substrings, puts these substrings into an array, and returns the array.
    /// The division is done by searching for a pattern; where the pattern is provided as the first parameter in the method's call.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.split
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/split
    pub(crate) fn split(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        let separator = args.get_or_undefined(0);
        let limit = args.get_or_undefined(1);

        // 2. If separator is neither undefined nor null, then
        if !separator.is_null_or_undefined() {
            // a. Let splitter be ? GetMethod(separator, @@split).
            let splitter = separator.get_method(JsSymbol::split(), context)?;
            // b. If splitter is not undefined, then
            if let Some(splitter) = splitter {
                // i. Return ? Call(splitter, separator, « O, limit »).
                return splitter.call(separator, &[this.clone(), limit.clone()], context);
            }
        }

        // 3. Let S be ? ToString(O).
        let this_str = this.to_string(context)?;

        // 4.  If limit is undefined, let lim be 2^32 - 1; else let lim be ℝ(? ToUint32(limit)).
        let lim = if limit.is_undefined() {
            u32::MAX
        } else {
            limit.to_u32(context)?
        } as usize;

        // 5. Let R be ? ToString(separator).
        let separator_str = separator.to_string(context)?;

        // 6. If lim = 0, return A.
        if lim == 0 {
            // a. Return ! CreateArrayFromList(« »).
            return Ok(Array::create_array_from_list([], context).into());
        }

        // 7. If separator is undefined, then
        if separator.is_undefined() {
            // a. Return ! CreateArrayFromList(« S »).
            return Ok(Array::create_array_from_list([this_str.into()], context).into());
        }

        // 8. Let separatorLength be the length of R.
        let separator_length = separator_str.len();

        // 9. If separatorLength is 0, then
        if separator_length == 0 {
            // a. Let head be the substring of S from 0 to lim.
            // b. Let codeUnits be a List consisting of the sequence of code units that are the elements of head.
            let head = this_str
                .get(..lim)
                .unwrap_or(&this_str[..])
                .iter()
                .map(|code| js_string!(std::slice::from_ref(code)).into());
            // c. Return ! CreateArrayFromList(codeUnits).
            return Ok(Array::create_array_from_list(head, context).into());
        }

        // 10. If S is the empty String, return ! CreateArrayFromList(« S »).
        if this_str.is_empty() {
            return Ok(Array::create_array_from_list([this_str.into()], context).into());
        }

        // 11. Let substrings be a new empty List.
        let mut substrings = vec![];

        // 12. Let i be 0.
        let mut i = 0;

        // 13. Let j be ! StringIndexOf(S, R, 0).
        let mut j = this_str.index_of(&separator_str, 0);

        // 14. Repeat, while j is not -1
        while let Some(index) = j {
            // a. Let T be the substring of S from i to j.
            // b. Append T as the last element of substrings.
            substrings.push(js_string!(&this_str[i..index]));

            // c. If the number of elements of substrings is lim, return ! CreateArrayFromList(substrings).
            if substrings.len() == lim {
                return Ok(Array::create_array_from_list(
                    substrings.into_iter().map(JsValue::from),
                    context,
                )
                .into());
            }
            // d. Set i to j + separatorLength.
            i = index + separator_length;

            // e. Set j to ! StringIndexOf(S, R, i).
            j = this_str.index_of(&separator_str, i);
        }

        // 15. Let T be the substring of S from i.
        // 16. Append T to substrings.
        substrings.push(js_string!(&this_str[i..]));

        // 17. Return ! CreateArrayFromList(substrings).
        Ok(
            Array::create_array_from_list(substrings.into_iter().map(JsValue::from), context)
                .into(),
        )
    }

    /// String.prototype.valueOf()
    ///
    /// The `valueOf()` method returns the primitive value of a `String` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.value_of
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/valueOf
    pub(crate) fn value_of(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Return ? thisStringValue(this value).
        Self::this_string_value(this).map(JsValue::from)
    }

    /// `String.prototype.matchAll( regexp )`
    ///
    /// The `matchAll()` method returns an iterator of all results matching a string against a [`regular expression`][regex], including [capturing groups][cg].
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.matchall
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/matchAll
    /// [regex]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_Expressions
    /// [cg]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_Expressions/Groups_and_Ranges
    pub(crate) fn match_all(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let o = this.require_object_coercible()?;

        // 2. If regexp is neither undefined nor null, then
        let regexp = args.get_or_undefined(0);
        if !regexp.is_null_or_undefined() {
            // a. Let isRegExp be ? IsRegExp(regexp).
            // b. If isRegExp is true, then
            if let Some(regexp_obj) = regexp.as_object() {
                if is_reg_exp_object(regexp_obj, context)? {
                    // i. Let flags be ? Get(regexp, "flags").
                    let flags = regexp_obj.get(utf16!("flags"), context)?;

                    // ii. Perform ? RequireObjectCoercible(flags).
                    flags.require_object_coercible()?;

                    // iii. If ? ToString(flags) does not contain "g", throw a TypeError exception.
                    if !flags.to_string(context)?.contains(&u16::from(b'g')) {
                        return Err(JsNativeError::typ()
                        .with_message(
                            "String.prototype.matchAll called with a non-global RegExp argument",
                        )
                        .into());
                    }
                }
            }
            // c. Let matcher be ? GetMethod(regexp, @@matchAll).
            let matcher = regexp.get_method(JsSymbol::match_all(), context)?;
            // d. If matcher is not undefined, then
            if let Some(matcher) = matcher {
                return matcher.call(regexp, &[o.clone()], context);
            }
        }

        // 3. Let S be ? ToString(O).
        let s = o.to_string(context)?;

        // 4. Let rx be ? RegExpCreate(regexp, "g").
        let rx = RegExp::create(regexp, &JsValue::new(js_string!("g")), context)?;

        // 5. Return ? Invoke(rx, @@matchAll, « S »).
        rx.invoke(JsSymbol::match_all(), &[JsValue::new(s)], context)
    }

    /// `String.prototype.normalize( [ form ] )`
    ///
    /// The normalize() method normalizes a string into a form specified in the Unicode® Standard Annex #15
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.normalize
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/normalize
    pub(crate) fn normalize(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        /// Represents the type of normalization applied to a [`JsString`]
        #[derive(Clone, Copy)]
        pub(crate) enum Normalization {
            Nfc,
            Nfd,
            Nfkc,
            Nfkd,
        }

        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let s = this.to_string(context)?;

        // 6. Let ns be the String value that is the result of normalizing S
        // into the normalization form named by f as specified in
        // https://unicode.org/reports/tr15/.
        let normalization = match args.get_or_undefined(0) {
            // 3. If form is undefined, let f be "NFC".
            &JsValue::Undefined => Normalization::Nfc,
            // 4. Else, let f be ? ToString(form).
            f => match f.to_string(context)? {
                ntype if &ntype == utf16!("NFC") => Normalization::Nfc,
                ntype if &ntype == utf16!("NFD") => Normalization::Nfd,
                ntype if &ntype == utf16!("NFKC") => Normalization::Nfkc,
                ntype if &ntype == utf16!("NFKD") => Normalization::Nfkd,
                // 5. If f is not one of "NFC", "NFD", "NFKC", or "NFKD", throw a RangeError exception.
                _ => {
                    return Err(JsNativeError::range()
                        .with_message(
                            "The normalization form should be one of NFC, NFD, NFKC, NFKD.",
                        )
                        .into());
                }
            },
        };

        let normalizers = {
            #[cfg(not(feature = "intl"))]
            {
                use once_cell::sync::Lazy;
                static NORMALIZERS: Lazy<StringNormalizers> = Lazy::new(|| {
                    let provider = &boa_icu_provider::minimal();
                    let nfc = ComposingNormalizer::try_new_nfc_unstable(provider)
                        .expect("minimal data should always be updated");
                    let nfkc = ComposingNormalizer::try_new_nfkc_unstable(provider)
                        .expect("minimal data should always be updated");
                    let nfd = DecomposingNormalizer::try_new_nfd_unstable(provider)
                        .expect("minimal data should always be updated");
                    let nfkd = DecomposingNormalizer::try_new_nfkd_unstable(provider)
                        .expect("minimal data should always be updated");

                    StringNormalizers {
                        nfc,
                        nfkc,
                        nfd,
                        nfkd,
                    }
                });
                &*NORMALIZERS
            }
            #[cfg(feature = "intl")]
            {
                context.icu().string_normalizers()
            }
        };

        let result = match normalization {
            Normalization::Nfc => normalizers.nfc.normalize_utf16(&s),
            Normalization::Nfd => normalizers.nfd.normalize_utf16(&s),
            Normalization::Nfkc => normalizers.nfkc.normalize_utf16(&s),
            Normalization::Nfkd => normalizers.nfkd.normalize_utf16(&s),
        };

        // 7. Return ns.
        Ok(js_string!(result).into())
    }

    /// `String.prototype.search( regexp )`
    ///
    /// The search() method executes a search for a match between a regular expression and this String object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.search
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/search
    pub(crate) fn search(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let o = this.require_object_coercible()?;

        // 2. If regexp is neither undefined nor null, then
        let regexp = args.get_or_undefined(0);
        if !regexp.is_null_or_undefined() {
            // a. Let searcher be ? GetMethod(regexp, @@search).
            let searcher = regexp.get_method(JsSymbol::search(), context)?;
            // b. If searcher is not undefined, then
            if let Some(searcher) = searcher {
                // i. Return ? Call(searcher, regexp, « O »).
                return searcher.call(regexp, &[o.clone()], context);
            }
        }

        // 3. Let string be ? ToString(O).
        let string = o.to_string(context)?;

        // 4. Let rx be ? RegExpCreate(regexp, undefined).
        let rx = RegExp::create(regexp, &JsValue::Undefined, context)?;

        // 5. Return ? Invoke(rx, @@search, « string »).
        rx.invoke(JsSymbol::search(), &[JsValue::new(string)], context)
    }

    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn iterator(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let o = this.require_object_coercible()?;
        // 2. Let s be ? ToString(O).
        let s = o.to_string(context)?;

        Ok(StringIterator::create_string_iterator(s, context).into())
    }
}

#[cfg(feature = "annex-b")]
impl String {
    /// `String.prototype.substr( start[, length] )`
    ///
    /// The `substr()` method returns a portion of the string, starting at the specified index and extending for a given number of characters afterward.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.substr
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substr
    /// <https://tc39.es/ecma262/#sec-string.prototype.substr>
    pub(crate) fn substr(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let o = this.require_object_coercible()?;

        // 2. Let S be ? ToString(O).
        let s = o.to_string(context)?;

        // 3. Let size be the length of S.
        let size = s.len() as i64;

        // 4. Let intStart be ? ToIntegerOrInfinity(start).
        let start = args.get_or_undefined(0);
        let int_start = start.to_integer_or_infinity(context)?;

        let int_start = match int_start {
            // 5. If intStart is -∞, set intStart to 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 6. Else if intStart < 0, set intStart to max(size + intStart, 0).
            IntegerOrInfinity::Integer(int_start) if int_start < 0 => max(size + int_start, 0),
            IntegerOrInfinity::Integer(int_start) => int_start,
            // 7. Else, set intStart to min(intStart, size).
            //
            // NOTE: size will always we smaller than +∞
            IntegerOrInfinity::PositiveInfinity => size,
        } as usize;

        // 8. If length is undefined, let intLength be size;
        //    otherwise let intLength be ? ToIntegerOrInfinity(length).
        let length = args.get_or_undefined(1);
        let int_length = if length.is_undefined() {
            IntegerOrInfinity::Integer(size)
        } else {
            length.to_integer_or_infinity(context)?
        };

        // 9. Set intLength to the result of clamping intLength between 0 and size.
        let int_length = match int_length {
            IntegerOrInfinity::NegativeInfinity => 0,
            IntegerOrInfinity::PositiveInfinity => size,
            IntegerOrInfinity::Integer(i) => i.clamp(0, size),
        } as usize;

        // 10. Let intEnd be min(intStart + intLength, size).
        let int_end = min(int_start + int_length, size as usize);

        // 11. Return the substring of S from intStart to intEnd.
        if let Some(substr) = s.get(int_start..int_end) {
            Ok(js_string!(substr).into())
        } else {
            Ok(js_string!().into())
        }
    }

    /// `CreateHTML(string, tag, attribute, value)`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createhtml
    pub(crate) fn create_html(
        string: &JsValue,
        tag: &[u16],
        attribute_and_value: Option<(&[u16], &JsValue)>,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let str be ? RequireObjectCoercible(string).
        let str = string.require_object_coercible()?;

        // 2. Let S be ? ToString(str).
        let s = str.to_string(context)?;

        // 3. Let p1 be the string-concatenation of "<" and tag.
        let mut p1 = JsString::concat_array(&[utf16!("<"), tag]);

        // 4. If attribute is not the empty String, then
        if let Some((attribute, value)) = attribute_and_value {
            // a. Let V be ? ToString(value).
            let v = value.to_string(context)?;

            // b. Let escapedV be the String value that is the same as V except that each occurrence
            //    of the code unit 0x0022 (QUOTATION MARK) in V has been replaced with the six
            //    code unit sequence "&quot;".
            let mut escaped_v = Vec::with_capacity(v.len());
            for c in v.as_slice().iter().copied() {
                if c == 0x0022 {
                    escaped_v.extend(utf16!("&quot;"));
                    continue;
                }
                escaped_v.push(c);
            }

            // c. Set p1 to the string-concatenation of:
            //    p1
            //    the code unit 0x0020 (SPACE)
            //    attribute
            //    the code unit 0x003D (EQUALS SIGN)
            //    the code unit 0x0022 (QUOTATION MARK)
            //    escapedV
            //    the code unit 0x0022 (QUOTATION MARK)
            p1 = JsString::concat_array(&[
                p1.as_slice(),
                utf16!(" "),
                attribute,
                utf16!("=\""),
                escaped_v.as_slice(),
                utf16!("\""),
            ]);
        }

        // 5. Let p2 be the string-concatenation of p1 and ">".
        // 6. Let p3 be the string-concatenation of p2 and S.
        // 7. Let p4 be the string-concatenation of p3, "</", tag, and ">".
        let p4 = JsString::concat_array(&[
            p1.as_slice(),
            utf16!(">"),
            s.as_slice(),
            utf16!("</"),
            tag,
            utf16!(">"),
        ]);

        // 8. Return p4.
        Ok(p4.into())
    }

    /// `String.prototype.anchor( name )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.anchor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/anchor
    pub(crate) fn anchor(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let name = args.get_or_undefined(0);

        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "a", "name", name).
        Self::create_html(s, utf16!("a"), Some((utf16!("name"), name)), context)
    }

    /// `String.prototype.big( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.big
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/big
    pub(crate) fn big(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "big", "", "").
        Self::create_html(s, utf16!("big"), None, context)
    }

    /// `String.prototype.blink( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.blink
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/blink
    pub(crate) fn blink(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "blink", "", "").
        Self::create_html(s, utf16!("blink"), None, context)
    }

    /// `String.prototype.bold( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.bold
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/bold
    pub(crate) fn bold(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "b", "", "").
        Self::create_html(s, utf16!("b"), None, context)
    }

    /// `String.prototype.fixed( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.fixed
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fixed
    pub(crate) fn fixed(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "big", "", "").
        Self::create_html(s, utf16!("tt"), None, context)
    }

    /// `String.prototype.fontcolor( color )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.fontcolor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fontcolor
    pub(crate) fn fontcolor(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let color = args.get_or_undefined(0);

        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "font", "color", color).
        Self::create_html(s, utf16!("font"), Some((utf16!("color"), color)), context)
    }

    /// `String.prototype.fontsize( size )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.fontsize
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/fontsize
    pub(crate) fn fontsize(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let size = args.get_or_undefined(0);

        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "font", "size", size).
        Self::create_html(s, utf16!("font"), Some((utf16!("size"), size)), context)
    }

    /// `String.prototype.italics( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.italics
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/italics
    pub(crate) fn italics(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "i", "", "").
        Self::create_html(s, utf16!("i"), None, context)
    }

    /// `String.prototype.link( url )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.link
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/link
    pub(crate) fn link(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let url = args.get_or_undefined(0);

        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "a", "href", url).
        Self::create_html(s, utf16!("a"), Some((utf16!("href"), url)), context)
    }

    /// `String.prototype.small( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.small
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/small
    pub(crate) fn small(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "small", "", "").
        Self::create_html(s, utf16!("small"), None, context)
    }

    /// `String.prototype.strike( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.strike
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/strike
    pub(crate) fn strike(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "strike", "", "").
        Self::create_html(s, utf16!("strike"), None, context)
    }

    /// `String.prototype.sub( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.sub
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/sub
    pub(crate) fn sub(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "sub", "", "").
        Self::create_html(s, utf16!("sub"), None, context)
    }

    /// `String.prototype.sup( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.sup
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/sup
    pub(crate) fn sup(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let S be the this value.
        let s = this;
        // 2. Return ? CreateHTML(S, "sup", "", "").
        Self::create_html(s, utf16!("sup"), None, context)
    }
}

/// Abstract operation `GetSubstitution ( matched, str, position, captures, namedCaptures, replacement )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-getsubstitution
pub(crate) fn get_substitution(
    matched: &JsString,
    str: &JsString,
    position: usize,
    captures: &[JsValue],
    named_captures: &JsValue,
    replacement: &JsString,
    context: &mut Context<'_>,
) -> JsResult<JsString> {
    let mut buf = [0; 2];
    // 1. Assert: Type(matched) is String.

    // 2. Let matchLength be the number of code units in matched.
    let match_length = matched.len();

    // 3. Assert: Type(str) is String.

    // 4. Let stringLength be the number of code units in str.
    let str_length = str.len();

    // 5. Assert: position ≤ stringLength.
    // 6. Assert: captures is a possibly empty List of Strings.
    // 7. Assert: Type(replacement) is String.

    // 8. Let tailPos be position + matchLength.
    let tail_pos = position + match_length;

    // 9. Let m be the number of elements in captures.
    let m = captures.len();

    // 10. Let result be the String value derived from replacement by copying code unit elements
    //     from replacement to result while performing replacements as specified in Table 58.
    //     These $ replacements are done left-to-right, and, once such a replacement is performed,
    //     the new replacement text is not subject to further replacements.
    let mut result = vec![];
    let mut chars = replacement.code_points().peekable();

    while let Some(first) = chars.next() {
        if first == CodePoint::Unicode('$') {
            let second = chars.next();
            let second_is_digit = second
                .and_then(CodePoint::as_char)
                .as_ref()
                .map_or(false, char::is_ascii_digit);
            // we use peek so that it is still in the iterator if not used
            let third = if second_is_digit {
                chars.peek().copied()
            } else {
                None
            };
            let third_is_digit = third
                .and_then(CodePoint::as_char)
                .as_ref()
                .map_or(false, char::is_ascii_digit);

            match (second, third) {
                // $$
                (Some(CodePoint::Unicode('$')), _) => {
                    // $
                    result.push('$' as u16);
                }
                // $&
                (Some(CodePoint::Unicode('&')), _) => {
                    // matched
                    result.extend_from_slice(matched);
                }
                // $`
                (Some(CodePoint::Unicode('`')), _) => {
                    // The replacement is the substring of str from 0 to position.
                    result.extend_from_slice(&str[..position]);
                }
                // $'
                (Some(CodePoint::Unicode('\'')), _) => {
                    // If tailPos ≥ stringLength, the replacement is the empty String.
                    // Otherwise the replacement is the substring of str from tailPos.
                    if tail_pos < str_length {
                        result.extend_from_slice(&str[tail_pos..]);
                    }
                }
                // $nn
                (Some(CodePoint::Unicode(second)), Some(CodePoint::Unicode(third)))
                    if second_is_digit && third_is_digit =>
                {
                    // The nnth element of captures, where nn is a two-digit decimal number in the range 01 to 99.
                    let tens = second
                        .to_digit(10)
                        .expect("could not convert character to digit after checking it")
                        as usize;
                    let units = third
                        .to_digit(10)
                        .expect("could not convert character to digit after checking it")
                        as usize;
                    let nn = 10 * tens + units;

                    // If nn ≤ m and the nnth element of captures is undefined, use the empty String instead.
                    // If nn is 00 or nn > m, no replacement is done.
                    if nn == 0 || nn > m {
                        result.extend_from_slice(&['$' as u16, second as u16, third as u16]);
                    } else if let Some(capture) = captures.get(nn - 1) {
                        if let Some(s) = capture.as_string() {
                            result.extend_from_slice(s);
                        }
                    }

                    chars.next();
                }
                // $n
                (Some(CodePoint::Unicode(second)), _) if second_is_digit => {
                    // The nth element of captures, where n is a single digit in the range 1 to 9.
                    let n = second
                        .to_digit(10)
                        .expect("could not convert character to digit after checking it")
                        as usize;

                    // If n ≤ m and the nth element of captures is undefined, use the empty String instead.
                    // If n > m, no replacement is done.
                    if n == 0 || n > m {
                        result.extend_from_slice(&['$' as u16, second as u16]);
                    } else if let Some(capture) = captures.get(n - 1) {
                        if let Some(s) = capture.as_string() {
                            result.extend_from_slice(s);
                        }
                    }
                }
                // $<
                (Some(CodePoint::Unicode('<')), _) => {
                    // 1. If namedCaptures is undefined, the replacement text is the String "$<".
                    // 2. Else,
                    if named_captures.is_undefined() {
                        result.extend_from_slice(utf16!("$<"));
                    } else {
                        // a. Assert: Type(namedCaptures) is Object.
                        let named_captures = named_captures
                            .as_object()
                            .expect("should be an object according to spec");

                        // b. Scan until the next > U+003E (GREATER-THAN SIGN).
                        let mut group_name = vec![];
                        let mut found = false;
                        loop {
                            match chars.next() {
                                Some(CodePoint::Unicode('>')) => {
                                    found = true;
                                    break;
                                }
                                Some(c) => group_name.extend_from_slice(c.encode_utf16(&mut buf)),
                                None => break,
                            }
                        }

                        // c. If none is found, the replacement text is the String "$<".
                        #[allow(clippy::if_not_else)]
                        if !found {
                            result.extend_from_slice(utf16!("$<"));
                            result.extend_from_slice(&group_name);
                        // d. Else,
                        } else {
                            // i. Let groupName be the enclosed substring.
                            let group_name = js_string!(group_name);
                            // ii. Let capture be ? Get(namedCaptures, groupName).
                            let capture = named_captures.get(group_name, context)?;

                            // iii. If capture is undefined, replace the text through > with the empty String.
                            // iv. Otherwise, replace the text through > with ? ToString(capture).
                            if !capture.is_undefined() {
                                result.extend_from_slice(&capture.to_string(context)?);
                            }
                        }
                    }
                }
                // $?, ? is none of the above
                _ => {
                    result.push('$' as u16);
                    if let Some(second) = second {
                        result.extend_from_slice(second.encode_utf16(&mut buf));
                    }
                }
            }
        } else {
            result.extend_from_slice(first.encode_utf16(&mut buf));
        }
    }

    // 11. Return result.
    Ok(js_string!(result))
}

/// Abstract operation `IsRegExp( argument )`
///
/// More information:
/// [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-isregexp
fn is_reg_exp(argument: &JsValue, context: &mut Context<'_>) -> JsResult<bool> {
    // 1. If Type(argument) is not Object, return false.
    let JsValue::Object(argument) = argument else {
        return Ok(false);
    };

    is_reg_exp_object(argument, context)
}
fn is_reg_exp_object(argument: &JsObject, context: &mut Context<'_>) -> JsResult<bool> {
    // 2. Let matcher be ? Get(argument, @@match).
    let matcher = argument.get(JsSymbol::r#match(), context)?;

    // 3. If matcher is not undefined, return ! ToBoolean(matcher).
    if !matcher.is_undefined() {
        return Ok(matcher.to_boolean());
    }

    // 4. If argument has a [[RegExpMatcher]] internal slot, return true.
    // 5. Return false.
    Ok(argument.is_regexp())
}
