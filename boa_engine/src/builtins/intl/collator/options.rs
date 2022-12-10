use std::str::FromStr;

use icu_collator::{CaseLevel, Strength};

use crate::builtins::intl::options::OptionTypeParsable;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub(crate) enum Sensitivity {
    Base,
    Accent,
    Case,
    Variant,
}

impl Sensitivity {
    /// Converts the sensitivity option to the corresponding ICU4X collator
    /// options.
    pub(crate) const fn to_collator_options(self) -> (Strength, Option<CaseLevel>) {
        match self {
            Sensitivity::Base => (Strength::Primary, None),
            Sensitivity::Accent => (Strength::Secondary, None),
            Sensitivity::Case => (Strength::Secondary, Some(CaseLevel::On)),
            Sensitivity::Variant => (Strength::Tertiary, None),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ParseSensitivityError;

impl std::fmt::Display for ParseSensitivityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "provided string was not `base`, `accent`, `case` or `variant`".fmt(f)
    }
}

impl FromStr for Sensitivity {
    type Err = ParseSensitivityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "base" => Ok(Self::Base),
            "accent" => Ok(Self::Accent),
            "case" => Ok(Self::Case),
            "variant" => Ok(Self::Variant),
            _ => Err(ParseSensitivityError),
        }
    }
}

impl OptionTypeParsable for Sensitivity {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Usage {
    #[default]
    Sort,
    Search,
}

#[derive(Debug)]
pub(crate) struct ParseUsageError;

impl std::fmt::Display for ParseUsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "provided string was not `sort` or `search`".fmt(f)
    }
}

impl FromStr for Usage {
    type Err = ParseUsageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sort" => Ok(Self::Sort),
            "search" => Ok(Self::Search),
            _ => Err(ParseUsageError),
        }
    }
}

impl OptionTypeParsable for Usage {}
