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
use std::{
    char::{decode_utf16, from_u32},
    cmp::{max, min},
    string::String as StdString,
};

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
        // 1. Let O be ? RequireObjectCoercible(this value).
        this.require_object_coercible(context)?;

        let search_value = args.get(0).cloned().unwrap_or_default();

        let replace_value = args.get(1).cloned().unwrap_or_default();

        // 2. If searchValue is neither undefined nor null, then
        if !search_value.is_null_or_undefined() {
            // a. Let replacer be ? GetMethod(searchValue, @@replace).
            let replacer = search_value
                .as_object()
                .unwrap_or_default()
                .get_method(context, WellKnownSymbols::replace())?;

            // b. If replacer is not undefined, then
            if let Some(replacer) = replacer {
                // i. Return ? Call(replacer, searchValue, « O, replaceValue »).
                return context.call(
                    &replacer.into(),
                    &search_value,
                    &[this.clone(), replace_value],
                );
            }
        }

        // 3. Let string be ? ToString(O).
        let this_str = this.to_string(context)?;

        // 4. Let searchString be ? ToString(searchValue).
        let search_str = search_value.to_string(context)?;

        // 5. Let functionalReplace be IsCallable(replaceValue).
        let functional_replace = replace_value.is_function();

        // 6. If functionalReplace is false, then
        // a. Set replaceValue to ? ToString(replaceValue).

        // 7. Let searchLength be the length of searchString.
        let search_length = search_str.len();

        // 8. Let position be ! StringIndexOf(string, searchString, 0).
        let position = this_str.find(search_str.as_str());

        // 9. If position is -1, return string.
        if position.is_none() {
            return Ok(this_str.into());
        }

        // 10. Let preserved be the substring of string from 0 to position.
        let preserved = this_str.get(..position.unwrap());

        // 11. If functionalReplace is true, then
        // 12. Else,
        let replacement: RcString;
        if functional_replace {
            // a. Let replacement be ? ToString(? Call(replaceValue, undefined, « searchString, 𝔽(position), string »)).
            replacement = context
                .call(
                    &replace_value,
                    &Value::Undefined,
                    &[
                        search_str.into(),
                        position.unwrap().into(),
                        this_str.clone().into(),
                    ],
                )?
                .to_string(context)?;
        } else {
            // a. Assert: Type(replaceValue) is String.
            // b. Let captures be a new empty List.
            let captures = Vec::new();

            // c. Let replacement be ! GetSubstitution(searchString, string, position, captures, undefined, replaceValue).
            replacement = get_substitution(
                search_str.to_string(),
                this_str.to_string(),
                position.unwrap(),
                captures,
                Value::undefined(),
                replace_value.to_string(context)?.to_string(),
            )?;
        }

        // 13. Return the string-concatenation of preserved, replacement, and the substring of string from position + searchLength.
        Ok(format!(
            "{}{}{}",
            preserved.unwrap_or_default(),
            replacement,
            this_str
                .get((position.unwrap() + search_length)..)
                .unwrap_or_default()
        )
        .into())
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
        // 1. Let O be ? RequireObjectCoercible(this value).
        let object = this.require_object_coercible(context)?;

        // 2. If regexp is neither undefined nor null, then
        let regexp = args.get(0).cloned().unwrap_or_default();
        if !regexp.is_null_or_undefined() {
            // a. Let matcher be ? GetMethod(regexp, @@match).
            // b. If matcher is not undefined, then
            if let Some(matcher) = regexp
                .as_object()
                .unwrap_or_default()
                .get_method(context, "match")?
            {
                // i. Return ? Call(matcher, regexp, « O »).
                return matcher.call(&regexp, &[object.clone()], context);
            }
        }

        // 3. Let S be ? ToString(O).
        let arg_str = object.to_string(context)?;

        // 4. Let rx be ? RegExpCreate(regexp, undefined).
        let rx = RegExp::constructor(&Value::from(Object::default()), &[regexp], context)?;

        // 5. Return ? Invoke(rx, @@match, « S »).
        RegExp::r#match(&rx, &[Value::from(arg_str)], context)
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

        let filter = fill_string.as_deref().unwrap_or(" ");

        let fill_len = max_length.wrapping_sub(primitive_length);
        let mut fill_str = StdString::new();

        while fill_str.len() < fill_len as usize {
            fill_str.push_str(filter);
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
    pub(crate) fn split(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let this = this.require_object_coercible(context)?;

        let separator = args.get(0).cloned().unwrap_or_default();
        let limit = args.get(1).cloned().unwrap_or_default();

        // 2. If separator is neither undefined nor null, then
        if !separator.is_null_or_undefined() {
            // a. Let splitter be ? GetMethod(separator, @@split).
            // b. If splitter is not undefined, then
            if let Some(splitter) = separator
                .as_object()
                .unwrap_or_default()
                .get_method(context, WellKnownSymbols::split())?
            {
                // i. Return ? Call(splitter, separator, « O, limit »).
                return splitter.call(&separator, &[this.clone(), limit], context);
            }
        }

        // 3. Let S be ? ToString(O).
        let this_str = this.to_string(context)?;

        // 4. Let A be ! ArrayCreate(0).
        let a = Array::array_create(0, None, context);

        // 5. Let lengthA be 0.
        let mut length_a = 0;

        // 6.  If limit is undefined, let lim be 2^32 - 1; else let lim be ℝ(? ToUint32(limit)).
        let lim = if limit.is_undefined() {
            u32::MAX
        } else {
            limit.to_u32(context)?
        };

        // 7. Let R be ? ToString(separator).
        let separator_str = separator.to_string(context)?;

        // 8. If lim = 0, return A.
        if lim == 0 {
            return Ok(a);
        }

        // 9. If separator is undefined, then
        if separator.is_undefined() {
            // a. Perform ! CreateDataPropertyOrThrow(A, "0", S).
            Array::add_to_array_object(&a, &[Value::from(this_str)], context)?;

            // b. Return A.
            return Ok(a);
        }

        // 10. Let s be the length of S.
        let this_str_length = this_str.encode_utf16().count();

        // 11. If s = 0, then
        if this_str_length == 0 {
            // a. If R is not the empty String, then
            if !separator_str.is_empty() {
                // i. Perform ! CreateDataPropertyOrThrow(A, "0", S).
                Array::add_to_array_object(&a, &[Value::from(this_str)], context)?;
            }

            // b. Return A.
            return Ok(a);
        }

        // 12. Let p be 0.
        // 13. Let q be p.
        let mut p = 0;
        let mut q = p;

        // 14. Repeat, while q ≠ s,
        while q != this_str_length {
            // a. Let e be SplitMatch(S, q, R).
            let e = split_match(&this_str, q, &separator_str);

            match e {
                // b. If e is not-matched, set q to q + 1.
                None => q += 1,
                // c. Else,
                Some(e) => {
                    // i. Assert: e is a non-negative integer ≤ s.
                    // ii. If e = p, set q to q + 1.
                    // iii. Else,
                    if e == p {
                        q += 1;
                    } else {
                        // 1. Let T be the substring of S from p to q.
                        let this_str_substring = StdString::from_utf16_lossy(
                            &this_str
                                .encode_utf16()
                                .skip(p)
                                .take(q - p)
                                .collect::<Vec<u16>>(),
                        );

                        // 2. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(lengthA)), T).
                        Array::add_to_array_object(
                            &a,
                            &[Value::from(this_str_substring)],
                            context,
                        )?;

                        // 3. Set lengthA to lengthA + 1.
                        length_a += 1;

                        // 4. If lengthA = lim, return A.
                        if length_a == lim {
                            return Ok(a);
                        }

                        // 5. Set p to e.
                        p = e;

                        // 6. Set q to p.
                        q = p;
                    }
                }
            }
        }

        // 15. Let T be the substring of S from p to s.
        let this_str_substring = StdString::from_utf16_lossy(
            &this_str
                .encode_utf16()
                .skip(p)
                .take(this_str_length - p)
                .collect::<Vec<u16>>(),
        );

        // 16. Perform ! CreateDataPropertyOrThrow(A, ! ToString(𝔽(lengthA)), T).
        Array::add_to_array_object(&a, &[Value::from(this_str_substring)], context)?;

        // 17. Return A.
        Ok(a)
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
    pub(crate) fn match_all(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let O be ? RequireObjectCoercible(this value).
        let object = this.require_object_coercible(context)?;

        // 2. If regexp is neither undefined nor null, then
        let regexp = args.get(0).cloned().unwrap_or_default();
        if !regexp.is_null_or_undefined() {
            // a. Let isRegExp be ? IsRegExp(regexp).
            // b. If isRegExp is true, then
            if regexp.as_object().unwrap_or_default().is_regexp() {
                // i. Let flags be ? Get(regexp, "flags").
                let flags = regexp.get_field("flags", context)?;

                // ii. Perform ? RequireObjectCoercible(flags).
                flags.require_object_coercible(context)?;

                // iii. If ? ToString(flags) does not contain "g", throw a TypeError exception.
                if !flags.to_string(context)?.contains('g') {
                    return context.throw_type_error(
                        "String.prototype.matchAll called with a non-global RegExp argument",
                    );
                }
            }

            // c. Let matcher be ? GetMethod(regexp, @@matchAll).
            // d. If matcher is not undefined, then
            if let Some(matcher) = regexp
                .as_object()
                .unwrap_or_default()
                .get_method(context, WellKnownSymbols::match_all())?
            {
                // i. Return ? Call(matcher, regexp, « O »).
                return matcher.call(&regexp, &[object.clone()], context);
            }
        }

        // 3. Let S be ? ToString(O).
        let arg_str = object.to_string(context)?;

        // 4. Let rx be ? RegExpCreate(regexp, "g").
        let rx = RegExp::constructor(
            &Value::from(Object::default()),
            &[regexp, Value::from("g")],
            context,
        )?;

        // 5. Return ? Invoke(rx, @@matchAll, « S »).
        RegExp::match_all(&rx, &[Value::from(arg_str)], context)
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

/// `22.1.3.17.1 GetSubstitution ( matched, str, position, captures, namedCaptures, replacement )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-getsubstitution
pub(crate) fn get_substitution(
    matched: StdString,
    str: StdString,
    position: usize,
    captures: Vec<Value>,
    _named_captures: Value,
    replacement: StdString,
) -> Result<RcString> {
    // 1. Assert: Type(matched) is String.

    // 2. Let matchLength be the number of code units in matched.
    let match_length = matched.chars().count();

    // 3. Assert: Type(str) is String.

    // 4. Let stringLength be the number of code units in str.
    let str_length = str.chars().count();

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
    let mut result = StdString::new();
    let mut chars = replacement.chars().peekable();

    while let Some(first) = chars.next() {
        if first == '$' {
            let second = chars.next();
            let second_is_digit = second.map_or(false, |ch| ch.is_digit(10));
            // we use peek so that it is still in the iterator if not used
            let third = if second_is_digit { chars.peek() } else { None };
            let third_is_digit = third.map_or(false, |ch| ch.is_digit(10));

            match (second, third) {
                // $$
                (Some('$'), _) => {
                    // $
                    result.push('$');
                }
                // $&
                (Some('&'), _) => {
                    // matched
                    result.push_str(&matched);
                }
                // $`
                (Some('`'), _) => {
                    // The replacement is the substring of str from 0 to position.
                    result.push_str(&str[..position]);
                }
                // $'
                (Some('\''), _) => {
                    // If tailPos ≥ stringLength, the replacement is the empty String.
                    // Otherwise the replacement is the substring of str from tailPos.
                    if tail_pos >= str_length {
                        result.push_str("");
                    } else {
                        result.push_str(&str[tail_pos..]);
                    }
                }
                // $nn
                (Some(second), Some(third)) if second_is_digit && third_is_digit => {
                    // The nnth element of captures, where nn is a two-digit decimal number in the range 01 to 99.
                    // If nn ≤ m and the nnth element of captures is undefined, use the empty String instead.
                    // If nn is 00 or nn > m, no replacement is done.
                    let tens = second.to_digit(10).unwrap() as usize;
                    let units = third.to_digit(10).unwrap() as usize;
                    let nn = 10 * tens + units;
                    let capture = if let Some(v) = captures.get(nn - 1) {
                        v.clone()
                    } else {
                        Value::undefined()
                    };

                    if nn <= m && capture.is_undefined() {
                        result.push_str("")
                    } else if nn == 0 || nn > m {
                        result.push('$');
                        result.push(first);
                        result.push(second);
                    } else if let Some(s) = capture.as_string() {
                        result.push_str(s);
                        break;
                    }
                }
                // $n
                (Some(first), second) if second_is_digit => {
                    // The nth element of captures, where n is a single digit in the range 1 to 9.
                    // If n ≤ m and the nth element of captures is undefined, use the empty String instead.
                    // If n > m, no replacement is done.
                    let n = first.to_digit(10).unwrap() as usize;
                    let capture = if let Some(v) = captures.get(n - 1) {
                        v.clone()
                    } else {
                        Value::undefined()
                    };

                    if n <= m && capture.is_undefined() {
                        result.push_str("")
                    } else if n > m {
                        result.push('$');
                        result.push(first);
                        if let Some(second) = second {
                            result.push(*second)
                        }
                    } else if let Some(s) = capture.as_string() {
                        result.push_str(s);
                    }
                }
                // $<
                (Some('<'), _) => {
                    // TODO: named capture groups
                    // 1. If namedCaptures is undefined, the replacement text is the String "$<".
                    result.push_str("$<");
                }
                // $?, ? is none of the above
                _ => {
                    result.push('$');
                    if let Some(second) = second {
                        result.push(second);
                    }
                }
            }
        } else {
            result.push(first);
        }
    }

    // 11. Return result.
    Ok(result.into())
}

/// `22.1.3.21.1 SplitMatch ( S, q, R )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-splitmatch
fn split_match(s_str: &str, q: usize, r_str: &str) -> Option<usize> {
    // 1. Let r be the number of code units in R.
    let r = r_str.encode_utf16().count();

    // 2. Let s be the number of code units in S.
    let s = s_str.encode_utf16().count();

    // 3. If q + r > s, return not-matched.
    if q + r > s {
        return None;
    }

    // 4. If there exists an integer i between 0 (inclusive) and r (exclusive)
    //    such that the code unit at index q + i within S is different from the code unit at index i within R,
    //    return not-matched.
    for i in 0..r {
        if let Some(s_char) = s_str.encode_utf16().nth(q + i) {
            if let Some(r_char) = r_str.encode_utf16().nth(i) {
                if s_char != r_char {
                    return None;
                }
            }
        }
    }

    // 5. Return q + r.
    Some(q + r)
}
