//! This module implements the global `Number` object.
//!
//! The `Number` JavaScript object is a wrapper object allowing you to work with numerical values.
//! A `Number` object is created using the `Number()` constructor. A primitive type object number is created using the `Number()` **function**.
//!
//! The JavaScript `Number` type is double-precision 64-bit binary format IEEE 754 value. In more recent implementations,
//! JavaScript also supports integers with arbitrary precision using the BigInt type.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-number-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number

#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        object::{internal_methods_trait::ObjectInternalMethods, Object, PROTOTYPE},
        value::{ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use num_traits::float::FloatCore;
use std::{borrow::Borrow, f64, ops::Deref};

/// Helper function that converts a Value to a Number.
fn to_number(value: &Value) -> Value {
    match *value.deref().borrow() {
        ValueData::Boolean(b) => {
            if b {
                Value::from(1)
            } else {
                Value::from(0)
            }
        }
        ValueData::Symbol(_) | ValueData::Undefined => Value::from(f64::NAN),
        ValueData::Integer(i) => Value::from(f64::from(i)),
        ValueData::Object(ref o) => (o).deref().borrow().get_internal_slot("NumberData"),
        ValueData::Null => Value::from(0),
        ValueData::Rational(n) => Value::from(n),
        ValueData::String(ref s) => match s.parse::<f64>() {
            Ok(n) => Value::from(n),
            Err(_) => Value::from(f64::NAN),
        },
    }
}

/// Helper function that formats a float as a ES6-style exponential number string.
fn num_to_exponential(n: f64) -> String {
    match n.abs() {
        x if x > 1.0 => format!("{:e}", n).replace("e", "e+"),
        x if x == 0.0 => format!("{:e}", n).replace("e", "e+"),
        _ => format!("{:e}", n),
    }
}

/// Create a new number `[[Construct]]`
pub fn make_number(this: &mut Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let data = match args.get(0) {
        Some(ref value) => to_number(value),
        None => to_number(&Value::from(0)),
    };
    this.set_internal_slot("NumberData", data);
    Ok(this.clone())
}

/// `Number()` function.
///
/// More Information https://tc39.es/ecma262/#sec-number-constructor-number-value
pub fn call_number(_this: &mut Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let data = match args.get(0) {
        Some(ref value) => to_number(value),
        None => to_number(&Value::from(0)),
    };
    Ok(data)
}

/// `Number.prototype.toExponential( [fractionDigits] )`
///
/// The `toExponential()` method returns a string representing the Number object in exponential notation.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.toexponential
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toExponential
pub fn to_exponential(this: &mut Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this).to_number();
    let this_str_num = num_to_exponential(this_num);
    Ok(Value::from(this_str_num))
}

/// `Number.prototype.toFixed( [digits] )`
///
/// The `toFixed()` method formats a number using fixed-point notation
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.tofixed
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toFixed
pub fn to_fixed(this: &mut Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this).to_number();
    let precision = match args.get(0) {
        Some(n) => match n.to_integer() {
            x if x > 0 => n.to_integer() as usize,
            _ => 0,
        },
        None => 0,
    };
    let this_fixed_num = format!("{:.*}", precision, this_num);
    Ok(Value::from(this_fixed_num))
}

/// `Number.prototype.toLocaleString( [locales [, options]] )`
///
/// The `toLocaleString()` method returns a string with a language-sensitive representation of this number.
///
/// Note that while this technically conforms to the Ecma standard, it does no actual
/// internationalization logic.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.tolocalestring
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toLocaleString
pub fn to_locale_string(this: &mut Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this).to_number();
    let this_str_num = format!("{}", this_num);
    Ok(Value::from(this_str_num))
}

/// `Number.prototype.toPrecision( [precision] )`
///
/// The `toPrecision()` method returns a string representing the Number object to the specified precision.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.toexponential
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toPrecision
pub fn to_precision(this: &mut Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this);
    let _num_str_len = format!("{}", this_num.to_number()).len();
    let _precision = match args.get(0) {
        Some(n) => match n.to_integer() {
            x if x > 0 => n.to_integer() as usize,
            _ => 0,
        },
        None => 0,
    };
    // TODO: Implement toPrecision
    unimplemented!("TODO: Implement toPrecision");
}

