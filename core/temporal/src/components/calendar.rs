//! This module implements the calendar traits and related components.
//!
//! The goal of the calendar module of `boa_temporal` is to provide
//! Temporal compatible calendar implementations.
//!
//! The implementation will only be of calendar's prexisting calendars. This library
//! does not come with a pre-existing `CustomCalendar` (i.e., an object that implements
//! the calendar protocol), but it does aim to provide the necessary tools and API for
//! implementing one.

use std::str::FromStr;

use crate::{
    components::{Date, DateTime, Duration, MonthDay, YearMonth},
    iso::{IsoDate, IsoDateSlots},
    options::{ArithmeticOverflow, TemporalUnit},
    TemporalError, TemporalFields, TemporalResult,
};

use icu_calendar::{
    types::{Era, MonthCode},
    week::{RelativeUnit, WeekCalculator},
    AnyCalendar, AnyCalendarKind, Calendar, Iso,
};
use tinystr::TinyAsciiStr;

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

// NOTE (nekevss): May be worth switching the below to "Custom" `DateLikes`, and
// allow the non-custom to be engine specific types.
//
// enum CalendarDateLike<C: CalendarProtocol, D: DateTypes<C>> {
//   Date(Date<C>),
//   CustomDate(D::Date),
//   ...
// }
/// The `DateLike` objects that can be provided to the `CalendarProtocol`.
#[derive(Debug)]
pub enum CalendarDateLike<C: CalendarProtocol> {
    /// Represents a user-defined `Date` datelike
    CustomDate(C::Date),
    /// Represents a user-defined `DateTime` datelike
    CustomDateTime(C::DateTime),
    /// Represents a user-defined `YearMonth` datelike
    CustomYearMonth(C::YearMonth),
    /// Represents a user-defined `MonthDay` datelike
    CustomMonthDay(C::MonthDay),
    /// Represents a `DateTime<C>`.
    DateTime(DateTime<C>),
    /// Represents a `Date<C>`.
    Date(Date<C>),
}

impl<C: CalendarProtocol> CalendarDateLike<C> {
    /// Retrieves the internal `IsoDate` field.
    #[inline]
    #[must_use]
    pub fn as_iso_date(&self) -> IsoDate {
        match self {
            CalendarDateLike::CustomDate(d) => d.iso_date(),
            CalendarDateLike::CustomMonthDay(md) => md.iso_date(),
            CalendarDateLike::CustomYearMonth(ym) => ym.iso_date(),
            CalendarDateLike::CustomDateTime(dt) => dt.iso_date(),
            CalendarDateLike::DateTime(dt) => dt.iso_date(),
            CalendarDateLike::Date(d) => d.iso_date(),
        }
    }
}

// ==== CalendarProtocol trait ====

/// A trait for implementing a Builtin Calendar's Calendar Protocol in Rust.
pub trait CalendarProtocol: Clone {
    /// A Custom `Date` Type for an associated `CalendarProtocol`. Default `Date<C>`
    type Date: IsoDateSlots + GetCalendarSlot<Self> + Clone + core::fmt::Debug;
    /// A Custom `DateTime` Type for an associated `CalendarProtocol`. Default `DateTime<C>`
    type DateTime: IsoDateSlots + GetCalendarSlot<Self> + Clone + core::fmt::Debug;
    /// A Custom `YearMonth` Type for an associated `CalendarProtocol`. Default `YearMonth<C>`
    type YearMonth: IsoDateSlots + GetCalendarSlot<Self> + Clone + core::fmt::Debug;
    /// A Custom `MonthDay` Type for an associated `CalendarProtocol`. Default `MonthDay<C>`
    type MonthDay: IsoDateSlots + GetCalendarSlot<Self> + Clone + core::fmt::Debug;
    /// The context passed to every method of the `CalendarProtocol`.
    type Context;

