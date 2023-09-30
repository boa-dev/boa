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
    js_string, Context, JsNativeError, JsObject, JsResult,
};
use std::{fmt, str::FromStr};

// TODO: Expand docs on the below options.

#[inline]
pub(crate) fn get_temporal_rounding_increment(
    options: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<f64> {
    // 1. Let increment be ? GetOption(normalizedOptions, "roundingIncrement", "number", undefined, 1𝔽).
    let value = options.get(js_string!("roundingIncrement"), context)?;

    let increment = if value.is_undefined() {
        1.0
    } else {
        value.to_number(context)?
    };

    // 2. If increment is not finite, throw a RangeError exception.
    if !increment.is_finite() {
        return Err(JsNativeError::range()
            .with_message("rounding increment was out of range.")
            .into());
    }

    // 3. Let integerIncrement be truncate(ℝ(increment)).
    let integer_increment = increment.trunc();

    // 4. If integerIncrement < 1 or integerIncrement > 109, throw a RangeError exception.
    if (1.0..=109.0).contains(&integer_increment) {
        return Err(JsNativeError::range()
            .with_message("rounding increment was out of range.")
            .into());
    }

    // 5. Return integerIncrement.
    Ok(integer_increment)
}

/// Gets the `TemporalUnit` from an options object.
#[inline]
pub(crate) fn get_temporal_unit(
    options: &JsObject,
    key: &[u16],
    unit_group: TemporalUnitGroup,
    required: Option<TemporalUnit>,
    extra_values: Option<Vec<TemporalUnit>>,
    context: &mut Context<'_>,
) -> JsResult<TemporalUnit> {
    let extra = extra_values.unwrap_or_default();
    let mut unit_values = unit_group.group();
    unit_values.extend(extra);

    let (required, default) = if let Some(unit) = required {
        (false, unit)
    } else {
        // Note: using auto here should be fine as the default value will not be used.
        (true, TemporalUnit::Auto)
    };

    let unit = get_option::<TemporalUnit>(options, key, required, context)?.unwrap_or(default);

    if !unit_values.contains(&unit) {
        return Err(JsNativeError::range()
            .with_message("TemporalUnit was not part of the valid UnitGroup.")
            .into());
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TemporalUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
    Auto,
    Undefined,
}

impl TemporalUnit {
    pub(crate) fn to_maximum_rounding_increment(self) -> Option<f64> {
        use TemporalUnit::{
            Day, Hour, Microsecond, Millisecond, Minute, Month, Nanosecond, Second, Week, Year,
        };
        // 1. If unit is "year", "month", "week", or "day", then
        // a. Return undefined.
        // 2. If unit is "hour", then
        // a. Return 24.
        // 3. If unit is "minute" or "second", then
        // a. Return 60.
        // 4. Assert: unit is one of "millisecond", "microsecond", or "nanosecond".
        // 5. Return 1000.
        match self {
            Year | Month | Week | Day => None,
            Hour => Some(24.0),
            Minute | Second => Some(60.0),
            Millisecond | Microsecond | Nanosecond => Some(1000.0),
            _ => unreachable!(),
        }
    }

    /// Returns if value of unit is "Undefined".
    pub(crate) fn is_undefined(self) -> bool {
        self == Self::Undefined
    }

    /// Returns if value of enum is "auto"
    pub(crate) fn is_auto(self) -> bool {
        self == Self::Auto
    }
}

#[derive(Debug)]
pub(crate) struct ParseTemporalUnitError;

impl fmt::Display for ParseTemporalUnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid TemporalUnit")
    }
}

impl FromStr for TemporalUnit {
    type Err = ParseTemporalUnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "year" | "years" => Ok(Self::Year),
            "month" | "months" => Ok(Self::Month),
            "week" | "weeks" => Ok(Self::Week),
            "day" | "days" => Ok(Self::Day),
            "hour" | "hours" => Ok(Self::Hour),
            "minute" | "minutes" => Ok(Self::Minute),
            "second" | "seconds" => Ok(Self::Second),
            "millisecond" | "milliseconds" => Ok(Self::Millisecond),
            "microsecond" | "microseconds" => Ok(Self::Microsecond),
            "nanosecond" | "nanoseconds" => Ok(Self::Nanosecond),
            // Note: undefined is an implementation value. It would be an error to parse undefined.
            _ => Err(ParseTemporalUnitError),
        }
    }
}

