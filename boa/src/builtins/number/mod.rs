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

use super::{
    function::{make_builtin_fn, make_constructor_fn},
    object::ObjectKind,
};
use crate::{
    builtins::{
        object::internal_methods_trait::ObjectInternalMethods,
        value::{ResultValue, Value, ValueData},
        RangeError,
    },
    exec::Interpreter,
};
use num_traits::float::FloatCore;
use std::{borrow::Borrow, f64, ops::Deref};

const BUF_SIZE: usize = 2200;

/// `Number` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Number;

impl Number {
    /// Helper function that converts a Value to a Number.
    #[allow(clippy::wrong_self_convention)]
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
            ValueData::BigInt(ref bigint) => Value::from(bigint.to_f64()),
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

    /// `[[Construct]]` - Creates a Number instance
    ///
    /// `[[Call]]` - Creates a number primitive
    pub(crate) fn make_number(
        this: &mut Value,
        args: &[Value],
        _ctx: &mut Interpreter,
    ) -> ResultValue {
        let data = match args.get(0) {
            Some(ref value) => Self::to_number(value),
            None => Self::to_number(&Value::from(0)),
        };
        this.set_kind(ObjectKind::Number);
        this.set_internal_slot("NumberData", data.clone());

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
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_exponential(
        this: &mut Value,
        _args: &[Value],
        _ctx: &mut Interpreter,
    ) -> ResultValue {
        let this_num = Self::to_number(this).to_number();
        let this_str_num = Self::num_to_exponential(this_num);
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
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_fixed(
        this: &mut Value,
        args: &[Value],
        _ctx: &mut Interpreter,
    ) -> ResultValue {
        let this_num = Self::to_number(this).to_number();
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
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_locale_string(
        this: &mut Value,
        _args: &[Value],
        _ctx: &mut Interpreter,
    ) -> ResultValue {
        let this_num = Self::to_number(this).to_number();
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
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_precision(
        this: &mut Value,
        args: &[Value],
        _ctx: &mut Interpreter,
    ) -> ResultValue {
        let this_num = Self::to_number(this);
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
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_native_string_radix(mut value: f64, radix: u8) -> String {
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
        let mut delta = 0.5 * (Self::next_after(value, f64::MAX) - value);
        delta = Self::next_after(0.0, f64::MAX).max(delta);
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
                frac_buf[fraction_cursor] =
                    std::char::from_digit(digit, radix as u32).unwrap() as u8;
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

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_native_string(x: f64) -> String {
        if x == -0. {
            return "0".to_owned();
        } else if x.is_nan() {
            return "NaN".to_owned();
        } else if x.is_infinite() && x.is_sign_positive() {
            return "Infinity".to_owned();
        } else if x.is_infinite() && x.is_sign_negative() {
            return "-Infinity".to_owned();
        }

        // FIXME: This is not spec compliant.
        format!("{}", x)
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
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        // 1. Let x be ? thisNumberValue(this value).
        let x = Self::to_number(this).to_number();
        // 2. If radix is undefined, let radixNumber be 10.
        // 3. Else, let radixNumber be ? ToInteger(radix).
        let radix = args.get(0).map_or(10, |arg| arg.to_integer()) as u8;

        // 4. If radixNumber < 2 or radixNumber > 36, throw a RangeError exception.
        if radix < 2 || radix > 36 {
            return Err(RangeError::run_new(
                "radix must be an integer at least 2 and no greater than 36",
                ctx,
            )?);
        }

        if x == -0. {
            return Ok(Value::from("0"));
        } else if x.is_nan() {
            return Ok(Value::from("NaN"));
        } else if x.is_infinite() && x.is_sign_positive() {
            return Ok(Value::from("Infinity"));
        } else if x.is_infinite() && x.is_sign_negative() {
            return Ok(Value::from("-Infinity"));
        }

        // 5. If radixNumber = 10, return ! ToString(x).
        // This part should use exponential notations for long integer numbers commented tests
        if radix == 10 {
            // return Ok(to_value(format!("{}", Self::to_number(this).to_num())));
            return Ok(Value::from(Self::to_native_string(x)));
        }

        // This is a Optimization from the v8 source code to print values that can fit in a single character
        // Since the actual num_to_string allocates a 2200 bytes buffer for actual conversion
        // I am not sure if this part is effective as the v8 equivalent https://chromium.googlesource.com/v8/v8/+/refs/heads/master/src/builtins/number.tq#53
        // // Fast case where the result is a one character string.
        // if x.is_sign_positive() && x.fract() == 0.0 && x < radix_number as f64 {
        //     return Ok(to_value(format!("{}", std::char::from_digit(x as u32, radix_number as u32).unwrap())))
        // }

        // 6. Return the String representation of this Number value using the radix specified by radixNumber.
        Ok(Value::from(Self::to_native_string_radix(x, radix)))
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
    pub(crate) fn value_of(
        this: &mut Value,
        _args: &[Value],
        _ctx: &mut Interpreter,
    ) -> ResultValue {
        Ok(Self::to_number(this))
    }

    /// Create a new `Number` object
    pub(crate) fn create(global: &Value) -> Value {
        let prototype = Value::new_object(Some(global));
        prototype.set_internal_slot("NumberData", Value::from(0));

        make_builtin_fn(Self::to_exponential, "toExponential", &prototype, 1);
        make_builtin_fn(Self::to_fixed, "toFixed", &prototype, 1);
        make_builtin_fn(Self::to_locale_string, "toLocaleString", &prototype, 0);
        make_builtin_fn(Self::to_precision, "toPrecision", &prototype, 1);
        make_builtin_fn(Self::to_string, "toString", &prototype, 1);
        make_builtin_fn(Self::value_of, "valueOf", &prototype, 0);

        let number = make_constructor_fn("Number", 1, Self::make_number, global, prototype, true);

        // Constants from:
        // https://tc39.es/ecma262/#sec-properties-of-the-number-constructor
        number.set_field("EPSILON", Value::from(std::f64::EPSILON));
        number.set_field("MAX_SAFE_INTEGER", Value::from(9_007_199_254_740_991_f64));
        number.set_field("MIN_SAFE_INTEGER", Value::from(-9_007_199_254_740_991_f64));
        number.set_field("MAX_VALUE", Value::from(std::f64::MAX));
        number.set_field("MIN_VALUE", Value::from(std::f64::MIN));
        number.set_field("NEGATIVE_INFINITY", Value::from(f64::NEG_INFINITY));
        number.set_field("POSITIVE_INFINITY", Value::from(f64::INFINITY));
        number.set_field("NaN", Value::from(f64::NAN));

        number
    }

    /// Initialise the `Number` object on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) {
        global.set_field("Number", Self::create(global));
    }

    /// The abstract operation Number::equal takes arguments
    /// x (a Number) and y (a Number). It performs the following steps when called:
    ///
    /// https://tc39.es/ecma262/#sec-numeric-types-number-equal
    #[allow(clippy::float_cmp)]
    pub(crate) fn equals(a: f64, b: f64) -> bool {
        a == b
    }

    /// The abstract operation Number::sameValue takes arguments
    /// x (a Number) and y (a Number). It performs the following steps when called:
    ///
    /// https://tc39.es/ecma262/#sec-numeric-types-number-sameValue
    #[allow(clippy::float_cmp)]
    pub(crate) fn same_value(a: f64, b: f64) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }

        if a == 0.0 && b == 0.0 {
            if (a.is_sign_negative() && b.is_sign_positive())
                || (a.is_sign_positive() && b.is_sign_negative())
            {
                return false;
            };
            true
        } else {
            a == b
        }
    }

    /// The abstract operation Number::sameValueZero takes arguments
    /// x (a Number) and y (a Number). It performs the following steps when called:
    ///
    /// https://tc39.es/ecma262/#sec-numeric-types-number-sameValueZero
    #[allow(clippy::float_cmp)]
    pub(crate) fn same_value_zero(a: f64, b: f64) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }

        a == b
    }
}