    /// Creates a `Temporal.PlainDate` object from provided fields.
    fn date_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut Self::Context,
    ) -> TemporalResult<Date<Self>>;
    /// Creates a `Temporal.PlainYearMonth` object from the provided fields.
    fn year_month_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut Self::Context,
    ) -> TemporalResult<YearMonth<Self>>;
    /// Creates a `Temporal.PlainMonthDay` object from the provided fields.
    fn month_day_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut Self::Context,
    ) -> TemporalResult<MonthDay<Self>>;
    /// Returns a `Temporal.PlainDate` based off an added date.
    fn date_add(
        &self,
        date: &Date<Self>,
        duration: &Duration,
        overflow: ArithmeticOverflow,
        context: &mut Self::Context,
    ) -> TemporalResult<Date<Self>>;
    /// Returns a `Temporal.Duration` representing the duration between two dates.
    fn date_until(
        &self,
        one: &Date<Self>,
        two: &Date<Self>,
        largest_unit: TemporalUnit,
        context: &mut Self::Context,
    ) -> TemporalResult<Duration>;
    /// Returns the era for a given `temporaldatelike`.
    fn era(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<Option<TinyAsciiStr<16>>>;
    /// Returns the era year for a given `temporaldatelike`
    fn era_year(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<Option<i32>>;
    /// Returns the `year` for a given `temporaldatelike`
    fn year(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<i32>;
    /// Returns the `month` for a given `temporaldatelike`
    fn month(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<u8>;
    // Note: Best practice would probably be to switch to a MonthCode enum after extraction.
    /// Returns the `monthCode` for a given `temporaldatelike`
    fn month_code(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<TinyAsciiStr<4>>;
    /// Returns the `day` for a given `temporaldatelike`
    fn day(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<u8>;
    /// Returns a value representing the day of the week for a date.
    fn day_of_week(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<u16>;
    /// Returns a value representing the day of the year for a given calendar.
    fn day_of_year(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<u16>;
    /// Returns a value representing the week of the year for a given calendar.
    fn week_of_year(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<u16>;
    /// Returns the year of a given week.
    fn year_of_week(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<i32>;
    /// Returns the days in a week for a given calendar.
    fn days_in_week(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<u16>;
    /// Returns the days in a month for a given calendar.
    fn days_in_month(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<u16>;
    /// Returns the days in a year for a given calendar.
    fn days_in_year(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<u16>;
    /// Returns the months in a year for a given calendar.
    fn months_in_year(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<u16>;
    /// Returns whether a value is within a leap year according to the designated calendar.
    fn in_leap_year(
        &self,
        date_like: &CalendarDateLike<Self>,
        context: &mut Self::Context,
    ) -> TemporalResult<bool>;
    /// Return the fields for a value.
    fn fields(
        &self,
        fields: Vec<String>,
        context: &mut Self::Context,
    ) -> TemporalResult<Vec<String>>;
    /// Merge fields based on the calendar and provided values.
    fn merge_fields(
        &self,
        fields: &TemporalFields,
        additional_fields: &TemporalFields,
        context: &mut Self::Context,
    ) -> TemporalResult<TemporalFields>;
    /// Debug name
    fn identifier(&self, context: &mut Self::Context) -> TemporalResult<String>;
}

/// A trait for retrieving an internal calendar slice.
pub trait GetCalendarSlot<C: CalendarProtocol> {
    /// Returns the `CalendarSlot<C>` value of the implementor.
    fn get_calendar(&self) -> CalendarSlot<C>;
}

// NOTE(nekevss): Builtin could be `Rc<AnyCalendar>`, but doing so may
// have an effect on the pattern matching for `CalendarSlot`'s methods.
/// The `[[Calendar]]` field slot of a Temporal Object.
#[derive(Debug)]
pub enum CalendarSlot<C: CalendarProtocol> {
    /// The calendar identifier string.
    Builtin(AnyCalendar),
    /// A `CalendarProtocol` implementation.
    Protocol(C),
}

impl<C: CalendarProtocol> Clone for CalendarSlot<C> {
    fn clone(&self) -> Self {
        match self {
            Self::Builtin(any) => {
                let clone = match any {
                    AnyCalendar::Buddhist(c) => AnyCalendar::Buddhist(*c),
                    AnyCalendar::Chinese(c) => AnyCalendar::Chinese(c.clone()),
                    AnyCalendar::Coptic(c) => AnyCalendar::Coptic(*c),
                    AnyCalendar::Dangi(c) => AnyCalendar::Dangi(c.clone()),
                    AnyCalendar::Ethiopian(c) => AnyCalendar::Ethiopian(*c),
                    AnyCalendar::Gregorian(c) => AnyCalendar::Gregorian(*c),
                    AnyCalendar::Hebrew(c) => AnyCalendar::Hebrew(c.clone()),
                    AnyCalendar::Indian(c) => AnyCalendar::Indian(*c),
                    AnyCalendar::IslamicCivil(c) => AnyCalendar::IslamicCivil(c.clone()),
                    AnyCalendar::IslamicObservational(c) => {
                        AnyCalendar::IslamicObservational(c.clone())
                    }
                    AnyCalendar::IslamicTabular(c) => AnyCalendar::IslamicTabular(c.clone()),
                    AnyCalendar::IslamicUmmAlQura(c) => AnyCalendar::IslamicUmmAlQura(c.clone()),
                    AnyCalendar::Iso(c) => AnyCalendar::Iso(*c),
                    AnyCalendar::Japanese(c) => AnyCalendar::Japanese(c.clone()),
                    AnyCalendar::JapaneseExtended(c) => AnyCalendar::JapaneseExtended(c.clone()),
                    AnyCalendar::Persian(c) => AnyCalendar::Persian(*c),
                    AnyCalendar::Roc(c) => AnyCalendar::Roc(*c),
                    _ => unimplemented!("There is a calendar that is missing a clone impl."),
                };
                Self::Builtin(clone)
            }

            Self::Protocol(proto) => CalendarSlot::Protocol(proto.clone()),
        }
    }
}

// `FromStr` essentially serves as a stand in for `IsBuiltinCalendar`.
impl<C: CalendarProtocol> FromStr for CalendarSlot<C> {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE(nekesss): Catch the iso identifier here, as `iso8601` is not a valid ID below.
        if s == "iso8601" {
            return Ok(CalendarSlot::Builtin(AnyCalendar::Iso(Iso)));
        }

        let Some(cal) = AnyCalendarKind::get_for_bcp47_bytes(s.as_bytes()) else {
            return Err(TemporalError::range().with_message("Not a builtin calendar."));
        };

        let any_calendar = AnyCalendar::new(cal);

        Ok(CalendarSlot::Builtin(any_calendar))
    }
}

impl<C: CalendarProtocol> Default for CalendarSlot<C> {
    fn default() -> Self {
        Self::Builtin(AnyCalendar::Iso(Iso))
    }
}

// ==== Public `CalendarSlot` methods ====

impl<C: CalendarProtocol> CalendarSlot<C> {
    /// Returns whether the current calendar is `ISO`
    pub fn is_iso(&self) -> bool {
        matches!(self, CalendarSlot::Builtin(AnyCalendar::Iso(_)))
    }
}

// ==== Abstract `CalendarProtocol` Methods ====

// NOTE: Below is functionally the `CalendarProtocol` implementation on `CalendarSlot`.

impl<C: CalendarProtocol> CalendarSlot<C> {
    /// `CalendarDateFromFields`
    pub fn date_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut C::Context,
    ) -> TemporalResult<Date<C>> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => {
                // Resolve month and monthCode;
                fields.iso_resolve_month()?;
                Date::new(
                    fields.year().unwrap_or(0),
                    fields.month().unwrap_or(0),
                    fields.day().unwrap_or(0),
                    self.clone(),
                    overflow,
                )
            }
            CalendarSlot::Builtin(builtin) => {
                // NOTE: This might preemptively throw as `ICU4X` does not support constraining.
                // Resolve month and monthCode;
                let calendar_date = builtin.date_from_codes(
                    Era::from(fields.era()),
                    fields.year().unwrap_or(0),
                    MonthCode(fields.month_code()),
                    fields.day().unwrap_or(0) as u8,
                )?;
                let iso = builtin.date_to_iso(&calendar_date);
                Date::new(
                    iso.year().number,
                    iso.month().ordinal as i32,
                    iso.day_of_month().0 as i32,
                    self.clone(),
                    overflow,
                )
            }
            CalendarSlot::Protocol(protocol) => {
                protocol.date_from_fields(fields, overflow, context)
            }
        }
    }

    /// `CalendarMonthDayFromFields`
    pub fn month_day_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut C::Context,
    ) -> TemporalResult<MonthDay<C>> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => {
                fields.iso_resolve_month()?;
                MonthDay::new(
                    fields.month().unwrap_or(0),
                    fields.day().unwrap_or(0),
                    self.clone(),
                    overflow,
                )
            }
            CalendarSlot::Builtin(_) => {
                // TODO: This may get complicated...
                // For reference: https://github.com/tc39/proposal-temporal/blob/main/polyfill/lib/calendar.mjs#L1275.
                Err(TemporalError::range().with_message("Not yet implemented/supported."))
            }
            CalendarSlot::Protocol(protocol) => {
                protocol.month_day_from_fields(fields, overflow, context)
            }
        }
    }

    /// `CalendarYearMonthFromFields`
    pub fn year_month_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        context: &mut C::Context,
    ) -> TemporalResult<YearMonth<C>> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => {
                fields.iso_resolve_month()?;
                YearMonth::new(
                    fields.year().unwrap_or(0),
                    fields.month().unwrap_or(0),
                    fields.day(),
                    self.clone(),
                    overflow,
                )
            }
            CalendarSlot::Builtin(builtin) => {
                // NOTE: This might preemptively throw as `ICU4X` does not support regulating.
                let calendar_date = builtin.date_from_codes(
                    Era::from(fields.era()),
                    fields.year().unwrap_or(0),
                    MonthCode(fields.month_code()),
                    fields.day().unwrap_or(1) as u8,
                )?;
                let iso = builtin.date_to_iso(&calendar_date);
                YearMonth::new(
                    iso.year().number,
                    iso.month().ordinal as i32,
                    Some(iso.day_of_month().0 as i32),
                    self.clone(),
                    overflow,
                )
            }
            CalendarSlot::Protocol(protocol) => {
                protocol.year_month_from_fields(fields, overflow, context)
            }
        }
    }

    /// `CalendarDateAdd`
    pub fn date_add(
        &self,
        date: &Date<C>,
        duration: &Duration,
        overflow: ArithmeticOverflow,
        context: &mut C::Context,
    ) -> TemporalResult<Date<C>> {
        match self {
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => {
                protocol.date_add(date, duration, overflow, context)
            }
        }
    }

    /// `CalendarDateUntil`
    pub fn date_until(
        &self,
        one: &Date<C>,
        two: &Date<C>,
        largest_unit: TemporalUnit,
        context: &mut C::Context,
    ) -> TemporalResult<Duration> {
        match self {
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => {
                protocol.date_until(one, two, largest_unit, context)
            }
        }
    }

    /// `CalendarEra`
    pub fn era(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => Ok(None),
            CalendarSlot::Builtin(builtin) => {
                let calendar_date = builtin.date_from_iso(date_like.as_iso_date().as_icu4x()?);
                Ok(Some(builtin.year(&calendar_date).era.0))
            }
            CalendarSlot::Protocol(protocol) => protocol.era(date_like, context),
        }
    }

    /// `CalendarEraYear`
    pub fn era_year(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<Option<i32>> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => Ok(None),
            CalendarSlot::Builtin(builtin) => {
                let calendar_date = builtin.date_from_iso(date_like.as_iso_date().as_icu4x()?);
                Ok(Some(builtin.year(&calendar_date).number))
            }
            CalendarSlot::Protocol(protocol) => protocol.era_year(date_like, context),
        }
    }

    /// `CalendarYear`
    pub fn year(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<i32> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => Ok(date_like.as_iso_date().year),
            CalendarSlot::Builtin(builtin) => {
                let calendar_date = builtin.date_from_iso(date_like.as_iso_date().as_icu4x()?);
                Ok(builtin.year(&calendar_date).number)
            }
            CalendarSlot::Protocol(protocol) => protocol.year(date_like, context),
        }
    }

    /// `CalendarMonth`
    pub fn month(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<u8> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => Ok(date_like.as_iso_date().month),
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.month(date_like, context),
        }
    }

    /// `CalendarMonthCode`
    pub fn month_code(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<TinyAsciiStr<4>> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => {
                Ok(date_like.as_iso_date().as_icu4x()?.month().code.0)
            }
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.month_code(date_like, context),
        }
    }

    /// `CalendarDay`
    pub fn day(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<u8> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => Ok(date_like.as_iso_date().day),
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.day(date_like, context),
        }
    }

    /// `CalendarDayOfWeek`
    pub fn day_of_week(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => {
                Ok(date_like.as_iso_date().as_icu4x()?.day_of_week() as u16)
            }
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.day_of_week(date_like, context),
        }
    }

    /// `CalendarDayOfYear`
    pub fn day_of_year(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => Ok(date_like
                .as_iso_date()
                .as_icu4x()?
                .day_of_year_info()
                .day_of_year),
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.day_of_year(date_like, context),
        }
    }

    /// `CalendarWeekOfYear`
    pub fn week_of_year(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => {
                let date = date_like.as_iso_date().as_icu4x()?;

                let week_calculator = WeekCalculator::default();

                let week_of = date
                    .week_of_year(&week_calculator)
                    .map_err(|err| TemporalError::range().with_message(err.to_string()))?;

                Ok(week_of.week)
            }
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.week_of_year(date_like, context),
        }
    }

    /// `CalendarYearOfWeek`
    pub fn year_of_week(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<i32> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => {
                let date = date_like.as_iso_date().as_icu4x()?;

                let week_calculator = WeekCalculator::default();

                let week_of = date
                    .week_of_year(&week_calculator)
                    .map_err(|err| TemporalError::range().with_message(err.to_string()))?;

                match week_of.unit {
                    RelativeUnit::Previous => Ok(date.year().number - 1),
                    RelativeUnit::Current => Ok(date.year().number),
                    RelativeUnit::Next => Ok(date.year().number + 1),
                }
            }
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.year_of_week(date_like, context),
        }
    }

    /// `CalendarDaysInWeek`
    pub fn days_in_week(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => Ok(7),
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.days_in_week(date_like, context),
        }
    }

    /// `CalendarDaysInMonth`
    pub fn days_in_month(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => {
                // NOTE: Cast shouldn't fail in this instance.
                Ok(date_like.as_iso_date().as_icu4x()?.day_of_month().0 as u16)
            }
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.days_in_month(date_like, context),
        }
    }

    /// `CalendarDaysInYear`
    pub fn days_in_year(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => {
                Ok(date_like.as_iso_date().as_icu4x()?.days_in_year())
            }
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.days_in_year(date_like, context),
        }
    }

    /// `CalendarMonthsInYear`
    pub fn months_in_year(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => Ok(12),
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.months_in_year(date_like, context),
        }
    }

    /// `CalendarInLeapYear`
    pub fn in_leap_year(
        &self,
        date_like: &CalendarDateLike<C>,
        context: &mut C::Context,
    ) -> TemporalResult<bool> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => {
                Ok(date_like.as_iso_date().as_icu4x()?.is_in_leap_year())
            }
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.in_leap_year(date_like, context),
        }
    }

    /// `CalendarFields`
    pub fn fields(
        &self,
        fields: Vec<String>,
        context: &mut C::Context,
    ) -> TemporalResult<Vec<String>> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => Ok(fields),
            CalendarSlot::Builtin(_) => {
                Err(TemporalError::range().with_message("Not yet implemented."))
            }
            CalendarSlot::Protocol(protocol) => protocol.fields(fields, context),
        }
    }

    /// `CalendarMergeFields`
    pub fn merge_fields(
        &self,
        fields: &TemporalFields,
        additional_fields: &TemporalFields,
        context: &mut C::Context,
    ) -> TemporalResult<TemporalFields> {
        match self {
            CalendarSlot::Builtin(_) => fields.merge_fields(additional_fields, self),
            CalendarSlot::Protocol(protocol) => {
                protocol.merge_fields(fields, additional_fields, context)
            }
        }
    }

    /// Returns the identifier of this calendar slot.
    pub fn identifier(&self, context: &mut C::Context) -> TemporalResult<String> {
        match self {
            CalendarSlot::Builtin(AnyCalendar::Iso(_)) => Ok(String::from("iso8601")),
            CalendarSlot::Builtin(builtin) => Ok(String::from(builtin.debug_name())),
            CalendarSlot::Protocol(protocol) => protocol.identifier(context),
        }
    }
}