const BUF_SIZE: usize = 2200;

// https://golang.org/src/math/nextafter.go
#[inline]
fn next_after(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        f64::NAN
    } else if (x - y) == 0. {
        x
    } else if x == 0.0 {
        f64::from_bits(1).copysign(y)
    } else if y > x || x > 0.0 {
        f64::from_bits(x.to_bits() + 1)
    } else {
        f64::from_bits(x.to_bits() - 1)
    }
}

// https://chromium.googlesource.com/v8/v8/+/refs/heads/master/src/numbers/conversions.cc#1230
pub fn num_to_string(mut value: f64, radix: u8) -> String {
    assert!(radix >= 2);
    assert!(radix <= 36);
    assert!(value.is_finite());
    // assert_ne!(0.0, value);

    // Character array used for conversion.
    // Temporary buffer for the result. We start with the decimal point in the
    // middle and write to the left for the integer part and to the right for the
    // fractional part. 1024 characters for the exponent and 52 for the mantissa
    // either way, with additional space for sign, decimal point and string
    // termination should be sufficient.
    let mut buffer: [u8; BUF_SIZE] = [0; BUF_SIZE];
    let (int_buf, frac_buf) = buffer.split_at_mut(BUF_SIZE / 2);
    let mut fraction_cursor = 0;
    let negative = value.is_sign_negative();
    if negative {
        value = -value
    }
    // Split the value into an integer part and a fractional part.
    // let mut integer = value.trunc();
    // let mut fraction = value.fract();
    let mut integer = value.floor();
    let mut fraction = value - integer;

    // We only compute fractional digits up to the input double's precision.
    let mut delta = 0.5 * (next_after(value, f64::MAX) - value);
    delta = next_after(0.0, f64::MAX).max(delta);
    assert!(delta > 0.0);
    if fraction >= delta {
        // Insert decimal point.
        frac_buf[fraction_cursor] = b'.';
        fraction_cursor += 1;
        loop {
            // Shift up by one digit.
            fraction *= radix as f64;
            delta *= radix as f64;
            // Write digit.
            let digit = fraction as u32;
            frac_buf[fraction_cursor] = std::char::from_digit(digit, radix as u32).unwrap() as u8;
            fraction_cursor += 1;
            // Calculate remainder.
            fraction -= digit as f64;
            // Round to even.
            if fraction + delta > 1.0
                && (fraction > 0.5 || (fraction - 0.5) < f64::EPSILON && digit & 1 != 0)
            {
                loop {
                    // We need to back trace already written digits in case of carry-over.
                    fraction_cursor -= 1;
                    if fraction_cursor == 0 {
                        //              CHECK_EQ('.', buffer[fraction_cursor]);
                        // Carry over to the integer part.
                        integer += 1.;
                        break;
                    } else {
                        let c: u8 = frac_buf[fraction_cursor];
                        // Reconstruct digit.
                        let digit_0 = (c as char).to_digit(10).unwrap();
                        if digit_0 + 1 >= radix as u32 {
                            continue;
                        }
                        frac_buf[fraction_cursor] =
                            std::char::from_digit(digit_0 + 1, radix as u32).unwrap() as u8;
                        fraction_cursor += 1;
                        break;
                    }
                }
                break;
            }
            if fraction < delta {
                break;
            }
        }
    }

    // Compute integer digits. Fill unrepresented digits with zero.
    let mut int_iter = int_buf.iter_mut().enumerate().rev(); //.rev();
    while FloatCore::integer_decode(integer / f64::from(radix)).1 > 0 {
        integer /= radix as f64;
        *int_iter.next().unwrap().1 = b'0';
    }

    loop {
        let remainder = integer % (radix as f64);
        *int_iter.next().unwrap().1 =
            std::char::from_digit(remainder as u32, radix as u32).unwrap() as u8;
        integer = (integer - remainder) / radix as f64;
        if integer <= 0f64 {
            break;
        }
    }
    // Add sign and terminate string.
    if negative {
        *int_iter.next().unwrap().1 = b'-';
    }
    assert!(fraction_cursor < BUF_SIZE);

    let integer_cursor = int_iter.next().unwrap().0 + 1;
    let fraction_cursor = fraction_cursor + BUF_SIZE / 2;
    // dbg!("Number: {}, Radix: {}, Cursors: {}, {}", value, radix, integer_cursor, fraction_cursor);
    String::from_utf8_lossy(&buffer[integer_cursor..fraction_cursor]).into()
}

