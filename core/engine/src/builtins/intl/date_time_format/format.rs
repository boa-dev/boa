use icu_datetime::{
    fieldsets::builder::{DateFields, ZoneStyle},
    options::{Length, SubsecondDigits as IcuSubsecondDigits, TimePrecision},
    preferences::HourCycle,
};

use crate::{
    Context, JsObject, JsResult,
    builtins::{
        intl::{date_time_format::FormatType, options::get_number_option},
        options::{OptionType, get_option},
    },
    js_error, js_string,
};

pub(super) struct FormatOptions {
    _hour_cycle: Option<HourCycle>,                    // -> ???
    week_day: Option<WeekDay>,                         // e -> Maps to DateField
    era: Option<Era>,                                  // G -> Maps to YearStyle
    year: Option<Year>,                                // Y -> Maps to DateField
    month: Option<Month>,                              // M -> Maps to DateField
    day: Option<Day>,                                  // D -> Maps to DateField
    day_period: Option<DayPeriod>,                     // a -> ???
    hour: Option<Hour>,                                // Maps to TimePrecision
    minute: Option<Minute>,                            // Maps to TimePrecision
    second: Option<Second>,                            // Maps to TimePrecision
    fractional_second_digits: Option<SubsecondDigits>, // Maps to TimePrecision
    time_zone_name: Option<TimeZoneName>,              // Maps to ZoneStyle
}

