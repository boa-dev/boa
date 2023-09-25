use std::fmt;

use crate::builtins::options::{ParsableOptionType, RoundingMode};

#[derive(Debug)]
pub(crate) struct DigitFormatOptions {
    pub(crate) minimum_integer_digits: u8,
    pub(crate) rounding_increment: u16,
    pub(crate) rounding_mode: RoundingMode,
    pub(crate) trailing_zero_display: TrailingZeroDisplay,
    pub(crate) rounding_type: RoundingType,
    pub(crate) rounding_priority: RoundingPriority,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub(crate) enum Notation {
    #[default]
    Standard,
    Scientific,
    Engineering,
    Compact,
}

#[derive(Debug)]
pub(crate) struct ParseNotationError;

impl fmt::Display for ParseNotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid notation option")
    }
}

impl std::str::FromStr for Notation {
    type Err = ParseNotationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standard" => Ok(Self::Standard),
            "scientific" => Ok(Self::Scientific),
            "engineering" => Ok(Self::Engineering),
            "compact" => Ok(Self::Compact),
            _ => Err(ParseNotationError),
        }
    }
}

impl ParsableOptionType for Notation {}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub(crate) enum RoundingPriority {
    #[default]
    Auto,
    MorePrecision,
    LessPrecision,
}

#[derive(Debug)]
pub(crate) struct ParseRoundingPriorityError;

impl fmt::Display for ParseRoundingPriorityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid rounding priority")
    }
}

impl std::str::FromStr for RoundingPriority {
    type Err = ParseRoundingPriorityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "morePrecision" => Ok(Self::MorePrecision),
            "lessPrecision" => Ok(Self::LessPrecision),
            _ => Err(ParseRoundingPriorityError),
        }
    }
}

impl ParsableOptionType for RoundingPriority {}

impl fmt::Display for RoundingPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => "auto",
            Self::MorePrecision => "morePrecision",
            Self::LessPrecision => "lessPrecision",
        }
        .fmt(f)
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub(crate) enum TrailingZeroDisplay {
    #[default]
    Auto,
    StripIfInteger,
}

#[derive(Debug)]
pub(crate) struct ParseTrailingZeroDisplayError;

impl fmt::Display for ParseTrailingZeroDisplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not a valid trailing zero display option")
    }
}

impl std::str::FromStr for TrailingZeroDisplay {
    type Err = ParseTrailingZeroDisplayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "stripIfInteger" => Ok(Self::StripIfInteger),
            _ => Err(ParseTrailingZeroDisplayError),
        }
    }
}

impl ParsableOptionType for TrailingZeroDisplay {}

impl fmt::Display for TrailingZeroDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => "auto",
            Self::StripIfInteger => "stripIfInteger",
        }
        .fmt(f)
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct Extrema<T> {
    pub(crate) minimum: T,
    pub(crate) maximum: T,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum RoundingType {
    MorePrecision {
        significant_digits: Extrema<u8>,
        fraction_digits: Extrema<u8>,
    },
    LessPrecision {
        significant_digits: Extrema<u8>,
        fraction_digits: Extrema<u8>,
    },
    SignificantDigits(Extrema<u8>),
    FractionDigits(Extrema<u8>),
}

impl RoundingType {
    /// Gets the significant digit limits of the rounding type, or `None` otherwise.
    pub(crate) const fn significant_digits(self) -> Option<Extrema<u8>> {
        match self {
            Self::MorePrecision {
                significant_digits, ..
            }
            | Self::LessPrecision {
                significant_digits, ..
            }
            | Self::SignificantDigits(significant_digits) => Some(significant_digits),
            Self::FractionDigits(_) => None,
        }
    }

    /// Gets the fraction digit limits of the rounding type, or `None` otherwise.
    pub(crate) const fn fraction_digits(self) -> Option<Extrema<u8>> {
        match self {
            Self::MorePrecision {
                fraction_digits, ..
            }
            | Self::LessPrecision {
                fraction_digits, ..
            }
            | Self::FractionDigits(fraction_digits) => Some(fraction_digits),
            Self::SignificantDigits(_) => None,
        }
    }
}
