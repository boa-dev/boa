//! This module implements the global `String` object.
//!
//! The `String` global object is a constructor for strings or a sequence of characters.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-string-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String

pub mod string_iterator;
#[cfg(test)]
mod tests;

use crate::builtins::Symbol;
use crate::object::PROTOTYPE;
use crate::property::DataDescriptor;
use crate::{
    builtins::{string::string_iterator::StringIterator, Array, BuiltIn, RegExp},
    object::{ConstructorBuilder, Object, ObjectData},
    property::Attribute,
    symbol::WellKnownSymbols,
    value::{RcString, Value},
    BoaProfiler, Context, Result,
};
use regress::Regex;
use std::{
    char::{decode_utf16, from_u32},
    cmp::{max, min},
    string::String as StdString,
};
use unicode_normalization::UnicodeNormalization;

pub(crate) fn code_point_at(string: RcString, position: i32) -> Option<(u32, u8, bool)> {
    let size = string.encode_utf16().count() as i32;
    if position < 0 || position >= size {
        return None;
    }
    let mut encoded = string.encode_utf16();
    let first = encoded.nth(position as usize)?;
    if !is_leading_surrogate(first) && !is_trailing_surrogate(first) {
        return Some((first as u32, 1, false));
    }
    if is_trailing_surrogate(first) || position + 1 == size {
        return Some((first as u32, 1, true));
    }
    let second = encoded.next()?;
    if !is_trailing_surrogate(second) {
        return Some((first as u32, 1, true));
    }
    let cp = (first as u32 - 0xD800) * 0x400 + (second as u32 - 0xDC00) + 0x10000;
    Some((cp, 2, false))
}

/// Helper function to check if a `char` is trimmable.
#[inline]
pub(crate) fn is_trimmable_whitespace(c: char) -> bool {
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

fn is_leading_surrogate(value: u16) -> bool {
    (0xD800..=0xDBFF).contains(&value)
}

fn is_trailing_surrogate(value: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&value)
}

/// JavaScript `String` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct String;

impl BuiltIn for String {
    const NAME: &'static str = "String";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let symbol_iterator = WellKnownSymbols::iterator();

        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT;
        let string_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().string_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .property("length", 0, attribute)
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
        .method(Self::r#match, "match", 1)
        .method(Self::normalize, "normalize", 1)
        .method(Self::pad_end, "padEnd", 1)
        .method(Self::pad_start, "padStart", 1)
        .method(Self::trim, "trim", 0)
        .method(Self::trim_start, "trimStart", 0)
        .method(Self::trim_end, "trimEnd", 0)
        .method(Self::to_lowercase, "toLowerCase", 0)
        .method(Self::to_uppercase, "toUpperCase", 0)
        .method(Self::substring, "substring", 2)
        .method(Self::substr, "substr", 2)
        .method(Self::split, "split", 2)
        .method(Self::value_of, "valueOf", 0)
        .method(Self::match_all, "matchAll", 1)
        .method(Self::replace, "replace", 2)
        .method(Self::iterator, (symbol_iterator, "[Symbol.iterator]"), 0)
        .method(Self::search, "search", 1)
        .build();

        (Self::NAME, string_object.into(), Self::attribute())
    }
}

impl String {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// JavaScript strings must be between `0` and less than positive `Infinity` and cannot be a negative number.
    /// The range of allowed values can be described like this: `[0, +∞)`.
    ///
    /// The resulting string can also not be larger than the maximum string size,
    /// which can differ in JavaScript engines. In Boa it is `2^32 - 1`
    pub(crate) const MAX_STRING_LENGTH: f64 = u32::MAX as f64;

    /// `String( value )`
    ///
    /// <https://tc39.es/ecma262/#sec-string-constructor-string-value>
    pub(crate) fn constructor(
        new_target: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        let string = match args.get(0) {
            Some(value) if value.is_symbol() && new_target.is_undefined() => {
                Symbol::to_string(value, &[], context)?
                    .as_string()
                    .expect("'Symbol::to_string' returns 'Value::String'")
                    .clone()
            }
            Some(ref value) => value.to_string(context)?,
            None => RcString::default(),
        };

        if new_target.is_undefined() {
            return Ok(string.into());
        }
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.get(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| context.standard_objects().object_object().prototype());
        let this = Value::new_object(context);

        this.as_object()
            .expect("this should be an object")
            .set_prototype_instance(prototype.into());

        let length = DataDescriptor::new(
            Value::from(string.encode_utf16().count()),
            Attribute::NON_ENUMERABLE,
        );
        this.set_property("length", length);

        this.set_data(ObjectData::String(string));

        Ok(this)
    }

