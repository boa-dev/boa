//! This module implements `DateTime` any directly related algorithms.

use crate::{
    components::{
        calendar::{CalendarProtocol, CalendarSlot},
        Instant,
    },
    iso::{IsoDate, IsoDateSlots, IsoDateTime, IsoTime},
    options::ArithmeticOverflow,
    parser::parse_date_time,
    TemporalError, TemporalResult,
};

use std::{any::Any, str::FromStr};
use tinystr::TinyAsciiStr;

use super::calendar::{CalendarDateLike, DateTypes, GetCalendarSlot};

/// The native Rust implementation of `Temporal.PlainDateTime`
#[derive(Debug, Default, Clone)]
pub struct DateTime<C: CalendarProtocol> {
    iso: IsoDateTime,
    calendar: CalendarSlot<C>,
}

// ==== Private DateTime API ====

impl<C: CalendarProtocol> DateTime<C> {
    /// Creates a new unchecked `DateTime`.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDateTime, calendar: CalendarSlot<C>) -> Self {
        Self { iso, calendar }
    }

    #[inline]
    #[must_use]
    /// Utility function for validating `IsoDate`s
    fn validate_iso(iso: IsoDate) -> bool {
        IsoDateTime::new_unchecked(iso, IsoTime::noon()).is_within_limits()
    }

    /// Create a new `DateTime` from an `Instant`.
    #[inline]
    pub(crate) fn from_instant(
        instant: &Instant,
        offset: f64,
        calendar: CalendarSlot<C>,
    ) -> TemporalResult<Self> {
        let iso = IsoDateTime::from_epoch_nanos(&instant.nanos, offset)?;
        Ok(Self { iso, calendar })
    }
}

// ==== Public DateTime API ====

impl<C: CalendarProtocol> DateTime<C> {
    /// Creates a new validated `DateTime`.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: i32,
        microsecond: i32,
        nanosecond: i32,
        calendar: CalendarSlot<C>,
    ) -> TemporalResult<Self> {
        let iso_date = IsoDate::new(year, month, day, ArithmeticOverflow::Reject)?;
        let iso_time = IsoTime::new(
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            ArithmeticOverflow::Reject,
        )?;
        Ok(Self::new_unchecked(
            IsoDateTime::new(iso_date, iso_time)?,
            calendar,
        ))
    }

    /// Validates whether ISO date slots are within iso limits at noon.
    #[inline]
    pub fn validate<T: IsoDateSlots>(target: &T) -> bool {
        Self::validate_iso(target.iso_date())
    }

    /// Returns this `Date`'s ISO year value.
    #[inline]
    #[must_use]
    pub const fn iso_year(&self) -> i32 {
        self.iso.date().year
    }

    /// Returns this `Date`'s ISO month value.
    #[inline]
    #[must_use]
    pub const fn iso_month(&self) -> u8 {
        self.iso.date().month
    }

    /// Returns this `Date`'s ISO day value.
    #[inline]
    #[must_use]
    pub const fn iso_day(&self) -> u8 {
        self.iso.date().day
    }

    /// Returns the hour value
    #[inline]
    #[must_use]
    pub fn hour(&self) -> u8 {
        self.iso.time().hour
    }

    /// Returns the minute value
    #[inline]
    #[must_use]
    pub fn minute(&self) -> u8 {
        self.iso.time().minute
    }

    /// Returns the second value
    #[inline]
    #[must_use]
    pub fn second(&self) -> u8 {
        self.iso.time().second
    }

    /// Returns the `millisecond` value
    #[inline]
    #[must_use]
    pub fn millisecond(&self) -> u16 {
        self.iso.time().millisecond
    }

    /// Returns the `microsecond` value
    #[inline]
    #[must_use]
    pub fn microsecond(&self) -> u16 {
        self.iso.time().microsecond
    }

    /// Returns the `nanosecond` value
    #[inline]
    #[must_use]
    pub fn nanosecond(&self) -> u16 {
        self.iso.time().nanosecond
    }

    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &CalendarSlot<C> {
        &self.calendar
    }
}

// ==== Calendar-derived public API ====

