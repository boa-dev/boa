use boa_macros::utf16;
use fixed_decimal::{FixedDecimal, FloatPrecision, RoundingIncrement as BaseMultiple};

use crate::{
    builtins::{
        intl::{
            number_format::{Extrema, RoundingType, TrailingZeroDisplay},
            options::{default_number_option, get_number_option},
        },
        options::{get_option, RoundingMode},
    },
    Context, JsNativeError, JsObject, JsResult,
};

use super::{DigitFormatOptions, Notation, RoundingPriority};

/// The increment of a rounding operation.
///
/// This differs from [`fixed_decimal::RoundingIncrement`] because ECMA402 accepts
/// several more increments than `fixed_decimal`, but all increments can be decomposed
/// into the target multiple and the magnitude offset.
///
/// For example, rounding the number `0.02456` to the increment 200 at position
/// -3 is equivalent to rounding the same number to the increment 2 at position -1, and adding
/// trailing zeroes.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct RoundingIncrement {
    multiple: BaseMultiple,
    // INVARIANT: can only be 0, 1, 2, or 3
    magnitude_offset: u8,
}

impl RoundingIncrement {
    /// Creates a `RoundingIncrement` from the numeric value of the increment.
    fn from_u16(increment: u16) -> Option<Self> {
        let mut offset = 0u8;
        let multiple = loop {
            let rem = increment % 10u16.checked_pow(u32::from(offset + 1))?;

            if rem != 0 {
                break increment / 10u16.pow(u32::from(offset));
            }

            offset += 1;
        };

        if offset > 3 {
            return None;
        }

        let multiple = match multiple {
            1 => BaseMultiple::MultiplesOf1,
            2 => BaseMultiple::MultiplesOf2,
            5 => BaseMultiple::MultiplesOf5,
            25 => BaseMultiple::MultiplesOf25,
            _ => return None,
        };

        Some(RoundingIncrement {
            multiple,
            magnitude_offset: offset,
        })
    }

    /// Gets the numeric value of this `RoundingIncrement`.
    pub(crate) fn to_u16(self) -> u16 {
        u16::from(self.magnitude_offset + 1)
            * match self.multiple {
                BaseMultiple::MultiplesOf1 => 1,
                BaseMultiple::MultiplesOf2 => 2,
                BaseMultiple::MultiplesOf5 => 5,
                BaseMultiple::MultiplesOf25 => 25,
                _ => {
                    debug_assert!(false, "base multiples can only be 1, 2, 5, or 25");
                    1
                }
            }
    }

    /// Gets the magnitude offset that needs to be added to the rounding position
    /// for this rounding increment.
    fn magnitude_offset(self) -> i16 {
        i16::from(self.magnitude_offset)
    }
}

