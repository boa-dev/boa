//! This module implements `Duration` along with it's methods and components.

use crate::{
    components::{Date, DateTime, ZonedDateTime},
    options::{ArithmeticOverflow, TemporalRoundingMode, TemporalUnit},
    parser::{duration::parse_duration, Cursor},
    TemporalError, TemporalResult,
};
use std::str::FromStr;

use super::{calendar::CalendarProtocol, tz::TzProtocol};

mod date;
mod time;

#[doc(inline)]
pub use date::DateDuration;
#[doc(inline)]
pub use time::TimeDuration;

/// The native Rust implementation of `Temporal.Duration`.
///
/// `Duration` is made up of a `DateDuration` and `TimeDuration` as primarily
/// defined by Abtract Operation 7.5.1-5.
#[derive(Debug, Clone, Copy, Default)]
pub struct Duration {
    date: DateDuration,
    time: TimeDuration,
}

// NOTE(nekevss): Structure of the below is going to be a little convoluted,
// but intended to section everything based on the below
//
// Notation - [section](sub-section(s)).
//
// Sections:
//   - Creation (private/public)
//   - Getters/Setters
//   - Methods (private/public/feature)
//

// ==== Private Creation methods ====

impl Duration {
    /// Creates a new `Duration` from a `DateDuration` and `TimeDuration`.
    pub(crate) const fn new_unchecked(date: DateDuration, time: TimeDuration) -> Self {
        Self { date, time }
    }

    /// Utility function to create a year duration.
    pub(crate) fn one_year(year_value: f64) -> Self {
        Self::from_date_duration(DateDuration::new_unchecked(year_value, 0f64, 0f64, 0f64))
    }

    /// Utility function to create a month duration.
    pub(crate) fn one_month(month_value: f64) -> Self {
        Self::from_date_duration(DateDuration::new_unchecked(0f64, month_value, 0f64, 0f64))
    }

    /// Utility function to create a week duration.
    pub(crate) fn one_week(week_value: f64) -> Self {
        Self::from_date_duration(DateDuration::new_unchecked(0f64, 0f64, week_value, 0f64))
    }
}

// ==== Public Duration API ====

impl Duration {
    /// Creates a new validated `Duration`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        years: f64,
        months: f64,
        weeks: f64,
        days: f64,
        hours: f64,
        minutes: f64,
        seconds: f64,
        milliseconds: f64,
        microseconds: f64,
        nanoseconds: f64,
    ) -> TemporalResult<Self> {
        let duration = Self::new_unchecked(
            DateDuration::new_unchecked(years, months, weeks, days),
            TimeDuration::new_unchecked(
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanoseconds,
            ),
        );
        if !is_valid_duration(&duration.into_iter().collect()) {
            return Err(TemporalError::range().with_message("Duration was not valid."));
        }
        Ok(duration)
    }

    /// Creates a partial `Duration` with all fields set to `NaN`.
    #[must_use]
    pub const fn partial() -> Self {
        Self {
            date: DateDuration::partial(),
            time: TimeDuration::partial(),
        }
    }

    /// Creates a `Duration` from only a `DateDuration`.
    #[must_use]
    pub fn from_date_duration(date: DateDuration) -> Self {
        Self {
            date,
            time: TimeDuration::default(),
        }
    }

    /// Creates a `Duration` from a provided a day and a `TimeDuration`.
    ///
    /// Note: `TimeDuration` records can store a day value to deal with overflow.
    #[must_use]
    pub fn from_day_and_time(day: f64, time: TimeDuration) -> Self {
        Self {
            date: DateDuration::new_unchecked(0.0, 0.0, 0.0, day),
            time,
        }
    }

    /// Creates a new valid `Duration` from a partial `Duration`.
    pub fn from_partial(partial: &Duration) -> TemporalResult<Self> {
        let duration = Self {
            date: DateDuration::from_partial(partial.date()),
            time: TimeDuration::from_partial(partial.time()),
        };
        if !is_valid_duration(&duration.into_iter().collect()) {
            return Err(TemporalError::range().with_message("Duration was not valid."));
        }
        Ok(duration)
    }

    /// Return if the Durations values are within their valid ranges.
    #[inline]
    #[must_use]
    pub fn is_time_within_range(&self) -> bool {
        self.time.is_within_range()
    }

    /// Returns whether `Duration`'s `DateDuration` isn't empty and is therefore a `DateDuration` or `Duration`.
    #[inline]
    #[must_use]
    pub fn is_date_duration(&self) -> bool {
        self.date().iter().any(|x| x != 0.0) && self.time().iter().all(|x| x == 0.0)
    }

    /// Returns whether `Duration`'s `DateDuration` is empty and is therefore a `TimeDuration`.
    #[inline]
    #[must_use]
    pub fn is_time_duration(&self) -> bool {
        self.time().iter().any(|x| x != 0.0) && self.date().iter().all(|x| x == 0.0)
    }
}