// TODO: Revert to `DateTime<C>`.
impl DateTime<()> {
    /// Returns the calendar year value.
    pub fn year(&self) -> TemporalResult<i32> {
        self.calendar
            .year(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar month value.
    pub fn month(&self) -> TemporalResult<u8> {
        self.calendar
            .month(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar month code value.
    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        self.calendar
            .month_code(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar day value.
    pub fn day(&self) -> TemporalResult<u8> {
        self.calendar
            .day(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar day of week value.
    pub fn day_of_week(&self) -> TemporalResult<u16> {
        self.calendar
            .day_of_week(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar day of year value.
    pub fn day_of_year(&self) -> TemporalResult<u16> {
        self.calendar
            .day_of_year(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar week of year value.
    pub fn week_of_year(&self) -> TemporalResult<u16> {
        self.calendar
            .week_of_year(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar year of week value.
    pub fn year_of_week(&self) -> TemporalResult<i32> {
        self.calendar
            .year_of_week(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar days in week value.
    pub fn days_in_week(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_week(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar days in month value.
    pub fn days_in_month(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_month(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar days in year value.
    pub fn days_in_year(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_year(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns the calendar months in year value.
    pub fn months_in_year(&self) -> TemporalResult<u16> {
        self.calendar
            .months_in_year(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }

    /// Returns returns whether the date in a leap year for the given calendar.
    pub fn in_leap_year(&self) -> TemporalResult<bool> {
        self.calendar
            .in_leap_year(&CalendarDateLike::DateTime(self.clone()), &mut ())
    }
}

impl<C: CalendarProtocol> DateTime<C> {
    /// Returns the calendar year value with provided context.
    pub fn contextualized_year(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        this.get_calendar()
            .year(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar month value with provided context.
    pub fn contextualized_month(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<u8> {
        this.get_calendar()
            .month(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar month code value with provided context.
    pub fn contextualized_month_code(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<TinyAsciiStr<4>> {
        this.get_calendar()
            .month_code(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar day value with provided context.
    pub fn contextualized_day(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<u8> {
        this.get_calendar()
            .day(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar day of week value with provided context.
    pub fn contextualized_day_of_week(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .day_of_week(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar day of year value with provided context.
    pub fn contextualized_day_of_year(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .day_of_year(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar week of year value with provided context.
    pub fn contextualized_week_of_year(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .week_of_year(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar year of week value with provided context.
    pub fn contextualized_year_of_week(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<i32> {
        this.get_calendar()
            .year_of_week(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar days in week value with provided context.
    pub fn contextualized_days_in_week(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .days_in_week(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar days in month value with provided context.
    pub fn contextualized_days_in_month(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .days_in_month(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar days in year value with provided context.
    pub fn contextualized_days_in_year(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .days_in_year(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns the calendar months in year value with provided context.
    pub fn contextualized_months_in_year(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .months_in_year(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }

    /// Returns whether the date is in a leap year for the given calendar with provided context.
    pub fn contextualized_in_leap_year(
        this: &<C::DateLikes as DateTypes<C>>::DateTime,
        context: &mut dyn Any,
    ) -> TemporalResult<bool> {
        this.get_calendar()
            .in_leap_year(&CalendarDateLike::CustomDateTime(this.clone()), context)
    }
}

// ==== Trait impls ====

impl<C: CalendarProtocol> GetCalendarSlot<C> for DateTime<C> {
    fn get_calendar(&self) -> CalendarSlot<C> {
        self.calendar.clone()
    }
}

impl<C: CalendarProtocol> IsoDateSlots for DateTime<C> {
    fn iso_date(&self) -> IsoDate {
        *self.iso.date()
    }
}

impl<C: CalendarProtocol> FromStr for DateTime<C> {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_record = parse_date_time(s)?;

        let calendar = parse_record.calendar.unwrap_or("iso8601".to_owned());

        let time = if let Some(time) = parse_record.time {
            IsoTime::from_components(
                i32::from(time.hour),
                i32::from(time.minute),
                i32::from(time.second),
                time.fraction,
            )?
        } else {
            IsoTime::default()
        };

        let date = IsoDate::new(
            parse_record.date.year,
            parse_record.date.month,
            parse_record.date.day,
            ArithmeticOverflow::Reject,
        )?;

        Ok(Self::new_unchecked(
            IsoDateTime::new(date, time)?,
            CalendarSlot::from_str(&calendar)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::components::calendar::CalendarSlot;

    use super::DateTime;

    #[test]
    #[allow(clippy::float_cmp)]
    fn plain_date_time_limits() {
        // This test is primarily to assert that the `expect` in the epoch methods is
        // valid, i.e., a valid instant is within the range of an f64.
        let negative_limit = DateTime::<()>::new(
            -271_821,
            4,
            19,
            0,
            0,
            0,
            0,
            0,
            0,
            CalendarSlot::from_str("iso8601").unwrap(),
        );
        let positive_limit = DateTime::<()>::new(
            275_760,
            9,
            14,
            0,
            0,
            0,
            0,
            0,
            0,
            CalendarSlot::from_str("iso8601").unwrap(),
        );

        assert!(negative_limit.is_err());
        assert!(positive_limit.is_err());
    }
}
