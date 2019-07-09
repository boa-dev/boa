use crate::js::{
    function::NativeFunctionData,
    object::{Property, PROTOTYPE},
    value::{from_value, to_value, ResultValue, Value, ValueData},
};
use gc::Gc;
use std::{
    cmp::{max, min},
    f64::NAN,
};

/// Create new string
/// <https://searchfox.org/mozilla-central/source/js/src/vm/StringObject.h#19>
// This gets called when a new String() is created, it's called by exec:346
pub fn make_string(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    // If we're constructing a string, we should set the initial length
    // To do this we need to convert the string back to a Rust String, then get the .len()
    // let a: String = from_value(args[0].clone()).unwrap();
    // this.set_field_slice("length", to_value(a.len() as i32));

    this.set_private_field_slice("PrimitiveValue", args[0].clone());
    Ok(this)
}

/// Get a string's length
pub fn get_string_length(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let this_str: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();
    Ok(to_value::<i32>(this_str.chars().count() as i32))
}

/// Get the string value to a primitive string
pub fn to_string(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    // Get String from String Object and send it back as a new value
    let primitive_val = this.get_private_field("PrimitiveValue");
    Ok(to_value(format!("{}", primitive_val).to_string()))
}

/// Returns a single element String containing the code unit at index pos within the String value
/// resulting from converting this object to a String. If there is no element at that index, the
/// result is the empty String. The result is a String value, not a String object.
/// <https://tc39.github.io/ecma262/#sec-string.prototype.charat>
pub fn char_at(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //         ^^ represents instance  ^^ represents arguments (we only care about the first one in this case)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();
    let pos: i32 = from_value(args[0].clone()).unwrap();

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
        primitive_val.chars().nth(pos as usize).unwrap(),
    ))
}

/// Returns a Number (a nonnegative integer less than 216) that is the numeric value of the code
/// unit at index pos within the String resulting from converting this object to a String. If there
/// is no element at that index, the result is NaN.
/// <https://tc39.github.io/ecma262/#sec-string.prototype.charcodeat>
pub fn char_code_at(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //              ^^ represents instance  ^^ represents arguments (we only care about the first one in this case)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();

    // Calling .len() on a string would give the wrong result, as they are bytes not the number of unicode code points
    // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of bytes is an O(1) operation.
    let length = primitive_val.chars().count();
    let pos: i32 = from_value(args[0].clone()).unwrap();

    if pos >= length as i32 || pos < 0 {
        return Ok(to_value(NAN));
    }

    let utf16_val = primitive_val.encode_utf16().nth(pos as usize).unwrap();
    // If there is no element at that index, the result is NaN
    // TODO: We currently don't have NaN
    Ok(to_value(f64::from(utf16_val)))
}

/// Returns a String that is the result of concatenating this String and all strings provided as
/// arguments
/// <https://tc39.github.io/ecma262/#sec-string.prototype.concat>
pub fn concat(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //        ^^ represents instance  ^^ represents arguments
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();

    let mut new_str = primitive_val.clone();

    for arg in args {
        let concat_str: String = from_value(arg).unwrap();
        new_str.push_str(&concat_str);
    }

    Ok(to_value(new_str))
}

/// Returns a String that is the result of repeating this String the number of times given by the
/// first argument
/// <https://tc39.github.io/ecma262/#sec-string.prototype.repeat>
pub fn repeat(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //        ^^ represents instance  ^^ represents arguments (only care about the first one in this case)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();

    let repeat_times: usize = from_value(args[0].clone()).unwrap();
    Ok(to_value(primitive_val.repeat(repeat_times)))
}

/// Returns a String which contains the slice of the JS String from character at "start" index up
/// to but not including character at "end" index
/// <https://tc39.github.io/ecma262/#sec-string.prototype.slice>
pub fn slice(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //       ^^ represents instance  ^^ represents arguments)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();

    let start: i32 = from_value(args[0].clone()).unwrap();
    let end: i32 = from_value(args[1].clone()).unwrap();

    // Calling .len() on a string would give the wrong result, as they are bytes not the number of unicode code points
    // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of bytes is an O(1) operation.
    let length: i32 = primitive_val.chars().count() as i32;

    let from: i32 = if start < 0 {
        max(length + start, 0)
    } else {
        min(start, length)
    };
    let to: i32 = if end < 0 {
        max(length + end, 0)
    } else {
        min(end, length)
    };

    let span = max(to - from, 0);

    let mut new_str = String::new();
    for i in from..from + span {
        new_str.push(primitive_val.chars().nth(i as usize).unwrap());
    }
    Ok(to_value(new_str))
}