// ==== Public `Duration` Getters/Setters ====

impl Duration {
    /// Returns a reference to the inner `TimeDuration`
    #[inline]
    #[must_use]
    pub fn time(&self) -> &TimeDuration {
        &self.time
    }

    /// Returns a reference to the inner `DateDuration`
    #[inline]
    #[must_use]
    pub fn date(&self) -> &DateDuration {
        &self.date
    }

    /// Set this `DurationRecord`'s `TimeDuration`.
    #[inline]
    pub fn set_time_duration(&mut self, time: TimeDuration) {
        self.time = time;
    }

    /// Set the value for `years`.
    #[inline]
    pub fn set_years(&mut self, y: f64) {
        self.date.years = y;
    }

    /// Returns the `years` field of duration.
    #[inline]
    #[must_use]
    pub const fn years(&self) -> f64 {
        self.date.years
    }

    /// Set the value for `months`.
    #[inline]
    pub fn set_months(&mut self, mo: f64) {
        self.date.months = mo;
    }

    /// Returns the `months` field of duration.
    #[inline]
    #[must_use]
    pub const fn months(&self) -> f64 {
        self.date.months
    }

    /// Set the value for `weeks`.
    #[inline]
    pub fn set_weeks(&mut self, w: f64) {
        self.date.weeks = w;
    }

    /// Returns the `weeks` field of duration.
    #[inline]
    #[must_use]
    pub const fn weeks(&self) -> f64 {
        self.date.weeks
    }

    /// Set the value for `days`.
    #[inline]
    pub fn set_days(&mut self, d: f64) {
        self.date.days = d;
    }

    /// Returns the `weeks` field of duration.
    #[inline]
    #[must_use]
    pub const fn days(&self) -> f64 {
        self.date.days
    }

    /// Set the value for `hours`.
    #[inline]
    pub fn set_hours(&mut self, h: f64) {
        self.time.hours = h;
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn hours(&self) -> f64 {
        self.time.hours
    }

    /// Set the value for `minutes`.
    #[inline]
    pub fn set_minutes(&mut self, m: f64) {
        self.time.minutes = m;
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn minutes(&self) -> f64 {
        self.time.minutes
    }

    /// Set the value for `seconds`.
    #[inline]
    pub fn set_seconds(&mut self, s: f64) {
        self.time.seconds = s;
    }

    /// Returns the `seconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn seconds(&self) -> f64 {
        self.time.seconds
    }

    /// Set the value for `milliseconds`.
    #[inline]
    pub fn set_milliseconds(&mut self, ms: f64) {
        self.time.milliseconds = ms;
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn milliseconds(&self) -> f64 {
        self.time.milliseconds
    }

    /// Set the value for `microseconds`.
    #[inline]
    pub fn set_microseconds(&mut self, mis: f64) {
        self.time.microseconds = mis;
    }

    /// Returns the `microseconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn microseconds(&self) -> f64 {
        self.time.microseconds
    }

    /// Set the value for `nanoseconds`.
    #[inline]
    pub fn set_nanoseconds(&mut self, ns: f64) {
        self.time.nanoseconds = ns;
    }

    /// Returns the `nanoseconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn nanoseconds(&self) -> f64 {
        self.time.nanoseconds
    }