impl<C: CalendarProtocol> CalendarSlot<C> {
    /// Returns the designated field descriptors for builtin calendars.
    pub fn field_descriptors(
        &self,
        _fields_type: CalendarFieldsType,
    ) -> TemporalResult<Vec<(String, bool)>> {
        // NOTE(nekevss): Can be called on a custom.
        if let CalendarSlot::Protocol(_) = self {
            return Ok(Vec::default());
        }

        // TODO: Research and implement the appropriate descriptors for all `BuiltinCalendars.`
        Err(TemporalError::range().with_message("FieldDescriptors is not yet implemented."))
    }

    /// Provides field keys to be ignored depending on the calendar.
    pub fn field_keys_to_ignore(&self, _keys: &[String]) -> TemporalResult<Vec<String>> {
        // TODO: Research and implement the appropriate KeysToIgnore for all `BuiltinCalendars.`
        Err(TemporalError::range().with_message("FieldKeysToIgnore is not yet implemented."))
    }

    /// `CalendarResolveFields`
    pub fn resolve_fields(
        &self,
        _fields: &mut TemporalFields,
        _typ: CalendarFieldsType,
    ) -> TemporalResult<()> {
        // TODO: Research and implement the appropriate ResolveFields for all `BuiltinCalendars.`
        Err(TemporalError::range().with_message("CalendarResolveFields is not yet implemented."))
    }
}

