use icu_datetime::preferences::HourCycle;

use crate::{
    Context, JsObject, JsResult,
    builtins::{
        intl::options::get_number_option,
        options::{OptionType, get_option},
    },
    js_error, js_string,
};

pub struct FormatOptions {
    hour_cyle: Option<HourCycle>,
    week_day: Option<WeekDay>,
    era: Option<Era>,
    year: Option<Year>,
    month: Option<Month>,
    day: Option<Day>,
    day_period: Option<DayPeriod>,
    hour: Option<Hour>,
    minute: Option<Minute>,
    second: Option<Second>,
    fractional_second_digits: Option<i32>,
    time_zone_name: Option<TimeZoneName>,
}

impl FormatOptions {
    pub fn try_init(
        options: &JsObject,
        hour_cyle: Option<HourCycle>, // TODO: Is option correct?
        context: &mut Context,
    ) -> JsResult<Self> {
        // Below is adapted and inlined from Step 24 of `CreateDateTimeFormat`
        let week_day = get_option::<WeekDay>(options, js_string!("weekDay"), context)?;
        let era = get_option::<Era>(options, js_string!("era"), context)?;
        let year = get_option::<Year>(options, js_string!("year"), context)?;
        let month = get_option::<Month>(options, js_string!("month"), context)?;
        let day = get_option::<Day>(options, js_string!("day"), context)?;
        let day_period = get_option::<DayPeriod>(options, js_string!("dayPeriod"), context)?;
        let hour = get_option::<Hour>(options, js_string!("hour"), context)?;
        let minute = get_option::<Minute>(options, js_string!("minute"), context)?;
        let second = get_option::<Second>(options, js_string!("second"), context)?;
        let fractional_second_digits =
            get_number_option(options, js_string!("fractionalSecondDigits"), 1, 3, context)?;
        let time_zone_name =
            get_option::<TimeZoneName>(options, js_string!("timeZoneName"), context)?;

        Ok(Self {
            hour_cyle,
            week_day,
            era,
            year,
            month,
            day,
            day_period,
            hour,
            minute,
            second,
            fractional_second_digits,
            time_zone_name,
        })
    }

    pub fn has_explicit_format_components(&self) -> bool {
        match self {
            Self {
                week_day: None,
                era: None,
                year: None,
                month: None,
                day: None,
                day_period: None,
                hour: None,
                minute: None,
                second: None,
                fractional_second_digits: None,
                time_zone_name: None,
                ..
            } => false,
            // If any of the format fields is Some(_), return true
            _ => true,
        }
    }
}

// ==== Format Options ====

// General thought: thse could probably be moved to `options.rs`

pub enum WeekDay {
    Narrow,
    Short,
    Long,
}

impl OptionType for WeekDay {
    fn from_value(value: crate::JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "narrow" => Ok(Self::Narrow),
            "short" => Ok(Self::Short),
            "long" => Ok(Self::Long),
            _ => Err(js_error!(RangeError: "unknown weekDay option")),
        }
    }
}

pub enum Era {
    Narrow,
    Short,
    Long,
}

impl OptionType for Era {
    fn from_value(value: crate::JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "narrow" => Ok(Self::Narrow),
            "short" => Ok(Self::Short),
            "long" => Ok(Self::Long),
            _ => Err(js_error!(RangeError: "unknown era option")),
        }
    }
}

pub enum Year {
    TwoDigit,
    Numeric,
}

impl OptionType for Year {
    fn from_value(value: crate::JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "2-digit" => Ok(Self::TwoDigit),
            "numeric" => Ok(Self::Numeric),
            _ => Err(js_error!(RangeError: "unknown year option")),
        }
    }
}

pub enum Month {
    TwoDigit,
    Numeric,
    Narrow,
    Short,
    Long,
}

impl OptionType for Month {
    fn from_value(value: crate::JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "2-digit" => Ok(Self::TwoDigit),
            "numeric" => Ok(Self::Numeric),
            "narrow" => Ok(Self::Narrow),
            "short" => Ok(Self::Short),
            "long" => Ok(Self::Long),
            _ => Err(js_error!(RangeError: "unknown month option")),
        }
    }
}

pub enum Day {
    TwoDigit,
    Numeric,
}

impl OptionType for Day {
    fn from_value(value: crate::JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "2-digit" => Ok(Self::TwoDigit),
            "numeric" => Ok(Self::Numeric),
            _ => Err(js_error!(RangeError: "unknown day option")),
        }
    }
}

pub enum DayPeriod {
    Narrow,
    Short,
    Long,
}

impl OptionType for DayPeriod {
    fn from_value(value: crate::JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "narrow" => Ok(Self::Narrow),
            "short" => Ok(Self::Short),
            "long" => Ok(Self::Long),
            _ => Err(js_error!(RangeError: "unknown dayPeriod option")),
        }
    }
}

pub enum Hour {
    TwoDigit,
    Numeric,
}

impl OptionType for Hour {
    fn from_value(value: crate::JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "2-digit" => Ok(Self::TwoDigit),
            "numeric" => Ok(Self::Numeric),
            _ => Err(js_error!(RangeError: "unknown hour option")),
        }
    }
}

pub enum Minute {
    TwoDigit,
    Numeric,
}

impl OptionType for Minute {
    fn from_value(value: crate::JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "2-digit" => Ok(Self::TwoDigit),
            "numeric" => Ok(Self::Numeric),
            _ => Err(js_error!(RangeError: "unknown minute option")),
        }
    }
}

pub enum Second {
    TwoDigit,
    Numeric,
}

impl OptionType for Second {
    fn from_value(value: crate::JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "2-digit" => Ok(Self::TwoDigit),
            "numeric" => Ok(Self::Numeric),
            _ => Err(js_error!(RangeError: "unknown second option")),
        }
    }
}

pub enum TimeZoneName {
    Short,
    Long,
    ShortOffset,
    LongOffset,
    ShortGeneric,
    LongGeneric,
}

impl OptionType for TimeZoneName {
    fn from_value(value: crate::JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_ref() {
            "short" => Ok(Self::Short),
            "long" => Ok(Self::Long),
            "shortOffset" => Ok(Self::ShortOffset),
            "longOffset" => Ok(Self::LongOffset),
            "shortGeneric" => Ok(Self::ShortGeneric),
            "longGeneric" => Ok(Self::LongGeneric),
            _ => Err(js_error!(RangeError: "unknown timeZoneName option")),
        }
    }
}