    /// Returns `Duration`'s iterator
    #[must_use]
    pub fn iter(&self) -> DurationIter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<'a> IntoIterator for &'a Duration {
    type Item = f64;
    type IntoIter = DurationIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DurationIter {
            duration: self,
            index: 0,
        }
    }
}

/// A Duration iterator that iterates through all duration fields.
#[derive(Debug)]
pub struct DurationIter<'a> {
    duration: &'a Duration,
    index: usize,
}

impl Iterator for DurationIter<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(self.duration.date.years()),
            1 => Some(self.duration.date.months()),
            2 => Some(self.duration.date.weeks()),
            3 => Some(self.duration.date.days()),
            4 => Some(self.duration.time.hours()),
            5 => Some(self.duration.time.minutes()),
            6 => Some(self.duration.time.seconds()),
            7 => Some(self.duration.time.milliseconds()),
            8 => Some(self.duration.time.microseconds()),
            9 => Some(self.duration.time.nanoseconds()),
            _ => None,
        };
        self.index += 1;
        result
    }
}

// ==== Private Duration methods ====

impl Duration {
    /// 7.5.21 `UnbalanceDateDurationRelative ( years, months, weeks, days, largestUnit, plainRelativeTo )`
    #[allow(dead_code)]
    pub(crate) fn unbalance_duration_relative<C: CalendarProtocol>(
        &self,
        largest_unit: TemporalUnit,
        plain_relative_to: Option<&Date<C>>,
        context: &mut C::Context,
    ) -> TemporalResult<DateDuration> {
        // 1. Let allZero be false.
        // 2. If years = 0, and months = 0, and weeks = 0, and days = 0, set allZero to true.
        let all_zero = self.date.years == 0_f64
            && self.date.months == 0_f64
            && self.date.weeks == 0_f64
            && self.date.days == 0_f64;

        // 3. If largestUnit is "year" or allZero is true, then
        if largest_unit == TemporalUnit::Year || all_zero {
            // a. Return ! CreateDateDurationRecord(years, months, weeks, days).
            return Ok(self.date);
        };

        // 4. Let sign be ! DurationSign(years, months, weeks, days, 0, 0, 0, 0, 0, 0).
        // 5. Assert: sign ≠ 0.
        let sign = f64::from(self.duration_sign());

        // 6. Let oneYear be ! CreateTemporalDuration(sign, 0, 0, 0, 0, 0, 0, 0, 0, 0).
        let one_year = Self::one_year(sign);
        // 7. Let oneMonth be ! CreateTemporalDuration(0, sign, 0, 0, 0, 0, 0, 0, 0, 0).
        let one_month = Self::one_month(sign);

        // 9. If plainRelativeTo is not undefined, then
        // a. Let calendar be plainRelativeTo.[[Calendar]].
        // 10. Else,
        // a. Let calendar be undefined.

        // 11. If largestUnit is "month", then
        if largest_unit == TemporalUnit::Month {
            // a. If years = 0, return ! CreateDateDurationRecord(0, months, weeks, days).
            if self.date.years == 0f64 {
                return DateDuration::new(0f64, self.date.months, self.date.weeks, self.date.days);
            }

            // b. If calendar is undefined, then
            let Some(mut plain_relative_to) = plain_relative_to.map(Clone::clone) else {
                // i. Throw a RangeError exception.
                return Err(TemporalError::range().with_message("Calendar cannot be undefined."));
            };

            // c. If calendar is an Object, then
            // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
            // ii. Let dateUntil be ? GetMethod(calendar, "dateUntil").
            // d. Else,
            // i. Let dateAdd be unused.
            // ii. Let dateUntil be unused.

            let mut years = self.date.years;
            let mut months = self.date.months;
            // e. Repeat, while years ≠ 0,
            while years != 0f64 {
                // i. Let newRelativeTo be ? CalendarDateAdd(calendar, plainRelativeTo, oneYear, undefined, dateAdd).
                let new_relative_to = plain_relative_to.calendar().date_add(
                    &plain_relative_to,
                    &one_year,
                    ArithmeticOverflow::Constrain,
                    context,
                )?;

                // ii. Let untilOptions be OrdinaryObjectCreate(null).
                // iii. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "month").
                // iv. Let untilResult be ? CalendarDateUntil(calendar, plainRelativeTo, newRelativeTo, untilOptions, dateUntil).
                let until_result = plain_relative_to.calendar().date_until(
                    &plain_relative_to,
                    &new_relative_to,
                    TemporalUnit::Month,
                    context,
                )?;

                // v. Let oneYearMonths be untilResult.[[Months]].
                let one_year_months = until_result.date.months;

                // vi. Set plainRelativeTo to newRelativeTo.
                plain_relative_to = new_relative_to;

                // vii. Set years to years - sign.
                years -= sign;
                // viii. Set months to months + oneYearMonths.
                months += one_year_months;
            }
            // f. Return ? CreateDateDurationRecord(0, months, weeks, days).
            return DateDuration::new(years, months, self.date.weeks, self.date.days);

        // 12. If largestUnit is "week", then
        } else if largest_unit == TemporalUnit::Week {
            // a. If years = 0 and months = 0, return ! CreateDateDurationRecord(0, 0, weeks, days).
            if self.date.years == 0f64 && self.date.months == 0f64 {
                return DateDuration::new(0f64, 0f64, self.date.weeks, self.date.days);
            }

            // b. If calendar is undefined, then
            let Some(mut plain_relative_to) = plain_relative_to.map(Clone::clone) else {
                // i. Throw a RangeError exception.
                return Err(TemporalError::range().with_message("Calendar cannot be undefined."));
            };

            // c. If calendar is an Object, then
            // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
            // d. Else,
            // i. Let dateAdd be unused.

            let mut years = self.date.years;
            let mut days = self.date.days;
            // e. Repeat, while years ≠ 0,
            while years != 0f64 {
                // i. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneYear, dateAdd).
                let move_result = plain_relative_to.move_relative_date(&one_year, context)?;

                // ii. Set plainRelativeTo to moveResult.[[RelativeTo]].
                plain_relative_to = move_result.0;
                // iii. Set days to days + moveResult.[[Days]].
                days += move_result.1;
                // iv. Set years to years - sign.
                years -= sign;
            }

            let mut months = self.date.months;
            // f. Repeat, while months ≠ 0,
            while months != 0f64 {
                // i. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
                let move_result = plain_relative_to.move_relative_date(&one_month, context)?;
                // ii. Set plainRelativeTo to moveResult.[[RelativeTo]].
                plain_relative_to = move_result.0;
                // iii. Set days to days + moveResult.[[Days]].
                days += move_result.1;
                // iv. Set months to months - sign.
                months -= sign;
            }
            // g. Return ? CreateDateDurationRecord(0, 0, weeks, days).
            return DateDuration::new(0f64, 0f64, self.date.weeks(), days);
        }

        // 13. If years = 0, and months = 0, and weeks = 0, return ! CreateDateDurationRecord(0, 0, 0, days).
        if self.date.years == 0f64 && self.date.months == 0f64 && self.date.weeks == 0f64 {
            return DateDuration::new(0f64, 0f64, 0f64, self.date.days);
        }

        // NOTE: Move 8 down to past 13 as we only use one_week after making it past 13.
        // 8. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
        let one_week = Self::one_week(sign);

        // 14. If calendar is undefined, then
        let Some(mut plain_relative_to) = plain_relative_to.map(Clone::clone) else {
            // a. Throw a RangeError exception.
            return Err(TemporalError::range().with_message("Calendar cannot be undefined."));
        };

        // 15. If calendar is an Object, then
        // a. Let dateAdd be ? GetMethod(calendar, "dateAdd").
        // 16. Else,
        // a. Let dateAdd be unused.

        let mut years = self.date.years;
        let mut days = self.date.days;
        // a. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneYear, dateAdd).
        while years != 0f64 {
            // a. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneYear, dateAdd).
            let move_result = plain_relative_to.move_relative_date(&one_year, context)?;

            // b. Set plainRelativeTo to moveResult.[[RelativeTo]].
            plain_relative_to = move_result.0;
            // c. Set days to days + moveResult.[[Days]].
            days += move_result.1;
            // d. Set years to years - sign.
            years -= sign;
        }

        let mut months = self.date.months;
        // 18. Repeat, while months ≠ 0,
        while months != 0f64 {
            // a. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
            let move_result = plain_relative_to.move_relative_date(&one_month, context)?;
            // b. Set plainRelativeTo to moveResult.[[RelativeTo]].
            plain_relative_to = move_result.0;
            // c. Set days to days +moveResult.[[Days]].
            days += move_result.1;
            // d. Set months to months - sign.
            months -= sign;
        }

        let mut weeks = self.date.weeks;
        // 19. Repeat, while weeks ≠ 0,
        while weeks != 0f64 {
            // a. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneWeek, dateAdd).
            let move_result = plain_relative_to.move_relative_date(&one_week, context)?;
            // b. Set plainRelativeTo to moveResult.[[RelativeTo]].
            plain_relative_to = move_result.0;
            // c. Set days to days + moveResult.[[Days]].
            days += move_result.1;
            // d. Set weeks to weeks - sign.
            weeks -= sign;
        }

        // 20. Return ? CreateDateDurationRecord(0, 0, 0, days).
        DateDuration::new(0f64, 0f64, 0f64, days)
    }

