//! The ECMAScript `Temporal` stage 3 built-in implementation.
//!
//! More information:
//!
//! [spec]: https://tc39.es/proposal-temporal/
#![allow(unreachable_code, dead_code, unused_imports)] // Unimplemented

mod calendar;
mod date_equations;
mod duration;
mod instant;
mod now;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod time_zone;
mod zoned_date_time;

use self::date_equations::mathematical_days_in_year;
pub use self::{
    calendar::*, duration::*, instant::*, now::*, plain_date::*, plain_date_time::*,
    plain_month_day::*, plain_time::*, plain_year_month::*, time_zone::*, zoned_date_time::*,
};
use super::{BuiltInBuilder, BuiltInObject, IntrinsicObject};
use crate::{
    context::intrinsics::{Intrinsics, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, ObjectData, ObjectInitializer},
    property::{Attribute, PropertyKey},
    realm::Realm,
    string::utf16,
    value::IntegerOrInfinity,
    Context, JsBigInt, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
    NativeFunction,
};
use boa_ast::temporal::{self, OffsetSign, UtcOffset};
use boa_profiler::Profiler;

// Relavant numeric constants
pub(crate) const NS_MAX_INSTANT: i128 = 8_640_000_000_000_000_000_000;
pub(crate) const NS_MIN_INSTANT: i128 = -8_640_000_000_000_000_000_000;
pub(crate) const NS_PER_DAY: i64 = 86_400_000_000_000;
pub(crate) const MICRO_PER_DAY: i64 = 8_640_000_000;
pub(crate) const MILLI_PER_DAY: i64 = 8_600_000;

// Datetime utf16 constants.
pub(crate) const YEAR: &[u16] = utf16!("year");
pub(crate) const MONTH: &[u16] = utf16!("month");
pub(crate) const WEEK: &[u16] = utf16!("week");
pub(crate) const DAY: &[u16] = utf16!("day");
pub(crate) const HOUR: &[u16] = utf16!("hour");
pub(crate) const MINUTE: &[u16] = utf16!("minute");
pub(crate) const SECOND: &[u16] = utf16!("second");
pub(crate) const MILLISECOND: &[u16] = utf16!("millisecond");
pub(crate) const MICROSECOND: &[u16] = utf16!("microsecond");
pub(crate) const NANOSECOND: &[u16] = utf16!("nanosecond");

// Rounding Mode string constants
pub(crate) const CEIL: &[u16] = utf16!("ceil");
pub(crate) const FLOOR: &[u16] = utf16!("floor");
pub(crate) const EXPAND: &[u16] = utf16!("expand");
pub(crate) const TRUNC: &[u16] = utf16!("trunc");
pub(crate) const HALFCEIL: &[u16] = utf16!("halfCeil");
pub(crate) const HALFFLOOR: &[u16] = utf16!("halfFloor");
pub(crate) const HALFEXPAND: &[u16] = utf16!("halfExpand");
pub(crate) const HALFTRUNC: &[u16] = utf16!("halfTrunc");
pub(crate) const HALFEVEN: &[u16] = utf16!("halfEven");

/// `TemporalUnits` represents the temporal relationship laid out in table 13 of the [ECMAScript Specification][spec]
///
/// [spec]: https://tc39.es/proposal-temporal/#table-temporal-units
#[derive(Debug)]
pub struct TemporalUnits {
    year: (&'static [u16], &'static [u16]),
    month: (&'static [u16], &'static [u16]),
    week: (&'static [u16], &'static [u16]),
    day: (&'static [u16], &'static [u16]),
    hour: (&'static [u16], &'static [u16]),
    minute: (&'static [u16], &'static [u16]),
    second: (&'static [u16], &'static [u16]),
    millisecond: (&'static [u16], &'static [u16]),
    microsecond: (&'static [u16], &'static [u16]),
    nanosecond: (&'static [u16], &'static [u16]),
}