/// Abstract operation [`SetNumberFormatDigitOptions ( intlObj, options, mnfdDefault, mxfdDefault, notation )`][spec].
///
/// Gets the digit format options of the number formatter from the options object and the requested notation.
///
/// [spec]: https://tc39.es/ecma402/#sec-setnfdigitoptions
pub(crate) fn get_digit_format_options(
    options: &JsObject,
    min_float_digits_default: u8,
    mut max_float_digits_default: u8,
    notation: Notation,
    context: &mut Context,
) -> JsResult<DigitFormatOptions> {
    // 1. Let mnid be ? GetNumberOption(options, "minimumIntegerDigits,", 1, 21, 1).
    let minimum_integer_digits =
        get_number_option(options, utf16!("minimumIntegerDigits"), 1, 21, context)?.unwrap_or(1);
    // 2. Let mnfd be ? Get(options, "minimumFractionDigits").
    let min_float_digits = options.get(utf16!("minimumFractionDigits"), context)?;
    // 3. Let mxfd be ? Get(options, "maximumFractionDigits").
    let max_float_digits = options.get(utf16!("maximumFractionDigits"), context)?;
    // 4. Let mnsd be ? Get(options, "minimumSignificantDigits").
    let min_sig_digits = options.get(utf16!("minimumSignificantDigits"), context)?;
    // 5. Let mxsd be ? Get(options, "maximumSignificantDigits").
    let max_sig_digits = options.get(utf16!("maximumSignificantDigits"), context)?;

    // 7. Let roundingPriority be ? GetOption(options, "roundingPriority", string, « "auto", "morePrecision", "lessPrecision" », "auto").
    let mut rounding_priority =
        get_option(options, utf16!("roundingPriority"), context)?.unwrap_or_default();

    // 8. Let roundingIncrement be ? GetNumberOption(options, "roundingIncrement", 1, 5000, 1).
    // 9. If roundingIncrement is not in « 1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500, 1000, 2000, 2500, 5000 », throw a RangeError exception.
    let rounding_increment =
        get_number_option(options, utf16!("roundingIncrement"), 1, 5000, context)?.unwrap_or(1);

    let rounding_increment = RoundingIncrement::from_u16(rounding_increment).ok_or_else(|| {
        JsNativeError::range().with_message("invalid value for option `roundingIncrement`")
    })?;

    // 10. Let roundingMode be ? GetOption(options, "roundingMode", string, « "ceil", "floor", "expand", "trunc", "halfCeil", "halfFloor", "halfExpand", "halfTrunc", "halfEven" », "halfExpand").
    let rounding_mode = get_option(options, utf16!("roundingMode"), context)?.unwrap_or_default();

    // 11. Let trailingZeroDisplay be ? GetOption(options, "trailingZeroDisplay", string, « "auto", "stripIfInteger" », "auto").
    let trailing_zero_display =
        get_option(options, utf16!("trailingZeroDisplay"), context)?.unwrap_or_default();

    // 12. NOTE: All fields required by SetNumberFormatDigitOptions have now been read from options. The remainder of this AO interprets the options and may throw exceptions.

    // 13. If roundingIncrement is not 1, set mxfdDefault to mnfdDefault.
    if rounding_increment.to_u16() != 1 {
        max_float_digits_default = min_float_digits_default;
    }

    // 17. If mnsd is not undefined or mxsd is not undefined, then
    //     a. Let hasSd be true.
    // 18. Else,
    //     a. Let hasSd be false.
    let has_sig_limits = !min_sig_digits.is_undefined() || !max_sig_digits.is_undefined();

    // 19. If mnfd is not undefined or mxfd is not undefined, then
    //     a. Let hasFd be true.
    // 20. Else,
    //     a. Let hasFd be false.
    let has_float_limits = !min_float_digits.is_undefined() || !max_float_digits.is_undefined();

    // 21. Let needSd be true.
    // 22. Let needFd be true.
    let (need_sig_limits, need_frac_limits) = if rounding_priority == RoundingPriority::Auto {
        // 23. If roundingPriority is "auto", then
        //     a. Set needSd to hasSd.
        //     b. If needSd is true, or hasFd is false and notation is "compact", then
        //         i. Set needFd to false.
        (
            has_sig_limits,
            !has_sig_limits && (has_float_limits || notation != Notation::Compact),
        )
    } else {
        (true, true)
    };

    // 24. If needSd is true, then
    let sig_digits = if need_sig_limits {
        // a. If hasSd is true, then
        let extrema = if has_sig_limits {
            // i. Set intlObj.[[MinimumSignificantDigits]] to ? DefaultNumberOption(mnsd, 1, 21, 1).
            let min_sig = default_number_option(&min_sig_digits, 1, 21, context)?.unwrap_or(1);
            // ii. Set intlObj.[[MaximumSignificantDigits]] to ? DefaultNumberOption(mxsd, intlObj.[[MinimumSignificantDigits]], 21, 21).
            let max_sig =
                default_number_option(&max_sig_digits, min_sig, 21, context)?.unwrap_or(21);

            Extrema {
                minimum: min_sig,
                maximum: max_sig,
            }
        } else {
            // b. Else,
            Extrema {
                // i. Set intlObj.[[MinimumSignificantDigits]] to 1.
                minimum: 1,
                // ii. Set intlObj.[[MaximumSignificantDigits]] to 21.
                maximum: 21,
            }
        };
        assert!(extrema.minimum <= extrema.maximum);
        Some(extrema)
    } else {
        None
    };

    // 25. If needFd is true, then
    let fractional_digits = if need_frac_limits {
        //     a. If hasFd is true, then
        let extrema = if has_float_limits {
            // i. Set mnfd to ? DefaultNumberOption(mnfd, 0, 100, undefined).
            let min_float_digits = default_number_option(&min_float_digits, 0, 100, context)?;
            // ii. Set mxfd to ? DefaultNumberOption(mxfd, 0, 100, undefined).
            let max_float_digits = default_number_option(&max_float_digits, 0, 100, context)?;

            let (min_float_digits, max_float_digits) = match (min_float_digits, max_float_digits) {
                (Some(min_float_digits), Some(max_float_digits)) => {
                    // v. Else if mnfd is greater than mxfd, throw a RangeError exception.
                    if min_float_digits > max_float_digits {
                        return Err(JsNativeError::range().with_message(
                            "`minimumFractionDigits` cannot be bigger than `maximumFractionDigits`",
                        ).into());
                    }
                    (min_float_digits, max_float_digits)
                }
                // iv. Else if mxfd is undefined, set mxfd to max(mxfdDefault, mnfd).
                (Some(min_float_digits), None) => (
                    min_float_digits,
                    u8::max(max_float_digits_default, min_float_digits),
                ),
                // iii. If mnfd is undefined, set mnfd to min(mnfdDefault, mxfd).
                (None, Some(max_float_digits)) => (
                    u8::min(min_float_digits_default, max_float_digits),
                    max_float_digits,
                ),
                (None, None) => {
                    unreachable!("`has_fd` can only be true if `mnfd` or `mxfd` is not undefined")
                }
            };

            Extrema {
                // vi. Set intlObj.[[MinimumFractionDigits]] to mnfd.
                minimum: min_float_digits,
                // vii. Set intlObj.[[MaximumFractionDigits]] to mxfd.
                maximum: max_float_digits,
            }
        } else {
            // b. Else,
            Extrema {
                //    i. Set intlObj.[[MinimumFractionDigits]] to mnfdDefault.
                minimum: min_float_digits_default,
                //    ii. Set intlObj.[[MaximumFractionDigits]] to mxfdDefault.
                maximum: max_float_digits_default,
            }
        };
        assert!(extrema.minimum <= extrema.maximum);
        Some(extrema)
    } else {
        None
    };

    let rounding_type = match (sig_digits, fractional_digits) {
        // 26. If needSd is false and needFd is false, then
        (None, None) => {
            // f. Set intlObj.[[ComputedRoundingPriority]] to "morePrecision".
            rounding_priority = RoundingPriority::MorePrecision;
            // e. Set intlObj.[[RoundingType]] to morePrecision.
            RoundingType::MorePrecision {
                significant_digits: Extrema {
                    // c. Set intlObj.[[MinimumSignificantDigits]] to 1.
                    minimum: 1,
                    // d. Set intlObj.[[MaximumSignificantDigits]] to 2.
                    maximum: 2,
                },
                fraction_digits: Extrema {
                    // a. Set intlObj.[[MinimumFractionDigits]] to 0.
                    minimum: 0,
                    // b. Set intlObj.[[MaximumFractionDigits]] to 0.
                    maximum: 0,
                },
            }
        }
        (Some(significant_digits), Some(fraction_digits)) => match rounding_priority {
            RoundingPriority::MorePrecision => RoundingType::MorePrecision {
                significant_digits,
                fraction_digits,
            },
            RoundingPriority::LessPrecision => RoundingType::LessPrecision {
                significant_digits,
                fraction_digits,
            },
            RoundingPriority::Auto => {
                unreachable!("Cannot have both roundings when the priority is `Auto`")
            }
        },
        (Some(sig), None) => RoundingType::SignificantDigits(sig),
        (None, Some(frac)) => RoundingType::FractionDigits(frac),
    };

    if rounding_increment.to_u16() != 1 {
        let RoundingType::FractionDigits(range) = rounding_type else {
            return Err(JsNativeError::typ()
                .with_message("option `roundingIncrement` invalid for the current set of options")
                .into());
        };

        if range.minimum != range.maximum {
            return Err(JsNativeError::range()
                .with_message("option `roundingIncrement` invalid for the current set of options")
                .into());
        }
    }

    Ok(DigitFormatOptions {
        // 6. Set intlObj.[[MinimumIntegerDigits]] to mnid.
        minimum_integer_digits,
        // 14. Set intlObj.[[RoundingIncrement]] to roundingIncrement.
        rounding_increment,
        // 15. Set intlObj.[[RoundingMode]] to roundingMode.
        rounding_mode,
        // 16. Set intlObj.[[TrailingZeroDisplay]] to trailingZeroDisplay.
        trailing_zero_display,
        rounding_type,
        rounding_priority,
    })
}