    // TODO: Move to DateDuration
    /// `BalanceDateDurationRelative`
    #[allow(unused)]
    pub fn balance_date_duration_relative<C: CalendarProtocol>(
        &self,
        largest_unit: TemporalUnit,
        plain_relative_to: Option<&Date<C>>,
        context: &mut C::Context,
    ) -> TemporalResult<DateDuration> {
        let mut result = self.date;

        // 1. Let allZero be false.
        // 2. If years = 0, and months = 0, and weeks = 0, and days = 0, set allZero to true.
        let all_zero = self.date.years == 0.0
            && self.date.months == 0.0
            && self.date.weeks == 0.0
            && self.date.days == 0.0;

        // 3. If largestUnit is not one of "year", "month", or "week", or allZero is true, then
        match largest_unit {
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week if !all_zero => {}
            _ => {
                // a. Return ! CreateDateDurationRecord(years, months, weeks, days).
                return Ok(result);
            }
        }

        // 4. If plainRelativeTo is undefined, then
        let Some(mut plain_relative_to) = plain_relative_to.map(Clone::clone) else {
            // a. Throw a RangeError exception.
            return Err(TemporalError::range().with_message("relativeTo cannot be undefined."));
        };

        // 5. Let sign be ! DurationSign(years, months, weeks, days, 0, 0, 0, 0, 0, 0).
        // 6. Assert: sign ≠ 0.
        let sign = f64::from(self.duration_sign());

        // 7. Let oneYear be ! CreateTemporalDuration(sign, 0, 0, 0, 0, 0, 0, 0, 0, 0).
        let one_year = Self::one_year(sign);
        // 8. Let oneMonth be ! CreateTemporalDuration(0, sign, 0, 0, 0, 0, 0, 0, 0, 0).
        let one_month = Self::one_month(sign);
        // 9. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
        let one_week = Self::one_week(sign);

        // 10. Let calendar be relativeTo.[[Calendar]].

        match largest_unit {
            // 12. If largestUnit is "year", then
            TemporalUnit::Year => {
                // a. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // b. Else,
                // i. Let dateAdd be unused.

                // c. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneYear, dateAdd).
                // d. Let newRelativeTo be moveResult.[[RelativeTo]].
                // e. Let oneYearDays be moveResult.[[Days]].
                let (mut new_relative_to, mut one_year_days) =
                    plain_relative_to.move_relative_date(&one_year, context)?;

                // f. Repeat, while abs(days) ≥ abs(oneYearDays),
                while result.days().abs() >= one_year_days.abs() {
                    // i. Set days to days - oneYearDays.
                    result.days -= one_year_days;

                    // ii. Set years to years + sign.
                    result.years += sign;

                    // iii. Set relativeTo to newRelativeTo.
                    plain_relative_to = new_relative_to;
                    // iv. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneYear, dateAdd).
                    let move_result = plain_relative_to.move_relative_date(&one_year, context)?;

                    // v. Set newRelativeTo to moveResult.[[RelativeTo]].
                    new_relative_to = move_result.0;
                    // vi. Set oneYearDays to moveResult.[[Days]].
                    one_year_days = move_result.1;
                }

                // g. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
                // h. Set newRelativeTo to moveResult.[[RelativeTo]].
                // i. Let oneMonthDays be moveResult.[[Days]].
                let (mut new_relative_to, mut one_month_days) =
                    plain_relative_to.move_relative_date(&one_month, context)?;

                // j. Repeat, while abs(days) ≥ abs(oneMonthDays),
                while result.days().abs() >= one_month_days.abs() {
                    // i. Set days to days - oneMonthDays.
                    result.days -= one_month_days;
                    // ii. Set months to months + sign.
                    result.months += sign;

                    // iii. Set relativeTo to newRelativeTo.
                    plain_relative_to = new_relative_to;
                    // iv. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
                    let move_result = plain_relative_to.move_relative_date(&one_month, context)?;

                    // v. Set newRelativeTo to moveResult.[[RelativeTo]].
                    new_relative_to = move_result.0;
                    // vi. Set oneMonthDays to moveResult.[[Days]].
                    one_month_days = move_result.1;
                }

                // k. Set newRelativeTo to ? CalendarDateAdd(calendar, relativeTo, oneYear, undefined, dateAdd).
                new_relative_to = plain_relative_to.calendar().date_add(
                    &plain_relative_to,
                    &one_year,
                    ArithmeticOverflow::Constrain,
                    context,
                )?;

                // l. If calendar is an Object, then
                // i. Let dateUntil be ? GetMethod(calendar, "dateUntil").
                // m. Else,
                // i. Let dateUntil be unused.
                // n. Let untilOptions be OrdinaryObjectCreate(null).
                // o. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "month").

                // p. Let untilResult be ? CalendarDateUntil(calendar, relativeTo, newRelativeTo, untilOptions, dateUntil).
                let mut until_result = plain_relative_to.calendar().date_until(
                    &plain_relative_to,
                    &new_relative_to,
                    TemporalUnit::Month,
                    context,
                )?;

                // q. Let oneYearMonths be untilResult.[[Months]].
                let mut one_year_months = until_result.date.months();

                // r. Repeat, while abs(months) ≥ abs(oneYearMonths),
                while result.months().abs() >= one_year_months.abs() {
                    // i. Set months to months - oneYearMonths.
                    result.months -= one_year_months;
                    // ii. Set years to years + sign.
                    result.years += sign;

                    // iii. Set relativeTo to newRelativeTo.
                    plain_relative_to = new_relative_to;

                    // iv. Set newRelativeTo to ? CalendarDateAdd(calendar, relativeTo, oneYear, undefined, dateAdd).
                    new_relative_to = plain_relative_to.calendar().date_add(
                        &plain_relative_to,
                        &one_year,
                        ArithmeticOverflow::Constrain,
                        context,
                    )?;

                    // v. Set untilOptions to OrdinaryObjectCreate(null).
                    // vi. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "month").
                    // vii. Set untilResult to ? CalendarDateUntil(calendar, relativeTo, newRelativeTo, untilOptions, dateUntil).
                    until_result = plain_relative_to.calendar().date_until(
                        &plain_relative_to,
                        &new_relative_to,
                        TemporalUnit::Month,
                        context,
                    )?;

                    // viii. Set oneYearMonths to untilResult.[[Months]].
                    one_year_months = until_result.date.months();
                }
            }
            // 13. Else if largestUnit is "month", then
            TemporalUnit::Month => {
                // a. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // b. Else,
                // i. Let dateAdd be unused.

                // c. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
                // d. Let newRelativeTo be moveResult.[[RelativeTo]].
                // e. Let oneMonthDays be moveResult.[[Days]].
                let (mut new_relative_to, mut one_month_days) =
                    plain_relative_to.move_relative_date(&one_month, context)?;

                // f. Repeat, while abs(days) ≥ abs(oneMonthDays),
                while result.days().abs() >= one_month_days.abs() {
                    // i. Set days to days - oneMonthDays.
                    result.days -= one_month_days;

                    // ii. Set months to months + sign.
                    result.months += sign;

                    // iii. Set relativeTo to newRelativeTo.
                    plain_relative_to = new_relative_to;

                    // iv. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
                    let move_result = plain_relative_to.move_relative_date(&one_month, context)?;

                    // v. Set newRelativeTo to moveResult.[[RelativeTo]].
                    new_relative_to = move_result.0;
                    // vi. Set oneMonthDays to moveResult.[[Days]].
                    one_month_days = move_result.1;
                }
            }
            // 14. Else,
            TemporalUnit::Week => {
                // a. Assert: largestUnit is "week".
                // b. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // c. Else,
                // i. Let dateAdd be unused.

                // d. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneWeek, dateAdd).
                // e. Let newRelativeTo be moveResult.[[RelativeTo]].
                // f. Let oneWeekDays be moveResult.[[Days]].
                let (mut new_relative_to, mut one_week_days) =
                    plain_relative_to.move_relative_date(&one_week, context)?;

                // g. Repeat, while abs(days) ≥ abs(oneWeekDays),
                while result.days().abs() >= one_week_days.abs() {
                    // i. Set days to days - oneWeekDays.
                    result.days -= one_week_days;
                    // ii. Set weeks to weeks + sign.
                    result.weeks += sign;
                    // iii. Set relativeTo to newRelativeTo.
                    plain_relative_to = new_relative_to;
                    // iv. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneWeek, dateAdd).
                    let move_result = plain_relative_to.move_relative_date(&one_week, context)?;

                    // v. Set newRelativeTo to moveResult.[[RelativeTo]].
                    new_relative_to = move_result.0;
                    // vi. Set oneWeekDays to moveResult.[[Days]].
                    one_week_days = move_result.1;
                }
            }
            _ => unreachable!(),
        }

        // 15. Return ! CreateDateDurationRecord(years, months, weeks, days).
        Ok(result)
    }

