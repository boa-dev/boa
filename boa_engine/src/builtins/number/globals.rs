use boa_macros::utf16;

use crate::{
    builtins::{string::is_trimmable_whitespace, BuiltInBuilder, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    object::JsObject,
    realm::Realm,
    string::{common::StaticJsStrings, Utf16Trim},
    Context, JsArgs, JsResult, JsString, JsValue,
};

use num_traits::Num;

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
fn is_finite(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    if let Some(value) = args.get(0) {
        let number = value.to_number(context)?;
        Ok(number.is_finite().into())
    } else {
        Ok(false.into())
    }
}

pub(crate) struct IsFinite;

impl IntrinsicObject for IsFinite {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, is_finite)
            .name(Self::NAME)
            .length(1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().is_finite().into()
    }
}

impl BuiltInObject for IsFinite {
    const NAME: JsString = StaticJsStrings::IS_FINITE;
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
pub(crate) fn is_nan(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    if let Some(value) = args.get(0) {
        let number = value.to_number(context)?;
        Ok(number.is_nan().into())
    } else {
        Ok(true.into())
    }
}

pub(crate) struct IsNaN;

impl IntrinsicObject for IsNaN {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, is_nan)
            .name(Self::NAME)
            .length(1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().is_nan().into()
    }
}

impl BuiltInObject for IsNaN {
    const NAME: JsString = StaticJsStrings::IS_NAN;
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
    context: &mut Context<'_>,
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
        if !var_s.is_empty() && (var_s.starts_with(utf16!("+")) || var_s.starts_with(utf16!("-"))) {
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

pub(crate) struct ParseInt;

impl IntrinsicObject for ParseInt {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, parse_int)
            .name(Self::NAME)
            .length(2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().parse_int().into()
    }
}

impl BuiltInObject for ParseInt {
    const NAME: JsString = StaticJsStrings::PARSE_INT;
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
    context: &mut Context<'_>,
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
pub(crate) struct ParseFloat;

impl IntrinsicObject for ParseFloat {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, parse_float)
            .name(Self::NAME)
            .length(1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().parse_float().into()
    }
}

impl BuiltInObject for ParseFloat {
    const NAME: JsString = StaticJsStrings::PARSE_FLOAT;
}
