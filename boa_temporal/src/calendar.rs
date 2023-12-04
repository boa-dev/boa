//! Temporal calendar traits and implementations.
//!
//! The goal of the calendar module of `boa_temporal` is to provide
//! Temporal compatible calendar implementations.
//!
//! The implementation will only be of calendar's prexisting calendars. This library
//! does not come with a pre-existing `CustomCalendar` (i.e., an object that implements
//! the calendar protocol), but it does aim to provide the necessary tools and API for
//! implementing one.

use std::{any::Any, str::FromStr};

use crate::{
    date::Date,
    datetime::DateTime,
    duration::Duration,
    fields::TemporalFields,
    iso::{IsoDate, IsoDateSlots},
    month_day::MonthDay,
    options::{ArithmeticOverflow, TemporalUnit},
    year_month::YearMonth,
    TemporalError, TemporalResult,
};

use tinystr::TinyAsciiStr;

use self::iso::IsoCalendar;

pub mod iso;

/// The ECMAScript defined protocol methods
pub const CALENDAR_PROTOCOL_METHODS: [&str; 21] = [
    "dateAdd",
    "dateFromFields",
    "dateUntil",
    "day",
    "dayOfWeek",
    "dayOfYear",
    "daysInMonth",
    "daysInWeek",
    "daysInYear",
    "fields",
    "id",
    "inLeapYear",
    "mergeFields",
    "month",
    "monthCode",
    "monthDayFromFields",
    "monthsInYear",
    "weekOfYear",
    "year",
    "yearMonthFromFields",
    "yearOfWeek",
];

/// Designate the type of `CalendarFields` needed
#[derive(Debug, Clone, Copy)]
pub enum CalendarFieldsType {
    /// Whether the Fields should return for a Date.
    Date,
    /// Whether the Fields should return for a YearMonth.
    YearMonth,
    /// Whether the Fields should return for a MonthDay.
    MonthDay,
}

// TODO: Optimize to TinyStr or &str.
impl From<&[String]> for CalendarFieldsType {
    fn from(value: &[String]) -> Self {
        let year_present = value.contains(&"year".to_owned());
        let day_present = value.contains(&"day".to_owned());

        if year_present && day_present {
            CalendarFieldsType::Date
        } else if year_present {
            CalendarFieldsType::YearMonth
        } else {
            CalendarFieldsType::MonthDay
        }
    }
}

/// `AvailableCalendars` lists the currently implemented `CalendarProtocols`
#[derive(Debug, Clone, Copy)]
pub enum AvailableCalendars {
    /// The ISO8601 calendar.
    Iso,
}

// NOTE: Should `s` be forced to lowercase or should the user be expected to provide the lowercase.
impl FromStr for AvailableCalendars {
    type Err = TemporalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "iso8601" => Ok(Self::Iso),
            _ => {
                Err(TemporalError::range().with_message("CalendarId is not an available Calendar"))
            }
        }
    }
}

impl AvailableCalendars {
    /// Returns the `CalendarProtocol` for the `AvailableCalendar`
    #[must_use]
    pub fn to_protocol(&self) -> Box<dyn CalendarProtocol> {
        match self {
            Self::Iso => Box::new(IsoCalendar),
        }
    }
}

/// The `DateLike` objects that can be provided to the `CalendarProtocol`.
#[derive(Debug)]
pub enum CalendarDateLike {
    /// Represents a `Date` datelike
    Date(Date),
    /// Represents a `DateTime` datelike
    DateTime(DateTime),
    /// Represents a `YearMonth` datelike
    YearMonth(YearMonth),
    /// Represents a `MonthDay` datelike
    MonthDay(MonthDay),
}

impl CalendarDateLike {
    /// Retrieves the internal `IsoDate` field.
    #[inline]
    #[must_use]
    pub fn as_iso_date(&self) -> IsoDate {
        match self {
            CalendarDateLike::Date(d) => d.iso_date(),
            CalendarDateLike::DateTime(dt) => dt.iso_date(),
            CalendarDateLike::MonthDay(md) => md.iso_date(),
            CalendarDateLike::YearMonth(ym) => ym.iso_date(),
        }
    }
}