    // TODO: Refactor relative_to's into a RelativeTo struct?
    /// Abstract Operation 7.5.26 `RoundDuration ( years, months, weeks, days, hours, minutes,
    ///   seconds, milliseconds, microseconds, nanoseconds, increment, unit,
    ///   roundingMode [ , plainRelativeTo [, zonedRelativeTo [, precalculatedDateTime]]] )`
    #[allow(clippy::type_complexity)]
    pub fn round_duration<C: CalendarProtocol, Z: TzProtocol>(
        &self,
        increment: f64,
        unit: TemporalUnit,
        rounding_mode: TemporalRoundingMode,
        relative_targets: (
            Option<&Date<C>>,
            Option<&ZonedDateTime<C, Z>>,
            Option<&DateTime<C>>,
        ),
        context: &mut C::Context,
    ) -> TemporalResult<(Self, f64)> {
        match unit {
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                let round_result = self.date().round(
                    Some(self.time),
                    increment,
                    unit,
                    rounding_mode,
                    relative_targets,
                    context,
                )?;
                let result = Self::from_date_duration(round_result.0);
                Ok((result, round_result.1))
            }
            TemporalUnit::Hour
            | TemporalUnit::Minute
            | TemporalUnit::Second
            | TemporalUnit::Millisecond
            | TemporalUnit::Microsecond
            | TemporalUnit::Nanosecond => {
                let round_result = self.time().round(increment, unit, rounding_mode)?;
                let result = Self {
                    date: self.date,
                    time: round_result.0,
                };
                Ok((result, round_result.1))
            }
            TemporalUnit::Auto => {
                Err(TemporalError::range().with_message("Invalid TemporalUnit for Duration.round"))
            }
        }

