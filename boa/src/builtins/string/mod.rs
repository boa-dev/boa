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

#[cfg(test)]
mod tests;

use super::function::{make_builtin_fn, make_constructor_fn};
use crate::{
    builtins::{
        object::{Object, ObjectData},
        property::Property,
        value::{ResultValue, Value, ValueData},
        RegExp,
    },
    exec::Interpreter,
    BoaProfiler,
};
use regex::Regex;
use std::string::String as StdString;
use std::{
    cmp::{max, min},
    f64::NAN,
    ops::Deref,
};

/// JavaScript `String` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct String;

impl String {
    fn this_string_value(this: &Value, ctx: &mut Interpreter) -> Result<StdString, Value> {
        match this.data() {
            ValueData::String(ref string) => return Ok(string.clone()),
            ValueData::Object(ref object) => {
                let object = object.borrow();
                if let Some(string) = object.as_string() {
                    return Ok(string.clone());
                }
            }
            _ => {}
        }

        Err(ctx
            .throw_type_error("'this' is not a string")
            .expect_err("throw_type_error() did not return an error"))
    }

    /// [[Construct]] - Creates a new instance `this`
    ///
    /// [[Call]] - Returns a new native `string`
    /// <https://tc39.es/ecma262/#sec-string-constructor-string-value>
    pub(crate) fn make_string(
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        // This value is used by console.log and other routines to match Obexpecty"failed to parse argument for String method"pe
        // to its Javascript Identifier (global constructor method name)
        let string = match args.get(0) {
            Some(ref value) => ctx.to_string(value)?,
            None => StdString::new(),
        };

        let length = string.chars().count();

        this.set_field("length", Value::from(length as i32));

        this.set_data(ObjectData::String(string.clone()));

        Ok(Value::from(string))
    }