/// Returns a Boolean indicating whether the sequence of code units of the
/// "search string" is the same as the corresponding code units of this string
/// starting at index "position"
/// <https://tc39.github.io/ecma262/#sec-string.prototype.startswith>
pub fn starts_with(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //             ^^ represents instance  ^^ represents arguments)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();

    // TODO: Should throw TypeError if pattern is regular expression
    let search_string: String = from_value(args[0].clone()).unwrap();

    let length: i32 = primitive_val.chars().count() as i32;
    let search_length: i32 = search_string.chars().count() as i32;

    // If less than 2 args specified, position is 'undefined', defaults to 0
    let position: i32 = if args.len() < 2 {
        0
    } else {
        from_value(args[1].clone()).unwrap()
    };

    let start = min(max(position, 0), length);
    let end = start + search_length;

    if end > length {
        Ok(to_value(false))
    } else {
        // Only use the part of the string from "start"
        let this_string: String = primitive_val.chars().skip(start as usize).collect();
        Ok(to_value(this_string.starts_with(&search_string)))
    }
}

/// Returns a Boolean indicating whether the sequence of code units of the
/// "search string"  is the same as the corresponding code units of this string
/// starting at position "end position" - length
/// <https://tc39.github.io/ecma262/#sec-string.prototype.endswith>
pub fn ends_with(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //           ^^ represents instance  ^^ represents arguments)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();

    // TODO: Should throw TypeError if search_string is regular expression
    let search_string: String = from_value(args[0].clone()).unwrap();

    let length: i32 = primitive_val.chars().count() as i32;
    let search_length: i32 = search_string.chars().count() as i32;

    // If less than 2 args specified, end_position is 'undefined', defaults to
    // length of this
    let end_position: i32 = if args.len() < 2 {
        length
    } else {
        from_value(args[1].clone()).unwrap()
    };

    let end = min(max(end_position, 0), length);
    let start = end - search_length;

    if start < 0 {
        Ok(to_value(false))
    } else {
        // Only use the part of the string up to "end"
        let this_string: String = primitive_val.chars().take(end as usize).collect();
        Ok(to_value(this_string.ends_with(&search_string)))
    }
}

/// Returns a Boolean indicating whether searchString appears as a substring of
/// the result of converting this object to a String, at one or more indices
/// that are greater than or equal to position. If position is undefined, 0 is
/// assumed, so as to search all of the String.
/// <https://tc39.github.io/ecma262/#sec-string.prototype.includes>
pub fn includes(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //          ^^ represents instance  ^^ represents arguments)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();

    // TODO: Should throw TypeError if search_string is regular expression
    let search_string: String = from_value(args[0].clone()).unwrap();

    let length: i32 = primitive_val.chars().count() as i32;

    // If less than 2 args specified, position is 'undefined', defaults to 0
    let position: i32 = if args.len() < 2 {
        0
    } else {
        from_value(args[1].clone()).unwrap()
    };

    let start = min(max(position, 0), length);

    // Take the string from "this" and use only the part of it after "start"
    let this_string: String = primitive_val.chars().skip(start as usize).collect();

    Ok(to_value(this_string.contains(&search_string)))
}

