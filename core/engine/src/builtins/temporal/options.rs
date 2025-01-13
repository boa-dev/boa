//! Temporal Option types.

// Implementation Note:
//
// The below Option types are adapted from the types laid out by
// the Temporal proposal's polyfill types that can be found at the
// below link.
//
// https://github.com/tc39/proposal-temporal/blob/main/polyfill/index.d.ts

use crate::{
    builtins::options::{get_option, OptionType, ParsableOptionType},
    js_string, Context, JsNativeError, JsObject, JsResult, JsString, JsValue,
};
use temporal_rs::{
    options::{
        ArithmeticOverflow, DifferenceSettings, Disambiguation, DisplayCalendar, DisplayOffset,
        DisplayTimeZone, DurationOverflow, OffsetDisambiguation, RoundingIncrement,
        TemporalRoundingMode, TemporalUnit,
    },
    parsers::Precision,
};

// TODO: Expand docs on the below options.

/// Gets the `TemporalUnit` from an options object.
#[inline]
pub(crate) fn get_temporal_unit(
    options: &JsObject,
    key: JsString,
    unit_group: TemporalUnitGroup,
    extra_values: Option<Vec<TemporalUnit>>,
    context: &mut Context,
) -> JsResult<Option<TemporalUnit>> {
    let extra = extra_values.unwrap_or_default();
    let mut unit_values = unit_group.group();
    unit_values.extend(extra);

    let unit = get_option(options, key, context)?;

    if let Some(u) = &unit {
        if !unit_values.contains(u) {
            return Err(JsNativeError::range()
                .with_message("TemporalUnit was not part of the valid UnitGroup.")
                .into());
        }
    }

    Ok(unit)
}

#[inline]
pub(crate) fn get_difference_settings(
    options: &JsObject,
    context: &mut Context,
) -> JsResult<DifferenceSettings> {
    let mut settings = DifferenceSettings::default();
    settings.largest_unit =
        get_option::<TemporalUnit>(options, js_string!("largestUnit"), context)?;
    settings.increment =
        get_option::<RoundingIncrement>(options, js_string!("roundingIncrement"), context)?;
    settings.rounding_mode =
        get_option::<TemporalRoundingMode>(options, js_string!("roundingMode"), context)?;
    settings.smallest_unit =
        get_option::<TemporalUnit>(options, js_string!("smallestUnit"), context)?;
    Ok(settings)
}

pub(crate) fn get_digits_option(options: &JsObject, context: &mut Context) -> JsResult<Precision> {
    // 1. Let digitsValue be ? Get(options, "fractionalSecondDigits").
    let digits_value = options.get(js_string!("fractionalSecondDigits"), context)?;
    // 2. If digitsValue is undefined, return auto.
    if digits_value.is_undefined() {
        return Ok(Precision::Auto);
    }
    // 3. If digitsValue is not a Number, then
    let Some(digits_number) = digits_value.as_number() else {
        // a. If ? ToString(digitsValue) is not "auto", throw a RangeError exception.
        if digits_value.to_string(context)? != js_string!("auto") {
            return Err(JsNativeError::range()
                .with_message("fractionalSecondDigits must be a digit or 'auto'")
                .into());
        }
        // b. Return auto.
        return Ok(Precision::Auto);
    };

    // 4. If digitsValue is NaN, +∞𝔽, or -∞𝔽, throw a RangeError exception.
    if !digits_number.is_finite() {
        return Err(JsNativeError::range()
            .with_message("fractionalSecondDigits must be a finite number")
            .into());
    }
    // 5. Let digitCount be floor(ℝ(digitsValue)).
    let digits = digits_number.floor() as i32;
    // 6. If digitCount < 0 or digitCount > 9, throw a RangeError exception.
    if !(0..=9).contains(&digits) {
        return Err(JsNativeError::range()
            .with_message("fractionalSecondDigits must be in an inclusive range of 0-9")
            .into());
    }
    // 7. Return digitCount.
    Ok(Precision::Digit(digits as u8))
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub(crate) enum TemporalUnitGroup {
    Date, // Need to assert if this is neede anymore with the removal of `Temporal.Calendar`
    Time,
    DateTime,
}

impl TemporalUnitGroup {
    fn group(self) -> Vec<TemporalUnit> {
        use TemporalUnitGroup::{Date, DateTime, Time};

        match self {
            Date => date_units().collect(),
            Time => time_units().collect(),
            DateTime => datetime_units().collect(),
        }
    }
}

fn time_units() -> impl Iterator<Item = TemporalUnit> {
    [
        TemporalUnit::Hour,
        TemporalUnit::Minute,
        TemporalUnit::Second,
        TemporalUnit::Millisecond,
        TemporalUnit::Microsecond,
        TemporalUnit::Nanosecond,
    ]
    .iter()
    .copied()
}

fn date_units() -> impl Iterator<Item = TemporalUnit> {
    [
        TemporalUnit::Year,
        TemporalUnit::Month,
        TemporalUnit::Week,
        TemporalUnit::Day,
    ]
    .iter()
    .copied()
}

fn datetime_units() -> impl Iterator<Item = TemporalUnit> {
    date_units().chain(time_units())
}

impl ParsableOptionType for TemporalUnit {}
impl ParsableOptionType for ArithmeticOverflow {}
impl ParsableOptionType for DurationOverflow {}
impl ParsableOptionType for Disambiguation {}
impl ParsableOptionType for OffsetDisambiguation {}
impl ParsableOptionType for TemporalRoundingMode {}
impl ParsableOptionType for DisplayCalendar {}
impl ParsableOptionType for DisplayOffset {}
impl ParsableOptionType for DisplayTimeZone {}

impl OptionType for RoundingIncrement {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        let value = value.to_number(context)?;

        Ok(RoundingIncrement::try_from(value)?)
    }
}