impl FormatOptions {
    pub(super) fn try_init(
        options: &JsObject,
        hour_cycle: Option<HourCycle>, // TODO: Is option correct?
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
            get_number_option(options, js_string!("fractionalSecondDigits"), 1, 3, context)?
                .map(SubsecondDigits::from_i32);
        let time_zone_name =
            get_option::<TimeZoneName>(options, js_string!("timeZoneName"), context)?;

        Ok(Self {
            _hour_cycle: hour_cycle,
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

    pub(super) fn set_date_defaults(&mut self) {
        self.year = Some(Year::Numeric);
        self.month = Some(Month::Numeric);
        self.day = Some(Day::Numeric);
    }

    pub(super) fn set_time_defaults(&mut self) {
        self.hour = Some(Hour::Numeric);
        self.minute = Some(Minute::Numeric);
        self.second = Some(Second::Numeric);
    }

    pub(super) fn has_explicit_format_components(&self) -> bool {
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

    pub(super) fn check_dtf_type(&self, required: FormatType) -> bool {
        // a. Let needDefaults be true.
        // b. If required is date or any, then
        // i. For each property name prop of « "weekday", "year", "month", "day" », do
        // 1. Let value be formatOptions.[[<prop>]].
        // 2. If value is not undefined, set needDefaults to false.
        // c. If required is time or any, then
        // i. For each property name prop of « "dayPeriod", "hour", "minute", "second", "fractionalSecondDigits" », do
        // 1. Let value be formatOptions.[[<prop>]].
        // 2. If value is not undefined, set needDefaults to false.
        if required != FormatType::Time && self.has_date_defaults() {
            return false;
        }
        if required != FormatType::Date && self.has_time_defaults() {
            return false;
        }
        true
    }

    pub(super) fn has_date_defaults(&self) -> bool {
        self.week_day.is_some() || self.year.is_some() || self.month.is_some() || self.day.is_some()
    }

    pub(super) fn has_time_defaults(&self) -> bool {
        self.day_period.is_some()
            || self.hour.is_some()
            || self.minute.is_some()
            || self.second.is_some()
            || self.fractional_second_digits.is_some()
    }

    pub(super) fn to_length(&self) -> Option<Length> {
        match (self.month, self.week_day) {
            (Some(month), _) => Some(month.to_length()),
            (None, Some(week_day)) => Some(week_day.to_length()),
            _ => None,
        }
    }

    /// Convert the current `FormatOptions` to a [`DateFields`].
    pub(super) fn to_date_fields(&self) -> Option<DateFields> {
        match (self.year, self.month, self.day, self.week_day) {
            (Some(_y), _m, _d, Some(_e)) => Some(DateFields::YMDE),
            (Some(_y), _m, Some(_d), None) => Some(DateFields::YMD),
            (Some(_y), Some(_m), None, None) => Some(DateFields::YM),
            (Some(_y), None, None, None) => Some(DateFields::Y),
            (None, Some(_m), _d, Some(_e)) => Some(DateFields::MDE),
            (None, Some(_m), Some(_d), None) => Some(DateFields::MD),
            (None, Some(_m), None, None) => Some(DateFields::M),
            (None, None, Some(_d), Some(_e)) => Some(DateFields::DE),
            (None, None, Some(_d), None) => Some(DateFields::D),
            (None, None, None, Some(_e)) => Some(DateFields::E),
            _ => None,
        }
    }

    /// Convert the current `FormatOptions` to a [`TimePrecision`].
    pub(super) fn to_time_fields(&self) -> Option<TimePrecision> {
        match (
            self.hour,
            self.minute,
            self.second,
            self.fractional_second_digits,
        ) {
            (_h, _m, _s, Some(digits)) => Some(TimePrecision::Subsecond(digits.into())),
            (_h, _m, Some(_s), None) => Some(TimePrecision::Second),
            (_h, Some(_m), None, None) => Some(TimePrecision::Minute),
            (Some(_h), None, None, None) => Some(TimePrecision::Hour),
            _ => None,
        }
    }

    /// Convert the current `FormatOptions` to a [`ZoneStyle`].
    pub(super) fn to_zone_style(&self) -> Option<ZoneStyle> {
        self.time_zone_name.map(TimeZoneName::to_zone_style)
    }
}

// ==== Format Options ====

#[derive(Debug, Clone, Copy)]
pub(crate) enum WeekDay {
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

impl WeekDay {
    pub(crate) fn to_length(self) -> Length {
        match self {
            Self::Long => Length::Long,
            Self::Short => Length::Medium,
            Self::Narrow => Length::Short,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Era {
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum Year {
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum Month {
    TwoDigit,
    Numeric,
    Narrow,
    Short,
    Long,
}

impl Month {
    pub(crate) fn to_length(self) -> Length {
        // NOTE (nekevss): after a brief glance, nnarrow does not appear to be
        // currently supported by ICU4X ... TBD
        match self {
            Self::Long => Length::Long,
            Self::Short => Length::Medium,
            _ => Length::Short,
        }
    }
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum Day {
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum DayPeriod {
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum Hour {
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum Minute {
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum Second {
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum SubsecondDigits {
    S1,
    S2,
    S3,
}

impl SubsecondDigits {
    fn from_i32(i: i32) -> SubsecondDigits {
        match i {
            1 => SubsecondDigits::S1,
            2 => SubsecondDigits::S2,
            3 => SubsecondDigits::S3,
            _ => unreachable!("subSecondDigits must be previously constrained."),
        }
    }
}

impl From<SubsecondDigits> for IcuSubsecondDigits {
    fn from(value: SubsecondDigits) -> Self {
        match value {
            SubsecondDigits::S1 => IcuSubsecondDigits::S1,
            SubsecondDigits::S2 => IcuSubsecondDigits::S2,
            SubsecondDigits::S3 => IcuSubsecondDigits::S3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TimeZoneName {
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

impl TimeZoneName {
    fn to_zone_style(self) -> ZoneStyle {
        match self {
            TimeZoneName::LongGeneric => ZoneStyle::GenericLong,
            TimeZoneName::ShortGeneric => ZoneStyle::GenericShort,
            TimeZoneName::LongOffset => ZoneStyle::LocalizedOffsetLong,
            TimeZoneName::ShortOffset => ZoneStyle::LocalizedOffsetShort,
            TimeZoneName::Long => ZoneStyle::SpecificLong,
            TimeZoneName::Short => ZoneStyle::SpecificShort,
        }
    }
}