// ==== CalendarProtocol trait ====

/// The `CalendarProtocol`'s Clone supertrait.
pub trait CalendarProtocolClone {
    /// Clone's the current `CalendarProtocol`
    fn clone_box(&self) -> Box<dyn CalendarProtocol>;
}

impl<P> CalendarProtocolClone for P
where
    P: 'static + CalendarProtocol + Clone,
{
    fn clone_box(&self) -> Box<dyn CalendarProtocol> {
        Box::new(self.clone())
    }
}

// TODO: Split further into `CalendarProtocol` and `BuiltinCalendar` to better handle
// fields and mergeFields.
/// A trait for implementing a Builtin Calendar's Calendar Protocol in Rust.
pub trait CalendarProtocol: CalendarProtocolClone {
    /// Creates a `Temporal.PlainDate` object from provided fields.
    fn date_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<Date>;
    /// Creates a `Temporal.PlainYearMonth` object from the provided fields.
    fn year_month_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<YearMonth>;
    /// Creates a `Temporal.PlainMonthDay` object from the provided fields.
    fn month_day_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<MonthDay>;
    /// Returns a `Temporal.PlainDate` based off an added date.
    fn date_add(
        &self,
        date: &Date,
        duration: &Duration,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<Date>;
    /// Returns a `Temporal.Duration` representing the duration between two dates.
    fn date_until(
        &self,
        one: &Date,
        two: &Date,
        largest_unit: TemporalUnit,
        context: &mut dyn Any,
    ) -> TemporalResult<Duration>;
    /// Returns the era for a given `temporaldatelike`.
    fn era(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<Option<TinyAsciiStr<8>>>;
    /// Returns the era year for a given `temporaldatelike`
    fn era_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<Option<i32>>;
    /// Returns the `year` for a given `temporaldatelike`
    fn year(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<i32>;
    /// Returns the `month` for a given `temporaldatelike`
    fn month(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<u8>;
    // Note: Best practice would probably be to switch to a MonthCode enum after extraction.
    /// Returns the `monthCode` for a given `temporaldatelike`
    fn month_code(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<TinyAsciiStr<4>>;
    /// Returns the `day` for a given `temporaldatelike`
    fn day(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<u8>;
    /// Returns a value representing the day of the week for a date.
    fn day_of_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16>;
    /// Returns a value representing the day of the year for a given calendar.
    fn day_of_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16>;
    /// Returns a value representing the week of the year for a given calendar.
    fn week_of_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16>;
    /// Returns the year of a given week.
    fn year_of_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32>;
    /// Returns the days in a week for a given calendar.
    fn days_in_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16>;
    /// Returns the days in a month for a given calendar.
    fn days_in_month(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16>;
    /// Returns the days in a year for a given calendar.
    fn days_in_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16>;
    /// Returns the months in a year for a given calendar.
    fn months_in_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16>;
    /// Returns whether a value is within a leap year according to the designated calendar.
    fn in_leap_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<bool>;
    /// Resolve the `TemporalFields` for the implemented Calendar
    fn resolve_fields(
        &self,
        fields: &mut TemporalFields,
        r#type: CalendarFieldsType,
    ) -> TemporalResult<()>;
    /// Return this calendar's a fieldName and whether it is required depending on type (date, day-month).
    fn field_descriptors(&self, r#type: CalendarFieldsType) -> Vec<(String, bool)>;
    /// Return the fields to ignore for this Calendar based on provided keys.
    fn field_keys_to_ignore(&self, additional_keys: Vec<String>) -> Vec<String>;
    /// Debug name
    fn identifier(&self, context: &mut dyn Any) -> TemporalResult<String>;
}

impl core::fmt::Debug for dyn CalendarProtocol {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            self.identifier(&mut ()).unwrap_or_default().as_str()
        )
    }
}

/// The `[[Calendar]]` field slot of a Temporal Object.
#[derive(Debug)]
pub enum CalendarSlot {
    /// The calendar identifier string.
    Identifier(String),
    /// A `CalendarProtocol` implementation.
    Protocol(Box<dyn CalendarProtocol>),
}

impl Clone for CalendarSlot {
    fn clone(&self) -> Self {
        match self {
            Self::Identifier(s) => Self::Identifier(s.clone()),
            Self::Protocol(b) => Self::Protocol(b.clone_box()),
        }
    }
}

impl Clone for Box<dyn CalendarProtocol + 'static> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Default for CalendarSlot {
    fn default() -> Self {
        Self::Identifier("iso8601".to_owned())
    }
}

// TODO: Handle `CalendarFields` and `CalendarMergeFields`
impl CalendarSlot {
    /// `CalendarDateAdd`
    ///
    /// TODO: More Docs
    pub fn date_add(
        &self,
        date: &Date,
        duration: &Duration,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<Date> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.date_add(date, duration, overflow, context)
            }
            Self::Protocol(protocol) => protocol.date_add(date, duration, overflow, context),
        }
    }

    /// `CalendarDateUntil`
    ///
    /// TODO: More Docs
    pub fn date_until(
        &self,
        one: &Date,
        two: &Date,
        largest_unit: TemporalUnit,
        context: &mut dyn Any,
    ) -> TemporalResult<Duration> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.date_until(one, two, largest_unit, context)
            }
            Self::Protocol(protocol) => protocol.date_until(one, two, largest_unit, context),
        }
    }

