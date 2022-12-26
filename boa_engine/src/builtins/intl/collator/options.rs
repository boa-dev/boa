use std::str::FromStr;

use icu_collator::{CaseLevel, Strength};

use crate::builtins::intl::options::OptionTypeParsable;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Sensitivity {
    Base,
    Accent,
    Case,
    Variant,
}

impl Sensitivity {
    /// Converts the sensitivity option to the equivalent ICU4X collator options.
    pub(crate) const fn to_collator_options(self) -> (Strength, CaseLevel) {
        match self {
            Sensitivity::Base => (Strength::Primary, CaseLevel::Off),
            Sensitivity::Accent => (Strength::Secondary, CaseLevel::Off),
            Sensitivity::Case => (Strength::Primary, CaseLevel::On),
            Sensitivity::Variant => (Strength::Tertiary, CaseLevel::On),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ParseSensitivityError;

impl std::fmt::Display for ParseSensitivityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not `base`, `accent`, `case` or `variant`")
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
pub(crate) enum Usage {
    #[default]
    Sort,
    Search,
}

#[derive(Debug)]
pub(crate) struct ParseUsageError;

impl std::fmt::Display for ParseUsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("provided string was not `sort` or `search`")
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