impl Default for TemporalUnits {
    fn default() -> Self {
        Self {
            year: (YEAR, utf16!("years")),
            month: (MONTH, utf16!("months")),
            week: (WEEK, utf16!("weeks")),
            day: (DAY, utf16!("days")),
            hour: (HOUR, utf16!("hours")),
            minute: (MINUTE, utf16!("minutes")),
            second: (SECOND, utf16!("seconds")),
            millisecond: (MILLISECOND, utf16!("milliseconds")),
            microsecond: (MICROSECOND, utf16!("microseconds")),
            nanosecond: (NANOSECOND, utf16!("nanoseconds")),
        }
    }
}

impl TemporalUnits {
    /// Returns a vector of all date singualar `TemporalUnits`.
    fn date_singulars(&self) -> Vec<JsString> {
        vec![
            self.year.0.into(),
            self.month.0.into(),
            self.week.0.into(),
            self.day.0.into(),
        ]
    }

    /// Returns a vector of all time singular `TemporalUnits`.
    fn time_singulars(&self) -> Vec<JsString> {
        vec![
            self.hour.0.into(),
            self.minute.0.into(),
            self.second.0.into(),
            self.millisecond.0.into(),
            self.microsecond.0.into(),
            self.nanosecond.0.into(),
        ]
    }

    /// Return a vector of all datetime singular `TemporalUnits`.
    fn datetime_singulars(&self) -> Vec<JsString> {
        let mut output = self.date_singulars();
        output.extend(self.time_singulars());
        output
    }

    fn all(&self) -> Vec<(&'static [u16], &'static [u16])> {
        vec![
            self.year,
            self.month,
            self.week,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.millisecond,
            self.microsecond,
            self.nanosecond,
        ]
    }

    fn append_plural_units(&self, singulars: &mut Vec<JsString>) {
        let units_table = self.all();
        for (singular, plural) in units_table {
            let singular_string: JsString = singular.into();
            if singulars.contains(&singular_string) {
                singulars.push(plural.into());
            }
        }
    }