impl IsoDateSlots for () {
    fn iso_date(&self) -> IsoDate {
        unreachable!()
    }
}

/// An empty `CalendarProtocol` implementation on `()`.
///
/// # Panics
///
/// Attempting to use this empty calendar implementation as a valid calendar is an error and will cause a panic.
impl CalendarProtocol for () {
    type Date = Date<()>;

    type DateTime = DateTime<()>;

    type YearMonth = YearMonth<()>;

    type MonthDay = MonthDay<()>;

    type Context = ();

    fn date_from_fields(
        &self,
        _: &mut TemporalFields,
        _: ArithmeticOverflow,
        (): &mut (),
    ) -> TemporalResult<Date<Self>> {
        unreachable!();
    }

    fn month_day_from_fields(
        &self,
        _: &mut TemporalFields,
        _: ArithmeticOverflow,
        (): &mut (),
    ) -> TemporalResult<MonthDay<()>> {
        unreachable!();
    }

    fn year_month_from_fields(
        &self,
        _: &mut TemporalFields,
        _: ArithmeticOverflow,
        (): &mut (),
    ) -> TemporalResult<YearMonth<Self>> {
        unreachable!()
    }

    fn date_add(
        &self,
        _: &Date<Self>,
        _: &Duration,
        _: ArithmeticOverflow,
        (): &mut (),
    ) -> TemporalResult<Date<Self>> {
        unreachable!();
    }

