//! The ECMAScript `Temporal` stage 3 built-in implementation.
//!
//! More information:
//!
//! [spec]: https://tc39.es/proposal-temporal/

mod calendar;
mod date_equations;
mod duration;
mod fields;
mod instant;
mod now;
mod options;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod time_zone;
mod zoned_date_time;

#[cfg(feature = "experimental")]
#[cfg(test)]
mod tests;

pub(crate) use fields::TemporalFields;

use self::options::{
    get_temporal_rounding_increment, get_temporal_unit, TemporalUnit, TemporalUnitGroup,
};
pub use self::{
    calendar::*, duration::*, instant::*, now::*, plain_date::*, plain_date_time::*,
    plain_month_day::*, plain_time::*, plain_year_month::*, time_zone::*, zoned_date_time::*,
};

use crate::{
    builtins::{
        iterable::IteratorRecord,
        options::{get_option, RoundingMode, UnsignedRoundingMode},
        BuiltInBuilder, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::Intrinsics,
    js_string,
    property::Attribute,
    realm::Realm,
    string::{common::StaticJsStrings, utf16},
    value::Type,
    Context, JsBigInt, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

// Relavant numeric constants
/// Nanoseconds per day constant: 8.64e+13
pub(crate) const NS_PER_DAY: i64 = 86_400_000_000_000;
/// Microseconds per day constant: 8.64e+10
pub(crate) const MIS_PER_DAY: i64 = 8_640_000_000;
/// Milliseconds per day constant: 8.64e+7
pub(crate) const MS_PER_DAY: i32 = 24 * 60 * 60 * 1000;

pub(crate) fn ns_max_instant() -> JsBigInt {
    JsBigInt::from(i128::from(NS_PER_DAY) * 100_000_000_i128)
}

pub(crate) fn ns_min_instant() -> JsBigInt {
    JsBigInt::from(i128::from(NS_PER_DAY) * -100_000_000_i128)
}

// An enum representing common fields across `Temporal` objects.
#[allow(unused)]
pub(crate) enum DateTimeValues {
    Year,
    Month,
    MonthCode,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

/// The [`Temporal`][spec] builtin object.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Temporal;

impl BuiltInObject for Temporal {
    const NAME: JsString = StaticJsStrings::TEMPORAL;
}

impl IntrinsicObject for Temporal {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("Now"),
                realm.intrinsics().objects().now(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("Calendar"),
                realm.intrinsics().constructors().calendar().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("Duration"),
                realm.intrinsics().constructors().duration().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("Instant"),
                realm.intrinsics().constructors().instant().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("PlainDate"),
                realm.intrinsics().constructors().plain_date().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("PlainDateTime"),
                realm
                    .intrinsics()
                    .constructors()
                    .plain_date_time()
                    .constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("PlainMonthDay"),
                realm
                    .intrinsics()
                    .constructors()
                    .plain_month_day()
                    .constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("PlainTime"),
                realm.intrinsics().constructors().plain_time().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("PlainYearMonth"),
                realm
                    .intrinsics()
                    .constructors()
                    .plain_year_month()
                    .constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("TimeZone"),
                realm.intrinsics().constructors().time_zone().constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                js_string!("ZonedDateTime"),
                realm
                    .intrinsics()
                    .constructors()
                    .zoned_date_time()
                    .constructor(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().temporal()
    }
}

// -- Temporal Abstract Operations --

/// Abstract operation `ToZeroPaddedDecimalString ( n, minLength )`
///
/// The abstract operation `ToZeroPaddedDecimalString` takes arguments `n` (a non-negative integer)
/// and `minLength` (a non-negative integer) and returns a String.
fn to_zero_padded_decimal_string(n: u64, min_length: usize) -> String {
    format!("{n:0min_length$}")
}

/// Abstract Operation 13.1 [`IteratorToListOfType`][proposal]
///
/// [proposal]: https://tc39.es/proposal-temporal/#sec-iteratortolistoftype
pub(crate) fn iterator_to_list_of_types(
    iterator: &mut IteratorRecord,
    element_types: &[Type],
    context: &mut Context<'_>,
) -> JsResult<Vec<JsValue>> {
    // 1. Let values be a new empty List.
    let mut values = Vec::new();

    // 2. Let next be true.
    // 3. Repeat, while next is not false,
    // a. Set next to ? IteratorStep(iteratorRecord).
    // b. If next is not false, then
    while iterator.step(context)? {
        // i. Let nextValue be ? IteratorValue(next).
        let next_value = iterator.value(context)?;
        // ii. If Type(nextValue) is not an element of elementTypes, then
        if element_types.contains(&next_value.get_type()) {
            // 1. Let completion be ThrowCompletion(a newly created TypeError object).
            let completion = JsNativeError::typ()
                .with_message("IteratorNext is not within allowed type values.");

            // NOTE: The below should return as we are forcing a ThrowCompletion.
            // 2. Return ? IteratorClose(iteratorRecord, completion).
            let _never = iterator.close(Err(completion.into()), context)?;
        }
        // iii. Append nextValue to the end of the List values.
        values.push(next_value);
    }

    // 4. Return values.
    Ok(values)
}

/// 13.2 `ISODateToEpochDays ( year, month, date )`
// Note: implemented on IsoDateRecord.

// Abstract Operation 13.3 `EpochDaysToEpochMs`
pub(crate) fn epoch_days_to_epoch_ms(day: i32, time: i32) -> f64 {
    f64::from(day).mul_add(f64::from(MS_PER_DAY), f64::from(time))
}

// 13.4 Date Equations
// implemented in temporal/date_equations.rs

// Abstract Operation 13.5 `GetOptionsObject ( options )`
// Implemented in builtin/options.rs

// 13.6 `GetOption ( options, property, type, values, default )`
// Implemented in builtin/options.rs

/// 13.7 `ToTemporalOverflow (options)`
// Now implemented in temporal/options.rs

/// 13.10 `ToTemporalRoundingMode ( normalizedOptions, fallback )`
// Now implemented in builtin/options.rs

// 13.11 `NegateTemporalRoundingMode ( roundingMode )`
// Now implemented in builtin/options.rs

// 13.16 `ToTemporalRoundingIncrement ( normalizedOptions )`
// Now implemented in temporal/options.rs

/// 13.17 `ValidateTemporalRoundingIncrement ( increment, dividend, inclusive )`
#[inline]
pub(crate) fn validate_temporal_rounding_increment(
    increment: f64,
    dividend: f64,
    inclusive: bool,
) -> JsResult<()> {
    // 1. If inclusive is true, then
    let maximum = if inclusive {
        // a. Let maximum be dividend.
        dividend
    // 2. Else,
    } else {
        // a. Assert: dividend > 1.
        assert!(dividend > 1.0);
        // b. Let maximum be dividend - 1.
        dividend - 1.0
    };

    // 3. If increment > maximum, throw a RangeError exception.
    if increment > maximum {
        return Err(JsNativeError::range()
            .with_message("increment is exceeds the range of the allowed maximum.")
            .into());
    }
    // 4. If dividend modulo increment ≠ 0, then
    if dividend % increment != 0.0 {
        // a. Throw a RangeError exception.
        return Err(JsNativeError::range()
            .with_message("Temporal rounding increment is not valid.")
            .into());
    }
    // 5. Return unused.
    Ok(())
}

/// 13.21 `ToRelativeTemporalObject ( options )`
pub(crate) fn to_relative_temporal_object(
    _options: &JsObject,
    _context: &mut Context<'_>,
) -> JsResult<JsValue> {
    Err(JsNativeError::range()
        .with_message("not yet implemented.")
        .into())
}

// 13.22 `LargerOfTwoTemporalUnits ( u1, u2 )`
// use core::cmp::max

// 13.23 `MaximumTemporalDurationRoundingIncrement ( unit )`
// Implemented on TemporalUnit in temporal/options.rs

// 13.26 `GetUnsignedRoundingMode ( roundingMode, isNegative )`
// Implemented on RoundingMode in builtins/options.rs

/// 13.27 `ApplyUnsignedRoundingMode ( x, r1, r2, unsignedRoundingMode )`
#[inline]
fn apply_unsigned_rounding_mode(
    x: f64,
    r1: f64,
    r2: f64,
    unsigned_rounding_mode: UnsignedRoundingMode,
) -> f64 {
    // 1. If x is equal to r1, return r1.
    if (x - r1).abs() == 0.0 {
        return r1;
    };
    // 2. Assert: r1 < x < r2.
    assert!(r1 < x && x < r2);
    // 3. Assert: unsignedRoundingMode is not undefined.

    // 4. If unsignedRoundingMode is zero, return r1.
    if unsigned_rounding_mode == UnsignedRoundingMode::Zero {
        return r1;
    };
    // 5. If unsignedRoundingMode is infinity, return r2.
    if unsigned_rounding_mode == UnsignedRoundingMode::Infinity {
        return r2;
    };

    // 6. Let d1 be x – r1.
    let d1 = x - r1;
    // 7. Let d2 be r2 – x.
    let d2 = r2 - x;
    // 8. If d1 < d2, return r1.
    if d1 < d2 {
        return r1;
    }
    // 9. If d2 < d1, return r2.
    if d2 < d1 {
        return r2;
    }
    // 10. Assert: d1 is equal to d2.
    assert!((d1 - d2).abs() == 0.0);

    // 11. If unsignedRoundingMode is half-zero, return r1.
    if unsigned_rounding_mode == UnsignedRoundingMode::HalfZero {
        return r1;
    };
    // 12. If unsignedRoundingMode is half-infinity, return r2.
    if unsigned_rounding_mode == UnsignedRoundingMode::HalfInfinity {
        return r2;
    };
    // 13. Assert: unsignedRoundingMode is half-even.
    assert!(unsigned_rounding_mode == UnsignedRoundingMode::HalfEven);
    // 14. Let cardinality be (r1 / (r2 – r1)) modulo 2.
    let cardinality = (r1 / (r2 - r1)) % 2.0;
    // 15. If cardinality is 0, return r1.
    if cardinality == 0.0 {
        return r1;
    }
    // 16. Return r2.
    r2
}

/// 13.28 `RoundNumberToIncrement ( x, increment, roundingMode )`
pub(crate) fn round_number_to_increment(
    x: f64,
    increment: f64,
    rounding_mode: RoundingMode,
) -> f64 {
    // 1. Let quotient be x / increment.
    let mut quotient = x / increment;

    // 2. If quotient < 0, then
    let is_negative = if quotient < 0_f64 {
        // a. Let isNegative be true.
        // b. Set quotient to -quotient.
        quotient = -quotient;
        true
    // 3. Else,
    } else {
        // a. Let isNegative be false.
        false
    };

    // 4. Let unsignedRoundingMode be GetUnsignedRoundingMode(roundingMode, isNegative).
    let unsigned_rounding_mode = rounding_mode.get_unsigned_round_mode(is_negative);
    // 5. Let r1 be the largest integer such that r1 ≤ quotient.
    let r1 = quotient.ceil();
    // 6. Let r2 be the smallest integer such that r2 > quotient.
    let r2 = quotient.floor();
    // 7. Let rounded be ApplyUnsignedRoundingMode(quotient, r1, r2, unsignedRoundingMode).
    let mut rounded = apply_unsigned_rounding_mode(quotient, r1, r2, unsigned_rounding_mode);
    // 8. If isNegative is true, set rounded to -rounded.
    if is_negative {
        rounded = -rounded;
    };
    // 9. Return rounded × increment.
    rounded * increment
}

/// 13.29 `RoundNumberToIncrementAsIfPositive ( x, increment, roundingMode )`
#[inline]
pub(crate) fn round_to_increment_as_if_positive(
    ns: &JsBigInt,
    increment: i64,
    rounding_mode: RoundingMode,
) -> JsResult<JsBigInt> {
    // 1. Let quotient be x / increment.
    let q = ns.to_f64() / increment as f64;
    // 2. Let unsignedRoundingMode be GetUnsignedRoundingMode(roundingMode, false).
    let unsigned_rounding_mode = rounding_mode.get_unsigned_round_mode(false);
    // 3. Let r1 be the largest integer such that r1 ≤ quotient.
    let r1 = q.trunc();
    // 4. Let r2 be the smallest integer such that r2 > quotient.
    let r2 = q.trunc() + 1.0;
    // 5. Let rounded be ApplyUnsignedRoundingMode(quotient, r1, r2, unsignedRoundingMode).
    let rounded = apply_unsigned_rounding_mode(q, r1, r2, unsigned_rounding_mode);

    // 6. Return rounded × increment.
    let rounded = JsBigInt::try_from(rounded)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;

    Ok(JsBigInt::mul(&rounded, &JsBigInt::from(increment)))
}

/// 13.43 `ToPositiveIntegerWithTruncation ( argument )`
#[inline]
pub(crate) fn to_positive_integer_with_trunc(
    value: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<i32> {
    // 1. Let integer be ? ToIntegerWithTruncation(argument).
    let int = to_integer_with_truncation(value, context)?;
    // 2. If integer ≤ 0, throw a RangeError exception.
    if int <= 0 {
        return Err(JsNativeError::range()
            .with_message("value is not a positive integer")
            .into());
    }
    // 3. Return integer.
    Ok(int)
}

/// 13.44 `ToIntegerWithTruncation ( argument )`
#[inline]
pub(crate) fn to_integer_with_truncation(
    value: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<i32> {
    // 1. Let number be ? ToNumber(argument).
    let number = value.to_number(context)?;
    // 2. If number is NaN, +∞𝔽 or -∞𝔽, throw a RangeError exception.
    if number.is_nan() || number.is_infinite() {
        return Err(JsNativeError::range()
            .with_message("truncation target must be an integer.")
            .into());
    }
    // 3. Return truncate(ℝ(number)).
    Ok(number.trunc() as i32)
}

/// Abstract operation 13.45 `ToIntegerIfIntegral( argument )`
#[inline]
pub(crate) fn to_integer_if_integral(arg: &JsValue, context: &mut Context<'_>) -> JsResult<i32> {
    // 1. Let number be ? ToNumber(argument).
    // 2. If IsIntegralNumber(number) is false, throw a RangeError exception.
    // 3. Return ℝ(number).
    if !arg.is_integer() {
        return Err(JsNativeError::range()
            .with_message("value to convert is not an integral number.")
            .into());
    }

    arg.to_i32(context)
}

// 13.46 `PrepareTemporalFields ( fields, fieldNames, requiredFields [ , duplicateBehaviour ] )`
// See fields.rs

// NOTE: op -> true == until | false == since
/// 13.47 `GetDifferenceSettings ( operation, options, unitGroup, disallowedUnits, fallbackSmallestUnit, smallestLargestDefaultUnit )`
#[inline]
pub(crate) fn get_diff_settings(
    op: bool,
    options: &JsObject,
    unit_group: TemporalUnitGroup,
    disallowed_units: &[TemporalUnit],
    fallback_smallest_unit: TemporalUnit,
    smallest_largest_default_unit: TemporalUnit,
    context: &mut Context<'_>,
) -> JsResult<(TemporalUnit, TemporalUnit, RoundingMode, f64)> {
    // 1. NOTE: The following steps read options and perform independent validation in alphabetical order (ToTemporalRoundingIncrement reads "roundingIncrement" and ToTemporalRoundingMode reads "roundingMode").
    // 2. Let largestUnit be ? GetTemporalUnit(options, "largestUnit", unitGroup, "auto").
    let largest_unit = get_temporal_unit(
        options,
        utf16!("largestUnit"),
        unit_group,
        false,
        Some(TemporalUnit::Auto),
        None,
        context,
    )?;

    let Some(mut largest_unit) = largest_unit else {
        return Err(JsNativeError::range()
            .with_message("largestUnit cannot be undefined in this context.")
            .into());
    };

    // 3. If disallowedUnits contains largestUnit, throw a RangeError exception.
    if disallowed_units.contains(&largest_unit) {
        return Err(JsNativeError::range()
            .with_message("largestUnit is not an allowed unit.")
            .into());
    }

    // 4. Let roundingIncrement be ? ToTemporalRoundingIncrement(options).
    let rounding_increment = get_temporal_rounding_increment(options, context)?;
    // 5. Let roundingMode be ? ToTemporalRoundingMode(options, "trunc").
    let mut rounding_mode =
        get_option::<RoundingMode>(options, utf16!("roundingMode"), false, context)?
            .unwrap_or(RoundingMode::Trunc);

    // 6. If operation is since, then
    if !op {
        // a. Set roundingMode to ! NegateTemporalRoundingMode(roundingMode).
        rounding_mode = rounding_mode.negate();
    }

    // 7. Let smallestUnit be ? GetTemporalUnit(options, "smallestUnit", unitGroup, fallbackSmallestUnit).
    let smallest_unit = get_temporal_unit(
        options,
        utf16!("smallestUnit"),
        unit_group,
        false,
        Some(fallback_smallest_unit),
        None,
        context,
    )?;

    let Some(smallest_unit) = smallest_unit else {
        return Err(JsNativeError::range()
            .with_message("smallestUnit cannot be undefined in this context.")
            .into());
    };

    // 8. If disallowedUnits contains smallestUnit, throw a RangeError exception.
    if disallowed_units.contains(&smallest_unit) {
        return Err(JsNativeError::range()
            .with_message("smallestUnit is not an allowed unit.")
            .into());
    }

    // 9. Let defaultLargestUnit be ! LargerOfTwoTemporalUnits(smallestLargestDefaultUnit, smallestUnit).
    let default_largest_unit = core::cmp::max(smallest_largest_default_unit, smallest_unit);

    // 10. If largestUnit is "auto", set largestUnit to defaultLargestUnit.
    if largest_unit == TemporalUnit::Auto {
        largest_unit = default_largest_unit;
    }

    // 11. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
    if largest_unit != core::cmp::max(largest_unit, smallest_unit) {
        return Err(JsNativeError::range()
            .with_message("largestUnit must be larger than smallestUnit")
            .into());
    }

    // 12. Let maximum be ! MaximumTemporalDurationRoundingIncrement(smallestUnit).
    let maximum = smallest_unit.to_maximum_rounding_increment();

    // 13. If maximum is not undefined, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
    if let Some(max) = maximum {
        validate_temporal_rounding_increment(rounding_increment, f64::from(max), false)?;
    }

    // 14. Return the Record { [[SmallestUnit]]: smallestUnit, [[LargestUnit]]: largestUnit, [[RoundingMode]]: roundingMode, [[RoundingIncrement]]: roundingIncrement, }.
    Ok((
        smallest_unit,
        largest_unit,
        rounding_mode,
        rounding_increment,
    ))
}

// NOTE: used for MergeFields methods. Potentially can be omitted in favor of `TemporalFields`.
/// 14.6 `CopyDataProperties ( target, source, excludedKeys [ , excludedValues ] )`
pub(crate) fn copy_data_properties(
    target: &JsObject,
    source: &JsValue,
    excluded_keys: &Vec<JsString>,
    excluded_values: Option<&Vec<JsValue>>,
    context: &mut Context<'_>,
) -> JsResult<()> {
    // 1. If source is undefined or null, return unused.
    if source.is_null_or_undefined() {
        return Ok(());
    }

    // 2. Let from be ! ToObject(source).
    let from = source.to_object(context)?;

    // 3. Let keys be ? from.[[OwnPropertyKeys]]().
    let keys = from.__own_property_keys__(context)?;

    // 4. For each element nextKey of keys, do
    for next_key in keys {
        // a. Let excluded be false.
        let mut excluded = false;
        // b. For each element e of excludedItemsexcludedKeys, do
        for e in excluded_keys {
            // i. If SameValue(e, nextKey) is true, then
            if next_key.to_string() == e.to_std_string_escaped() {
                // 1. Set excluded to true.
                excluded = true;
            }
        }

        // c. If excluded is false, then
        if !excluded {
            // i. Let desc be ? from.[[GetOwnProperty]](nextKey).
            let desc = from.__get_own_property__(&next_key, context)?;
            // ii. If desc is not undefined and desc.[[Enumerable]] is true, then
            match desc {
                Some(d)
                    if d.enumerable()
                        .expect("enumerable field must be set per spec.") =>
                {
                    // 1. Let propValue be ? Get(from, nextKey).
                    let prop_value = from.get(next_key.clone(), context)?;
                    // 2. If excludedValues is present, then
                    if let Some(values) = excluded_values {
                        // a. For each element e of excludedValues, do
                        for e in values {
                            // i. If SameValue(e, propValue) is true, then
                            if JsValue::same_value(e, &prop_value) {
                                // i. Set excluded to true.
                                excluded = true;
                            }
                        }
                    }

                    // 3. PerformIf excluded is false, perform ! CreateDataPropertyOrThrow(target, nextKey, propValue).
                    if !excluded {
                        target.create_data_property_or_throw(next_key, prop_value, context)?;
                    }
                }
                _ => {}
            }
        }
    }

    // 5. Return unused.
    Ok(())
}

// Note: Deviates from Proposal spec -> proto appears to be always null across the specification.
/// 14.7 `SnapshotOwnProperties ( source, proto [ , excludedKeys [ , excludedValues ] ] )`
fn snapshot_own_properties(
    source: &JsObject,
    excluded_keys: Option<Vec<JsString>>,
    excluded_values: Option<Vec<JsValue>>,
    context: &mut Context<'_>,
) -> JsResult<JsObject> {
    // 1. Let copy be OrdinaryObjectCreate(proto).
    let copy = JsObject::with_null_proto();
    // 2. If excludedKeys is not present, set excludedKeys to « ».
    let keys = excluded_keys.unwrap_or_default();
    // 3. If excludedValues is not present, set excludedValues to « ».
    let values = excluded_values.unwrap_or_default();
    // 4. Perform ? CopyDataProperties(copy, source, excludedKeys, excludedValues).
    copy_data_properties(&copy, &source.clone().into(), &keys, Some(&values), context)?;
    // 5. Return copy.
    Ok(copy)
}
