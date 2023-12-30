//! This module implements `Date` and any directly related algorithms.

use crate::{
    components::{
        calendar::{CalendarProtocol, CalendarSlot},
        duration::DateDuration,
        DateTime, Duration,
    },
    iso::{IsoDate, IsoDateSlots},
    options::{ArithmeticOverflow, TemporalUnit},
    parser::parse_date_time,
    TemporalError, TemporalResult,
};
use std::{any::Any, str::FromStr};

use super::duration::TimeDuration;

/// The native Rust implementation of `Temporal.PlainDate`.
#[derive(Debug, Default, Clone)]
pub struct Date<C: CalendarProtocol> {
    iso: IsoDate,
    calendar: CalendarSlot<C>,
}

// ==== Private API ====

impl<C: CalendarProtocol> Date<C> {
    /// Create a new `Date` with the date values and calendar slot.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: CalendarSlot<C>) -> Self {
        Self { iso, calendar }
    }

    #[inline]
    /// Returns a new moved date and the days associated with that adjustment
    pub(crate) fn move_relative_date(
        &self,
        duration: &Duration,
        context: &mut dyn Any,
    ) -> TemporalResult<(Self, f64)> {
        let new_date =
            self.contextual_add_date(duration, ArithmeticOverflow::Constrain, context)?;
        let days = f64::from(self.days_until(&new_date));
        Ok((new_date, days))
    }
}

// ==== Public API ====

impl<C: CalendarProtocol> Date<C> {
    /// Creates a new `Date` while checking for validity.
    pub fn new(
        year: i32,
        month: i32,
        day: i32,
        calendar: CalendarSlot<C>,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let iso = IsoDate::new(year, month, day, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
    }

    #[must_use]
    /// Creates a `Date` from a `DateTime`.
    pub fn from_datetime(dt: &DateTime<C>) -> Self {
        Self {
            iso: dt.iso_date(),
            calendar: dt.calendar().clone(),
        }
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s year value.
    pub const fn year(&self) -> i32 {
        self.iso.year()
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s month value.
    pub const fn month(&self) -> u8 {
        self.iso.month()
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s day value.
    pub const fn day(&self) -> u8 {
        self.iso.day()
    }

    #[inline]
    #[must_use]
    /// Returns the `Date`'s inner `IsoDate` record.
    pub const fn iso_date(&self) -> IsoDate {
        self.iso
    }

    #[inline]
    #[must_use]
    /// Returns a reference to this `Date`'s calendar slot.
    pub fn calendar(&self) -> &CalendarSlot<C> {
        &self.calendar
    }

    /// 3.5.7 `IsValidISODate`
    ///
    /// Checks if the current date is a valid `ISODate`.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.iso.is_valid()
    }

    /// `DaysUntil`
    ///
    /// Calculates the epoch days between two `Date`s
    #[inline]
    #[must_use]
    pub fn days_until(&self, other: &Self) -> i32 {
        other.iso.to_epoch_days() - self.iso.to_epoch_days()
    }
}

impl<C: CalendarProtocol> IsoDateSlots for Date<C> {
    /// Returns the structs `IsoDate`
    fn iso_date(&self) -> IsoDate {
        self.iso
    }
}

// ==== Context based API ====

impl<C: CalendarProtocol> Date<C> {
    /// Returns the date after adding the given duration to date with a provided context.
    ///
    /// Temporal Equivalent: 3.5.13 `AddDate ( calendar, plainDate, duration [ , options [ , dateAdd ] ] )`
    #[inline]
    pub fn contextual_add_date(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
        context: &mut dyn Any,
    ) -> TemporalResult<Self> {
        // 1. If options is not present, set options to undefined.
        // 2. If duration.[[Years]] ≠ 0, or duration.[[Months]] ≠ 0, or duration.[[Weeks]] ≠ 0, then
        if duration.date().years() != 0.0
            || duration.date().months() != 0.0
            || duration.date().weeks() != 0.0
        {
            // a. If dateAdd is not present, then
            // i. Set dateAdd to unused.
            // ii. If calendar is an Object, set dateAdd to ? GetMethod(calendar, "dateAdd").
            // b. Return ? CalendarDateAdd(calendar, plainDate, duration, options, dateAdd).
            return self.calendar().date_add(self, duration, overflow, context);
        }

        // 3. Let overflow be ? ToTemporalOverflow(options).
        // 4. Let days be ? BalanceTimeDuration(duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]], "day").[[Days]].
        let (days, _) = TimeDuration::new_unchecked(
            duration.hours(),
            duration.minutes(),
            duration.seconds(),
            duration.milliseconds(),
            duration.microseconds(),
            duration.nanoseconds(),
        )
        .balance(duration.days(), TemporalUnit::Day)?;

        // 5. Let result be ? AddISODate(plainDate.[[ISOYear]], plainDate.[[ISOMonth]], plainDate.[[ISODay]], 0, 0, 0, days, overflow).
        let result = self
            .iso
            .add_iso_date(&DateDuration::new(0f64, 0f64, 0f64, days)?, overflow)?;

        Ok(Self::new_unchecked(result, self.calendar().clone()))
    }

    /// Returns the date after adding the given duration to date.
    ///
    /// Temporal Equivalent: 3.5.13 `AddDate ( calendar, plainDate, duration [ , options [ , dateAdd ] ] )`
    #[inline]
    pub fn add_date(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        self.contextual_add_date(duration, overflow, &mut ())
    }

    /// Returns a duration representing the difference between the dates one and two with a provided context.
    ///
    /// Temporal Equivalent: 3.5.6 `DifferenceDate ( calendar, one, two, options )`
    #[inline]
    pub fn contextual_difference_date(
        &self,
        other: &Self,
        largest_unit: TemporalUnit,
        context: &mut dyn Any,
    ) -> TemporalResult<Duration> {
        if self.iso.year() == other.iso.year()
            && self.iso.month() == other.iso.month()
            && self.iso.day() == other.iso.day()
        {
            return Ok(Duration::default());
        }

        if largest_unit == TemporalUnit::Day {
            let days = self.days_until(other);
            return Ok(Duration::from_date_duration(DateDuration::new(
                0f64,
                0f64,
                0f64,
                f64::from(days),
            )?));
        }

        self.calendar()
            .date_until(self, other, largest_unit, context)
    }

    /// Returns a duration representing the difference between the dates one and two.
    ///
    /// Temporal Equivalent: 3.5.6 `DifferenceDate ( calendar, one, two, options )`
    #[inline]
    pub fn difference_date(
        &self,
        other: &Self,
        largest_unit: TemporalUnit,
    ) -> TemporalResult<Duration> {
        self.contextual_difference_date(other, largest_unit, &mut ())
    }
}

// ==== Trait impls ====

impl<C: CalendarProtocol> FromStr for Date<C> {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_record = parse_date_time(s)?;

        let calendar = parse_record.calendar.unwrap_or("iso8601".to_owned());

        let date = IsoDate::new(
            parse_record.date.year,
            parse_record.date.month,
            parse_record.date.day,
            ArithmeticOverflow::Reject,
        )?;

        Ok(Self::new_unchecked(
            date,
            CalendarSlot::from_str(&calendar)?,
        ))
    }
}
