use crate::{builtins::options::OptionType, js_error};

pub(crate) enum HourCycle {
    H11,
    H12,
    H23,
    H24,
}

impl HourCycle {
    pub(crate) fn as_utf8(&self) -> &[u8] {
        match self {
            Self::H11 => "h11".as_bytes(),
            Self::H12 => "h12".as_bytes(),
            Self::H23 => "h23".as_bytes(),
            Self::H24 => "h24".as_bytes(),
        }
    }
}

impl OptionType for HourCycle {
    fn from_value(value: crate::JsValue, context: &mut crate::Context) -> crate::JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "h11" => Ok(Self::H11),
            "h12" => Ok(Self::H12),
            "h23" => Ok(Self::H23),
            "h24" => Ok(Self::H24),
            _ => Err(js_error!(RangeError: "unknown hourCycle option")),
        }
    }
}

pub(super) enum FormatMatcher {
    Basic,
    BestFit,
}

impl OptionType for FormatMatcher {
    fn from_value(value: crate::JsValue, context: &mut crate::Context) -> crate::JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "basic" => Ok(Self::Basic),
            "best fit" => Ok(Self::BestFit),
            _ => Err(js_error!(RangeError: "unknown formatMatcher option")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum DateStyle {
    Full,
    Long,
    Medium,
    Short,
}

impl OptionType for DateStyle {
    fn from_value(value: crate::JsValue, context: &mut crate::Context) -> crate::JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "full" => Ok(Self::Full),
            "long" => Ok(Self::Long),
            "medium" => Ok(Self::Medium),
            "short" => Ok(Self::Short),
            _ => Err(js_error!(RangeError: "unknown dateStyle option")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum TimeStyle {
    Full,
    Long,
    Medium,
    Short,
}

impl OptionType for TimeStyle {
    fn from_value(value: crate::JsValue, context: &mut crate::Context) -> crate::JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "full" => Ok(Self::Full),
            "long" => Ok(Self::Long),
            "medium" => Ok(Self::Medium),
            "short" => Ok(Self::Short),
            _ => Err(js_error!(RangeError: "unknown timeStyle option")),
        }
    }
}
