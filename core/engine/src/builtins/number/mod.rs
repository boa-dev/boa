//! Boa's implementation of ECMAScript's global `Number` object.
//!
//! The `Number` ECMAScript object is a wrapper object allowing you to work with numerical values.
//! A `Number` object is created using the `Number()` constructor. A primitive type object number is created using the `Number()` **function**.
//!
//! The ECMAScript `Number` type is double-precision 64-bit binary format IEEE 754 value. In more recent implementations,
//! ECMAScript also supports integers with arbitrary precision using the `BigInt` type.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-number-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number

use crate::{
    Context, JsArgs, JsResult, JsString,
    builtins::BuiltInObject,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::{AbstractRelation, IntegerOrInfinity, JsValue},
};
use cow_utils::CowUtils;
use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::{One, ToPrimitive};

mod globals;
pub(crate) use globals::{IsFinite, IsNaN, ParseFloat, ParseInt};

mod conversions;

use super::{BuiltInBuilder, BuiltInConstructor, IntrinsicObject};
use crate::value::JsVariant;
pub(crate) use conversions::{f64_to_int32, f64_to_uint32};

#[cfg(test)]
mod tests;

/// `Number` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Number;

impl IntrinsicObject for Number {
    fn init(realm: &Realm) {
        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT;

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(js_string!("EPSILON"), f64::EPSILON, attribute)
            .static_property(
                js_string!("MAX_SAFE_INTEGER"),
                Self::MAX_SAFE_INTEGER,
                attribute,
            )
            .static_property(
                js_string!("MIN_SAFE_INTEGER"),
                Self::MIN_SAFE_INTEGER,
                attribute,
            )
            .static_property(js_string!("MAX_VALUE"), Self::MAX_VALUE, attribute)
            .static_property(js_string!("MIN_VALUE"), Self::MIN_VALUE, attribute)
            .static_property(
                js_string!("NEGATIVE_INFINITY"),
                f64::NEG_INFINITY,
                attribute,
            )
            .static_property(js_string!("POSITIVE_INFINITY"), f64::INFINITY, attribute)
            .static_property(js_string!("NaN"), f64::NAN, attribute)
            .static_property(
                js_string!("parseInt"),
                realm.intrinsics().objects().parse_int(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("parseFloat"),
                realm.intrinsics().objects().parse_float(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_method(Self::number_is_finite, js_string!("isFinite"), 1)
            .static_method(Self::number_is_nan, js_string!("isNaN"), 1)
            .static_method(Self::is_safe_integer, js_string!("isSafeInteger"), 1)
            .static_method(Self::number_is_integer, js_string!("isInteger"), 1)
            .method(Self::to_exponential, js_string!("toExponential"), 1)
            .method(Self::to_fixed, js_string!("toFixed"), 1)
            .method(Self::to_locale_string, js_string!("toLocaleString"), 0)
            .method(Self::to_precision, js_string!("toPrecision"), 1)
            .method(Self::to_string, js_string!("toString"), 1)
            .method(Self::value_of, js_string!("valueOf"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Number {
    const NAME: JsString = StaticJsStrings::NUMBER;
}

impl BuiltInConstructor for Number {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 6;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 14;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::number;

    /// `Number( value )`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let data = match args.first() {
            Some(value) => value.to_numeric_number(context)?,
            None => 0.0,
        };
        if new_target.is_undefined() {
            return Ok(JsValue::new(data));
        }
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::number, context)?;
        let this =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        Ok(this.into())
    }
}

impl Number {
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
    pub(crate) const MIN_VALUE: f64 = 5e-324;

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
            .or_else(|| {
                value
                    .as_object()
                    .and_then(|obj| obj.downcast_ref::<f64>().as_deref().copied())
            })
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
        let precision = match args.first() {
            None => None,
            Some(x) if x.is_undefined() => None,
            // 2. Let f be ? ToIntegerOrInfinity(fractionDigits).
            Some(n) => Some(n.to_integer_or_infinity(context)?),
        };
        // 4. If x is not finite, return ! Number::toString(x).
        if !this_num.is_finite() {
            return Ok(JsValue::new(JsString::from(this_num)));
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
                    .into());
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
            })? as u8;

        let mut buffer = ryu_js::Buffer::new();
        let string = buffer.format_to_fixed(this_num, precision);

        Ok(js_string!(string).into())
    }

    /// `Number.prototype.toLocaleString( [locales [, options]] )`
    ///
    /// The `toLocaleString()` method returns a string with a language-sensitive representation of this number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sup-number.prototype.tolocalestring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toLocaleString
    #[allow(clippy::wrong_self_convention)]
    #[allow(
        unused_variables,
        reason = "`args` and `context` are used if the `intl` feature is enabled"
    )]
    pub(crate) fn to_locale_string(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let x be ? ThisNumberValue(this value).
        let x = Self::this_number_value(this)?;

        #[cfg(feature = "intl")]
        {
            use fixed_decimal::{Decimal, FloatPrecision};
            use writeable::Writeable;

            use crate::builtins::intl::NumberFormat;

            if !x.is_finite() {
                return Ok(JsValue::new(js_string!(x)));
            }

            let locales = args.get_or_undefined(0).clone();
            let options = args.get_or_undefined(1).clone();

            // 2. Let numberFormat be ? Construct(%Intl.NumberFormat%, « locales, options »).
            let number_format = NumberFormat::new(&locales, &options, context)?;
            let mut x = Decimal::try_from_f64(x, FloatPrecision::RoundTrip)
                .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

            // 3. Return FormatNumeric(numberFormat, ! ToIntlMathematicalValue(x)).
            Ok(js_string!(number_format.format(&mut x).write_to_string()).into())
        }

        #[cfg(not(feature = "intl"))]
        {
            Ok(JsValue::new(js_string!(x)))
        }
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
            if let Some(first) = to_round.chars().next()
                && first > '4'
            {
                digit += 1;
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

                return Ok(JsValue::new(js_string!(prefix + &suffix)));
            }
        }

        // 11
        let e_inc = exponent + 1;
        if e_inc == precision_i32 {
            return Ok(JsValue::new(js_string!(prefix + &suffix)));
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
        Ok(JsValue::new(js_string!(prefix + &suffix)))
    }

    /// Returns the shortest digits in the given radix whose value rounds back to `value`.
    ///
    /// This is the free-format algorithm from Burger and Dybvig, "Printing Floating-Point
    /// Numbers Quickly and Accurately" (1996). It works on exact integers, so the digits are
    /// correct for every radix, not only for powers of two.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-numeric-types-number-tostring
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_js_string_radix(value: f64, radix: u8) -> JsString {
        assert!((2..=36).contains(&radix));
        assert!(value.is_finite());

        if value == 0.0 {
            return js_string!("0");
        }

        let (digits, point) = Self::shortest_radix_digits(value.abs(), radix);

        // The digits denote 0.d1d2…dk × radix^point.
        let mut result = String::with_capacity(digits.len() + point.unsigned_abs() as usize + 3);
        if value.is_sign_negative() {
            result.push('-');
        }
        let digit = |d: &u8| {
            std::char::from_digit(u32::from(*d), u32::from(radix)).expect("digit is below radix")
        };
        if point <= 0 {
            result.push_str("0.");
            result.extend(std::iter::repeat_n('0', point.unsigned_abs() as usize));
            result.extend(digits.iter().map(digit));
        } else if point as usize >= digits.len() {
            result.extend(digits.iter().map(digit));
            result.extend(std::iter::repeat_n('0', point as usize - digits.len()));
        } else {
            let (int, frac) = digits.split_at(point as usize);
            result.extend(int.iter().map(digit));
            result.push('.');
            result.extend(frac.iter().map(digit));
        }
        js_string!(result)
    }

    /// Generates the shortest digits of a positive finite `value` in `radix`, returning them
    /// with the position `point` of the radix point, so that `value` is 0.d1d2…dk × radix^point.
    fn shortest_radix_digits(value: f64, radix: u8) -> (Vec<u8>, i32) {
        const HIDDEN_BIT: u64 = 1 << 52;
        const MIN_EXPONENT: i32 = -1074;

        let bits = value.to_bits();
        let biased = ((bits >> 52) & 0x7ff) as i32;
        let (f, e) = if biased == 0 {
            (bits & (HIDDEN_BIT - 1), MIN_EXPONENT)
        } else {
            ((bits & (HIDDEN_BIT - 1)) | HIDDEN_BIT, biased - 1075)
        };

        // `value` is f × 2^e. The rounding interval around it is (value - m- / s, value + m+ / s)
        // after scaling, with r / s = value. The gap below is half as wide when f is a power of
        // two, because the exponent changes there. Round-to-nearest-even includes the interval
        // ends when f is even.
        let f = BigUint::from(f);
        let unequal_gaps = f == BigUint::from(HIDDEN_BIT) && e > MIN_EXPONENT;
        let (mut r, mut s, mut m_plus, mut m_minus) = match (e >= 0, unequal_gaps) {
            (true, false) => {
                let be = BigUint::one() << e as usize;
                ((&f * &be) << 1, BigUint::from(2u8), be.clone(), be)
            }
            (true, true) => {
                let be = BigUint::one() << e as usize;
                ((&f * &be) << 2, BigUint::from(4u8), &be << 1, be)
            }
            (false, false) => (
                &f << 1,
                BigUint::one() << (1 - e) as usize,
                BigUint::one(),
                BigUint::one(),
            ),
            (false, true) => (
                &f << 2,
                BigUint::one() << (2 - e) as usize,
                BigUint::from(2u8),
                BigUint::one(),
            ),
        };
        let inclusive = (bits & 1) == 0;
        let radix_big = BigUint::from(radix);

        // Find the smallest `point` such that value + m+ / s fits below radix^point. Start from
        // an estimate (at most one too small) and fix it up.
        let log2_value = f64::from(e) + f.bits() as f64 - 1.0;
        let mut point = (log2_value / f64::from(radix).log2() - 1e-9).ceil() as i32 - 1;
        if point >= 0 {
            s *= radix_big.pow(point.unsigned_abs());
        } else {
            let scale = radix_big.pow(point.unsigned_abs());
            r *= &scale;
            m_plus *= &scale;
            m_minus *= &scale;
        }
        let too_low = |r: &BigUint, m_plus: &BigUint, s: &BigUint| {
            let high = r + m_plus;
            if inclusive { high >= *s } else { high > *s }
        };
        while too_low(&r, &m_plus, &s) {
            s *= &radix_big;
            point += 1;
        }

        let mut digits = Vec::new();
        loop {
            r *= &radix_big;
            m_plus *= &radix_big;
            m_minus *= &radix_big;
            let (digit, rest) = r.div_rem(&s);
            r = rest;
            let digit = digit.to_u8().expect("digit is below radix");

            let low_ok = if inclusive { r <= m_minus } else { r < m_minus };
            let high_ok = too_low(&r, &m_plus, &s);
            match (low_ok, high_ok) {
                (false, false) => digits.push(digit),
                (true, false) => {
                    digits.push(digit);
                    break;
                }
                (false, true) => {
                    digits.push(digit + 1);
                    break;
                }
                (true, true) => {
                    // Both digits round back to `value`, so pick the closer one.
                    digits.push(if &r << 1 < s { digit } else { digit + 1 });
                    break;
                }
            }
        }
        (digits, point)
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
            return Ok(JsValue::new(JsString::from(x)));
        }

        if x == -0. {
            return Ok(JsValue::new(js_string!("0")));
        } else if x.is_nan() {
            return Ok(JsValue::new(js_string!("NaN")));
        } else if x.is_infinite() && x.is_sign_positive() {
            return Ok(JsValue::new(js_string!("Infinity")));
        } else if x.is_infinite() && x.is_sign_negative() {
            return Ok(JsValue::new(js_string!("-Infinity")));
        }

        // This is a Optimization from the v8 source code to print values that can fit in a single character
        // Since the actual num_to_string allocates a 2200 bytes buffer for actual conversion
        // I am not sure if this part is effective as the v8 equivalent https://chromium.googlesource.com/v8/v8/+/refs/heads/master/src/builtins/number.tq#53
        // // Fast case where the result is a one character string.
        // if x.is_sign_positive() && x.fract() == 0.0 && x < radix_number as f64 {
        //     return Ok(std::char::from_digit(x as u32, radix_number as u32).unwrap().to_string().into())
        // }

        // 6. Return the String representation of this Number value using the radix specified by radixNumber.
        Ok(JsValue::new(Self::to_js_string_radix(x, radix_number)))
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
        // 1. If number is not a Number, return false.
        // 2. If number is not finite, return false.
        // 3. Otherwise, return true.
        Ok(JsValue::new(args.first().is_some_and(
            |val| match val.variant() {
                JsVariant::Integer32(_) => true,
                JsVariant::Float64(number) => number.is_finite(),
                _ => false,
            },
        )))
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
        Ok(args.first().is_some_and(Self::is_integer).into())
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
            if let Some(number) = args.first().and_then(JsValue::as_number) {
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
        Ok(JsValue::new(match args.first().map(JsValue::variant) {
            Some(JsVariant::Integer32(_)) => true,
            Some(JsVariant::Float64(number)) if Self::is_float_integer(number) => {
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
    pub(crate) fn is_integer(val: &JsValue) -> bool {
        match val.variant() {
            JsVariant::Integer32(_) => true,
            JsVariant::Float64(number) => Self::is_float_integer(number),
            _ => false,
        }
    }

    /// Checks if the float argument is an integer.
    #[allow(clippy::float_cmp)]
    pub(crate) fn is_float_integer(number: f64) -> bool {
        number.is_finite() && number.trunc() == number
    }

    /// The abstract operation `Number::equal` takes arguments
    /// x (a Number) and y (a Number). It performs the following steps when called:
    ///
    /// <https://tc39.es/ecma262/#sec-numeric-types-number-equal>
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
    #[allow(clippy::float_cmp)]
    pub(crate) fn same_value_zero(x: f64, y: f64) -> bool {
        if x.is_nan() && y.is_nan() {
            return true;
        }

        x == y
    }

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

    pub(crate) fn not(x: f64) -> i32 {
        let x = f64_to_int32(x);
        !x
    }
}

/// Helper function that formats a float as a ES6-style exponential number string.
fn f64_to_exponential(n: f64) -> JsString {
    let s = format!("{n:e}");
    match n.abs() {
        x if x >= 1.0 || x == 0.0 => js_string!(s.cow_replace('e', "e+")),
        _ => js_string!(s),
    }
}

/// Helper function that formats a float as a ES6-style exponential number string with a given precision.
// We can't use the same approach as in `f64_to_exponential`
// because in cases like (0.999).toExponential(0) the result will be 1e0.
// Instead we get the index of 'e', and if the next character is not '-' we insert the plus sign
fn f64_to_exponential_with_precision(n: f64, prec: usize) -> JsString {
    let mut res = format!("{n:.prec$e}");
    let idx = res.find('e').expect("'e' not found in exponential string");
    if res.as_bytes()[idx + 1] != b'-' {
        res.insert(idx + 1, '+');
    }
    js_string!(res)
}