/// `Number.prototype.toString( [radix] )`
///
/// The `toString()` method returns a string representing the specified Number object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.tostring
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toString
pub fn to_string(this: &mut Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    // 1. Let x be ? thisNumberValue(this value).
    let x = to_number(this).to_number();
    // 2. If radix is undefined, let radixNumber be 10.
    // 3. Else, let radixNumber be ? ToInteger(radix).
    let radix_number = args.get(0).map_or(10, |arg| arg.to_integer()) as u8;

    if x == -0. {
        return Ok(Value::from("0"));
    } else if x.is_nan() {
        return Ok(Value::from("NaN"));
    } else if x.is_infinite() && x.is_sign_positive() {
        return Ok(Value::from("Infinity"));
    } else if x.is_infinite() && x.is_sign_negative() {
        return Ok(Value::from("-Infinity"));
    }

    // 4. If radixNumber < 2 or radixNumber > 36, throw a RangeError exception.
    if radix_number < 2 || radix_number > 36 {
        panic!("Radix must be between 2 and 36");
    }

    // 5. If radixNumber = 10, return ! ToString(x).
    // This part should use exponential notations for long integer numbers commented tests
    if radix_number == 10 {
        // return Ok(to_value(format!("{}", to_number(this).to_num())));
        return Ok(Value::from(format!("{}", x)));
    }

    // This is a Optimization from the v8 source code to print values that can fit in a single character
    // Since the actual num_to_string allocates a 2200 bytes buffer for actual conversion
    // I am not sure if this part is effective as the v8 equivalent https://chromium.googlesource.com/v8/v8/+/refs/heads/master/src/builtins/number.tq#53
    // // Fast case where the result is a one character string.
    // if x.is_sign_positive() && x.fract() == 0.0 && x < radix_number as f64 {
    //     return Ok(to_value(format!("{}", std::char::from_digit(x as u32, radix_number as u32).unwrap())))
    // }

    // 6. Return the String representation of this Number value using the radix specified by radixNumber.
    Ok(Value::from(num_to_string(x, radix_number)))
}

/// `Number.prototype.toString()`
///
/// The `valueOf()` method returns the wrapped primitive value of a Number object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.valueof
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/valueOf
pub fn value_of(this: &mut Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    Ok(to_number(this))
}

/// Create a new `Number` object
pub fn create(global: &Value) -> Value {
    let prototype = Value::new_object(Some(global));
    prototype.set_internal_slot("NumberData", Value::from(0));

    make_builtin_fn!(to_exponential, named "toExponential", with length 1, of prototype);
    make_builtin_fn!(to_fixed, named "toFixed", with length 1, of prototype);
    make_builtin_fn!(to_locale_string, named "toLocaleString", of prototype);
    make_builtin_fn!(to_precision, named "toPrecision", with length 1, of prototype);
    make_builtin_fn!(to_string, named "toString", with length 1, of prototype);
    make_builtin_fn!(value_of, named "valueOf", of prototype);

    make_constructor_fn!(make_number, call_number, global, prototype)
}

/// Initialise the `Number` object on the global object.
#[inline]
pub fn init(global: &Value) {
    global.set_field_slice("Number", create(global));
}
