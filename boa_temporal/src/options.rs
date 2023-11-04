//! Temporal Options

use core::{fmt, str::FromStr};

use crate::TemporalError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TemporalUnit {
    Auto = 0,
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl TemporalUnit {
    pub fn to_maximum_rounding_increment(self) -> Option<u16> {
        use TemporalUnit::{
            Auto, Day, Hour, Microsecond, Millisecond, Minute, Month, Nanosecond, Second, Week,
            Year,
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
            Hour => Some(24),
            Minute | Second => Some(60),
            Millisecond | Microsecond | Nanosecond => Some(1000),
            Auto => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct ParseTemporalUnitError;

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
            _ => Err(ParseTemporalUnitError),
        }
    }
}

impl fmt::Display for TemporalUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => "auto",
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
        }
        .fmt(f)
    }
}

/// `ArithmeticOverflow` can also be used as an
/// assignment overflow and consists of the "constrain"
/// and "reject" options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOverflow {
    Constrain,
    Reject,
}

#[derive(Debug)]
pub struct ParseArithmeticOverflowError;

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
pub enum DurationOverflow {
    Constrain,
    Balance,
}

#[derive(Debug)]
pub struct ParseDurationOverflowError;

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
pub enum InstantDisambiguation {
    Compatible,
    Earlier,
    Later,
    Reject,
}

#[derive(Debug)]
pub struct ParseInstantDisambiguationError;

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
pub enum OffsetDisambiguation {
    Use,
    Prefer,
    Ignore,
    Reject,
}

#[derive(Debug)]
pub struct ParseOffsetDisambiguationError;

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

// TODO: Figure out what to do with intl's RoundingMode

#[derive(Debug, Copy, Clone, Default)]
pub enum TemporalRoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    #[default]
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalUnsignedRoundingMode {
    Infinity,
    Zero,
    HalfInfinity,
    HalfZero,
    HalfEven,
}

impl TemporalRoundingMode {
    pub const fn negate(self) -> Self {
        use TemporalRoundingMode::{
            Ceil, Expand, Floor, HalfCeil, HalfEven, HalfExpand, HalfFloor, HalfTrunc, Trunc,
        };

        match self {
            Ceil => Self::Floor,
            Floor => Self::Ceil,
            HalfCeil => Self::HalfFloor,
            HalfFloor => Self::HalfCeil,
            Trunc => Self::Trunc,
            Expand => Self::Expand,
            HalfTrunc => Self::HalfTrunc,
            HalfExpand => Self::HalfExpand,
            HalfEven => Self::HalfEven,
        }
    }

    pub const fn get_unsigned_round_mode(self, is_negative: bool) -> TemporalUnsignedRoundingMode {
        use TemporalRoundingMode::{
            Ceil, Expand, Floor, HalfCeil, HalfEven, HalfExpand, HalfFloor, HalfTrunc, Trunc,
        };

        match self {
            Ceil if !is_negative => TemporalUnsignedRoundingMode::Infinity,
            Ceil => TemporalUnsignedRoundingMode::Zero,
            Floor if !is_negative => TemporalUnsignedRoundingMode::Zero,
            Floor | Trunc | Expand => TemporalUnsignedRoundingMode::Infinity,
            HalfCeil if !is_negative => TemporalUnsignedRoundingMode::HalfInfinity,
            HalfCeil | HalfTrunc => TemporalUnsignedRoundingMode::HalfZero,
            HalfFloor if !is_negative => TemporalUnsignedRoundingMode::HalfZero,
            HalfFloor | HalfExpand => TemporalUnsignedRoundingMode::HalfInfinity,
            HalfEven => TemporalUnsignedRoundingMode::HalfEven,
        }
    }
}

impl FromStr for TemporalRoundingMode {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ceil" => Ok(Self::Ceil),
            "floor" => Ok(Self::Floor),
            "expand" => Ok(Self::Expand),
            "trunc" => Ok(Self::Trunc),
            "halfCeil" => Ok(Self::HalfCeil),
            "halfFloor" => Ok(Self::HalfFloor),
            "halfExpand" => Ok(Self::HalfExpand),
            "halfTrunc" => Ok(Self::HalfTrunc),
            "halfEven" => Ok(Self::HalfEven),
            _ => Err(TemporalError::range().with_message("RoundingMode not an accepted value.")),
        }
    }
}

impl fmt::Display for TemporalRoundingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ceil => "ceil",
            Self::Floor => "floor",
            Self::Expand => "expand",
            Self::Trunc => "trunc",
            Self::HalfCeil => "halfCeil",
            Self::HalfFloor => "halfFloor",
            Self::HalfExpand => "halfExpand",
            Self::HalfTrunc => "halfTrunc",
            Self::HalfEven => "halfEven",
        }
        .fmt(f)
    }
}