    fn plural_lookup(&self, value: &JsString) -> JsString {
        let units_table = self.all();
        for (singular, plural) in units_table {
            if plural == value {
                return singular.into();
            }
        }
        value.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnsignedRoundingMode {
    Infinity,
    Zero,
    HalfInfinity,
    HalfZero,
    HalfEven,
}

/// The [`Temporal`][spec] builtin object.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Temporal;

impl BuiltInObject for Temporal {
    const NAME: &'static str = "Temporal";
}

impl IntrinsicObject for Temporal {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                "Now",
                Now::init(realm),
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

/// 13.2 ISODateToEpochDays ( year, month, date )
pub(crate) fn iso_date_to_epoch_days(year: i32, month: i32, date: i32) -> i32 {
    // 1. Let resolvedYear be year + floor(month / 12).
    let resolved_year = year + (month as f64 / 12_f64).floor() as i32;
    // 2. Let resolvedMonth be month modulo 12.
    let resolved_month = month % 12;

    // 3. Find a time t such that EpochTimeToEpochYear(t) is resolvedYear, EpochTimeToMonthInYear(t) is resolvedMonth, and EpochTimeToDate(t) is 1.
    let year_t = self::date_equations::epoch_time_for_year(resolved_year as f64);
    let month_t = self::date_equations::epoch_time_for_month_given_year(resolved_month, resolved_year);

    // 4. Return EpochTimeToDayNumber(t) + date - 1.
    self::date_equations::epoch_time_to_day_number(year_t + month_t) as i32 + date - 1
}

/// Abstract Operation 13.5 GetOptionsObject ( options )
#[inline]
pub(crate) fn get_option_object(options: &JsValue) -> JsResult<JsObject> {
    // 1. If options is undefined, then
    if options.is_undefined() {
        // a. Return OrdinaryObjectCreate(null).
        return Ok(JsObject::with_null_proto());
    // 2. If Type(options) is Object, then
    } else if options.is_object() {
        // a. Return options.
        return Ok(options.as_object().unwrap().clone());
    }
    // 3. Throw a TypeError exception.
    return Err(JsNativeError::typ()
        .with_message("options value was not an object.")
        .into());
}

/// 13.6 CopyOptions ( options )
#[inline]
pub(crate) fn copy_options(options: &JsValue, context: &mut Context<'_>) -> JsResult<JsObject> {
    // 1. Let optionsCopy be OrdinaryObjectCreate(null).
    let options_copy = JsObject::with_null_proto();
    // 2. Perform ? CopyDataProperties(optionsCopy, ? GetOptionsObject(options), « »).
    let option_object = get_option_object(options)?;
    let excluded_keys: Vec<PropertyKey> = Vec::new();
    options_copy.copy_data_properties(&option_object.into(), excluded_keys, context)?;
    // 3. Return optionsCopy.
    Ok(options_copy)
}

pub(crate) enum OptionType {
    String,
    Bool,
    Number,
}

/// 13.7 GetOption ( options, property, type, values, default )
#[inline]
pub(crate) fn get_option(
    options: &JsObject,
    property: PropertyKey,
    r#type: OptionType,
    values: Option<&[JsString]>,
    default: Option<&JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. Let value be ? Get(options, property).
    let initial_value = options.get(property, context)?;

    // 2. If value is undefined, then
    if initial_value.is_undefined() {
        match default {
            // a. If default is required, throw a RangeError exception.
            None => {
                return Err(JsNativeError::range()
                    .with_message("options object is required.")
                    .into())
            }
            // b. Return default.
            Some(option_value) => return Ok(option_value.clone()),
        }
    }

    let value: JsValue = match r#type {
        // 3. If type is "boolean", then
        OptionType::Bool => {
            // a. Set value to ToBoolean(value).
            initial_value.to_boolean().into()
        }
        // 4. Else if type is "number", then
        OptionType::Number => {
            // a. Set value to ? ToNumber(value).
            let value = initial_value.to_number(context)?;
            // b. If value is NaN, throw a RangeError exception.
            if value.is_nan() {
                return Err(JsNativeError::range()
                    .with_message("option value is NaN")
                    .into());
            };

            value.into()
        }
        // 5. Else,
        // a. Assert: type is "string".
        OptionType::String => {
            // b. Set value to ? ToString(value).
            initial_value.to_string(context)?.into()
        }
    };

    // 6. If values is not undefined and values does not contain an element equal to value, throw a RangeError exception.
    // NOTE: per spec, values is only provided/defined in string cases, so the below should be correct.
    if let (Some(vals), Some(value_as_string)) = (values, value.as_string()) {
        if !vals.contains(value_as_string) {
            return Err(JsNativeError::range()
                .with_message("Option value is not in the provided options.")
                .into());
        }
    }

    // 7. Return value.
    Ok(value)
}

/// 13.10 ToTemporalRoundingMode ( normalizedOptions, fallback )
#[inline]
pub(crate) fn to_temporal_rounding_mode(
    normalized_options: &JsObject,
    fallback: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsString> {
    // 1. Return ? GetOption(normalizedOptions, "roundingMode", "string", [CEIL, FLOOR, "expand", "trunc",
    // "halfCeil", "halfFloor", "halfExpand", "halfTrunc", "halfEven"], fallback).
    let option_value = get_option(
        normalized_options,
        PropertyKey::from("roundingMode"),
        OptionType::String,
        Some(&[
            CEIL.into(),
            FLOOR.into(),
            "expand".into(),
            "trunc".into(),
            "halfCeil".into(),
            "halfFloor".into(),
            "halfExpand".into(),
            "halfTrunc".into(),
            "halfEven".into(),
        ]),
        Some(fallback),
        context,
    )?;

    match option_value.as_string() {
        Some(string) => Ok(string.clone()),
        // TODO: validate
        None => Err(JsNativeError::typ()
            .with_message("roundingMode must be a string value.")
            .into()),
    }
}

// 13.11 NegateTemporalRoundingMode ( roundingMode )
fn negate_temporal_rounding_mode(rounding_mode: JsString) -> JsString {
    match rounding_mode.as_slice() {
        CEIL => FLOOR.into(),
        FLOOR => CEIL.into(),
        HALFCEIL => HALFFLOOR.into(),
        HALFFLOOR => HALFCEIL.into(),
        _ => rounding_mode,
    }
}

/// 13.16 ToTemporalRoundingIncrement ( normalizedOptions )
#[inline]
pub(crate) fn to_temporal_rounding_increment(
    normalized_options: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. Let increment be ? GetOption(normalizedOptions, "roundingIncrement", "number", undefined, 1𝔽).
    let increment = get_option(
        normalized_options,
        PropertyKey::from("roundingIncrement"),
        OptionType::Number,
        None,
        Some(&JsValue::from(1.0)),
        context,
    )?;
    // 2. If increment is not finite, throw a RangeError exception.
    let num = match increment.to_number(context) {
        Ok(number) if number.is_finite() => number,
        _ => {
            return Err(JsNativeError::range()
                .with_message("rounding increment was out of range.")
                .into())
        }
    };
    // 3. Let integerIncrement be truncate(ℝ(increment)).
    let integer_increment = num.trunc();
    // 4. If integerIncrement < 1 or integerIncrement > 109, throw a RangeError exception.
    if integer_increment < 1.0 || integer_increment > 109.0 {
        return Err(JsNativeError::range()
            .with_message("rounding increment was out of range.")
            .into());
    }
    // 5. Return integerIncrement.
    Ok(integer_increment)
}

/// 13.17 ValidateTemporalRoundingIncrement ( increment, dividend, inclusive )
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

/// Abstract operation 13.20 GetTemporalUnit ( normalizedOptions, key, unitGroup, default [ , extraValues ] )
#[inline]
pub(crate) fn get_temporal_unit(
    normalized_options: &JsObject,
    key: PropertyKey,
    unit_group: &JsString,               // JsString or temporal
    default: Option<&JsValue>,           // Must be required (none), undefined, or JsString.
    extra_values: Option<Vec<JsString>>, // Vec<JsString>
    context: &mut Context<'_>,
) -> JsResult<JsString> {
    // 1. Let singularNames be a new empty List.
    let temporal_units = TemporalUnits::default();
    // 2. For each row of Table 13, except the header row, in table order, do
    // a. Let unit be the value in the Singular column of the row.  // b. If the Category column of the row is date and unitGroup is date or datetime, append unit to singularNames.
    // c. Else if the Category column of the row is time and unitGroup is time or datetime, append unit to singularNames.
    let mut singular_names = if unit_group.as_slice() == utf16!("date") {
        temporal_units.date_singulars()
    } else if unit_group.as_slice() == utf16!("time") {
        temporal_units.time_singulars()
    } else {
        temporal_units.datetime_singulars()
    };
    // 3. If extraValues is present, then
    // a. Set singularNames to the list-concatenation of singularNames and extraValues.
    if let Some(values) = extra_values {
        singular_names.extend(values);
    }
    // 4. If default is required, then
    // a. Let defaultValue be undefined.
    // 5. Else,
    // a. Let defaultValue be default.
    // b. If defaultValue is not undefined and singularNames does not contain defaultValue, then
    // i. Append defaultValue to singularNames.
    let default_value = if let Some(value) = default {
        // NOTE: singular name must be either undefined or a JsString, any other value is an implementation error.
        if !value.is_undefined() {
            if let Some(value_string) = value.as_string() {
                if singular_names.contains(value_string) {
                    singular_names.push(value_string.clone());
                }
            }
        }
        Some(value)
    } else {
        None
    };

    // 6. Let allowedValues be a copy of singularNames.
    // 7. For each element singularName of singularNames, do
    // a. If singularName is listed in the Singular column of Table 13, then
    // i. Let pluralName be the value in the Plural column of the corresponding row.
    // ii. Append pluralName to allowedValues.
    // 8. NOTE: For each singular Temporal unit name that is contained within allowedValues, the
    // corresponding plural name is also contained within it.
    temporal_units.append_plural_units(&mut singular_names);

    // 9. Let value be ? GetOption(normalizedOptions, key, "string", allowedValues, defaultValue).
    let value = get_option(
        normalized_options,
        key,
        OptionType::String,
        Some(&singular_names),
        default_value,
        context,
    )?;

    // 10. If value is undefined and default is required, throw a RangeError exception.
    if value.is_undefined() && default.is_none() {
        return Err(JsNativeError::range()
            .with_message("option cannot be undefined when required.")
            .into());
    }

    // NOTE: Should spec probably be asserting that value is not undefined at this point.
    assert!(!value.is_undefined());

    // 11. If value is listed in the Plural column of Table 13, then
    // a. Set value to the value in the Singular column of the corresponding row.
    // 12. Return value.
    match value.as_string() {
        Some(string) => Ok(temporal_units.plural_lookup(string)),
        // TODO: verify that this is correct to specification, i.e. is it possible for default value to exist and value to be undefined?
        _ => unreachable!(),
    }
}

/// 13.22 LargerOfTwoTemporalUnits ( u1, u2 )
fn larger_of_two_temporal_units(u1: &JsString, u2: &JsString) -> JsString {
    // 1. Assert: Both u1 and u2 are listed in the Singular column of Table 13.
    let unit_table = TemporalUnits::default();
    assert!(
        unit_table.datetime_singulars().contains(&u1)
            && unit_table.datetime_singulars().contains(&u2)
    );

    // 2. For each row of Table 13, except the header row, in table order, do
    // a. Let unit be the value in the Singular column of the row.
    let mut result = JsString::default();
    for unit in unit_table.all() {
        // b. If SameValue(u1, unit) is true, return unit.
        if u1.as_slice() == unit.0 {
            result = u1.clone();
            break;
        };

        // c. If SameValue(u2, unit) is true, return unit.
        if u2.as_slice() == unit.1 {
            result = u2.clone();
            break;
        };
    }
    return result;
}

/// 13.23 MaximumTemporalDurationRoundingIncrement ( unit )
fn maximum_temporal_duration_rounding_increment(unit: &JsString) -> JsValue {
    match unit.as_slice() {
        // 1. If unit is "year", "month", "week", or "day", then
        // a. Return undefined.
        YEAR | MONTH | WEEK | DAY => JsValue::undefined(),
        // 2. If unit is "hour", then
        // a. Return 24.
        HOUR => JsValue::from(24),
        // 3. If unit is "minute" or "second", then
        // a. Return 60.
        MINUTE | SECOND => JsValue::from(60),
        // 4. Assert: unit is one of "millisecond", "microsecond", or "nanosecond".
        // 5. Return 1000.
        MILLISECOND | MICROSECOND | NANOSECOND => JsValue::from(1000),
        _ => unreachable!(),
    }
}

/// 13.26 GetUnsignedRoundingMode ( roundingMode, isNegative )
#[inline]
pub(crate) fn get_unsigned_round_mode(
    rounding_mode: &JsString,
    is_negative: bool,
) -> UnsignedRoundingMode {
    match rounding_mode.as_slice() {
        CEIL if !is_negative => UnsignedRoundingMode::Infinity,
        CEIL => UnsignedRoundingMode::Zero,
        FLOOR if !is_negative => UnsignedRoundingMode::Zero,
        FLOOR => UnsignedRoundingMode::Infinity,
        EXPAND => UnsignedRoundingMode::Infinity,
        TRUNC => UnsignedRoundingMode::Zero,
        HALFCEIL if !is_negative => UnsignedRoundingMode::HalfInfinity,
        HALFCEIL => UnsignedRoundingMode::HalfZero,
        HALFFLOOR if !is_negative => UnsignedRoundingMode::HalfZero,
        HALFFLOOR => UnsignedRoundingMode::HalfInfinity,
        HALFEXPAND => UnsignedRoundingMode::HalfInfinity,
        HALFTRUNC => UnsignedRoundingMode::HalfZero,
        HALFEVEN => UnsignedRoundingMode::HalfEven,
        _ => unreachable!(),
    }
}

/// 13.27 ApplyUnsignedRoundingMode ( x, r1, r2, unsignedRoundingMode )
#[inline]
fn apply_unsigned_rounding_mode(
    x: f64,
    r1: f64,
    r2: f64,
    unsigned_rounding_mode: UnsignedRoundingMode,
) -> f64 {
    // 1. If x is equal to r1, return r1.
    if x == r1 {
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
    assert!(d1 == d2);

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

/// 13.29 RoundNumberToIncrementAsIfPositive ( x, increment, roundingMode )
#[inline]
pub(crate) fn round_to_increment_as_if_positive(
    ns: &JsBigInt,
    increment: i64,
    rounding_mode: &JsString,
) -> JsResult<JsBigInt> {
    // 1. Let quotient be x / increment.
    let q = ns.to_f64() / increment as f64;
    // 2. Let unsignedRoundingMode be GetUnsignedRoundingMode(roundingMode, false).
    let unsigned_rounding_mode = get_unsigned_round_mode(rounding_mode, false);
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

/// 13.47 GetDifferenceSettings ( operation, options, unitGroup, disallowedUnits, fallbackSmallestUnit, smallestLargestDefaultUnit )
// IMPLEMENTATION NOTE: op -> true == until | false == since
#[inline]
pub(crate) fn get_diff_settings(
    op: bool,
    options: &JsObject,
    unit_group: JsString,
    disallowed_units: Vec<JsString>,
    fallback_smallest_unit: JsString,
    smallest_largest_default_unit: JsString,
    context: &mut Context<'_>,
) -> JsResult<(JsString, JsString, JsString, f64)> {
    // 1. NOTE: The following steps read options and perform independent validation in alphabetical order (ToTemporalRoundingIncrement reads "roundingIncrement" and ToTemporalRoundingMode reads "roundingMode").
    // 2. Let largestUnit be ? GetTemporalUnit(options, "largestUnit", unitGroup, "auto").
    let mut largest_unit = get_temporal_unit(
        options,
        PropertyKey::from("largestUnit"),
        &unit_group,
        Some(&JsValue::from("auto")),
        None,
        context,
    )?;

    // 3. If disallowedUnits contains largestUnit, throw a RangeError exception.
    if disallowed_units.contains(&largest_unit) {
        return Err(JsNativeError::range()
            .with_message("largestUnit is not an allowed unit.")
            .into());
    }

    // 4. Let roundingIncrement be ? ToTemporalRoundingIncrement(options).
    let rounding_increment = to_temporal_rounding_increment(options, context)?;
    // 5. Let roundingMode be ? ToTemporalRoundingMode(options, "trunc").
    let mut rounding_mode = to_temporal_rounding_mode(options, &JsValue::from("trunc"), context)?;

    // 6. If operation is since, then
    if !op {
        // a. Set roundingMode to ! NegateTemporalRoundingMode(roundingMode).
        rounding_mode = negate_temporal_rounding_mode(rounding_mode);
    }

    // 7. Let smallestUnit be ? GetTemporalUnit(options, "smallestUnit", unitGroup, fallbackSmallestUnit).
    let smallest_unit = get_temporal_unit(
        options,
        PropertyKey::from("smallestUnit"),
        &unit_group,
        Some(&fallback_smallest_unit.into()),
        None,
        context,
    )?;

    // 8. If disallowedUnits contains smallestUnit, throw a RangeError exception.
    if disallowed_units.contains(&smallest_unit) {
        return Err(JsNativeError::range()
            .with_message("smallestUnit is not an allowed unit.")
            .into());
    }

    // 9. Let defaultLargestUnit be ! LargerOfTwoTemporalUnits(smallestLargestDefaultUnit, smallestUnit).
    let default_largest_unit =
        larger_of_two_temporal_units(&smallest_largest_default_unit, &smallest_unit);
    // 10. If largestUnit is "auto", set largestUnit to defaultLargestUnit.
    if largest_unit.as_slice() == utf16!("auto") {
        largest_unit = default_largest_unit;
    }

    // 11. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
    if largest_unit != larger_of_two_temporal_units(&largest_unit, &smallest_unit) {
        return Err(JsNativeError::range()
            .with_message("largestUnit must be larger than smallestUnit")
            .into());
    }

    // 12. Let maximum be ! MaximumTemporalDurationRoundingIncrement(smallestUnit).
    let maximum = maximum_temporal_duration_rounding_increment(&smallest_unit);

    // 13. If maximum is not undefined, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
    if !maximum.is_undefined() {
        validate_temporal_rounding_increment(
            rounding_increment,
            maximum.as_number().unwrap(),
            false,
        )?
    }

    // 14. Return the Record { [[SmallestUnit]]: smallestUnit, [[LargestUnit]]: largestUnit, [[RoundingMode]]: roundingMode, [[RoundingIncrement]]: roundingIncrement, }.
    Ok((
        smallest_unit,
        largest_unit,
        rounding_mode,
        rounding_increment,
    ))
}