        // 18. Let duration be ? CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
        // 19. Return the Record { [[DurationRecord]]: duration, [[Total]]: total }.
    }
}

// ==== Public Duration methods ====

impl Duration {
    /// Returns the absolute value of `Duration`.
    #[inline]
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            date: self.date().abs(),
            time: self.time().abs(),
        }
    }

    /// 7.5.10 `DurationSign ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )`
    ///
    /// Determines the sign for the current self.
    #[inline]
    #[must_use]
    pub fn duration_sign(&self) -> i32 {
        duration_sign(&self.into_iter().collect())
    }

    /// 7.5.12 `DefaultTemporalLargestUnit ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds )`
    #[inline]
    #[must_use]
    pub fn default_temporal_largest_unit(&self) -> TemporalUnit {
        for (index, value) in self.into_iter().enumerate() {
            if value == 0f64 {
                continue;
            }

            match index {
                0 => return TemporalUnit::Year,
                1 => return TemporalUnit::Month,
                2 => return TemporalUnit::Week,
                3 => return TemporalUnit::Day,
                4 => return TemporalUnit::Hour,
                5 => return TemporalUnit::Minute,
                6 => return TemporalUnit::Second,
                7 => return TemporalUnit::Millisecond,
                8 => return TemporalUnit::Microsecond,
                _ => {}
            }
        }

        TemporalUnit::Nanosecond
    }

    /// Calls `TimeDuration`'s balance method on the current `Duration`.
    #[inline]
    pub fn balance_time_duration(&self, unit: TemporalUnit) -> TemporalResult<(f64, TimeDuration)> {
        TimeDuration::new_unchecked(
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.milliseconds(),
            self.microseconds(),
            self.nanoseconds(),
        )
        .balance(self.days(), unit)
    }
}