    /// Get the string value to a primitive string
    #[allow(clippy::wrong_self_convention)]
    #[inline]
    pub(crate) fn to_string(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // Get String from String Object and send it back as a new value
        Ok(Value::from(Self::this_string_value(this, ctx)?))
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
    pub(crate) fn char_at(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;
        let pos = i32::from(
            args.get(0)
                .expect("failed to get argument for String method"),
        );

        // Calling .len() on a string would give the wrong result, as they are bytes not the number of
        // unicode code points
        // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of
        // bytes is an O(1) operation.
        let length = primitive_val.chars().count();

        // We should return an empty string is pos is out of range
        if pos >= length as i32 || pos < 0 {
            return Ok("".into());
        }

        Ok(Value::from(
            primitive_val
                .chars()
                .nth(pos as usize)
                .expect("failed to get value"),
        ))
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
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;

        // Calling .len() on a string would give the wrong result, as they are bytes not the number of unicode code points
        // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of bytes is an O(1) operation.
        let length = primitive_val.chars().count();
        let pos = i32::from(
            args.get(0)
                .expect("failed to get argument for String method"),
        );

        if pos >= length as i32 || pos < 0 {
            return Ok(Value::from(NAN));
        }

        let utf16_val = primitive_val
            .encode_utf16()
            .nth(pos as usize)
            .expect("failed to get utf16 value");
        // If there is no element at that index, the result is NaN
        // TODO: We currently don't have NaN
        Ok(Value::from(f64::from(utf16_val)))
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
    pub(crate) fn concat(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let object = ctx.require_object_coercible(this)?;
        let mut string = ctx.to_string(object)?;

        for arg in args {
            string.push_str(&ctx.to_string(arg)?);
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
    pub(crate) fn repeat(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;

        let repeat_times = usize::from(
            args.get(0)
                .expect("failed to get argument for String method"),
        );

        Ok(Value::from(primitive_val.repeat(repeat_times)))
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
    pub(crate) fn slice(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;

        let start = i32::from(
            args.get(0)
                .expect("failed to get argument for String method"),
        );

        let end = i32::from(args.get(1).expect("failed to get argument in slice"));

        // Calling .len() on a string would give the wrong result, as they are bytes not the number of unicode code points
        // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of bytes is an O(1) operation.
        let length = primitive_val.chars().count() as i32;

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
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;

        // TODO: Should throw TypeError if pattern is regular expression
        let search_string = ctx.to_string(
            args.get(0)
                .expect("failed to get argument for String method"),
        )?;

        let length = primitive_val.chars().count() as i32;
        let search_length = search_string.chars().count() as i32;

        // If less than 2 args specified, position is 'undefined', defaults to 0
        let position = if args.len() < 2 {
            0
        } else {
            i32::from(args.get(1).expect("failed to get arg"))
        };

        let start = min(max(position, 0), length);
        let end = start.wrapping_add(search_length);

        if end > length {
            Ok(Value::from(false))
        } else {
            // Only use the part of the string from "start"
            let this_string: StdString = primitive_val.chars().skip(start as usize).collect();
            Ok(Value::from(this_string.starts_with(&search_string)))
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
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;

        // TODO: Should throw TypeError if search_string is regular expression
        let search_string = ctx.to_string(
            args.get(0)
                .expect("failed to get argument for String method"),
        )?;

        let length = primitive_val.chars().count() as i32;
        let search_length = search_string.chars().count() as i32;

        // If less than 2 args specified, end_position is 'undefined', defaults to
        // length of this
        let end_position = if args.len() < 2 {
            length
        } else {
            i32::from(args.get(1).expect("Could not get argumetn"))
        };

        let end = min(max(end_position, 0), length);
        let start = end.wrapping_sub(search_length);

        if start < 0 {
            Ok(Value::from(false))
        } else {
            // Only use the part of the string up to "end"
            let this_string: StdString = primitive_val.chars().take(end as usize).collect();
            Ok(Value::from(this_string.ends_with(&search_string)))
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
    pub(crate) fn includes(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;

        // TODO: Should throw TypeError if search_string is regular expression
        let search_string = ctx.to_string(
            args.get(0)
                .expect("failed to get argument for String method"),
        )?;

        let length = primitive_val.chars().count() as i32;

        // If less than 2 args specified, position is 'undefined', defaults to 0
        let position = if args.len() < 2 {
            0
        } else {
            i32::from(args.get(1).expect("Could not get argument"))
        };

        let start = min(max(position, 0), length);

        // Take the string from "this" and use only the part of it after "start"
        let this_string: StdString = primitive_val.chars().skip(start as usize).collect();

        Ok(Value::from(this_string.contains(&search_string)))
    }

    /// Return either the string itself or the string of the regex equivalent
    fn get_regex_string(value: &Value) -> StdString {
        match value.deref() {
            ValueData::String(ref body) => body.into(),
            ValueData::Object(ref obj) => {
                let obj = obj.borrow();

                if obj.internal_slots().get("RegExpMatcher").is_some() {
                    // first argument is another `RegExp` object, so copy its pattern and flags
                    if let Some(body) = obj.internal_slots().get("OriginalSource") {
                        return body.to_string();
                    }
                }
                "undefined".to_string()
            }
            _ => "undefined".to_string(),
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
    pub(crate) fn replace(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // TODO: Support Symbol replacer
        let primitive_val = ctx.to_string(this)?;
        if args.is_empty() {
            return Ok(Value::from(primitive_val));
        }

        let regex_body = Self::get_regex_string(args.get(0).expect("Value needed"));
        let re = Regex::new(&regex_body).expect("unable to convert regex to regex object");
        let mat = re.find(&primitive_val).expect("unable to find value");
        let caps = re
            .captures(&primitive_val)
            .expect("unable to get capture groups from text");

        let replace_value = if args.len() > 1 {
            // replace_object could be a string or function or not exist at all
            let replace_object: &Value = args.get(1).expect("second argument expected");
            match replace_object.deref() {
                ValueData::String(val) => {
                    // https://tc39.es/ecma262/#table-45
                    let mut result = val.to_string();
                    let re = Regex::new(r"\$(\d)").unwrap();

                    if val.find("$$").is_some() {
                        result = val.replace("$$", "$")
                    }

                    if val.find("$`").is_some() {
                        let start_of_match = mat.start();
                        let slice = &primitive_val[..start_of_match];
                        result = val.replace("$`", slice);
                    }

                    if val.find("$'").is_some() {
                        let end_of_match = mat.end();
                        let slice = &primitive_val[end_of_match..];
                        result = val.replace("$'", slice);
                    }

                    if val.find("$&").is_some() {
                        // get matched value
                        let matched = caps.get(0).expect("cannot get matched value");
                        result = val.replace("$&", matched.as_str());
                    }

                    // Capture $1, $2, $3 etc
                    if re.is_match(&result) {
                        let mat_caps = re.captures(&result).unwrap();
                        let group_str = mat_caps.get(1).unwrap().as_str();
                        let group_int = group_str.parse::<usize>().unwrap();
                        result = re
                            .replace(result.as_str(), caps.get(group_int).unwrap().as_str())
                            .to_string()
                    }

                    result
                }
                ValueData::Object(_) => {
                    // This will return the matched substring first, then captured parenthesized groups later
                    let mut results: Vec<Value> = caps
                        .iter()
                        .map(|capture| Value::from(capture.unwrap().as_str()))
                        .collect();

                    // Returns the starting byte offset of the match
                    let start = caps
                        .get(0)
                        .expect("Unable to get Byte offset from string for match")
                        .start();
                    results.push(Value::from(start));
                    // Push the whole string being examined
                    results.push(Value::from(primitive_val.to_string()));

                    let result = ctx.call(&replace_object, this, &results).unwrap();

                    ctx.to_string(&result)?
                }
                _ => "undefined".to_string(),
            }
        } else {
            "undefined".to_string()
        };

        Ok(Value::from(primitive_val.replacen(
            &mat.as_str(),
            &replace_value,
            1,
        )))
    }

    /// `String.prototype.indexOf( searchValue[, fromIndex] )`
    ///
    /// The `indexOf()` method returns the index within the calling `String` object of the first occurrence of the specified value, starting the search at `fromIndex`.
    ///
    /// Returns -1 if the value is not found.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.indexof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/indexOf
    pub(crate) fn index_of(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;

        // TODO: Should throw TypeError if search_string is regular expression
        let search_string = ctx.to_string(
            args.get(0)
                .expect("failed to get argument for String method"),
        )?;

        let length = primitive_val.chars().count() as i32;

        // If less than 2 args specified, position is 'undefined', defaults to 0
        let position = if args.len() < 2 {
            0
        } else {
            i32::from(args.get(1).expect("Could not get argument"))
        };

        let start = min(max(position, 0), length);

        // Here cannot use the &str method "find", because this returns the byte
        // index: we need to return the char index in the JS String
        // Instead, iterate over the part we're checking until the slice we're
        // checking "starts with" the search string
        for index in start..length {
            let this_string: StdString = primitive_val.chars().skip(index as usize).collect();
            if this_string.starts_with(&search_string) {
                // Explicitly return early with the index value
                return Ok(Value::from(index));
            }
        }
        // Didn't find a match, so return -1
        Ok(Value::from(-1))
    }

    /// `String.prototype.lastIndexOf( searchValue[, fromIndex] )`
    ///
    /// The `lastIndexOf()` method returns the index within the calling `String` object of the last occurrence of the specified value, searching backwards from `fromIndex`.
    ///
    /// Returns -1 if the value is not found.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string.prototype.lastindexof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/lastIndexOf
    pub(crate) fn last_index_of(
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;

        // TODO: Should throw TypeError if search_string is regular expression
        let search_string = ctx.to_string(
            args.get(0)
                .expect("failed to get argument for String method"),
        )?;

        let length = primitive_val.chars().count() as i32;

        // If less than 2 args specified, position is 'undefined', defaults to 0
        let position = if args.len() < 2 {
            0
        } else {
            i32::from(args.get(1).expect("Could not get argument"))
        };

        let start = min(max(position, 0), length);

        // Here cannot use the &str method "rfind", because this returns the last
        // byte index: we need to return the last char index in the JS String
        // Instead, iterate over the part we're checking keeping track of the higher
        // index we found that "starts with" the search string
        let mut highest_index = -1;
        for index in start..length {
            let this_string: StdString = primitive_val.chars().skip(index as usize).collect();
            if this_string.starts_with(&search_string) {
                highest_index = index;
            }
        }

        // This will still be -1 if no matches were found, else with be >= 0
        Ok(Value::from(highest_index))
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
    pub(crate) fn r#match(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let mut re =
            RegExp::make_regexp(&mut Value::from(Object::default()), &[args[0].clone()], ctx)?;
        RegExp::r#match(&mut re, ctx.to_string(this)?, ctx)
    }

    /// Abstract method `StringPad`.
    ///
    /// Performs the actual string padding for padStart/End.
    /// <https://tc39.es/ecma262/#sec-stringpad/>
    fn string_pad(
        primitive: StdString,
        max_length: i32,
        fill_string: Option<StdString>,
        at_start: bool,
    ) -> ResultValue {
        let primitive_length = primitive.len() as i32;

        if max_length <= primitive_length {
            return Ok(Value::from(primitive));
        }

        let filler = match fill_string {
            Some(filler) => filler,
            None => " ".to_owned(),
        };

        if filler == "" {
            return Ok(Value::from(primitive));
        }

        let fill_len = max_length.wrapping_sub(primitive_length);
        let mut fill_str = StdString::new();

        while fill_str.len() < fill_len as usize {
            fill_str.push_str(&filler);
        }
        // Cut to size max_length
        let concat_fill_str: StdString = fill_str.chars().take(fill_len as usize).collect();

        if at_start {
            Ok(Value::from(format!("{}{}", concat_fill_str, &primitive)))
        } else {
            Ok(Value::from(format!("{}{}", primitive, &concat_fill_str)))
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
    pub(crate) fn pad_end(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let primitive_val = ctx.to_string(this)?;
        if args.is_empty() {
            return Err(Value::from("padEnd requires maxLength argument"));
        }
        let max_length = i32::from(
            args.get(0)
                .expect("failed to get argument for String method"),
        );

        let fill_string = if args.len() != 1 {
            Some(ctx.to_string(args.get(1).expect("Could not get argument"))?)
        } else {
            None
        };

        Self::string_pad(primitive_val, max_length, fill_string, false)
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
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        let primitive_val = ctx.to_string(this)?;
        if args.is_empty() {
            return Err(Value::from("padStart requires maxLength argument"));
        }
        let max_length = i32::from(
            args.get(0)
                .expect("failed to get argument for String method"),
        );

        let fill_string = match args.len() {
            1 => None,
            _ => Some(ctx.to_string(args.get(1).expect("Could not get argument"))?),
        };

        Self::string_pad(primitive_val, max_length, fill_string, true)
    }

    /// Helper function to check if a `char` is trimmable.
    fn is_trimmable_whitespace(c: char) -> bool {
        // The rust implementation of `trim` does not regard the same characters whitespace as ecma standard does
        //
        // Rust uses \p{White_Space} by default, which also includes:
        // `\u{0085}' (next line)
        // And does not include:
        // '\u{FEFF}' (zero width non-breaking space)
        match c {
        // Explicit whitespace: https://tc39.es/ecma262/#sec-white-space
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{FEFF}' |
        // Unicode Space_Seperator category
        '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' |
        // Line terminators: https://tc39.es/ecma262/#sec-line-terminators
        '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => true,
        _ => false,
    }
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
    pub(crate) fn trim(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let this_str = ctx.to_string(this)?;
        Ok(Value::from(
            this_str.trim_matches(Self::is_trimmable_whitespace),
        ))
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
    pub(crate) fn trim_start(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let this_str = ctx.to_string(this)?;
        Ok(Value::from(
            this_str.trim_start_matches(Self::is_trimmable_whitespace),
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
    pub(crate) fn trim_end(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let this_str = ctx.to_string(this)?;
        Ok(Value::from(
            this_str.trim_end_matches(Self::is_trimmable_whitespace),
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
    pub(crate) fn to_lowercase(
        this: &mut Value,
        _: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let this_str = ctx.to_string(this)?;
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
    pub(crate) fn to_uppercase(
        this: &mut Value,
        _: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let this_str = ctx.to_string(this)?;
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
    pub(crate) fn substring(
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;
        // If no args are specified, start is 'undefined', defaults to 0
        let start = if args.is_empty() {
            0
        } else {
            i32::from(
                args.get(0)
                    .expect("failed to get argument for String method"),
            )
        };
        let length = primitive_val.chars().count() as i32;
        // If less than 2 args specified, end is the length of the this object converted to a String
        let end = if args.len() < 2 {
            length
        } else {
            i32::from(args.get(1).expect("Could not get argument"))
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
        let extracted_string: StdString = primitive_val
            .chars()
            .skip(from)
            .take(to.wrapping_sub(from))
            .collect();
        Ok(Value::from(extracted_string))
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
    pub(crate) fn substr(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // First we get it the actual string a private field stored on the object only the engine has access to.
        // Then we convert it into a Rust String by wrapping it in from_value
        let primitive_val = ctx.to_string(this)?;
        // If no args are specified, start is 'undefined', defaults to 0
        let mut start = if args.is_empty() {
            0
        } else {
            i32::from(
                args.get(0)
                    .expect("failed to get argument for String method"),
            )
        };
        let length = primitive_val.chars().count() as i32;
        // If less than 2 args specified, end is +infinity, the maximum number value.
        // Using i32::max_value() should be safe because the final length used is at most
        // the number of code units from start to the end of the string,
        // which should always be smaller or equals to both +infinity and i32::max_value
        let end = if args.len() < 2 {
            i32::max_value()
        } else {
            i32::from(args.get(1).expect("Could not get argument"))
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
    pub(crate) fn value_of(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // Use the to_string method because it is specified to do the same thing in this case
        Self::to_string(this, args, ctx)
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
    pub(crate) fn match_all(
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        let mut re: Value = match args.get(0) {
            Some(arg) => {
                if arg.is_null() {
                    RegExp::make_regexp(
                        &mut Value::from(Object::default()),
                        &[Value::from(ctx.to_string(arg)?), Value::from("g")],
                        ctx,
                    )
                } else if arg.is_undefined() {
                    RegExp::make_regexp(
                        &mut Value::from(Object::default()),
                        &[Value::undefined(), Value::from("g")],
                        ctx,
                    )
                } else {
                    Ok(arg.clone())
                }
            }
            None => RegExp::make_regexp(
                &mut Value::from(Object::default()),
                &[Value::from(""), Value::from("g")],
                ctx,
            ),
        }?;

        RegExp::match_all(&mut re, ctx.to_string(this)?)
    }

    /// Create a new `String` object.
    pub(crate) fn create(global: &Value) -> Value {
        // Create prototype
        let prototype = Value::new_object(Some(global));
        let length = Property::default().value(Value::from(0));

        prototype.set_property("length", length);

        make_builtin_fn(Self::char_at, "charAt", &prototype, 1);
        make_builtin_fn(Self::char_code_at, "charCodeAt", &prototype, 1);
        make_builtin_fn(Self::to_string, "toString", &prototype, 0);
        make_builtin_fn(Self::concat, "concat", &prototype, 1);
        make_builtin_fn(Self::repeat, "repeat", &prototype, 1);
        make_builtin_fn(Self::slice, "slice", &prototype, 2);
        make_builtin_fn(Self::starts_with, "startsWith", &prototype, 1);
        make_builtin_fn(Self::ends_with, "endsWith", &prototype, 1);
        make_builtin_fn(Self::includes, "includes", &prototype, 1);
        make_builtin_fn(Self::index_of, "indexOf", &prototype, 1);
        make_builtin_fn(Self::last_index_of, "lastIndexOf", &prototype, 1);
        make_builtin_fn(Self::r#match, "match", &prototype, 1);
        make_builtin_fn(Self::pad_end, "padEnd", &prototype, 1);
        make_builtin_fn(Self::pad_start, "padStart", &prototype, 1);
        make_builtin_fn(Self::trim, "trim", &prototype, 0);
        make_builtin_fn(Self::trim_start, "trimStart", &prototype, 0);
        make_builtin_fn(Self::trim_end, "trimEnd", &prototype, 0);
        make_builtin_fn(Self::to_lowercase, "toLowerCase", &prototype, 0);
        make_builtin_fn(Self::to_uppercase, "toUpperCase", &prototype, 0);
        make_builtin_fn(Self::substring, "substring", &prototype, 2);
        make_builtin_fn(Self::substr, "substr", &prototype, 2);
        make_builtin_fn(Self::value_of, "valueOf", &prototype, 0);
        make_builtin_fn(Self::match_all, "matchAll", &prototype, 1);
        make_builtin_fn(Self::replace, "replace", &prototype, 2);

        make_constructor_fn("String", 1, Self::make_string, global, prototype, true)
    }

    /// Initialise the `String` object on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) -> (&str, Value) {
        let _timer = BoaProfiler::global().start_event("string", "init");

        ("String", Self::create(global))
    }
}
