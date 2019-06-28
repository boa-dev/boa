use crate::js::function::NativeFunctionData;
use crate::js::object::{Property, PROTOTYPE};
use crate::js::value::{from_value, to_value, ResultValue, Value, ValueData};
use gc::Gc;
use std::f64::NAN;

/// Create new string
/// https://searchfox.org/mozilla-central/source/js/src/vm/StringObject.h#19
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
    let this_str: String =
        from_value(this.get_private_field(String::from("PrimitiveValue"))).unwrap();
    Ok(to_value::<i32>(this_str.len() as i32))
}

/// Get the string value to a primitive string
pub fn to_string(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    // Get String from String Object and send it back as a new value
    let primitive_val = this.get_private_field(String::from("PrimitiveValue"));
    Ok(to_value(format!("{}", primitive_val).to_string()))
}

/// Returns a single element String containing the code unit at index pos within the String value resulting from converting this object to a String. If there is no element at that index, the result is the empty String. The result is a String value, not a String object.
/// https://tc39.github.io/ecma262/#sec-string.prototype.charat
pub fn char_at(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //         ^^ represents instance  ^^ represents arguments (we only care about the first one in this case)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String =
        from_value(this.get_private_field(String::from("PrimitiveValue"))).unwrap();
    let pos = from_value(args[0].clone()).unwrap();

    // Calling .len() on a string would give the wrong result, as they are bytes not the number of unicode code points
    // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of bytes is an O(1) operation.
    let length = primitive_val.chars().count();

    // We should return an empty string is pos is out of range
    if pos >= length || pos < 0 as usize {
        return Ok(to_value::<String>(String::new()));
    }

    Ok(to_value::<char>(primitive_val.chars().nth(pos).unwrap()))
}

/// Returns a Number (a nonnegative integer less than 216) that is the numeric value of the code unit at index pos within the String resulting from converting this object to a String. If there is no element at that index, the result is NaN.
/// https://tc39.github.io/ecma262/#sec-string.prototype.charcodeat
pub fn char_code_at(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    //              ^^ represents instance  ^^ represents arguments (we only care about the first one in this case)
    // First we get it the actual string a private field stored on the object only the engine has access to.
    // Then we convert it into a Rust String by wrapping it in from_value
    let primitive_val: String =
        from_value(this.get_private_field(String::from("PrimitiveValue"))).unwrap();

    // Calling .len() on a string would give the wrong result, as they are bytes not the number of unicode code points
    // Note that this is an O(N) operation (because UTF-8 is complex) while getting the number of bytes is an O(1) operation.
    let length = primitive_val.chars().count();
    let pos = from_value(args[0].clone()).unwrap();

    if pos >= length || pos < 0 as usize {
        return Ok(to_value(NAN));
    }

    let utf16_val = primitive_val.encode_utf16().nth(pos).unwrap();
    // If there is no element at that index, the result is NaN
    // TODO: We currently don't have NaN
    Ok(to_value(utf16_val as f64))
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
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{FEFF}' => true,
        // Unicode Space_Seperator category
        '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}'  => true,
        // Line terminators: https://tc39.es/ecma262/#sec-line-terminators
        '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}' => true,
        _ => false,
    }
}

pub fn trim(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let this_str: String =
        from_value(this.get_private_field(String::from("PrimitiveValue"))).unwrap();
    Ok(to_value(this_str.trim_matches(is_trimmable_whitespace)))
}

pub fn trim_start(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let this_str: String =
        from_value(this.get_private_field(String::from("PrimitiveValue"))).unwrap();
    Ok(to_value(this_str.trim_start_matches(is_trimmable_whitespace)))
}

pub fn trim_end(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let this_str: String =
        from_value(this.get_private_field(String::from("PrimitiveValue"))).unwrap();
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
    proto.set_field_slice("trim", to_value(trim as NativeFunctionData));
    proto.set_field_slice("trimStart", to_value(trim_start as NativeFunctionData));
    proto.set_field_slice("trimEnd", to_value(trim_end as NativeFunctionData));
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
    #[test]
    fn check_string_constructor_is_function() {
        let global = ValueData::new_obj(None);
        let string_constructor = _create(&global);
        assert_eq!(string_constructor.is_function(), true);
    }
}