/// Utility function to check whether the `Duration` fields are valid.
#[inline]
#[must_use]
pub(crate) fn is_valid_duration(set: &Vec<f64>) -> bool {
    // 1. Let sign be ! DurationSign(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
    let sign = duration_sign(set);
    // 2. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
    for v in set {
        // a. If 𝔽(v) is not finite, return false.
        if !v.is_finite() {
            return false;
        }
        // b. If v < 0 and sign > 0, return false.
        if *v < 0f64 && sign > 0 {
            return false;
        }
        // c. If v > 0 and sign < 0, return false.
        if *v > 0f64 && sign < 0 {
            return false;
        }
    }
    // 3. Return true.
    true
}

/// Utility function for determining the sign for the current set of `Duration` fields.
#[inline]
#[must_use]
fn duration_sign(set: &Vec<f64>) -> i32 {
    // 1. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
    for v in set {
        // a. If v < 0, return -1.
        if *v < 0f64 {
            return -1;
        // b. If v > 0, return 1.
        } else if *v > 0f64 {
            return 1;
        }
    }
    // 2. Return 0.
    0
}

impl From<TimeDuration> for Duration {
    fn from(value: TimeDuration) -> Self {
        Self {
            time: value,
            date: DateDuration::default(),
        }
    }
}

// ==== FromStr trait impl ====

