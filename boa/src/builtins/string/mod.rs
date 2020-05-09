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

use crate::{
    builtins::{
        object::{internal_methods_trait::ObjectInternalMethods, Object, ObjectKind, PROTOTYPE},
        property::Property,
        regexp::{make_regexp, match_all as regexp_match_all, r#match as regexp_match},
        value::{from_value, to_value, undefined, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::Gc;
use regex::Regex;
use std::{
    cmp::{max, min},
    f64::NAN,
    ops::Deref,
};

/// Create new string [[Construct]]
// This gets called when a new String() is created, it's called by exec:346
pub fn make_string(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    // If we're constructing a string, we should set the initial length
    // To do this we need to convert the string back to a Rust String, then get the .len()
    // let a: String = from_value(args.get(0).expect("failed to get argument for String method").clone()).unwrap();
    // this.set_field_slice("length", to_value(a.len() as i32));

    // This value is used by console.log and other routines to match Obexpecty"failed to parse argument for String method"pe
    // to its Javascript Identifier (global constructor method name)
    this.set_kind(ObjectKind::String);
    this.set_internal_slot(
        "StringData",
        args.get(0)
            .expect("failed to get StringData for make_string()")
            .clone(),
    );
    Ok(this.clone())
}

/// Call new string [[Call]]
///
/// More information: [ECMAScript reference](https://tc39.es/ecma262/#sec-string-constructor-string-value)
pub fn call_string(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let arg = match args.get(0) {
        Some(v) => v.clone(),
        None => undefined(),
    };

    if arg.is_undefined() {
        return Ok(to_value(""));
    }

    Ok(to_value(arg.to_string()))
}

/// Get the string value to a primitive string
pub fn to_string(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    // Get String from String Object and send it back as a new value
    let primitive_val = this.get_internal_slot("StringData");
    Ok(to_value(format!("{}", primitive_val)))
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
pub fn char_at(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val = ctx.value_to_rust_string(this);
    let pos: i32 = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");

    // Calling .len() on a string would give the wrong result, as they are bytes not the number of
    // unicode code points
    // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of
    // bytes is an O(1) operation.
    let length = primitive_val.chars().count();

    // We should return an empty string is pos is out of range
    if pos >= length as i32 || pos < 0 {
        return Ok(to_value::<String>(String::new()));
    }

    Ok(to_value::<char>(
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
pub fn char_code_at(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = ctx.value_to_rust_string(this);

    // Calling .len() on a string would give the wrong result, as they are bytes not the number of unicode code points
    // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of bytes is an O(1) operation.
    let length = primitive_val.chars().count();
    let pos: i32 = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");

    if pos >= length as i32 || pos < 0 {
        return Ok(to_value(NAN));
    }

    let utf16_val = primitive_val
        .encode_utf16()
        .nth(pos as usize)
        .expect("failed to get utf16 value");
    // If there is no element at that index, the result is NaN
    // TODO: We currently don't have NaN
    Ok(to_value(f64::from(utf16_val)))
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
pub fn concat(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let mut new_str = ctx.value_to_rust_string(this);

    for arg in args {
        let concat_str: String = from_value(arg.clone()).expect("failed to get argument value");
        new_str.push_str(&concat_str);
    }

    Ok(to_value(new_str))
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
pub fn repeat(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = ctx.value_to_rust_string(this);

    let repeat_times: usize = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");
    Ok(to_value(primitive_val.repeat(repeat_times)))
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
pub fn slice(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = ctx.value_to_rust_string(this);

    let start: i32 = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");
    let end: i32 = from_value(
        args.get(1)
            .expect("failed to get argument in slice")
            .clone(),
    )
    .expect("failed to parse argument");

    // Calling .len() on a string would give the wrong result, as they are bytes not the number of unicode code points
    // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of bytes is an O(1) operation.
    let length: i32 = primitive_val.chars().count() as i32;

    let from: i32 = if start < 0 {
        max(length.wrapping_add(start), 0)
    } else {
        min(start, length)
    };
    let to: i32 = if end < 0 {
        max(length.wrapping_add(end), 0)
    } else {
        min(end, length)
    };

    let span = max(to.wrapping_sub(from), 0);

    let mut new_str = String::new();
    for i in from..from.wrapping_add(span) {
        new_str.push(
            primitive_val
                .chars()
                .nth(i as usize)
                .expect("Could not get nth char"),
        );
    }
    Ok(to_value(new_str))
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
pub fn starts_with(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = ctx.value_to_rust_string(this);

    // TODO: Should throw TypeError if pattern is regular expression
    let search_string: String = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");

    let length: i32 = primitive_val.chars().count() as i32;
    let search_length: i32 = search_string.chars().count() as i32;

    // If less than 2 args specified, position is 'undefined', defaults to 0
    let position: i32 = if args.len() < 2 {
        0
    } else {
        from_value(args.get(1).expect("failed to get arg").clone()).expect("failed to get argument")
    };

    let start = min(max(position, 0), length);
    let end = start.wrapping_add(search_length);

    if end > length {
        Ok(to_value(false))
    } else {
        // Only use the part of the string from "start"
        let this_string: String = primitive_val.chars().skip(start as usize).collect();
        Ok(to_value(this_string.starts_with(&search_string)))
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
pub fn ends_with(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = ctx.value_to_rust_string(this);

    // TODO: Should throw TypeError if search_string is regular expression
    let search_string: String = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");

    let length: i32 = primitive_val.chars().count() as i32;
    let search_length: i32 = search_string.chars().count() as i32;

    // If less than 2 args specified, end_position is 'undefined', defaults to
    // length of this
    let end_position: i32 = if args.len() < 2 {
        length
    } else {
        from_value(args.get(1).expect("Could not get argumetn").clone())
            .expect("Could not convert value to i32")
    };

    let end = min(max(end_position, 0), length);
    let start = end.wrapping_sub(search_length);

    if start < 0 {
        Ok(to_value(false))
    } else {
        // Only use the part of the string up to "end"
        let this_string: String = primitive_val.chars().take(end as usize).collect();
        Ok(to_value(this_string.ends_with(&search_string)))
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
pub fn includes(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = ctx.value_to_rust_string(this);

    // TODO: Should throw TypeError if search_string is regular expression
    let search_string: String = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");

    let length: i32 = primitive_val.chars().count() as i32;

    // If less than 2 args specified, position is 'undefined', defaults to 0
    let position: i32 = if args.len() < 2 {
        0
    } else {
        from_value(args.get(1).expect("Could not get argument").clone())
            .expect("Could not convert value to i32")
    };

    let start = min(max(position, 0), length);

    // Take the string from "this" and use only the part of it after "start"
    let this_string: String = primitive_val.chars().skip(start as usize).collect();

    Ok(to_value(this_string.contains(&search_string)))
}

/// Return either the string itself or the string of the regex equivalent
fn get_regex_string(value: &Value) -> String {
    match value.deref() {
        ValueData::String(ref body) => body.into(),
        ValueData::Object(ref obj) => {
            let slots = &obj.borrow().internal_slots;
            if slots.get("RegExpMatcher").is_some() {
                // first argument is another `RegExp` object, so copy its pattern and flags
                if let Some(body) = slots.get("OriginalSource") {
                    return from_value(r#body.clone())
                        .expect("unable to get body from regex value");
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
pub fn replace(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // TODO: Support Symbol replacer
    let primitive_val: String = ctx.value_to_rust_string(this);
    if args.is_empty() {
        return Ok(to_value(primitive_val));
    }

    let regex_body = get_regex_string(args.get(0).expect("Value needed"));
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
                let mut result: String = val.to_string();
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
                    .map(|capture| to_value(capture.unwrap().as_str()))
                    .collect();

                // Returns the starting byte offset of the match
                let start = caps
                    .get(0)
                    .expect("Unable to get Byte offset from string for match")
                    .start();
                results.push(to_value(start));
                // Push the whole string being examined
                results.push(to_value(primitive_val.to_string()));

                let result = ctx.call(&replace_object, this, &results).unwrap();

                ctx.value_to_rust_string(&result)
            }
            _ => "undefined".to_string(),
        }
    } else {
        "undefined".to_string()
    };

    Ok(to_value(primitive_val.replacen(
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
pub fn index_of(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = ctx.value_to_rust_string(this);

    // TODO: Should throw TypeError if search_string is regular expression
    let search_string: String = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");

    let length: i32 = primitive_val.chars().count() as i32;

    // If less than 2 args specified, position is 'undefined', defaults to 0
    let position: i32 = if args.len() < 2 {
        0
    } else {
        from_value(args.get(1).expect("Could not get argument").clone())
            .expect("Could not convert value to i32")
    };

    let start = min(max(position, 0), length);

    // Here cannot use the &str method "find", because this returns the byte
    // index: we need to return the char index in the JS String
    // Instead, iterate over the part we're checking until the slice we're
    // checking "starts with" the search string
    for index in start..length {
        let this_string: String = primitive_val.chars().skip(index as usize).collect();
        if this_string.starts_with(&search_string) {
            // Explicitly return early with the index value
            return Ok(to_value(index));
        }
    }
    // Didn't find a match, so return -1
    Ok(to_value(-1))
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
pub fn last_index_of(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = ctx.value_to_rust_string(this);

    // TODO: Should throw TypeError if search_string is regular expression
    let search_string: String = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");

    let length: i32 = primitive_val.chars().count() as i32;

    // If less than 2 args specified, position is 'undefined', defaults to 0
    let position: i32 = if args.len() < 2 {
        0
    } else {
        from_value(args.get(1).expect("Could not get argument").clone())
            .expect("Could not convert value to i32")
    };

    let start = min(max(position, 0), length);

    // Here cannot use the &str method "rfind", because this returns the last
    // byte index: we need to return the last char index in the JS String
    // Instead, iterate over the part we're checking keeping track of the higher
    // index we found that "starts with" the search string
    let mut highest_index: i32 = -1;
    for index in start..length {
        let this_string: String = primitive_val.chars().skip(index as usize).collect();
        if this_string.starts_with(&search_string) {
            highest_index = index;
        }
    }

    // This will still be -1 if no matches were found, else with be >= 0
    Ok(to_value(highest_index))
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
pub fn r#match(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let mut re = make_regexp(&mut to_value(Object::default()), &[args[0].clone()], ctx)?;
    regexp_match(&mut re, ctx.value_to_rust_string(this), ctx)
}

/// Abstract method `StringPad`.
///
/// Performs the actual string padding for padStart/End.
/// <https://tc39.es/ecma262/#sec-stringpad/>
fn string_pad(
    primitive: String,
    max_length: i32,
    fill_string: Option<String>,
    at_start: bool,
) -> ResultValue {
    let primitive_length = primitive.len() as i32;

    if max_length <= primitive_length {
        return Ok(to_value(primitive));
    }

    let filler = match fill_string {
        Some(filler) => filler,
        None => String::from(" "),
    };

    if filler == "" {
        return Ok(to_value(primitive));
    }

    let fill_len = max_length.wrapping_sub(primitive_length);
    let mut fill_str = String::new();

    while fill_str.len() < fill_len as usize {
        fill_str.push_str(&filler);
    }
    // Cut to size max_length
    let concat_fill_str: String = fill_str.chars().take(fill_len as usize).collect();

    if at_start {
        Ok(to_value(format!("{}{}", concat_fill_str, &primitive)))
    } else {
        Ok(to_value(format!("{}{}", primitive, &concat_fill_str)))
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
pub fn pad_end(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let primitive_val: String = ctx.value_to_rust_string(this);
    if args.is_empty() {
        return Err(to_value("padEnd requires maxLength argument"));
    }
    let max_length = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");
    let fill_string: Option<String> = match args.len() {
        1 => None,
        _ => Some(
            from_value(args.get(1).expect("Could not get argument").clone())
                .expect("Could not convert value to Option<String>"),
        ),
    };

    string_pad(primitive_val, max_length, fill_string, false)
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
pub fn pad_start(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let primitive_val: String = ctx.value_to_rust_string(this);
    if args.is_empty() {
        return Err(to_value("padStart requires maxLength argument"));
    }
    let max_length = from_value(
        args.get(0)
            .expect("failed to get argument for String method")
            .clone(),
    )
    .expect("failed to parse argument for String method");
    let fill_string: Option<String> = match args.len() {
        1 => None,
        _ => Some(
            from_value(args.get(1).expect("Could not get argument").clone())
                .expect("Could not convert value to Option<String>"),
        ),
    };

    string_pad(primitive_val, max_length, fill_string, true)
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
pub fn trim(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let this_str: String = ctx.value_to_rust_string(this);
    Ok(to_value(this_str.trim_matches(is_trimmable_whitespace)))
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
pub fn trim_start(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let this_str: String = ctx.value_to_rust_string(this);
    Ok(to_value(
        this_str.trim_start_matches(is_trimmable_whitespace),
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
pub fn trim_end(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let this_str: String = ctx.value_to_rust_string(this);
    Ok(to_value(this_str.trim_end_matches(is_trimmable_whitespace)))
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
pub fn to_lowercase(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let this_str: String = ctx.value_to_rust_string(this);
    // The Rust String is mapped to uppercase using the builtin .to_lowercase().
    // There might be corner cases where it does not behave exactly like Javascript expects
    Ok(to_value(this_str.to_lowercase()))
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
pub fn to_uppercase(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let this_str: String = ctx.value_to_rust_string(this);
    // The Rust String is mapped to uppercase using the builtin .to_uppercase().
    // There might be corner cases where it does not behave exactly like Javascript expects
    Ok(to_value(this_str.to_uppercase()))
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
pub fn substring(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = ctx.value_to_rust_string(this);
    // If no args are specified, start is 'undefined', defaults to 0
    let start = if args.is_empty() {
        0
    } else {
        from_value(
            args.get(0)
                .expect("failed to get argument for String method")
                .clone(),
        )
        .expect("failed to parse argument for String method")
    };
    let length: i32 = primitive_val.chars().count() as i32;
    // If less than 2 args specified, end is the length of the this object converted to a String
    let end = if args.len() < 2 {
        length
    } else {
        from_value(args.get(1).expect("Could not get argument").clone())
            .expect("failed to parse argument for String method")
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
    let extracted_string: String = primitive_val
        .chars()
        .skip(from)
        .take(to.wrapping_sub(from))
        .collect();
    Ok(to_value(extracted_string))
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
pub fn substr(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = ctx.value_to_rust_string(this);
    // If no args are specified, start is 'undefined', defaults to 0
    let mut start = if args.is_empty() {
        0
    } else {
        from_value(
            args.get(0)
                .expect("failed to get argument for String method")
                .clone(),
        )
        .expect("failed to parse argument for String method")
    };
    let length: i32 = primitive_val.chars().count() as i32;
    // If less than 2 args specified, end is +infinity, the maximum number value.
    // Using i32::max_value() should be safe because the final length used is at most
    // the number of code units from start to the end of the string,
    // which should always be smaller or equals to both +infinity and i32::max_value
    let end = if args.len() < 2 {
        i32::max_value()
    } else {
        from_value(args.get(1).expect("Could not get argument").clone())
            .expect("failed to parse argument for String method")
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
        Ok(to_value("".to_string()))
    } else {
        let extracted_string: String = primitive_val
            .chars()
            .skip(start as usize)
            .take(result_length as usize)
            .collect();
        Ok(to_value(extracted_string))
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
pub fn value_of(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // Use the to_string method because it is specified to do the same thing in this case
    to_string(this, args, ctx)
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
pub fn match_all(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let mut re: Value = match args.get(0) {
        Some(arg) => {
            if arg == &Gc::new(ValueData::Null) {
                make_regexp(
                    &mut to_value(Object::default()),
                    &[
                        to_value(ctx.value_to_rust_string(arg)),
                        to_value(String::from("g")),
                    ],
                    ctx,
                )
            } else if arg == &undefined() {
                make_regexp(
                    &mut to_value(Object::default()),
                    &[undefined(), to_value(String::from("g"))],
                    ctx,
                )
            } else {
                from_value(arg.clone()).map_err(to_value)
            }
        }
        None => make_regexp(
            &mut to_value(Object::default()),
            &[to_value(String::new()), to_value(String::from("g"))],
            ctx,
        ),
    }?;

    regexp_match_all(&mut re, ctx.value_to_rust_string(this))
}

/// Create a new `String` object.
pub fn create(global: &Value) -> Value {
    // Create prototype
    let prototype = ValueData::new_obj(Some(global));
    let length = Property::default().value(to_value(0_i32));

    prototype.set_prop_slice("length", length);
    make_builtin_fn!(char_at, named "charAt", with length 1, of prototype);
    make_builtin_fn!(char_code_at, named "charCodeAt", with length 1, of prototype);
    make_builtin_fn!(to_string, named "toString", of prototype);
    make_builtin_fn!(concat, named "concat", with length 1, of prototype);
    make_builtin_fn!(repeat, named "repeat", with length 1, of prototype);
    make_builtin_fn!(slice, named "slice", with length 2, of prototype);
    make_builtin_fn!(starts_with, named "startsWith", with length 1, of prototype);
    make_builtin_fn!(ends_with, named "endsWith", with length 1, of prototype);
    make_builtin_fn!(includes, named "includes", with length 1, of prototype);
    make_builtin_fn!(index_of, named "indexOf", with length 1, of prototype);
    make_builtin_fn!(last_index_of, named "lastIndexOf", with length 1, of prototype);
    make_builtin_fn!(r#match, named "match", with length 1, of prototype);
    make_builtin_fn!(pad_end, named "padEnd", with length 1, of prototype);
    make_builtin_fn!(pad_start, named "padStart", with length 1, of prototype);
    make_builtin_fn!(trim, named "trim", of prototype);
    make_builtin_fn!(trim_start, named "trimStart", of prototype);
    make_builtin_fn!(to_lowercase, named "toLowerCase", of prototype);
    make_builtin_fn!(to_uppercase, named "toUpperCase", of prototype);
    make_builtin_fn!(substring, named "substring", with length 2, of prototype);
    make_builtin_fn!(substr, named "substr", with length 2, of prototype);
    make_builtin_fn!(value_of, named "valueOf", of prototype);
    make_builtin_fn!(match_all, named "matchAll", with length 1, of prototype);
    make_builtin_fn!(replace, named "replace", with length 2, of prototype);

    make_constructor_fn!(make_string, call_string, global, prototype)
}

/// Initialise the `String` object on the global object.
#[inline]
pub fn init(global: &Value) {
    global.set_field_slice("String", create(global));
}