/// Abstract operation [`FormatNumericToString ( intlObject, x )`][spec].
///
/// Converts the input number to a `FixedDecimal` with the specified digit format options.
///
/// [spec]: https://tc39.es/ecma402/#sec-formatnumberstring
pub(crate) fn f64_to_formatted_fixed_decimal(
    number: f64,
    options: &DigitFormatOptions,
) -> FixedDecimal {
    fn round(number: &mut FixedDecimal, position: i16, mode: RoundingMode, multiple: BaseMultiple) {
        match mode {
            RoundingMode::Ceil => number.ceil_to_increment(position, multiple),
            RoundingMode::Floor => number.floor_to_increment(position, multiple),
            RoundingMode::Expand => number.expand_to_increment(position, multiple),
            RoundingMode::Trunc => number.trunc_to_increment(position, multiple),
            RoundingMode::HalfCeil => number.half_ceil_to_increment(position, multiple),
            RoundingMode::HalfFloor => number.half_floor_to_increment(position, multiple),
            RoundingMode::HalfExpand => number.half_expand_to_increment(position, multiple),
            RoundingMode::HalfTrunc => number.half_trunc_to_increment(position, multiple),
            RoundingMode::HalfEven => number.half_even_to_increment(position, multiple),
        }
    }

    // <https://tc39.es/ecma402/#sec-torawprecision>
    fn to_raw_precision(
        number: &mut FixedDecimal,
        min_precision: u8,
        max_precision: u8,
        rounding_mode: RoundingMode,
    ) -> i16 {
        let msb = *number.magnitude_range().end();
        let min_msb = msb - i16::from(min_precision) + 1;
        let max_msb = msb - i16::from(max_precision) + 1;
        number.pad_end(min_msb);
        round(number, max_msb, rounding_mode, BaseMultiple::MultiplesOf1);
        max_msb
    }

    // <https://tc39.es/ecma402/#sec-torawfixed>
    fn to_raw_fixed(
        number: &mut FixedDecimal,
        min_fraction: u8,
        max_fraction: u8,
        rounding_increment: RoundingIncrement,
        rounding_mode: RoundingMode,
    ) -> i16 {
        #[cfg(debug_assertions)]
        if rounding_increment.to_u16() != 1 {
            assert_eq!(min_fraction, max_fraction);
        }

        number.pad_end(-i16::from(min_fraction));
        round(
            number,
            rounding_increment.magnitude_offset() - i16::from(max_fraction),
            rounding_mode,
            rounding_increment.multiple,
        );
        -i16::from(max_fraction)
    }

    // 1. If x is negative-zero, then
    //     a. Let isNegative be true.
    //     b. Set x to 0.
    // 2. Else,
    //     a. Assert: x is a mathematical value.
    //     b. If x < 0, let isNegative be true; else let isNegative be false.
    //     c. If isNegative is true, then
    //         i. Set x to -x.
    // We can skip these steps, because `FixedDecimal` already provides support for
    // negative zeroes.
    let mut number = FixedDecimal::try_from_f64(number, FloatPrecision::Floating)
        .expect("`number` must be finite");

    // 3. Let unsignedRoundingMode be GetUnsignedRoundingMode(intlObject.[[RoundingMode]], isNegative).
    // Skipping because `FixedDecimal`'s API already provides methods equivalent to `RoundingMode`s.

    match options.rounding_type {
        // 4. If intlObject.[[RoundingType]] is significantDigits, then
        RoundingType::SignificantDigits(Extrema { minimum, maximum }) => {
            // a. Let result be ToRawPrecision(x, intlObject.[[MinimumSignificantDigits]], intlObject.[[MaximumSignificantDigits]], unsignedRoundingMode).
            to_raw_precision(&mut number, minimum, maximum, options.rounding_mode);
        }
        // 5. Else if intlObject.[[RoundingType]] is fractionDigits, then
        RoundingType::FractionDigits(Extrema { minimum, maximum }) => {
            // a. Let result be ToRawFixed(x, intlObject.[[MinimumFractionDigits]], intlObject.[[MaximumFractionDigits]], intlObject.[[RoundingIncrement]], unsignedRoundingMode).
            to_raw_fixed(
                &mut number,
                minimum,
                maximum,
                options.rounding_increment,
                options.rounding_mode,
            );
        }
        // 6. Else,
        RoundingType::MorePrecision {
            significant_digits,
            fraction_digits,
        }
        | RoundingType::LessPrecision {
            significant_digits,
            fraction_digits,
        } => {
            let prefer_more_precision =
                matches!(options.rounding_type, RoundingType::MorePrecision { .. });
            // a. Let sResult be ToRawPrecision(x, intlObject.[[MinimumSignificantDigits]], intlObject.[[MaximumSignificantDigits]], unsignedRoundingMode).
            let mut fixed = number.clone();
            let s_magnitude = to_raw_precision(
                &mut number,
                significant_digits.maximum,
                significant_digits.minimum,
                options.rounding_mode,
            );
            // b. Let fResult be ToRawFixed(x, intlObject.[[MinimumFractionDigits]], intlObject.[[MaximumFractionDigits]], intlObject.[[RoundingIncrement]], unsignedRoundingMode).
            let f_magnitude = to_raw_fixed(
                &mut fixed,
                fraction_digits.maximum,
                fraction_digits.minimum,
                options.rounding_increment,
                options.rounding_mode,
            );

            // c. If intlObject.[[RoundingType]] is morePrecision, then
            //     i. If sResult.[[RoundingMagnitude]] ≤ fResult.[[RoundingMagnitude]], then
            //         1. Let result be sResult.
            //     ii. Else,
            //         1. Let result be fResult.
            // d. Else,
            //     i. Assert: intlObject.[[RoundingType]] is lessPrecision.
            //     ii. If sResult.[[RoundingMagnitude]] ≤ fResult.[[RoundingMagnitude]], then
            //         1. Let result be fResult.
            //     iii. Else,
            //         1. Let result be sResult.
            if (prefer_more_precision && f_magnitude < s_magnitude)
                || (!prefer_more_precision && s_magnitude <= f_magnitude)
            {
                number = fixed;
            }
        }
    }

    // 7. Set x to result.[[RoundedNumber]].
    // 8. Let string be result.[[FormattedString]].
    // 9. If intlObject.[[TrailingZeroDisplay]] is "stripIfInteger" and x modulo 1 = 0, then
    if options.trailing_zero_display == TrailingZeroDisplay::StripIfInteger
        && number.nonzero_magnitude_end() >= 0
    {
        // a. Let i be StringIndexOf(string, ".", 0).
        // b. If i ≠ -1, set string to the substring of string from 0 to i.
        number.trim_end();
    }

    // 10. Let int be result.[[IntegerDigitsCount]].
    // 11. Let minInteger be intlObject.[[MinimumIntegerDigits]].
    // 12. If int < minInteger, then
    //     a. Let forwardZeros be the String consisting of minInteger - int occurrences of the code unit 0x0030 (DIGIT ZERO).
    //     b. Set string to the string-concatenation of forwardZeros and string.
    number.pad_start(i16::from(options.minimum_integer_digits));

    // 13. If isNegative is true, then
    //     a. If x is 0, set x to negative-zero. Otherwise, set x to -x.
    // As mentioned above, `FixedDecimal` has support for this.

    // 14. Return the Record { [[RoundedNumber]]: x, [[FormattedString]]: string }.
    number
}