    /// `CalendarYear`
    ///
    /// TODO: More docs.
    pub fn year(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<i32> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.year(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.year(date_like, context),
        }
    }

    /// `CalendarMonth`
    ///
    /// TODO: More docs.
    pub fn month(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<u8> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.month(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.month(date_like, context),
        }
    }

    /// `CalendarMonthCode`
    ///
    /// TODO: More docs.
    pub fn month_code(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<TinyAsciiStr<4>> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.month_code(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.month_code(date_like, context),
        }
    }

    /// `CalendarDay`
    ///
    /// TODO: More docs.
    pub fn day(&self, date_like: &CalendarDateLike, context: &mut dyn Any) -> TemporalResult<u8> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.day(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.day(date_like, context),
        }
    }

    /// `CalendarDayOfWeek`
    ///
    /// TODO: More docs.
    pub fn day_of_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.day_of_week(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.day_of_week(date_like, context),
        }
    }

    /// `CalendarDayOfYear`
    ///
    /// TODO: More docs.
    pub fn day_of_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.day_of_year(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.day_of_year(date_like, context),
        }
    }

    /// `CalendarWeekOfYear`
    ///
    /// TODO: More docs.
    pub fn week_of_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.week_of_year(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.week_of_year(date_like, context),
        }
    }

    /// `CalendarYearOfWeek`
    ///
    /// TODO: More docs.
    pub fn year_of_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.year_of_week(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.year_of_week(date_like, context),
        }
    }

    /// `CalendarDaysInWeek`
    ///
    /// TODO: More docs.
    pub fn days_in_week(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.days_in_week(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.days_in_week(date_like, context),
        }
    }

    /// `CalendarDaysInMonth`
    ///
    /// TODO: More docs.
    pub fn days_in_month(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.days_in_month(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.days_in_month(date_like, context),
        }
    }

    /// `CalendarDaysInYear`
    ///
    /// TODO: More docs.
    pub fn days_in_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.days_in_year(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.days_in_year(date_like, context),
        }
    }

    /// `CalendarMonthsInYear`
    ///
    /// TODO: More docs.
    pub fn months_in_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.months_in_year(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.months_in_year(date_like, context),
        }
    }

    /// `CalendarInLeapYear`
    ///
    /// TODO: More docs.
    pub fn in_leap_year(
        &self,
        date_like: &CalendarDateLike,
        context: &mut dyn Any,
    ) -> TemporalResult<bool> {
        match self {
            Self::Identifier(id) => {
                let protocol = AvailableCalendars::from_str(id)?.to_protocol();
                protocol.in_leap_year(date_like, &mut ())
            }
            Self::Protocol(protocol) => protocol.in_leap_year(date_like, context),
        }
    }
}