    fn date_until(
        &self,
        _: &Date<()>,
        _: &Date<()>,
        _: TemporalUnit,
        (): &mut (),
    ) -> TemporalResult<Duration> {
        unreachable!();
    }

    fn era(
        &self,
        _: &CalendarDateLike<Self>,
        (): &mut (),
    ) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        unreachable!();
    }

    fn era_year(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<Option<i32>> {
        unreachable!();
    }

    fn year(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<i32> {
        unreachable!();
    }

    fn month(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<u8> {
        unreachable!();
    }

    fn month_code(
        &self,
        _: &CalendarDateLike<Self>,
        (): &mut (),
    ) -> TemporalResult<TinyAsciiStr<4>> {
        unreachable!();
    }

    fn day(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<u8> {
        unreachable!();
    }

    fn day_of_week(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<u16> {
        unreachable!();
    }

    fn day_of_year(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<u16> {
        unreachable!();
    }

    fn week_of_year(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<u16> {
        unreachable!();
    }

    fn year_of_week(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<i32> {
        unreachable!();
    }

    fn days_in_week(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<u16> {
        unreachable!();
    }

    fn days_in_month(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<u16> {
        unreachable!();
    }

    fn days_in_year(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<u16> {
        unreachable!();
    }

    fn months_in_year(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<u16> {
        unreachable!();
    }

    fn in_leap_year(&self, _: &CalendarDateLike<Self>, (): &mut ()) -> TemporalResult<bool> {
        unreachable!();
    }

    fn fields(&self, _: Vec<String>, (): &mut ()) -> TemporalResult<Vec<String>> {
        unreachable!();
    }

    fn merge_fields(
        &self,
        _: &TemporalFields,
        _: &TemporalFields,
        (): &mut (),
    ) -> TemporalResult<TemporalFields> {
        unreachable!();
    }

    fn identifier(&self, (): &mut ()) -> TemporalResult<String> {
        unreachable!();
    }
}