#[cfg(test)]
mod tests {
    use crate::builtins::intl::number_format::RoundingIncrement;
    use fixed_decimal::RoundingIncrement::*;

    #[test]
    fn u16_to_rounding_increment_sunny_day() {
        #[rustfmt::skip]
        const VALID_CASES: [(u16, RoundingIncrement); 15] = [
            // Singles
            (1, RoundingIncrement { multiple: MultiplesOf1, magnitude_offset: 0 }),
            (2, RoundingIncrement { multiple: MultiplesOf2, magnitude_offset: 0 }),
            (5, RoundingIncrement { multiple: MultiplesOf5, magnitude_offset: 0 }),
            // Tens
            (10, RoundingIncrement { multiple: MultiplesOf1, magnitude_offset: 1 }),
            (20, RoundingIncrement { multiple: MultiplesOf2, magnitude_offset: 1 }),
            (25, RoundingIncrement { multiple: MultiplesOf25, magnitude_offset: 0 }),
            (50, RoundingIncrement { multiple: MultiplesOf5, magnitude_offset: 1 }),
            // Hundreds
            (100, RoundingIncrement { multiple: MultiplesOf1, magnitude_offset: 2 }),
            (200, RoundingIncrement { multiple: MultiplesOf2, magnitude_offset: 2 }),
            (250, RoundingIncrement { multiple: MultiplesOf25, magnitude_offset: 1 }),
            (500, RoundingIncrement { multiple: MultiplesOf5, magnitude_offset: 2 }),
            // Thousands
            (1000, RoundingIncrement { multiple: MultiplesOf1, magnitude_offset: 3 }),
            (2000, RoundingIncrement { multiple: MultiplesOf2, magnitude_offset: 3 }),
            (2500, RoundingIncrement { multiple: MultiplesOf25, magnitude_offset: 2 }),
            (5000, RoundingIncrement { multiple: MultiplesOf5, magnitude_offset: 3 }),
        ];

        for (num, increment) in VALID_CASES {
            assert_eq!(RoundingIncrement::from_u16(num), Some(increment));
        }
    }

    #[test]
    fn u16_to_rounding_increment_rainy_day() {
        const INVALID_CASES: [u16; 9] = [0, 4, 6, 24, 10000, 65535, 7373, 140, 1500];

        for num in INVALID_CASES {
            assert!(RoundingIncrement::from_u16(num).is_none());
        }
    }
}