impl ParsableOptionType for TemporalUnit {}

impl fmt::Display for TemporalUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Year => "constrain",
            Self::Month => "month",
            Self::Week => "week",
            Self::Day => "day",
            Self::Hour => "hour",
            Self::Minute => "minute",
            Self::Second => "second",
            Self::Millisecond => "millsecond",
            Self::Microsecond => "microsecond",
            Self::Nanosecond => "nanosecond",
            Self::Auto => "auto",
            Self::Undefined => "undefined",
        }
        .fmt(f)
    }
}

/// `ArithmeticOverflow` can also be used as an
/// assignment overflow and consists of the "constrain"
/// and "reject" options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArithmeticOverflow {
    Constrain,
    Reject,
}

#[derive(Debug)]
pub(crate) struct ParseArithmeticOverflowError;

impl fmt::Display for ParseArithmeticOverflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid overflow value")
    }
}

impl FromStr for ArithmeticOverflow {
    type Err = ParseArithmeticOverflowError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "constrain" => Ok(Self::Constrain),
            "reject" => Ok(Self::Reject),
            _ => Err(ParseArithmeticOverflowError),
        }
    }
}

impl ParsableOptionType for ArithmeticOverflow {}

impl fmt::Display for ArithmeticOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constrain => "constrain",
            Self::Reject => "reject",
        }
        .fmt(f)
    }
}

/// `Duration` overflow options.
pub(crate) enum DurationOverflow {
    Constrain,
    Balance,
}

#[derive(Debug)]
pub(crate) struct ParseDurationOverflowError;

impl fmt::Display for ParseDurationOverflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid duration overflow value")
    }
}

impl FromStr for DurationOverflow {
    type Err = ParseDurationOverflowError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "constrain" => Ok(Self::Constrain),
            "balance" => Ok(Self::Balance),
            _ => Err(ParseDurationOverflowError),
        }
    }
}

impl ParsableOptionType for DurationOverflow {}

impl fmt::Display for DurationOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constrain => "constrain",
            Self::Balance => "balance",
        }
        .fmt(f)
    }
}

/// The disambiguation options for an instant.
pub(crate) enum InstantDisambiguation {
    Compatible,
    Earlier,
    Later,
    Reject,
}

#[derive(Debug)]
pub(crate) struct ParseInstantDisambiguationError;

impl fmt::Display for ParseInstantDisambiguationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid instant disambiguation value")
    }
}
impl FromStr for InstantDisambiguation {
    type Err = ParseInstantDisambiguationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compatible" => Ok(Self::Compatible),
            "earlier" => Ok(Self::Earlier),
            "later" => Ok(Self::Later),
            "reject" => Ok(Self::Reject),
            _ => Err(ParseInstantDisambiguationError),
        }
    }
}

impl ParsableOptionType for InstantDisambiguation {}

impl fmt::Display for InstantDisambiguation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compatible => "compatible",
            Self::Earlier => "earlier",
            Self::Later => "later",
            Self::Reject => "reject",
        }
        .fmt(f)
    }
}

/// Offset disambiguation options.
pub(crate) enum OffsetDisambiguation {
    Use,
    Prefer,
    Ignore,
    Reject,
}

#[derive(Debug)]
pub(crate) struct ParseOffsetDisambiguationError;

impl fmt::Display for ParseOffsetDisambiguationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid offset disambiguation value")
    }
}

impl FromStr for OffsetDisambiguation {
    type Err = ParseOffsetDisambiguationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "use" => Ok(Self::Use),
            "prefer" => Ok(Self::Prefer),
            "ignore" => Ok(Self::Ignore),
            "reject" => Ok(Self::Reject),
            _ => Err(ParseOffsetDisambiguationError),
        }
    }
}

impl ParsableOptionType for OffsetDisambiguation {}

impl fmt::Display for OffsetDisambiguation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Use => "use",
            Self::Prefer => "prefer",
            Self::Ignore => "ignore",
            Self::Reject => "reject",
        }
        .fmt(f)
    }
}
