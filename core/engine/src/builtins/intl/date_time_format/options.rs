use icu_datetime::preferences::{CalendarAlgorithm, HourCycle as IcuHourCycle};
use icu_locale::extensions::unicode::Value;

use crate::{builtins::options::OptionType, js_error, Context, JsNativeError, JsResult, JsValue, JsError};

pub(crate) enum HourCycle {
    H11,
    H12,
    H23,
    H24,
}

impl OptionType for HourCycle {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "h11" => Ok(Self::H11),
            "h12" => Ok(Self::H12),
            "h23" => Ok(Self::H23),
            "h24" => Ok(Self::H24),
            _ => Err(js_error!(RangeError: "unknown hourCycle option")),
        }
    }
}

impl TryFrom<HourCycle> for IcuHourCycle {
    type Error = JsError;
    fn try_from(hc: HourCycle) -> Result<Self, Self::Error> {
        match hc {
            HourCycle::H11 => Ok(IcuHourCycle::H11),
            HourCycle::H12 => Ok(IcuHourCycle::H12),
            HourCycle::H23 => Ok(IcuHourCycle::H23),
            // TODO: Work on support for H24, potentially remove depending on fate
            // of H24 option.
            HourCycle::H24 => Err(js_error!(RangeError: "h24 not currently supported.")),
        }
    }
}

pub(super) enum FormatMatcher {
    Basic,
    BestFit,
}

impl OptionType for FormatMatcher {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
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
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
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
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "full" => Ok(Self::Full),
            "long" => Ok(Self::Long),
            "medium" => Ok(Self::Medium),
            "short" => Ok(Self::Short),
            _ => Err(js_error!(RangeError: "unknown timeStyle option")),
        }
    }
}

impl OptionType for CalendarAlgorithm {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        let s = value.to_string(context)?.to_std_string_escaped();
        Value::try_from_str(&s)
            .ok()
            .and_then(|v| CalendarAlgorithm::try_from(&v).ok())
            .ok_or_else(|| {
                JsNativeError::range()
                    .with_message(format!("provided calendar `{s}` is invalid"))
                    .into()
            })
    }
}

// TODO: track https://github.com/unicode-org/icu4x/issues/6597 and
// https://github.com/tc39/ecma402/issues/1002 for resolution on
// `HourCycle::H24`.
impl OptionType for IcuHourCycle {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_str() {
            "h11" => Ok(IcuHourCycle::H11),
            "h12" => Ok(IcuHourCycle::H12),
            "h23" => Ok(IcuHourCycle::H23),
            _ => Err(js_error!(RangeError: "provided hour cycle was not `h11`, `h12` or `h23`"))
        }
    }
}
