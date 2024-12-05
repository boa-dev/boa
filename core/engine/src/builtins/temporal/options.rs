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
use temporal_rs::options::{
    ArithmeticOverflow, CalendarName, DifferenceSettings, Disambiguation, DurationOverflow,
    OffsetDisambiguation, RoundingIncrement, TemporalRoundingMode, TemporalUnit,
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
impl ParsableOptionType for CalendarName {}

impl OptionType for RoundingIncrement {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        let value = value.to_number(context)?;

        Ok(RoundingIncrement::try_from(value)?)
    }
}