/// If searchString appears as a substring of the result of converting this
/// object to a String, at one or more indices that are greater than or equal to
/// position, then the smallest such index is returned; otherwise, -1 is
/// returned. If position is undefined, 0 is assumed, so as to search all of the
/// String.
/// <https://tc39.github.io/ecma262/#sec-string.prototype.includes>
pub fn index_of(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //          ^^ represents instance  ^^ represents arguments)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();

    // TODO: Should throw TypeError if search_string is regular expression
    let search_string: String = from_value(args[0].clone()).unwrap();

    let length: i32 = primitive_val.chars().count() as i32;

    // If less than 2 args specified, position is 'undefined', defaults to 0
    let position: i32 = if args.len() < 2 {
        0
    } else {
        from_value(args[1].clone()).unwrap()
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

//// If searchString appears as a substring of the result of converting this
/// object to a String at one or more indices that are smaller than or equal to
/// position, then the greatest such index is returned; otherwise, -1 is
/// returned. If position is undefined, the length of the String value is
/// assumed, so as to search all of the String.
/// <https://tc39.github.io/ecma262/#sec-string.prototype.lastindexof>
pub fn last_index_of(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //               ^^ represents instance  ^^ represents arguments)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();

    // TODO: Should throw TypeError if search_string is regular expression
    let search_string: String = from_value(args[0].clone()).unwrap();

    let length: i32 = primitive_val.chars().count() as i32;

    // If less than 2 args specified, position is 'undefined', defaults to 0
    let position: i32 = if args.len() < 2 {
        0
    } else {
        from_value(args[1].clone()).unwrap()
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

pub fn trim(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let this_str: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();
    Ok(to_value(this_str.trim_matches(is_trimmable_whitespace)))
}

pub fn trim_start(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let this_str: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();
    Ok(to_value(
        this_str.trim_start_matches(is_trimmable_whitespace),
    ))
}

pub fn trim_end(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let this_str: String = from_value(this.get_private_field("PrimitiveValue")).unwrap();
    Ok(to_value(this_str.trim_end_matches(is_trimmable_whitespace)))
}

/// Create a new `String` object
pub fn _create(global: &Value) -> Value {
    let string = to_value(make_string as NativeFunctionData);
    let proto = ValueData::new_obj(Some(global));
    let prop = Property {
        configurable: false,
        enumerable: false,
        writable: false,
        value: Gc::new(ValueData::Undefined),
        get: to_value(get_string_length as NativeFunctionData),
        set: Gc::new(ValueData::Undefined),
    };
    proto.set_prop_slice("length", prop);
    proto.set_field_slice("charAt", to_value(char_at as NativeFunctionData));
    proto.set_field_slice("charCodeAt", to_value(char_code_at as NativeFunctionData));
    proto.set_field_slice("toString", to_value(to_string as NativeFunctionData));
    proto.set_field_slice("concat", to_value(concat as NativeFunctionData));
    proto.set_field_slice("repeat", to_value(repeat as NativeFunctionData));
    proto.set_field_slice("slice", to_value(slice as NativeFunctionData));
    proto.set_field_slice("startsWith", to_value(starts_with as NativeFunctionData));
    proto.set_field_slice("endsWith", to_value(ends_with as NativeFunctionData));
    proto.set_field_slice("includes", to_value(includes as NativeFunctionData));
    proto.set_field_slice("indexOf", to_value(index_of as NativeFunctionData));
    proto.set_field_slice("lastIndexOf", to_value(last_index_of as NativeFunctionData));
    proto.set_field_slice("trim", to_value(trim as NativeFunctionData));
    proto.set_field_slice("trimStart", to_value(trim_start as NativeFunctionData));
    string.set_field_slice(PROTOTYPE, proto);
    string
}

/// Initialise the `String` object on the global object
pub fn init(global: &Value) {
    global.set_field_slice("String", _create(global));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::Executor;
    use crate::forward;

    #[test]
    fn check_string_constructor_is_function() {
        let global = ValueData::new_obj(None);
        let string_constructor = _create(&global);
        assert_eq!(string_constructor.is_function(), true);
    }

    #[test]
    fn length() {
        //TEST262: https://github.com/tc39/test262/blob/master/test/built-ins/String/length.js
        let mut engine = Executor::new();
        let init = r#"
        const a = new String(' ');
        const b = new String('\ud834\udf06');
        const c = new String(' \b ');
        cosnt d = new String('中文长度')
        "#;
        forward(&mut engine, init);
        let a = dbg!(forward(&mut engine, "a.length"));
        assert_eq!(a, String::from("1"));
        let b = dbg!(forward(&mut engine, "b.length"));
        // TODO: fix this
        // unicode surrogate pair length should be 1
        // utf16/usc2 length should be 2
        // utf8 length should be 4
        //assert_eq!(b, String::from("2"));
        let c = dbg!(forward(&mut engine, "c.length"));
        assert_eq!(c, String::from("3"));
        let d = dbg!(forward(&mut engine, "d.length"));
        assert_eq!(d, String::from("4"));
    }

    #[test]
    fn concat() {
        let mut engine = Executor::new();
        let init = r#"
        const hello = "Hello, ";
        const world = "world! ";
        const nice = "Have a nice day."
        "#;
        forward(&mut engine, init);
        let a = dbg!(forward(&mut engine, "hello.concat(world, nice)"));
        let b = dbg!(forward(&mut engine, "hello + world + nice"));
        //assert_eq!(a, String::from("Hello, world! Have a nice day."));
        assert_eq!(b, String::from("Hello, world! Have a nice day."));
    }
}
