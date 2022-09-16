//! This module implements the global `Number` object.
//!
//! The `Number` JavaScript object is a wrapper object allowing you to work with numerical values.
//! A `Number` object is created using the `Number()` constructor. A primitive type object number is created using the `Number()` **function**.
//!
//! The JavaScript `Number` type is double-precision 64-bit binary format IEEE 754 value. In more recent implementations,
//! JavaScript also supports integers with arbitrary precision using the `BigInt` type.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-number-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number

use crate::{
    builtins::{string::is_trimmable_whitespace, BuiltIn, JsArgs},
    context::intrinsics::StandardConstructors,
    error::JsNativeError,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        JsObject, ObjectData,
    },
    property::Attribute,
    string::utf16,
    string::Utf16Trim,
    value::{AbstractRelation, IntegerOrInfinity, JsValue},
    Context, JsResult,
};
use boa_profiler::Profiler;
use num_traits::{float::FloatCore, Num};

mod conversions;

pub(crate) use conversions::{f64_to_int32, f64_to_uint32};
use tap::{Conv, Pipe};

#[cfg(test)]
mod tests;

const BUF_SIZE: usize = 2200;

/// `Number` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Number;

impl BuiltIn for Number {
    const NAME: &'static str = "Number";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let parse_int = FunctionBuilder::native(context, Self::parse_int)
            .name("parseInt")
            .length(2)
            .constructor(false)
            .build();

        let parse_float = FunctionBuilder::native(context, Self::parse_float)
            .name("parseFloat")
            .length(1)
            .constructor(false)
            .build();