impl FromStr for Duration {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_record = parse_duration(&mut Cursor::new(s))?;

        let minutes = if parse_record.time.fhours > 0.0 {
            parse_record.time.fhours * 60.0
        } else {
            f64::from(parse_record.time.minutes)
        };

        let seconds = if parse_record.time.fminutes > 0.0 {
            parse_record.time.fminutes * 60.0
        } else if parse_record.time.seconds > 0 {
            f64::from(parse_record.time.seconds)
        } else {
            minutes.rem_euclid(1.0) * 60.0
        };

        let milliseconds = if parse_record.time.fseconds > 0.0 {
            parse_record.time.fseconds * 1000.0
        } else {
            seconds.rem_euclid(1.0) * 1000.0
        };

        let micro = milliseconds.rem_euclid(1.0) * 1000.0;
        let nano = micro.rem_euclid(1.0) * 1000.0;

        let sign = if parse_record.sign { 1f64 } else { -1f64 };

        Ok(Self {
            date: DateDuration::new(
                f64::from(parse_record.date.years) * sign,
                f64::from(parse_record.date.months) * sign,
                f64::from(parse_record.date.weeks) * sign,
                f64::from(parse_record.date.days) * sign,
            )?,
            time: TimeDuration::new(
                f64::from(parse_record.time.hours) * sign,
                minutes.floor() * sign,
                seconds.floor() * sign,
                milliseconds.floor() * sign,
                micro.floor() * sign,
                nano.floor() * sign,
            )?,
        })
    }
}
