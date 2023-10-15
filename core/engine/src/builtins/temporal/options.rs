//! Temporal Option types.

// Implementation Note:
//
// The below Option types are adapted from the types laid out by
// the Temporal proposal's polyfill types that can be found at the
// below link.
//
// https://github.com/tc39/proposal-temporal/blob/main/polyfill/index.d.ts

use crate::{
    builtins::options::{get_option, ParsableOptionType},
    js_string,
    string::JsStr,
    Context, JsNativeError, JsObject, JsResult,
};
use temporal_rs::options::{
    ArithmeticOverflow, DurationOverflow, InstantDisambiguation, OffsetDisambiguation,
    TemporalRoundingMode, TemporalUnit,
};

// TODO: Expand docs on the below options.

// TODO: Remove and refactor: migrate to `boa_temporal`
#[inline]
pub(crate) fn get_temporal_rounding_increment(
    options: &JsObject,
    context: &mut Context,
) -> JsResult<Option<f64>> {
    // 1. Let increment be ? GetOption(normalizedOptions, "roundingIncrement", "number", undefined, 1𝔽).
    let value = options.get(js_string!("roundingIncrement"), context)?;

    if value.is_undefined() {
        return Ok(None);
    }
    let increment = value.to_number(context)?;

    // 2. If increment is not finite, throw a RangeError exception.
    if !increment.is_finite() {
        return Err(JsNativeError::range()
            .with_message("rounding increment was out of range.")
            .into());
    }

    // 3. Let integerIncrement be truncate(ℝ(increment)).
    let integer_increment = increment.trunc();

    // 4. If integerIncrement < 1 or integerIncrement > 10^9, throw a RangeError exception.
    if !(1.0..=1_000_000_000.0).contains(&integer_increment) {
        return Err(JsNativeError::range()
            .with_message("rounding increment was out of range.")
            .into());
    }

    // 5. Return integerIncrement.
    Ok(Some(integer_increment))
}

/// Gets the `TemporalUnit` from an options object.
#[inline]
pub(crate) fn get_temporal_unit(
    options: &JsObject,
    key: JsStr<'_>,
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum TemporalUnitGroup {
    Date,
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
impl ParsableOptionType for InstantDisambiguation {}
impl ParsableOptionType for OffsetDisambiguation {}
impl ParsableOptionType for TemporalRoundingMode {}