        context.register_global_property(
            "parseInt",
            parse_int.clone(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );
        context.register_global_property(
            "parseFloat",
            parse_float.clone(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        context.register_global_builtin_function("isFinite", 1, Self::global_is_finite);
        context.register_global_builtin_function("isNaN", 1, Self::global_is_nan);

        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT;

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().number().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .static_property("EPSILON", f64::EPSILON, attribute)
        .static_property("MAX_SAFE_INTEGER", Self::MAX_SAFE_INTEGER, attribute)
        .static_property("MIN_SAFE_INTEGER", Self::MIN_SAFE_INTEGER, attribute)
        .static_property("MAX_VALUE", Self::MAX_VALUE, attribute)
        .static_property("MIN_VALUE", Self::MIN_VALUE, attribute)
        .static_property("NEGATIVE_INFINITY", f64::NEG_INFINITY, attribute)
        .static_property("POSITIVE_INFINITY", f64::INFINITY, attribute)
        .static_property("NaN", f64::NAN, attribute)
        .static_property(
            "parseInt",
            parse_int,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .static_property(
            "parseFloat",
            parse_float,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .static_method(Self::number_is_finite, "isFinite", 1)
        .static_method(Self::number_is_nan, "isNaN", 1)
        .static_method(Self::is_safe_integer, "isSafeInteger", 1)
        .static_method(Self::number_is_integer, "isInteger", 1)
        .method(Self::to_exponential, "toExponential", 1)
        .method(Self::to_fixed, "toFixed", 1)
        .method(Self::to_locale_string, "toLocaleString", 0)
        .method(Self::to_precision, "toPrecision", 1)
        .method(Self::to_string, "toString", 1)
        .method(Self::value_of, "valueOf", 0)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl Number {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// The `Number.MAX_SAFE_INTEGER` constant represents the maximum safe integer in JavaScript (`2^53 - 1`).
    ///
    /// /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-number.max_safe_integer
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER
    pub(crate) const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991_f64;

    /// The `Number.MIN_SAFE_INTEGER` constant represents the minimum safe integer in JavaScript (`-(2^53 - 1)`).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-number.min_safe_integer
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MIN_SAFE_INTEGER
    pub(crate) const MIN_SAFE_INTEGER: f64 = -9_007_199_254_740_991_f64;

    /// The `Number.MAX_VALUE` property represents the maximum numeric value representable in JavaScript.
    ///
    /// The `MAX_VALUE` property has a value of approximately `1.79E+308`, or `2^1024`.
    /// Values larger than `MAX_VALUE` are represented as `Infinity`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-number.max_value
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_VALUE
    pub(crate) const MAX_VALUE: f64 = f64::MAX;

    /// The `Number.MIN_VALUE` property represents the smallest positive numeric value representable in JavaScript.
    ///
    /// The `MIN_VALUE` property is the number closest to `0`, not the most negative number, that JavaScript can represent.
    /// It has a value of approximately `5e-324`. Values smaller than `MIN_VALUE` ("underflow values") are converted to `0`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-number.min_value
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MIN_VALUE
    pub(crate) const MIN_VALUE: f64 = f64::MIN_POSITIVE;

    /// `Number( value )`
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let data = match args.get(0) {
            Some(value) => value.to_numeric_number(context)?,
            None => 0.0,
        };
        if new_target.is_undefined() {
            return Ok(JsValue::new(data));
        }
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::number, context)?;
        let this = JsObject::from_proto_and_data(prototype, ObjectData::number(data));
        Ok(this.into())
    }

    /// This function returns a `JsResult` of the number `Value`.
    ///
    /// If the `Value` is a `Number` primitive of `Number` object the number is returned.
    /// Otherwise an `TypeError` is thrown.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-thisnumbervalue
    fn this_number_value(value: &JsValue) -> JsResult<f64> {
        value
            .as_number()
            .or_else(|| value.as_object().and_then(|obj| obj.borrow().as_number()))
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a number")
                    .into()
            })
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
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let x be ? thisNumberValue(this value).
        let this_num = Self::this_number_value(this)?;
        let precision = match args.get(0) {
            None | Some(JsValue::Undefined) => None,
            // 2. Let f be ? ToIntegerOrInfinity(fractionDigits).
            Some(n) => Some(n.to_integer_or_infinity(context)?),
        };
        // 4. If x is not finite, return ! Number::toString(x).
        if !this_num.is_finite() {
            return Ok(JsValue::new(Self::to_native_string(this_num)));
        }
        // Get rid of the '-' sign for -0.0
        let this_num = if this_num == 0. { 0. } else { this_num };
        let this_str_num = match precision {
            None => f64_to_exponential(this_num),
            Some(IntegerOrInfinity::Integer(precision)) if (0..=100).contains(&precision) =>
            // 5. If f < 0 or f > 100, throw a RangeError exception.
            {
                f64_to_exponential_with_precision(this_num, precision as usize)
            }
            _ => {
                return Err(JsNativeError::range()
                    .with_message("toExponential() argument must be between 0 and 100")
                    .into())
            }
        };
        Ok(JsValue::new(this_str_num))
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
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let this_num be ? thisNumberValue(this value).
        let this_num = Self::this_number_value(this)?;

        // 2. Let f be ? ToIntegerOrInfinity(fractionDigits).
        // 3. Assert: If fractionDigits is undefined, then f is 0.
        let precision = args.get_or_undefined(0).to_integer_or_infinity(context)?;

        // 4, 5. If f < 0 or f > 100, throw a RangeError exception.
        let precision = precision
            .as_integer()
            .filter(|i| (0..=100).contains(i))
            .ok_or_else(|| {
                JsNativeError::range()
                    .with_message("toFixed() digits argument must be between 0 and 100")
            })? as usize;

        // 6. If x is not finite, return ! Number::toString(x).
        if !this_num.is_finite() {
            Ok(JsValue::new(Self::to_native_string(this_num)))
        // 10. If x ≥ 10^21, then let m be ! ToString(𝔽(x)).
        } else if this_num >= 1.0e21 {
            Ok(JsValue::new(f64_to_exponential(this_num)))
        } else {
            // Get rid of the '-' sign for -0.0 because of 9. If x < 0, then set s to "-".
            let this_num = if this_num == 0_f64 { 0_f64 } else { this_num };
            let this_fixed_num = format!("{this_num:.precision$}");
            Ok(JsValue::new(this_fixed_num))
        }
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
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        let this_num = Self::this_number_value(this)?;
        let this_str_num = this_num.to_string();
        Ok(JsValue::new(this_str_num))
    }

    /// `flt_str_to_exp` - used in `to_precision`
    ///
    /// This function traverses a string representing a number,
    /// returning the floored log10 of this number.
    fn flt_str_to_exp(flt: &str) -> i32 {
        let mut non_zero_encountered = false;
        let mut dot_encountered = false;
        for (i, c) in flt.chars().enumerate() {
            if c == '.' {
                if non_zero_encountered {
                    return (i as i32) - 1;
                }
                dot_encountered = true;
            } else if c != '0' {
                if dot_encountered {
                    return 1 - (i as i32);
                }
                non_zero_encountered = true;
            }
        }
        (flt.len() as i32) - 1
    }

    /// `round_to_precision` - used in `to_precision`
    ///
    /// This procedure has two roles:
    /// - If there are enough or more than enough digits in the
    ///   string to show the required precision, the number
    ///   represented by these digits is rounded using string
    ///   manipulation.
    /// - Else, zeroes are appended to the string.
    /// - Additionally, sometimes the exponent was wrongly computed and
    ///   while up-rounding we find that we need an extra digit. When this
    ///   happens, we return true so that the calling context can adjust
    ///   the exponent. The string is kept at an exact length of `precision`.
    ///
    /// When this procedure returns, `digits` is exactly `precision` long.
    fn round_to_precision(digits: &mut String, precision: usize) -> bool {
        if digits.len() > precision {
            let to_round = digits.split_off(precision);
            let mut digit = digits
                .pop()
                .expect("already checked that length is bigger than precision")
                as u8;
            if let Some(first) = to_round.chars().next() {
                if first > '4' {
                    digit += 1;
                }
            }

            if digit as char == ':' {
                // ':' is '9' + 1
                // need to propagate the increment backward
                let mut replacement = String::from("0");
                let mut propagated = false;
                for c in digits.chars().rev() {
                    let d = match (c, propagated) {
                        ('0'..='8', false) => (c as u8 + 1) as char,
                        (_, false) => '0',
                        (_, true) => c,
                    };
                    replacement.push(d);
                    if d != '0' {
                        propagated = true;
                    }
                }
                digits.clear();
                let replacement = if propagated {
                    replacement.as_str()
                } else {
                    digits.push('1');
                    &replacement.as_str()[1..]
                };
                for c in replacement.chars().rev() {
                    digits.push(c);
                }
                !propagated
            } else {
                digits.push(digit as char);
                false
            }
        } else {
            digits.push_str(&"0".repeat(precision - digits.len()));
            false
        }
    }

    /// `Number.prototype.toPrecision( [precision] )`
    ///
    /// The `toPrecision()` method returns a string representing the Number object to the specified precision.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-number.prototype.toprecision
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toPrecision
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_precision(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let precision = args.get_or_undefined(0);

        // 1 & 6
        let mut this_num = Self::this_number_value(this)?;
        // 2
        if precision.is_undefined() {
            return Self::to_string(this, &[], context);
        }

        // 3
        let precision = precision.to_integer_or_infinity(context)?;

        // 4
        if !this_num.is_finite() {
            return Self::to_string(this, &[], context);
        }

        let precision = match precision {
            IntegerOrInfinity::Integer(x) if (1..=100).contains(&x) => x as usize,
            _ => {
                // 5
                return Err(JsNativeError::range()
                    .with_message("precision must be an integer at least 1 and no greater than 100")
                    .into());
            }
        };
        let precision_i32 = precision as i32;

        // 7
        let mut prefix = String::new(); // spec: 's'
        let mut suffix: String; // spec: 'm'
        let mut exponent: i32; // spec: 'e'

        // 8
        if this_num < 0.0 {
            prefix.push('-');
            this_num = -this_num;
        }

        // 9
        if this_num == 0.0 {
            suffix = "0".repeat(precision);
            exponent = 0;
        // 10
        } else {
            // Due to f64 limitations, this part differs a bit from the spec,
            // but has the same effect. It manipulates the string constructed
            // by `format`: digits with an optional dot between two of them.
            suffix = format!("{this_num:.100}");

            // a: getting an exponent
            exponent = Self::flt_str_to_exp(&suffix);
            // b: getting relevant digits only
            if exponent < 0 {
                suffix = suffix.split_off((1 - exponent) as usize);
            } else if let Some(n) = suffix.find('.') {
                suffix.remove(n);
            }
            // impl: having exactly `precision` digits in `suffix`
            if Self::round_to_precision(&mut suffix, precision) {
                exponent += 1;
            }

            // c: switching to scientific notation
            let great_exp = exponent >= precision_i32;
            if exponent < -6 || great_exp {
                // ii
                if precision > 1 {
                    suffix.insert(1, '.');
                }
                // vi
                suffix.push('e');
                // iii
                if great_exp {
                    suffix.push('+');
                }
                // iv, v
                suffix.push_str(&exponent.to_string());

                return Ok(JsValue::new(prefix + &suffix));
            }
        }

        // 11
        let e_inc = exponent + 1;
        if e_inc == precision_i32 {
            return Ok(JsValue::new(prefix + &suffix));
        }

        // 12
        if exponent >= 0 {
            suffix.insert(e_inc as usize, '.');
        // 13
        } else {
            prefix.push('0');
            prefix.push('.');
            prefix.push_str(&"0".repeat(-e_inc as usize));
        }

        // 14
        Ok(JsValue::new(prefix + &suffix))
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
            value = -value;
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
                fraction *= f64::from(radix);
                delta *= f64::from(radix);
                // Write digit.
                let digit = fraction as u32;
                frac_buf[fraction_cursor] = std::char::from_digit(digit, u32::from(radix))
                    .expect("radix already checked")
                    as u8;
                fraction_cursor += 1;
                // Calculate remainder.
                fraction -= f64::from(digit);
                // Round to even.
                if fraction + delta > 1.0
                    && (fraction > 0.5 || (fraction - 0.5).abs() < f64::EPSILON && digit & 1 != 0)
                {
                    loop {
                        // We need to back trace already written digits in case of carry-over.
                        fraction_cursor -= 1;
                        if fraction_cursor == 0 {
                            //              CHECK_EQ('.', buffer[fraction_cursor]);
                            // Carry over to the integer part.
                            integer += 1.;
                        } else {
                            let c: u8 = frac_buf[fraction_cursor];
                            // Reconstruct digit.
                            let digit_0 = (c as char)
                                .to_digit(10)
                                .expect("character was not a valid digit");
                            if digit_0 + 1 >= u32::from(radix) {
                                continue;
                            }
                            frac_buf[fraction_cursor] =
                                std::char::from_digit(digit_0 + 1, u32::from(radix))
                                    .expect("digit was not a valid number in the given radix")
                                    as u8;
                            fraction_cursor += 1;
                        }
                        break;
                    }
                    break;
                }
                if fraction < delta {
                    break;
                }
            }
        }

        // Compute integer digits. Fill unrepresented digits with zero.
        let mut int_iter = int_buf.iter_mut().enumerate().rev();
        while FloatCore::integer_decode(integer / f64::from(radix)).1 > 0 {
            integer /= f64::from(radix);
            *int_iter.next().expect("integer buffer exhausted").1 = b'0';
        }

        loop {
            let remainder = integer % f64::from(radix);
            *int_iter.next().expect("integer buffer exhausted").1 =
                std::char::from_digit(remainder as u32, u32::from(radix))
                    .expect("remainder not a digit in the given number") as u8;
            integer = (integer - remainder) / f64::from(radix);
            if integer <= 0f64 {
                break;
            }
        }
        // Add sign and terminate string.
        if negative {
            *int_iter.next().expect("integer buffer exhausted").1 = b'-';
        }
        assert!(fraction_cursor < BUF_SIZE);

        let integer_cursor = int_iter.next().expect("integer buffer exhausted").0 + 1;
        let fraction_cursor = fraction_cursor + BUF_SIZE / 2;
        String::from_utf8_lossy(&buffer[integer_cursor..fraction_cursor]).into()
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_native_string(x: f64) -> String {
        let mut buffer = ryu_js::Buffer::new();
        buffer.format(x).to_string()
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
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let x be ? thisNumberValue(this value).
        let x = Self::this_number_value(this)?;

        let radix = args.get_or_undefined(0);
        let radix_number = if radix.is_undefined() {
            // 2. If radix is undefined, let radixNumber be 10.
            10
        } else {
            // 3. Else, let radixMV be ? ToIntegerOrInfinity(radix).
            radix
                .to_integer_or_infinity(context)?
                .as_integer()
                // 4. If radixNumber < 2 or radixNumber > 36, throw a RangeError exception.
                .filter(|i| (2..=36).contains(i))
                .ok_or_else(|| {
                    JsNativeError::range()
                        .with_message("radix must be an integer at least 2 and no greater than 36")
                })?
        } as u8;

        // 5. If radixNumber = 10, return ! ToString(x).
        if radix_number == 10 {
            return Ok(JsValue::new(Self::to_native_string(x)));
        }

        if x == -0. {
            return Ok(JsValue::new("0"));
        } else if x.is_nan() {
            return Ok(JsValue::new("NaN"));
        } else if x.is_infinite() && x.is_sign_positive() {
            return Ok(JsValue::new("Infinity"));
        } else if x.is_infinite() && x.is_sign_negative() {
            return Ok(JsValue::new("-Infinity"));
        }

        // This is a Optimization from the v8 source code to print values that can fit in a single character
        // Since the actual num_to_string allocates a 2200 bytes buffer for actual conversion
        // I am not sure if this part is effective as the v8 equivalent https://chromium.googlesource.com/v8/v8/+/refs/heads/master/src/builtins/number.tq#53
        // // Fast case where the result is a one character string.
        // if x.is_sign_positive() && x.fract() == 0.0 && x < radix_number as f64 {
        //     return Ok(std::char::from_digit(x as u32, radix_number as u32).unwrap().to_string().into())
        // }

        // 6. Return the String representation of this Number value using the radix specified by radixNumber.
        Ok(JsValue::new(Self::to_native_string_radix(x, radix_number)))
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
    pub(crate) fn value_of(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::new(Self::this_number_value(this)?))
    }

    /// Builtin javascript 'parseInt(str, radix)' function.
    ///
    /// Parses the given string as an integer using the given radix as a base.
    ///
    /// An argument of type Number (i.e. Integer or Rational) is also accepted in place of string.
    ///
    /// The radix must be an integer in the range [2, 36] inclusive.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parseint-string-radix
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseInt
    pub(crate) fn parse_int(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let (Some(val), radix) = (args.get(0), args.get_or_undefined(1)) {
            // 1. Let inputString be ? ToString(string).
            let input_string = val.to_string(context)?;

            // 2. Let S be ! TrimString(inputString, start).
            let mut var_s = input_string.trim_start();

            // 3. Let sign be 1.
            // 4. If S is not empty and the first code unit of S is the code unit 0x002D (HYPHEN-MINUS),
            //    set sign to -1.
            let sign = if !var_s.is_empty() && var_s.starts_with(utf16!("-")) {
                -1
            } else {
                1
            };

            // 5. If S is not empty and the first code unit of S is the code unit 0x002B (PLUS SIGN) or
            //    the code unit 0x002D (HYPHEN-MINUS), remove the first code unit from S.
            if !var_s.is_empty()
                && (var_s.starts_with(utf16!("+")) || var_s.starts_with(utf16!("-")))
            {
                var_s = &var_s[1..];
            }

            // 6. Let R be ℝ(? ToInt32(radix)).
            let mut var_r = radix.to_i32(context)?;

            // 7. Let stripPrefix be true.
            let mut strip_prefix = true;

            // 8. If R ≠ 0, then
            #[allow(clippy::if_not_else)]
            if var_r != 0 {
                //     a. If R < 2 or R > 36, return NaN.
                if !(2..=36).contains(&var_r) {
                    return Ok(JsValue::nan());
                }

                //     b. If R ≠ 16, set stripPrefix to false.
                if var_r != 16 {
                    strip_prefix = false;
                }
            } else {
                // 9. Else,
                //     a. Set R to 10.
                var_r = 10;
            }

            // 10. If stripPrefix is true, then
            //     a. If the length of S is at least 2 and the first two code units of S are either "0x" or "0X", then
            //         i. Remove the first two code units from S.
            //         ii. Set R to 16.
            if strip_prefix
                && var_s.len() >= 2
                && (var_s.starts_with(utf16!("0x")) || var_s.starts_with(utf16!("0X")))
            {
                var_s = &var_s[2..];

                var_r = 16;
            }

            // 11. If S contains a code unit that is not a radix-R digit, let end be the index within S of the
            //     first such code unit; otherwise, let end be the length of S.
            let end = char::decode_utf16(var_s.iter().copied())
                .position(|code| !code.map(|c| c.is_digit(var_r as u32)).unwrap_or_default())
                .unwrap_or(var_s.len());

            // 12. Let Z be the substring of S from 0 to end.
            let var_z = String::from_utf16_lossy(&var_s[..end]);

            // 13. If Z is empty, return NaN.
            if var_z.is_empty() {
                return Ok(JsValue::nan());
            }

            // 14. Let mathInt be the integer value that is represented by Z in radix-R notation, using the
            //     letters A-Z and a-z for digits with values 10 through 35. (However, if R is 10 and Z contains
            //     more than 20 significant digits, every significant digit after the 20th may be replaced by a
            //     0 digit, at the option of the implementation; and if R is not 2, 4, 8, 10, 16, or 32, then
            //     mathInt may be an implementation-approximated value representing the integer value that is
            //     represented by Z in radix-R notation.)
            let math_int = u64::from_str_radix(&var_z, var_r as u32).map_or_else(
                |_| f64::from_str_radix(&var_z, var_r as u32).expect("invalid_float_conversion"),
                |i| i as f64,
            );

            // 15. If mathInt = 0, then
            //     a. If sign = -1, return -0𝔽.
            //     b. Return +0𝔽.
            if math_int == 0_f64 {
                if sign == -1 {
                    return Ok(JsValue::new(-0_f64));
                }

                return Ok(JsValue::new(0_f64));
            }

            // 16. Return 𝔽(sign × mathInt).
            Ok(JsValue::new(f64::from(sign) * math_int))
        } else {
            // Not enough arguments to parseInt.
            Ok(JsValue::nan())
        }
    }

    /// Builtin javascript 'parseFloat(str)' function.
    ///
    /// Parses the given string as a floating point value.
    ///
    /// An argument of type Number (i.e. Integer or Rational) is also accepted in place of string.
    ///
    /// To improve performance an Integer type Number is returned in place of a Rational if the given
    /// string can be parsed and stored as an Integer.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parsefloat-string
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseFloat
    pub(crate) fn parse_float(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let Some(val) = args.get(0) {
            // TODO: parse float with optimal utf16 algorithm
            let input_string = val.to_string(context)?.to_std_string_escaped();
            let s = input_string.trim_start_matches(is_trimmable_whitespace);
            let s_prefix_lower = s.chars().take(4).collect::<String>().to_ascii_lowercase();

            // TODO: write our own lexer to match syntax StrDecimalLiteral
            if s.starts_with("Infinity") || s.starts_with("+Infinity") {
                Ok(JsValue::new(f64::INFINITY))
            } else if s.starts_with("-Infinity") {
                Ok(JsValue::new(f64::NEG_INFINITY))
            } else if s_prefix_lower.starts_with("inf")
                || s_prefix_lower.starts_with("+inf")
                || s_prefix_lower.starts_with("-inf")
            {
                // Prevent fast_float from parsing "inf", "+inf" as Infinity and "-inf" as -Infinity
                Ok(JsValue::nan())
            } else {
                Ok(fast_float::parse_partial::<f64, _>(s).map_or_else(
                    |_| JsValue::nan(),
                    |(f, len)| {
                        if len > 0 {
                            JsValue::new(f)
                        } else {
                            JsValue::nan()
                        }
                    },
                ))
            }
        } else {
            // Not enough arguments to parseFloat.
            Ok(JsValue::nan())
        }
    }

    /// Builtin javascript 'isFinite(number)' function.
    ///
    /// Converts the argument to a number, throwing a type error if the conversion is invalid.
    ///
    /// If the number is `NaN`, `+∞`, or `-∞`, `false` is returned.
    ///
    /// Otherwise true is returned.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isfinite-number
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/isFinite
    pub(crate) fn global_is_finite(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let Some(value) = args.get(0) {
            let number = value.to_number(context)?;
            Ok(number.is_finite().into())
        } else {
            Ok(false.into())
        }
    }

    /// Builtin javascript 'isNaN(number)' function.
    ///
    /// Converts the argument to a number, throwing a type error if the conversion is invalid.
    ///
    /// If the number is `NaN`, `true` is returned.
    ///
    /// Otherwise false is returned.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isnan-number
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/isNaN
    pub(crate) fn global_is_nan(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let Some(value) = args.get(0) {
            let number = value.to_number(context)?;
            Ok(number.is_nan().into())
        } else {
            Ok(true.into())
        }
    }

    /// `Number.isFinite( number )`
    ///
    /// Checks if the argument is a number, returning false if it isn't.
    ///
    /// If the number is `NaN`, `+∞`, or `-∞`, `false` is returned.
    ///
    /// Otherwise true is returned.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-number.isfinite
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isFinite
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn number_is_finite(
        _: &JsValue,
        args: &[JsValue],
        _ctx: &mut Context,
    ) -> JsResult<JsValue> {
        Ok(JsValue::new(if let Some(val) = args.get(0) {
            match val {
                JsValue::Integer(_) => true,
                JsValue::Rational(number) => number.is_finite(),
                _ => false,
            }
        } else {
            false
        }))
    }

    /// `Number.isInteger( number )`
    ///
    /// Checks if the argument is an integer.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-number.isinteger
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isInteger
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn number_is_integer(
        _: &JsValue,
        args: &[JsValue],
        _ctx: &mut Context,
    ) -> JsResult<JsValue> {
        Ok(args.get(0).map_or(false, Self::is_integer).into())
    }

    /// `Number.isNaN( number )`
    ///
    /// Checks if the argument is a number, returning false if it isn't.
    ///
    /// If the number is `NaN`, `true` is returned.
    ///
    /// Otherwise false is returned.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isnan-number
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isNaN
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn number_is_nan(
        _: &JsValue,
        args: &[JsValue],
        _ctx: &mut Context,
    ) -> JsResult<JsValue> {
        Ok(JsValue::new(
            if let Some(&JsValue::Rational(number)) = args.get(0) {
                number.is_nan()
            } else {
                false
            },
        ))
    }

    /// `Number.isSafeInteger( number )`
    ///
    /// Checks if the argument is an integer, returning false if it isn't.
    ///
    /// If `abs(number) ≤ MAX_SAFE_INTEGER`, `true` is returned.
    ///
    /// Otherwise false is returned.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isnan-number
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isNaN
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn is_safe_integer(
        _: &JsValue,
        args: &[JsValue],
        _ctx: &mut Context,
    ) -> JsResult<JsValue> {
        Ok(JsValue::new(match args.get(0) {
            Some(JsValue::Integer(_)) => true,
            Some(JsValue::Rational(number)) if Self::is_float_integer(*number) => {
                number.abs() <= Self::MAX_SAFE_INTEGER
            }
            _ => false,
        }))
    }

    /// Checks if the argument is a finite integer number value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isinteger
    #[inline]
    pub(crate) fn is_integer(val: &JsValue) -> bool {
        match val {
            JsValue::Integer(_) => true,
            JsValue::Rational(number) => Self::is_float_integer(*number),
            _ => false,
        }
    }

    /// Checks if the float argument is an integer.
    #[inline]
    #[allow(clippy::float_cmp)]
    pub(crate) fn is_float_integer(number: f64) -> bool {
        number.is_finite() && number.abs().floor() == number.abs()
    }

    /// The abstract operation `Number::equal` takes arguments
    /// x (a Number) and y (a Number). It performs the following steps when called:
    ///
    /// <https://tc39.es/ecma262/#sec-numeric-types-number-equal>
    #[inline]
    #[allow(clippy::float_cmp)]
    pub(crate) fn equal(x: f64, y: f64) -> bool {
        x == y
    }

    /// The abstract operation `Number::sameValue` takes arguments
    /// x (a Number) and y (a Number). It performs the following steps when called:
    ///
    /// <https://tc39.es/ecma262/#sec-numeric-types-number-sameValue>
    #[allow(clippy::float_cmp)]
    pub(crate) fn same_value(a: f64, b: f64) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }
        a == b && a.signum() == b.signum()
    }

    /// The abstract operation `Number::sameValueZero` takes arguments
    /// x (a Number) and y (a Number). It performs the following steps when called:
    ///
    /// <https://tc39.es/ecma262/#sec-numeric-types-number-sameValueZero>
    #[inline]
    #[allow(clippy::float_cmp)]
    pub(crate) fn same_value_zero(x: f64, y: f64) -> bool {
        if x.is_nan() && y.is_nan() {
            return true;
        }

        x == y
    }

    #[inline]
    #[allow(clippy::float_cmp)]
    pub(crate) fn less_than(x: f64, y: f64) -> AbstractRelation {
        if x.is_nan() || y.is_nan() {
            return AbstractRelation::Undefined;
        }
        if x == y || x == 0.0 && y == -0.0 || x == -0.0 && y == 0.0 {
            return AbstractRelation::False;
        }
        if x.is_infinite() && x.is_sign_positive() {
            return AbstractRelation::False;
        }
        if y.is_infinite() && y.is_sign_positive() {
            return AbstractRelation::True;
        }
        if x.is_infinite() && x.is_sign_negative() {
            return AbstractRelation::True;
        }
        if y.is_infinite() && y.is_sign_negative() {
            return AbstractRelation::False;
        }
        (x < y).into()
    }

    #[inline]
    pub(crate) fn not(x: f64) -> i32 {
        let x = f64_to_int32(x);
        !x
    }
}

/// Helper function that formats a float as a ES6-style exponential number string.
fn f64_to_exponential(n: f64) -> String {
    match n.abs() {
        x if x >= 1.0 || x == 0.0 => format!("{n:e}").replace('e', "e+"),
        _ => format!("{n:e}"),
    }
}

/// Helper function that formats a float as a ES6-style exponential number string with a given precision.
// We can't use the same approach as in `f64_to_exponential`
// because in cases like (0.999).toExponential(0) the result will be 1e0.
// Instead we get the index of 'e', and if the next character is not '-' we insert the plus sign
fn f64_to_exponential_with_precision(n: f64, prec: usize) -> String {
    let mut res = format!("{n:.prec$e}");
    let idx = res.find('e').expect("'e' not found in exponential string");
    if res.as_bytes()[idx + 1] != b'-' {
        res.insert(idx + 1, '+');
    }
    res
}