    fn this_string_value(this: &Value, context: &mut Context) -> Result<RcString> {
        match this {
            Value::String(ref string) => return Ok(string.clone()),
            Value::Object(ref object) => {
                let object = object.borrow();
                if let Some(string) = object.as_string() {
                    return Ok(string);
                }
            }
            _ => {}
        }

        Err(context.construct_type_error("'this' is not a string"))
    }

    /// Get the string value to a primitive string
    #[allow(clippy::wrong_self_convention)]
    #[inline]
    pub(crate) fn to_string(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        // Get String from String Object and send it back as a new value
        Ok(Value::from(Self::this_string_value(this, context)?))
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
    pub(crate) fn char_at(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = this.to_string(context)?;
        let pos = args
            .get(0)
            .cloned()
            .unwrap_or_else(Value::undefined)
            .to_integer(context)? as i32;

        // Fast path returning empty string when pos is obviously out of range
        if pos < 0 || pos >= primitive_val.len() as i32 {
            return Ok("".into());
        }

        // Calling .len() on a string would give the wrong result, as they are bytes not the number of
        // unicode code points
        // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of
        // bytes is an O(1) operation.
        if let Some(utf16_val) = primitive_val.encode_utf16().nth(pos as usize) {
            Ok(Value::from(from_u32(utf16_val as u32).unwrap()))
        } else {
            Ok("".into())
        }
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
        this: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = this.to_string(context)?;
        let pos = args
            .get(0)
            .cloned()
            .unwrap_or_else(Value::undefined)
            .to_integer(context)? as i32;

        // Fast path returning undefined when pos is obviously out of range
        if pos < 0 || pos >= primitive_val.len() as i32 {
            return Ok(Value::undefined());
        }

        if let Some((code_point, _, _)) = code_point_at(primitive_val, pos) {
            Ok(Value::from(code_point))
        } else {
            Ok(Value::undefined())
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
        this: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = this.to_string(context)?;
        let pos = args
            .get(0)
            .cloned()
            .unwrap_or_else(Value::undefined)
            .to_integer(context)? as i32;

        // Fast path returning NaN when pos is obviously out of range
        if pos < 0 || pos >= primitive_val.len() as i32 {
            return Ok(Value::nan());
        }

        // Calling .len() on a string would give the wrong result, as they are bytes not the number of unicode code points
        // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of bytes is an O(1) operation.
        // If there is no element at that index, the result is NaN
        if let Some(utf16_val) = primitive_val.encode_utf16().nth(pos as usize) {
            Ok(Value::from(f64::from(utf16_val)))
        } else {
            Ok(Value::nan())
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
    pub(crate) fn concat(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let object = this.require_object_coercible(context)?;
        let mut string = object.to_string(context)?.to_string();

        for arg in args {
            string.push_str(&arg.to_string(context)?);
        }

        Ok(Value::from(string))
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
    pub(crate) fn repeat(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let object = this.require_object_coercible(context)?;
        let string = object.to_string(context)?;

        if let Some(arg) = args.get(0) {
            let n = arg.to_integer(context)?;
            if n < 0.0 {
                return context.throw_range_error("repeat count cannot be a negative number");
            }

            if n.is_infinite() {
                return context.throw_range_error("repeat count cannot be infinity");
            }

            if n * (string.len() as f64) > Self::MAX_STRING_LENGTH {
                return context
                    .throw_range_error("repeat count must not overflow maximum string length");
            }
            Ok(string.repeat(n as usize).into())
        } else {
            Ok("".into())
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
    pub(crate) fn slice(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = this.to_string(context)?;

        // Calling .len() on a string would give the wrong result, as they are bytes not the number of unicode code points
        // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of bytes is an O(1) operation.
        let length = primitive_val.chars().count() as i32;

        let start = args
            .get(0)
            .cloned()
            .unwrap_or_else(Value::undefined)
            .to_integer(context)? as i32;
        let end = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| Value::integer(length))
            .to_integer(context)? as i32;

        let from = if start < 0 {
            max(length.wrapping_add(start), 0)
        } else {
            min(start, length)
        };
        let to = if end < 0 {
            max(length.wrapping_add(end), 0)
        } else {
            min(end, length)
        };

        let span = max(to.wrapping_sub(from), 0);

        let new_str: StdString = primitive_val
            .chars()
            .skip(from as usize)
            .take(span as usize)
            .collect();
        Ok(Value::from(new_str))
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
        this: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = this.to_string(context)?;

        let arg = args.get(0).cloned().unwrap_or_else(Value::undefined);

        if Self::is_regexp_object(&arg) {
            context.throw_type_error(
                "First argument to String.prototype.startsWith must not be a regular expression",
            )?;
        }

        let search_string = arg.to_string(context)?;

        let length = primitive_val.chars().count() as i32;
        let search_length = search_string.chars().count() as i32;

        // If less than 2 args specified, position is 'undefined', defaults to 0
        let position = if args.len() < 2 {
            0
        } else {
            args.get(1)
                .expect("failed to get arg")
                .to_integer(context)? as i32
        };

        let start = min(max(position, 0), length);
        let end = start.wrapping_add(search_length);

        if end > length {
            Ok(Value::from(false))
        } else {
            // Only use the part of the string from "start"
            let this_string: StdString = primitive_val.chars().skip(start as usize).collect();
            Ok(Value::from(this_string.starts_with(search_string.as_str())))
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
    pub(crate) fn ends_with(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = this.to_string(context)?;

        let arg = args.get(0).cloned().unwrap_or_else(Value::undefined);

        if Self::is_regexp_object(&arg) {
            context.throw_type_error(
                "First argument to String.prototype.endsWith must not be a regular expression",
            )?;
        }

        let search_string = arg.to_string(context)?;

        let length = primitive_val.chars().count() as i32;
        let search_length = search_string.chars().count() as i32;

        // If less than 2 args specified, end_position is 'undefined', defaults to
        // length of this
        let end_position = if args.len() < 2 {
            length
        } else {
            args.get(1)
                .expect("Could not get argumetn")
                .to_integer(context)? as i32
        };

        let end = min(max(end_position, 0), length);
        let start = end.wrapping_sub(search_length);

        if start < 0 {
            Ok(Value::from(false))
        } else {
            // Only use the part of the string up to "end"
            let this_string: StdString = primitive_val.chars().take(end as usize).collect();
            Ok(Value::from(this_string.ends_with(search_string.as_str())))
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
    pub(crate) fn includes(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = this.to_string(context)?;

        let arg = args.get(0).cloned().unwrap_or_else(Value::undefined);

        if Self::is_regexp_object(&arg) {
            context.throw_type_error(
                "First argument to String.prototype.includes must not be a regular expression",
            )?;
        }

        let search_string = arg.to_string(context)?;

        let length = primitive_val.chars().count() as i32;

        // If less than 2 args specified, position is 'undefined', defaults to 0
        let position = if args.len() < 2 {
            0
        } else {
            args.get(1)
                .expect("Could not get argument")
                .to_integer(context)? as i32
        };

        let start = min(max(position, 0), length);

        // Take the string from "this" and use only the part of it after "start"
        let this_string: StdString = primitive_val.chars().skip(start as usize).collect();

        Ok(Value::from(this_string.contains(search_string.as_str())))
    }

    /// Return either the string itself or the string of the regex equivalent
    fn get_regex_string(value: &Value) -> StdString {
        match value {
            Value::String(ref body) => body.to_string(),
            Value::Object(ref obj) => {
                let obj = obj.borrow();

                if let Some(regexp) = obj.as_regexp() {
                    // first argument is another `RegExp` object, so copy its pattern and flags
                    return regexp.original_source.clone().into();
                }
                "undefined".to_string()
            }
            _ => "undefined".to_string(),
        }
    }

    fn is_regexp_object(value: &Value) -> bool {
        match value {
            Value::Object(ref obj) => obj.borrow().is_regexp(),
            _ => false,
        }
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
    pub(crate) fn replace(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // TODO: Support Symbol replacer
        let primitive_val = this.to_string(context)?;
        if args.is_empty() {
            return Ok(Value::from(primitive_val));
        }

        let regex_body = Self::get_regex_string(args.get(0).expect("Value needed"));
        let re = Regex::new(&regex_body).expect("unable to convert regex to regex object");
        let mat = match re.find(&primitive_val) {
            Some(mat) => mat,
            None => return Ok(Value::from(primitive_val)),
        };
        let caps = re
            .find(&primitive_val)
            .expect("unable to get capture groups from text")
            .captures;

        let replace_value = if args.len() > 1 {
            // replace_object could be a string or function or not exist at all
            let replace_object: &Value = args.get(1).expect("second argument expected");
            match replace_object {
                Value::String(val) => {
                    // https://tc39.es/ecma262/#table-45
                    let mut result = StdString::new();
                    let mut chars = val.chars().peekable();

                    let m = caps.len();

                    while let Some(first) = chars.next() {
                        if first == '$' {
                            let second = chars.next();
                            let second_is_digit = second.map_or(false, |ch| ch.is_digit(10));
                            // we use peek so that it is still in the iterator if not used
                            let third = if second_is_digit { chars.peek() } else { None };
                            let third_is_digit = third.map_or(false, |ch| ch.is_digit(10));

                            match (second, third) {
                                (Some('$'), _) => {
                                    // $$
                                    result.push('$');
                                }
                                (Some('&'), _) => {
                                    // $&
                                    result.push_str(&primitive_val[mat.range()]);
                                }
                                (Some('`'), _) => {
                                    // $`
                                    let start_of_match = mat.start();
                                    result.push_str(&primitive_val[..start_of_match]);
                                }
                                (Some('\''), _) => {
                                    // $'
                                    let end_of_match = mat.end();
                                    result.push_str(&primitive_val[end_of_match..]);
                                }
                                (Some(second), Some(third))
                                    if second_is_digit && third_is_digit =>
                                {
                                    // $nn
                                    let tens = second.to_digit(10).unwrap() as usize;
                                    let units = third.to_digit(10).unwrap() as usize;
                                    let nn = 10 * tens + units;
                                    if nn == 0 || nn > m {
                                        result.push(first);
                                        result.push(second);
                                        if let Some(ch) = chars.next() {
                                            result.push(ch);
                                        }
                                    } else {
                                        let group = match mat.group(nn) {
                                            Some(range) => &primitive_val[range.clone()],
                                            _ => "",
                                        };
                                        result.push_str(group);
                                        chars.next(); // consume third
                                    }
                                }
                                (Some(second), _) if second_is_digit => {
                                    // $n
                                    let n = second.to_digit(10).unwrap() as usize;
                                    if n == 0 || n > m {
                                        result.push(first);
                                        result.push(second);
                                    } else {
                                        let group = match mat.group(n) {
                                            Some(range) => &primitive_val[range.clone()],
                                            _ => "",
                                        };
                                        result.push_str(group);
                                    }
                                }
                                (Some('<'), _) => {
                                    // $<
                                    // TODO: named capture groups
                                    result.push_str("$<");
                                }
                                _ => {
                                    // $?, ? is none of the above
                                    // we can consume second because it isn't $
                                    result.push(first);
                                    if let Some(second) = second {
                                        result.push(second);
                                    }
                                }
                            }
                        } else {
                            result.push(first);
                        }
                    }

                    result
                }
                Value::Object(_) => {
                    // This will return the matched substring first, then captured parenthesized groups later
                    let mut results: Vec<Value> = mat
                        .groups()
                        .map(|group| match group {
                            Some(range) => Value::from(&primitive_val[range]),
                            None => Value::undefined(),
                        })
                        .collect();

                    // Returns the starting byte offset of the match
                    let start = mat.start();
                    results.push(Value::from(start));
                    // Push the whole string being examined
                    results.push(Value::from(primitive_val.to_string()));

                    let result = context.call(&replace_object, this, &results)?;

                    result.to_string(context)?.to_string()
                }
                _ => "undefined".to_string(),
            }
        } else {
            "undefined".to_string()
        };

        Ok(Value::from(primitive_val.replacen(
            &primitive_val[mat.range()],
            &replace_value,
            1,
        )))
    }

    // pub(crate) fn replace_all(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
    //
    // }

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
    pub(crate) fn index_of(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let this = this.require_object_coercible(context)?;
        let string = this.to_string(context)?;

        let search_string = args
            .get(0)
            .cloned()
            .unwrap_or_else(Value::undefined)
            .to_string(context)?;

        let length = string.chars().count();
        let start = args
            .get(1)
            .map(|position| position.to_integer(context))
            .transpose()?
            .map_or(0, |position| position.max(0.0).min(length as f64) as usize);

        if search_string.is_empty() {
            return Ok(start.min(length).into());
        }

        if start < length {
            if let Some(position) = string.find(search_string.as_str()) {
                return Ok(string[..position].chars().count().into());
            }
        }

        Ok(Value::from(-1))
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
        this: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        let this = this.require_object_coercible(context)?;
        let string = this.to_string(context)?;

        let search_string = args
            .get(0)
            .cloned()
            .unwrap_or_else(Value::undefined)
            .to_string(context)?;

        let length = string.chars().count();
        let start = args
            .get(1)
            .map(|position| position.to_integer(context))
            .transpose()?
            .map_or(0, |position| position.max(0.0).min(length as f64) as usize);

        if search_string.is_empty() {
            return Ok(start.min(length).into());
        }

        if start < length {
            if let Some(position) = string.rfind(search_string.as_str()) {
                return Ok(string[..position].chars().count().into());
            }
        }

        Ok(Value::from(-1))
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
    pub(crate) fn r#match(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let re = RegExp::constructor(
            &Value::from(Object::default()),
            &[args.get(0).cloned().unwrap_or_default()],
            context,
        )?;
        RegExp::r#match(&re, this.to_string(context)?, context)
    }

    /// Abstract method `StringPad`.
    ///
    /// Performs the actual string padding for padStart/End.
    /// <https://tc39.es/ecma262/#sec-stringpad/>
    fn string_pad(
        primitive: RcString,
        max_length: i32,
        fill_string: Option<RcString>,
        at_start: bool,
    ) -> Value {
        let primitive_length = primitive.len() as i32;

        if max_length <= primitive_length {
            return Value::from(primitive);
        }

        let filler = fill_string.as_deref().unwrap_or(" ");

        if filler.is_empty() {
            return Value::from(primitive);
        }

        let fill_len = max_length.wrapping_sub(primitive_length);
        let mut fill_str = StdString::new();

        while fill_str.len() < fill_len as usize {
            fill_str.push_str(filler);
        }
        // Cut to size max_length
        let concat_fill_str: StdString = fill_str.chars().take(fill_len as usize).collect();

        if at_start {
            Value::from(format!("{}{}", concat_fill_str, &primitive))
        } else {
            Value::from(format!("{}{}", primitive, &concat_fill_str))
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
    pub(crate) fn pad_end(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let primitive = this.to_string(context)?;
        if args.is_empty() {
            return Err(Value::from("padEnd requires maxLength argument"));
        }
        let max_length = args
            .get(0)
            .expect("failed to get argument for String method")
            .to_integer(context)? as i32;

        let fill_string = args.get(1).map(|arg| arg.to_string(context)).transpose()?;

        Ok(Self::string_pad(primitive, max_length, fill_string, false))
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
    pub(crate) fn pad_start(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let primitive = this.to_string(context)?;
        if args.is_empty() {
            return Err(Value::from("padStart requires maxLength argument"));
        }
        let max_length = args
            .get(0)
            .expect("failed to get argument for String method")
            .to_integer(context)? as i32;

        let fill_string = args.get(1).map(|arg| arg.to_string(context)).transpose()?;

        Ok(Self::string_pad(primitive, max_length, fill_string, true))
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
    pub(crate) fn trim(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        let this = this.require_object_coercible(context)?;
        let string = this.to_string(context)?;
        Ok(Value::from(string.trim_matches(is_trimmable_whitespace)))
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
    pub(crate) fn trim_start(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        let string = this.to_string(context)?;
        Ok(Value::from(
            string.trim_start_matches(is_trimmable_whitespace),
        ))
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
    pub(crate) fn trim_end(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        let this = this.require_object_coercible(context)?;
        let string = this.to_string(context)?;
        Ok(Value::from(
            string.trim_end_matches(is_trimmable_whitespace),
        ))
    }

    /// `String.prototype.toLowerCase()`
    ///
    /// The `toLowerCase()` method returns the calling string value converted to lower case.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.tolowercase
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/toLowerCase
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_lowercase(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let this_str = this.to_string(context)?;
        // The Rust String is mapped to uppercase using the builtin .to_lowercase().
        // There might be corner cases where it does not behave exactly like Javascript expects
        Ok(Value::from(this_str.to_lowercase()))
    }

    /// `String.prototype.toUpperCase()`
    ///
    /// The `toUpperCase()` method returns the calling string value converted to uppercase.
    ///
    /// The value will be **converted** to a string if it isn't one
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.toUppercase
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/toUpperCase
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_uppercase(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let this_str = this.to_string(context)?;
        // The Rust String is mapped to uppercase using the builtin .to_uppercase().
        // There might be corner cases where it does not behave exactly like Javascript expects
        Ok(Value::from(this_str.to_uppercase()))
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
    pub(crate) fn substring(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = this.to_string(context)?;
        // If no args are specified, start is 'undefined', defaults to 0
        let start = if args.is_empty() {
            0
        } else {
            args.get(0)
                .expect("failed to get argument for String method")
                .to_integer(context)? as i32
        };
        let length = primitive_val.encode_utf16().count() as i32;
        // If less than 2 args specified, end is the length of the this object converted to a String
        let end = if args.len() < 2 {
            length
        } else {
            args.get(1)
                .expect("Could not get argument")
                .to_integer(context)? as i32
        };
        // Both start and end args replaced by 0 if they were negative
        // or by the length of the String if they were greater
        let final_start = min(max(start, 0), length);
        let final_end = min(max(end, 0), length);
        // Start and end are swapped if start is greater than end
        let from = min(final_start, final_end) as usize;
        let to = max(final_start, final_end) as usize;
        // Extract the part of the string contained between the start index and the end index
        // where start is guaranteed to be smaller or equals to end
        let extracted_string: std::result::Result<StdString, _> = decode_utf16(
            primitive_val
                .encode_utf16()
                .skip(from)
                .take(to.wrapping_sub(from)),
        )
        .collect();
        Ok(Value::from(extracted_string.expect("Invalid string")))
    }

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
    pub(crate) fn substr(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // First we get it the actual string a private field stored on the object only the context has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = this.to_string(context)?;
        // If no args are specified, start is 'undefined', defaults to 0
        let mut start = if args.is_empty() {
            0
        } else {
            args.get(0)
                .expect("failed to get argument for String method")
                .to_integer(context)? as i32
        };
        let length = primitive_val.chars().count() as i32;
        // If less than 2 args specified, end is +infinity, the maximum number value.
        // Using i32::max_value() should be safe because the final length used is at most
        // the number of code units from start to the end of the string,
        // which should always be smaller or equals to both +infinity and i32::max_value
        let end = if args.len() < 2 {
            i32::MAX
        } else {
            args.get(1)
                .expect("Could not get argument")
                .to_integer(context)? as i32
        };
        // If start is negative it become the number of code units from the end of the string
        if start < 0 {
            start = max(length.wrapping_add(start), 0);
        }
        // length replaced by 0 if it was negative
        // or by the number of code units from start to the end of the string if it was greater
        let result_length = min(max(end, 0), length.wrapping_sub(start));
        // If length is negative we return an empty string
        // otherwise we extract the part of the string from start and is length code units long
        if result_length <= 0 {
            Ok(Value::from(""))
        } else {
            let extracted_string: StdString = primitive_val
                .chars()
                .skip(start as usize)
                .take(result_length as usize)
                .collect();

            Ok(Value::from(extracted_string))
        }
    }

    /// String.prototype.split()
    ///
    /// The `split()` method divides a String into an ordered list of substrings, puts these substrings into an array, and returns the array.
    ///
    /// The division is done by searching for a pattern; where the pattern is provided as the first parameter in the method's call.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.split
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/split
    pub(crate) fn split(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let this = this.require_object_coercible(context)?;
        let string = this.to_string(context)?;

        let separator = args.get(0).filter(|value| !value.is_null_or_undefined());

        if let Some(result) = separator
            .and_then(|separator| separator.as_object())
            .and_then(|separator| {
                let key = WellKnownSymbols::split();

                match separator.get_method(context, key) {
                    Ok(splitter) => splitter.map(|splitter| {
                        let arguments = &[
                            Value::from(string.clone()),
                            args.get(1)
                                .map(|x| x.to_owned())
                                .unwrap_or(Value::Undefined),
                        ];
                        splitter.call(this, arguments, context)
                    }),
                    Err(_) => Some(Err(
                        context.construct_type_error("separator[Symbol.split] is not a function")
                    )),
                }
            })
        {
            return result;
        }

        let separator = separator
            .map(|separator| separator.to_string(context))
            .transpose()?;

        let limit = args
            .get(1)
            .map(|arg| arg.to_integer(context).map(|limit| limit as usize))
            .transpose()?
            .unwrap_or(u32::MAX as usize);

        let values: Vec<Value> = match separator {
            None if limit == 0 => vec![],
            None => vec![Value::from(string)],
            Some(separator) if separator.is_empty() => string
                .encode_utf16()
                // TODO: Support keeping invalid code point in string
                .map(|cp| Value::from(std::string::String::from_utf16_lossy(&[cp])))
                .take(limit)
                .collect(),
            Some(separator) => string
                .split(separator.as_str())
                .map(&Value::from)
                .take(limit)
                .collect(),
        };

        let new = Array::new_array(context);
        Array::construct_array(&new, &values, context)
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
    pub(crate) fn value_of(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // Use the to_string method because it is specified to do the same thing in this case
        Self::to_string(this, args, context)
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
    // TODO: update this method to return iterator
    pub(crate) fn match_all(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let re: Value = match args.get(0) {
            Some(arg) => {
                if arg.is_null() {
                    RegExp::constructor(
                        &Value::from(Object::default()),
                        &[Value::from(arg.to_string(context)?), Value::from("g")],
                        context,
                    )
                } else if arg.is_undefined() {
                    RegExp::constructor(
                        &Value::from(Object::default()),
                        &[Value::undefined(), Value::from("g")],
                        context,
                    )
                } else {
                    Ok(arg.clone())
                }
            }
            None => RegExp::constructor(
                &Value::from(Object::default()),
                &[Value::from(""), Value::from("g")],
                context,
            ),
        }?;

        RegExp::match_all(&re, this.to_string(context)?.to_string(), context)
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

    pub(crate) fn normalize(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let this = this.require_object_coercible(context)?;
        let s = this.to_string(context)?;
        let form = args.get(0).cloned().unwrap_or_default();

        let f_str;

        let f = match form.is_undefined() {
            true => "NFC",
            false => {
                f_str = form.to_string(context)?;
                f_str.as_str()
            }
        };

        match f {
            "NFC" => Ok(Value::from(s.nfc().collect::<StdString>())),
            "NFD" => Ok(Value::from(s.nfd().collect::<StdString>())),
            "NFKC" => Ok(Value::from(s.nfkc().collect::<StdString>())),
            "NFKD" => Ok(Value::from(s.nfkd().collect::<StdString>())),
            _ => context
                .throw_range_error("The normalization form should be one of NFC, NFD, NFKC, NFKD."),
        }
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
    pub(crate) fn search(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible(context)?;

        // 2. If regexp is neither undefined nor null, then
        let regexp = args.get(0).cloned().unwrap_or_default();
        if !regexp.is_null_or_undefined() {
            // a. Let searcher be ? GetMethod(regexp, @@search).
            // b. If searcher is not undefined, then
            if let Some(searcher) = regexp
                .to_object(context)?
                .get_method(context, WellKnownSymbols::search())?
            {
                // i. Return ? Call(searcher, regexp, « O »).
                return searcher.call(&regexp, &[this.clone()], context);
            }
        }

        // 3. Let string be ? ToString(O).
        let s = this.to_string(context)?;

        // 4. Let rx be ? RegExpCreate(regexp, undefined).
        let rx = RegExp::constructor(&Value::from(Object::default()), &[regexp], context)?;

        // 5. Return ? Invoke(rx, @@search, « string »).
        if let Some(searcher) = rx
            .to_object(context)?
            .get_method(context, WellKnownSymbols::search())?
        {
            searcher.call(&rx, &[Value::from(s)], context)
        } else {
            context.throw_type_error("regexp[Symbol.search] is not a function")
        }
    }

    pub(crate) fn iterator(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        StringIterator::create_string_iterator(context, this.clone())
    }
}
