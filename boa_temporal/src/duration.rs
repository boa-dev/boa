//! The Temporal Duration.
//!
//! TODO: Docs

use crate::{
    date::TemporalDate,
    datetime::TemporalDateTime,
    options::{ArithmeticOverflow, TemporalRoundingMode, TemporalUnit},
    utils,
    zoneddatetime::TemporalZonedDateTime,
    TemporalError, TemporalResult, NS_PER_DAY,
};
use std::any::Any;

// ==== `DateDuration` ====

/// `DateDuration` represents the [date duration record][spec] of the `Duration.`
///
/// These fields are laid out in the [Temporal Proposal][field spec] as 64-bit floating point numbers.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-date-duration-records
/// [field spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[derive(Debug, Default, Clone, Copy)]
pub struct DateDuration {
    years: f64,
    months: f64,
    weeks: f64,
    days: f64,
}

impl DateDuration {
    /// Creates a new `DateDuration` with provided values.
    pub const fn new(years: f64, months: f64, weeks: f64, days: f64) -> Self {
        Self {
            years,
            months,
            weeks,
            days,
        }
    }

    /// Returns a `PartialDateDuration` with all fields set to NaN.
    pub const fn partial() -> Self {
        Self {
            years: f64::NAN,
            months: f64::NAN,
            weeks: f64::NAN,
            days: f64::NAN,
        }
    }

    /// Returns the `[[years]]` value.
    pub const fn years(&self) -> f64 {
        self.years
    }

    /// Returns the `[[months]]` value.
    pub const fn months(&self) -> f64 {
        self.months
    }

    /// Returns the `[[weeks]]` value.
    pub const fn weeks(&self) -> f64 {
        self.weeks
    }

    /// Returns the `[[days]]` value.
    pub const fn days(&self) -> f64 {
        self.days
    }
}

impl<'a> IntoIterator for &'a DateDuration {
    type Item = f64;
    type IntoIter = DateIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DateIter {
            date: self,
            index: 0,
        }
    }
}

/// An iterator over the `DateDuration`
#[derive(Debug)]
pub struct DateIter<'a> {
    date: &'a DateDuration,
    index: usize,
}

impl Iterator for DateIter<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(self.date.years),
            1 => Some(self.date.months),
            2 => Some(self.date.weeks),
            3 => Some(self.date.days),
            _ => None,
        };
        self.index += 1;
        result
    }
}

// ==== `TimeDuration` ====

/// `TimeDuration` represents the [Time Duration record][spec] of the `Duration.`
///
/// These fields are laid out in the [Temporal Proposal][field spec] as 64-bit floating point numbers.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-time-duration-records
/// [field spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[derive(Debug, Default, Clone, Copy)]
pub struct TimeDuration {
    hours: f64,
    minutes: f64,
    seconds: f64,
    milliseconds: f64,
    microseconds: f64,
    nanoseconds: f64,
}

impl TimeDuration {
    /// Creates a new `TimeDuration`.
    pub const fn new(
        hours: f64,
        minutes: f64,
        seconds: f64,
        milliseconds: f64,
        microseconds: f64,
        nanoseconds: f64,
    ) -> Self {
        Self {
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        }
    }

    /// Creates a partial `TimeDuration` with all values set to NaN.
    pub const fn partial() -> Self {
        Self {
            hours: f64::NAN,
            minutes: f64::NAN,
            seconds: f64::NAN,
            milliseconds: f64::NAN,
            microseconds: f64::NAN,
            nanoseconds: f64::NAN,
        }
    }

    /// Utility function for returning if values in a valid range.
    #[inline]
    pub fn is_within_range(&self) -> bool {
        self.hours.abs() < 24f64
            && self.minutes.abs() < 60f64
            && self.seconds.abs() < 60f64
            && self.milliseconds.abs() < 1000f64
            && self.milliseconds.abs() < 1000f64
            && self.milliseconds.abs() < 1000f64
    }

    /// Returns the `[[hours]]` value.
    pub const fn hours(&self) -> f64 {
        self.hours
    }

    /// Returns the `[[minutes]]` value.
    pub const fn minutes(&self) -> f64 {
        self.minutes
    }

    /// Returns the `[[seconds]]` value.
    pub const fn seconds(&self) -> f64 {
        self.seconds
    }

    /// Returns the `[[milliseconds]]` value.
    pub const fn milliseconds(&self) -> f64 {
        self.milliseconds
    }

    /// Returns the `[[microseconds]]` value.
    pub const fn microseconds(&self) -> f64 {
        self.microseconds
    }

    /// Returns the `[[nanoseconds]]` value.
    pub const fn nanoseconds(&self) -> f64 {
        self.nanoseconds
    }
}

impl<'a> IntoIterator for &'a TimeDuration {
    type Item = f64;
    type IntoIter = TimeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TimeIter {
            time: self,
            index: 0,
        }
    }
}

/// An iterator over a `TimeDuration`.
#[derive(Debug)]
pub struct TimeIter<'a> {
    time: &'a TimeDuration,
    index: usize,
}

impl Iterator for TimeIter<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(self.time.hours),
            1 => Some(self.time.minutes),
            2 => Some(self.time.seconds),
            3 => Some(self.time.milliseconds),
            4 => Some(self.time.microseconds),
            5 => Some(self.time.nanoseconds),
            _ => None,
        };
        self.index += 1;
        result
    }
}

// ==== `Duration` ====

/// The `Duration` is a native Rust implementation of the `Duration` builtin
/// object internal fields and is primarily defined by Abtract Operation 7.5.1-5.
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
        Self::from_date_duration(DateDuration::new(year_value, 0f64, 0f64, 0f64))
    }

    /// Utility function to create a month duration.
    pub(crate) fn one_month(month_value: f64) -> Self {
        Self::from_date_duration(DateDuration::new(0f64, month_value, 0f64, 0f64))
    }

    /// Utility function to create a week duration.
    pub(crate) fn one_week(week_value: f64) -> Self {
        Self::from_date_duration(DateDuration::new(0f64, 0f64, week_value, 0f64))
    }
}

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
            DateDuration::new(years, months, weeks, days),
            TimeDuration::new(
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanoseconds,
            ),
        );
        if !duration.is_valid() {
            return Err(TemporalError::range().with_message("Duration was not valid."));
        }
        Ok(duration)
    }

    /// Creates a partial `Duration` with all fields set to NaN.
    pub const fn partial() -> Self {
        Self {
            date: DateDuration::partial(),
            time: TimeDuration::partial(),
        }
    }

    /// Creates a `Duration` from only a DateDuration.
    pub fn from_date_duration(date: DateDuration) -> Self {
        Self {
            date,
            time: TimeDuration::default(),
        }
    }

    /// Creates a `Duration` from a provided a day and a `TimeDuration`.
    ///
    /// Note: `TimeDuration` records can store a day value to deal with overflow.
    pub const fn from_day_and_time(day: f64, time: TimeDuration) -> Self {
        Self {
            date: DateDuration::new(0.0, 0.0, 0.0, day),
            time,
        }
    }

    /// Return if the Durations values are within their valid ranges.
    #[inline]
    pub fn is_time_within_range(&self) -> bool {
        self.time.is_within_range()
    }
}

// ==== Public `Duration` Getters/Setters ====

impl Duration {
    /// Returns a reference to the inner `TimeDuration`
    pub fn time(&self) -> &TimeDuration {
        &self.time
    }

    /// Returns a reference to the inner `DateDuration`
    pub fn date(&self) -> &DateDuration {
        &self.date
    }

    /// Set this `DurationRecord`'s `TimeDuration`.
    pub fn set_time_duration(&mut self, time: TimeDuration) {
        self.time = time;
    }

    /// Set the value for `years`.
    pub fn set_years(&mut self, y: f64) {
        self.date.years = y;
    }

    /// Set the value for `months`.
    pub fn set_months(&mut self, mo: f64) {
        self.date.months = mo;
    }

    /// Set the value for `weeks`.
    pub fn set_weeks(&mut self, w: f64) {
        self.date.weeks = w;
    }

    /// Set the value for `days`.
    pub fn set_days(&mut self, d: f64) {
        self.date.days = d;
    }

    /// Set the value for `hours`.
    pub fn set_hours(&mut self, h: f64) {
        self.time.hours = h;
    }

    /// Set the value for `minutes`.
    pub fn set_minutes(&mut self, m: f64) {
        self.time.minutes = m;
    }

    /// Set the value for `seconds`.
    pub fn set_seconds(&mut self, s: f64) {
        self.time.seconds = s;
    }

    /// Set the value for `milliseconds`.
    pub fn set_milliseconds(&mut self, ms: f64) {
        self.time.milliseconds = ms;
    }

    /// Set the value for `microseconds`.
    pub fn set_microseconds(&mut self, mis: f64) {
        self.time.microseconds = mis;
    }

    /// Set the value for `nanoseconds`.
    pub fn set_nanoseconds(&mut self, ns: f64) {
        self.time.nanoseconds = ns;
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
    /// Returns the duration time values as a vec
    pub(crate) fn time_values(&self) -> Vec<f64> {
        let mut values = Vec::from([self.date.days]);
        values.extend(self.time.into_iter());
        values
    }

    /// 7.5.11 `IsValidDuration ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )`
    ///
    /// Checks if the current `DurationRecord` is a valid self.
    pub(crate) fn is_valid(&self) -> bool {
        // 1. Let sign be ! DurationSign(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
        let sign = self.duration_sign();
        // 2. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
        for v in self {
            // a. If 𝔽(v) is not finite, return false.
            if !v.is_finite() {
                return false;
            }
            // b. If v < 0 and sign > 0, return false.
            if v < 0_f64 && sign > 0 {
                return false;
            }
            // c. If v > 0 and sign < 0, return false.
            if v > 0_f64 && sign < 0 {
                return false;
            }
        }
        // 3. Return true.
        true
    }

    /// 7.5.17 `TotalDurationNanoseconds ( days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, offsetShift )`
    pub(crate) fn total_duration_nanoseconds(&self, offset_shift: f64) -> f64 {
        let nanoseconds = if self.date.days == 0_f64 {
            self.time.nanoseconds
        } else {
            self.time.nanoseconds - offset_shift
        };

        self.date
            .days
            .mul_add(24_f64, self.time.hours)
            .mul_add(60_f64, self.time.minutes)
            .mul_add(60_f64, self.time.seconds)
            .mul_add(1_000_f64, self.time.milliseconds)
            .mul_add(1_000_f64, self.time.microseconds)
            .mul_add(1_000_f64, nanoseconds)
    }

    /// Abstract Operation 7.5.18 `BalancePossiblyInfiniteDuration ( days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, largestUnit )`
    pub(crate) fn balance_possibly_infinite_time_duration(
        &self,
        largest_unit: TemporalUnit,
    ) -> TemporalResult<Option<(f64, TimeDuration)>> {
        let mut result = TimeDuration::default();
        let mut result_days = 0f64;

        // 1. Set hours to hours + days × 24.
        result.hours = self.time.hours + (self.date.days * 24f64);

        // 2. Set nanoseconds to TotalDurationNanoseconds(hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
        result.nanoseconds = self.total_duration_nanoseconds(0f64);

        // 3. Set days, hours, minutes, seconds, milliseconds, and microseconds to 0.

        // 4. If nanoseconds < 0, let sign be -1; else, let sign be 1.
        let sign = if result.nanoseconds < 0f64 { -1 } else { 1 };
        // 5. Set nanoseconds to abs(nanoseconds).
        result.nanoseconds = result.nanoseconds.abs();

        match largest_unit {
            // 9. If largestUnit is "year", "month", "week", "day", or "hour", then
            TemporalUnit::Year
            | TemporalUnit::Month
            | TemporalUnit::Week
            | TemporalUnit::Day
            | TemporalUnit::Hour => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                result.microseconds = (result.nanoseconds / 1000f64).floor();
                // b. Set nanoseconds to nanoseconds modulo 1000.
                result.nanoseconds = result.nanoseconds % 1000f64;

                // c. Set milliseconds to floor(microseconds / 1000).
                result.milliseconds = (result.microseconds / 1000f64).floor();
                // d. Set microseconds to microseconds modulo 1000.
                result.microseconds = result.microseconds % 1000f64;

                // e. Set seconds to floor(milliseconds / 1000).
                result.seconds = (result.milliseconds / 1000f64).floor();
                // f. Set milliseconds to milliseconds modulo 1000.
                result.milliseconds = result.milliseconds % 1000f64;

                // g. Set minutes to floor(seconds / 60).
                result.minutes = (result.seconds / 60f64).floor();
                // h. Set seconds to seconds modulo 60.
                result.seconds = result.seconds % 60f64;

                // i. Set hours to floor(minutes / 60).
                result.hours = (result.minutes / 60f64).floor();
                // j. Set minutes to minutes modulo 60.
                result.minutes = result.minutes % 60f64;

                // k. Set days to floor(hours / 24).
                result_days = (result.hours / 24f64).floor();
                // l. Set hours to hours modulo 24.
                result.hours = result.hours % 24f64;
            }
            // 10. Else if largestUnit is "minute", then
            TemporalUnit::Minute => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                result.microseconds = (result.nanoseconds / 1000f64).floor();
                result.nanoseconds = result.nanoseconds % 1000f64;

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                result.milliseconds = (result.microseconds / 1000f64).floor();
                result.microseconds = result.microseconds % 1000f64;

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                result.minutes = (result.seconds / 60f64).floor();
                result.seconds = result.seconds % 60f64;

                // g. Set minutes to floor(seconds / 60).
                // h. Set seconds to seconds modulo 60.
                result.minutes = (result.seconds / 60f64).floor();
                result.seconds = result.seconds % 60f64;
            }
            // 11. Else if largestUnit is "second", then
            TemporalUnit::Second => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                result.microseconds = (result.nanoseconds / 1000f64).floor();
                result.nanoseconds = result.nanoseconds % 1000f64;

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                result.milliseconds = (result.microseconds / 1000f64).floor();
                result.microseconds = result.microseconds % 1000f64;

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                result.minutes = (result.seconds / 60f64).floor();
                result.seconds = result.seconds % 60f64;
            }
            // 12. Else if largestUnit is "millisecond", then
            TemporalUnit::Millisecond => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                result.microseconds = (result.nanoseconds / 1000f64).floor();
                result.nanoseconds = result.nanoseconds % 1000f64;

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                result.milliseconds = (result.microseconds / 1000f64).floor();
                result.microseconds = result.microseconds % 1000f64;
            }
            // 13. Else if largestUnit is "microsecond", then
            TemporalUnit::Microsecond => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                result.microseconds = (result.nanoseconds / 1000f64).floor();
                result.nanoseconds = result.nanoseconds % 1000f64;
            }
            // 14. Else,
            // a. Assert: largestUnit is "nanosecond".
            _ => debug_assert!(largest_unit == TemporalUnit::Nanosecond),
        }

        // 15. For each value v of « days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
        for value in self.time_values() {
            // a. If 𝔽(v) is not finite, then
            if !value.is_finite() {
                // i. If sign = 1, then
                if sign == 1 {
                    // 1. Return positive overflow.
                    return Ok(None);
                }
                // ii. Else if sign = -1, then
                // 1. Return negative overflow.
                return Ok(None);
            }
        }

        let sign = f64::from(sign);

        // 16. Return ? CreateTimeDurationRecord(days, hours × sign, minutes × sign, seconds × sign, milliseconds × sign, microseconds × sign, nanoseconds × sign).
        result.hours *= sign;
        result.minutes *= sign;
        result.seconds *= sign;
        result.milliseconds *= sign;
        result.microseconds *= sign;
        result.nanoseconds *= sign;

        // `CreateTimeDurationRecord` validates that the record that would be created is a valid duration, so validate here
        if !self.is_valid() {
            return Err(TemporalError::range()
                .with_message("TimeDurationRecord is not a valid duration.")
                .into());
        }

        Ok(Some((result_days, result)))
    }

    /// 7.5.21 UnbalanceDateDurationRelative ( years, months, weeks, days, largestUnit, plainRelativeTo )`
    #[allow(dead_code)]
    pub(crate) fn unbalance_duration_relative(
        &self,
        largest_unit: TemporalUnit,
        plain_relative_to: Option<&TemporalDate>,
        context: &mut dyn Any,
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
                return Ok(DateDuration::new(
                    0f64,
                    self.date.months,
                    self.date.weeks,
                    self.date.days,
                ));
            }

            // b. If calendar is undefined, then
            let Some(mut plain_relative_to) = plain_relative_to.map(Clone::clone) else {
                // i. Throw a RangeError exception.
                return Err(TemporalError::range()
                    .with_message("Calendar cannot be undefined.")
                    .into());
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
            return Ok(DateDuration::new(
                years,
                months,
                self.date.weeks,
                self.date.days,
            ));

        // 12. If largestUnit is "week", then
        } else if largest_unit == TemporalUnit::Week {
            // a. If years = 0 and months = 0, return ! CreateDateDurationRecord(0, 0, weeks, days).
            if self.date.years == 0f64 && self.date.months == 0f64 {
                return Ok(DateDuration::new(
                    0f64,
                    0f64,
                    self.date.weeks,
                    self.date.days,
                ));
            }

            // b. If calendar is undefined, then
            let Some(mut plain_relative_to) = plain_relative_to.map(Clone::clone) else {
                // i. Throw a RangeError exception.
                return Err(TemporalError::range()
                    .with_message("Calendar cannot be undefined.")
                    .into());
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
            return Ok(DateDuration::new(0f64, 0f64, self.date.weeks(), days));
        }

        // 13. If years = 0, and months = 0, and weeks = 0, return ! CreateDateDurationRecord(0, 0, 0, days).
        if self.date.years == 0f64 && self.date.months == 0f64 && self.date.weeks == 0f64 {
            return Ok(DateDuration::new(0f64, 0f64, 0f64, self.date.days));
        }

        // NOTE: Move 8 down to past 13 as we only use one_week after making it past 13.
        // 8. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
        let one_week = Self::one_week(sign);

        // 14. If calendar is undefined, then
        let Some(mut plain_relative_to) = plain_relative_to.map(Clone::clone) else {
            // a. Throw a RangeError exception.
            return Err(TemporalError::range()
                .with_message("Calendar cannot be undefined.")
                .into());
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
        Ok(DateDuration::new(0f64, 0f64, 0f64, days))
    }

    // TODO: Move to DateDuration
    /// `BalanceDateDurationRelative`
    #[allow(unused)]
    pub fn balance_date_duration_relative(
        &self,
        largest_unit: TemporalUnit,
        plain_relative_to: Option<&TemporalDate>,
        context: &mut dyn Any,
    ) -> TemporalResult<DateDuration> {
        let mut result = DateDuration::from(self.date);

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
            return Err(TemporalError::range()
                .with_message("relativeTo cannot be undefined.")
                .into());
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
    pub fn round_duration(
        &self,
        unbalance_date_duration: DateDuration,
        increment: f64,
        unit: TemporalUnit,
        rounding_mode: TemporalRoundingMode,
        relative_targets: (
            Option<&TemporalDate>,
            Option<&TemporalZonedDateTime>,
            Option<&TemporalDateTime>,
        ),
        context: &mut dyn Any,
    ) -> TemporalResult<(Self, f64)> {
        let mut result = Duration::new_unchecked(unbalance_date_duration, self.time);

        // 1. If plainRelativeTo is not present, set plainRelativeTo to undefined.
        let plain_relative_to = relative_targets.0;
        // 2. If zonedRelativeTo is not present, set zonedRelativeTo to undefined.
        let zoned_relative_to = relative_targets.1;
        // 3. If precalculatedPlainDateTime is not present, set precalculatedPlainDateTime to undefined.
        let _precalc_pdt = relative_targets.2;

        let (frac_days, frac_secs) = match unit {
            // 4. If unit is "year", "month", or "week", and plainRelativeTo is undefined, then
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week
                if plain_relative_to.is_none() =>
            {
                // a. Throw a RangeError exception.
                return Err(TemporalError::range()
                    .with_message("plainRelativeTo canot be undefined with given TemporalUnit")
                    .into());
            }
            // 5. If unit is one of "year", "month", "week", or "day", then
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                // a. Let nanoseconds be TotalDurationNanoseconds(hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
                let nanos =
                    Self::from_day_and_time(0.0, *result.time()).total_duration_nanoseconds(0.0);

                // b. If zonedRelativeTo is not undefined, then
                // i. Let intermediate be ? MoveRelativeZonedDateTime(zonedRelativeTo, years, months, weeks, days, precalculatedPlainDateTime).
                // ii. Let result be ? NanosecondsToDays(nanoseconds, intermediate).
                // iii. Let fractionalDays be days + result.[[Days]] + result.[[Nanoseconds]] / result.[[DayLength]].
                // c. Else,
                // i. Let fractionalDays be days + nanoseconds / nsPerDay.
                let frac_days = if zoned_relative_to.is_none() {
                    result.date.days + nanos / NS_PER_DAY as f64
                } else {
                    // implementation of b: i-iii needed.
                    return Err(TemporalError::range()
                        .with_message("Not yet implemented.")
                        .into());
                };
                // d. Set days, hours, minutes, seconds, milliseconds, microseconds, and nanoseconds to 0.
                result.date.days = 0f64;
                result.time = TimeDuration::default();
                // e. Assert: fractionalSeconds is not used below.
                (Some(frac_days), None)
            }
            // 6. Else,
            _ => {
                // a. Let fractionalSeconds be nanoseconds × 10-9 + microseconds × 10-6 + milliseconds × 10-3 + seconds.
                let frac_secs = result.time.nanoseconds.mul_add(
                    1_000_000_000f64,
                    result.time.microseconds.mul_add(
                        1_000_000f64,
                        result
                            .time
                            .milliseconds
                            .mul_add(1_000f64, result.time.seconds),
                    ),
                );

                // b. Assert: fractionalDays is not used below.
                (None, Some(frac_secs))
            }
        };

        // 7. let total be unset.
        // We begin matching against unit and return the remainder value.
        let total = match unit {
            // 8. If unit is "year", then
            TemporalUnit::Year => {
                let mut frac_days =
                    frac_days.expect("assert that fractionalDays exists for TemporalUnit == year");

                let plain_relative_to = plain_relative_to.expect("this must exist.");
                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let calendar = plain_relative_to.calendar();

                // b. Let yearsDuration be ! CreateTemporalDuration(years, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let years = DateDuration::new(result.date.years, 0.0, 0.0, 0.0);
                let years_duration = Duration::new_unchecked(years, TimeDuration::default());

                // c. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // d. Else,
                // i. Let dateAdd be unused.

                // e. Let yearsLater be ? AddDate(calendar, plainRelativeTo, yearsDuration, undefined, dateAdd).
                let years_later = plain_relative_to
                    .add_date(&years_duration, ArithmeticOverflow::Constrain, context)?;

                // f. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = Self::new_unchecked(
                    DateDuration::new(
                        result.date.years,
                        result.date.months,
                        result.date.weeks,
                        0.0,
                    ),
                    TimeDuration::default(),
                );

                // g. Let yearsMonthsWeeksLater be ? AddDate(calendar, plainRelativeTo, yearsMonthsWeeks, undefined, dateAdd).
                let years_months_weeks_later = plain_relative_to
                    .add_date(&years_months_weeks, ArithmeticOverflow::Constrain, context)?;

                // h. Let monthsWeeksInDays be DaysUntil(yearsLater, yearsMonthsWeeksLater).
                let months_weeks_in_days = years_later.days_until(&years_months_weeks_later);

                // i. Set plainRelativeTo to yearsLater.
                let plain_relative_to = years_later;

                // j. Set fractionalDays to fractionalDays + monthsWeeksInDays.
                frac_days += f64::from(months_weeks_in_days);

                // k. Let isoResult be ! AddISODate(plainRelativeTo.[[ISOYear]]. plainRelativeTo.[[ISOMonth]], plainRelativeTo.[[ISODay]], 0, 0, 0, truncate(fractionalDays), "constrain").
                let iso_result = plain_relative_to.iso_date().add_iso_date(
                    &DateDuration::new(0.0, 0.0, 0.0, frac_days.trunc()),
                    ArithmeticOverflow::Constrain,
                )?;

                // l. Let wholeDaysLater be ? CreateTemporalDate(isoResult.[[Year]], isoResult.[[Month]], isoResult.[[Day]], calendar).
                let whole_days_later = TemporalDate::new_unchecked(iso_result, calendar.clone());

                // m. Let untilOptions be OrdinaryObjectCreate(null).
                // n. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "year").
                // o. Let timePassed be ? DifferenceDate(calendar, plainRelativeTo, wholeDaysLater, untilOptions).
                let time_passed =
                    plain_relative_to.diff_date(&whole_days_later, TemporalUnit::Year, context)?;

                // p. Let yearsPassed be timePassed.[[Years]].
                let years_passed = time_passed.date.years();

                // q. Set years to years + yearsPassed.
                result.date.years += years_passed;

                // r. Let yearsDuration be ! CreateTemporalDuration(yearsPassed, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_duration = Self::one_year(years_passed);

                // s. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, yearsDuration, dateAdd).
                // t. Set plainRelativeTo to moveResult.[[RelativeTo]].
                // u. Let daysPassed be moveResult.[[Days]].
                let (plain_relative_to, days_passed) =
                    plain_relative_to.move_relative_date(&years_duration, context)?;

                // v. Set fractionalDays to fractionalDays - daysPassed.
                frac_days -= days_passed;

                // w. If fractionalDays < 0, let sign be -1; else, let sign be 1.
                let sign = if frac_days < 0.0 { -1 } else { 1 };

                // x. Let oneYear be ! CreateTemporalDuration(sign, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let one_year = Self::one_year(f64::from(sign));

                // y. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneYear, dateAdd).
                // z. Let oneYearDays be moveResult.[[Days]].
                let (_, one_year_days) =
                    plain_relative_to.move_relative_date(&one_year, context)?;

                // aa. Let fractionalYears be years + fractionalDays / abs(oneYearDays).
                let frac_years = result.date.years() + (frac_days / one_year_days.abs());

                // ab. Set years to RoundNumberToIncrement(fractionalYears, increment, roundingMode).
                result.date.years =
                    utils::round_number_to_increment(frac_years, increment, rounding_mode);

                // ac. Set total to fractionalYears.
                // ad. Set months and weeks to 0.
                result.date.months = 0f64;
                result.date.weeks = 0f64;

                frac_years
            }
            // 9. Else if unit is "month", then
            TemporalUnit::Month => {
                let mut frac_days =
                    frac_days.expect("assert that fractionalDays exists for TemporalUnit::Month");

                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let plain_relative_to = plain_relative_to.expect("this must exist.");

                // b. Let yearsMonths be ! CreateTemporalDuration(years, months, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_months = Self::from_date_duration(DateDuration::new(
                    result.date.years(),
                    result.date.months(),
                    0.0,
                    0.0,
                ));

                // c. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // d. Else,
                // i. Let dateAdd be unused.

                // e. Let yearsMonthsLater be ? AddDate(calendar, plainRelativeTo, yearsMonths, undefined, dateAdd).
                let years_months_later =
                    plain_relative_to.add_date(&years_months, ArithmeticOverflow::Constrain, context)?;

                // f. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = Self::from_date_duration(DateDuration::new(
                    result.date.years(),
                    result.date.months(),
                    result.date.weeks(),
                    0.0,
                ));

                // g. Let yearsMonthsWeeksLater be ? AddDate(calendar, plainRelativeTo, yearsMonthsWeeks, undefined, dateAdd).
                let years_months_weeks_later = plain_relative_to
                    .add_date(&years_months_weeks, ArithmeticOverflow::Constrain, context)?;

                // h. Let weeksInDays be DaysUntil(yearsMonthsLater, yearsMonthsWeeksLater).
                let weeks_in_days = years_months_later.days_until(&years_months_weeks_later);

                // i. Set plainRelativeTo to yearsMonthsLater.
                let plain_relative_to = years_months_later;

                // j. Set fractionalDays to fractionalDays + weeksInDays.
                frac_days += f64::from(weeks_in_days);

                // k. If fractionalDays < 0, let sign be -1; else, let sign be 1.
                let sign = if frac_days < 0.0 { -1f64 } else { 1f64 };

                // l. Let oneMonth be ! CreateTemporalDuration(0, sign, 0, 0, 0, 0, 0, 0, 0, 0).
                let one_month = Self::one_month(sign);

                // m. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
                // n. Set plainRelativeTo to moveResult.[[RelativeTo]].
                // o. Let oneMonthDays be moveResult.[[Days]].
                let (mut plain_relative_to, mut one_month_days) =
                    plain_relative_to.move_relative_date(&one_month, context)?;

                // p. Repeat, while abs(fractionalDays) ≥ abs(oneMonthDays),
                while frac_days.abs() >= one_month_days.abs() {
                    // i. Set months to months + sign.
                    result.date.months += sign;

                    // ii. Set fractionalDays to fractionalDays - oneMonthDays.
                    frac_days -= one_month_days;

                    // iii. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
                    let move_result = plain_relative_to.move_relative_date(&one_month, context)?;

                    // iv. Set plainRelativeTo to moveResult.[[RelativeTo]].
                    plain_relative_to = move_result.0;
                    // v. Set oneMonthDays to moveResult.[[Days]].
                    one_month_days = move_result.1;
                }

                // q. Let fractionalMonths be months + fractionalDays / abs(oneMonthDays).
                let frac_months = result.date.months() + frac_days / one_month_days.abs();

                // r. Set months to RoundNumberToIncrement(fractionalMonths, increment, roundingMode).
                result.date.months =
                    utils::round_number_to_increment(frac_months, increment, rounding_mode);

                // s. Set total to fractionalMonths.
                // t. Set weeks to 0.
                result.date.weeks = 0.0;
                frac_months
            }
            // 10. Else if unit is "week", then
            TemporalUnit::Week => {
                let mut frac_days =
                    frac_days.expect("assert that fractionalDays exists for TemporalUnit::Month");
                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let plain_relative_to = plain_relative_to.expect("date must exist given Week");

                // b. If fractionalDays < 0, let sign be -1; else, let sign be 1.
                let sign = if frac_days < 0.0 { -1f64 } else { 1f64 };

                // c. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
                let one_week = Self::one_week(sign);

                // d. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // e. Else,
                // i. Let dateAdd be unused.

                // f. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneWeek, dateAdd).
                // g. Set plainRelativeTo to moveResult.[[RelativeTo]].
                // h. Let oneWeekDays be moveResult.[[Days]].
                let (mut plain_relative_to, mut one_week_days) =
                    plain_relative_to.move_relative_date(&one_week, context)?;

                // i. Repeat, while abs(fractionalDays) ≥ abs(oneWeekDays),
                while frac_days.abs() >= one_week_days.abs() {
                    // i. Set weeks to weeks + sign.
                    result.date.weeks += sign;

                    // ii. Set fractionalDays to fractionalDays - oneWeekDays.
                    frac_days -= one_week_days;

                    // iii. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneWeek, dateAdd).
                    let move_result = plain_relative_to.move_relative_date(&one_week, context)?;

                    // iv. Set plainRelativeTo to moveResult.[[RelativeTo]].
                    plain_relative_to = move_result.0;
                    // v. Set oneWeekDays to moveResult.[[Days]].
                    one_week_days = move_result.1;
                }

                // j. Let fractionalWeeks be weeks + fractionalDays / abs(oneWeekDays).
                let frac_weeks = result.date.weeks() + frac_days / one_week_days.abs();

                // k. Set weeks to RoundNumberToIncrement(fractionalWeeks, increment, roundingMode).
                result.date.weeks =
                    utils::round_number_to_increment(frac_weeks, increment, rounding_mode);
                // l. Set total to fractionalWeeks.
                frac_weeks
            }
            // 11. Else if unit is "day", then
            TemporalUnit::Day => {
                let frac_days =
                    frac_days.expect("assert that fractionalDays exists for TemporalUnit::Day");

                // a. Set days to RoundNumberToIncrement(fractionalDays, increment, roundingMode).
                result.date.days =
                    utils::round_number_to_increment(frac_days, increment, rounding_mode);
                // b. Set total to fractionalDays.
                frac_days
            }
            // 12. Else if unit is "hour", then
            TemporalUnit::Hour => {
                let frac_secs =
                    frac_secs.expect("Assert fractionSeconds exists for Temporal::Hour");
                // a. Let fractionalHours be (fractionalSeconds / 60 + minutes) / 60 + hours.
                let frac_hours =
                    (frac_secs / 60f64 + result.time.minutes) / 60f64 + result.time.hours;
                // b. Set hours to RoundNumberToIncrement(fractionalHours, increment, roundingMode).
                let rounded_hours =
                    utils::round_number_to_increment(frac_hours, increment, rounding_mode);
                // c. Set total to fractionalHours.
                // d. Set minutes, seconds, milliseconds, microseconds, and nanoseconds to 0.
                result.time = TimeDuration::new(rounded_hours, 0.0, 0.0, 0.0, 0.0, 0.0);
                frac_hours
            }
            // 13. Else if unit is "minute", then
            TemporalUnit::Minute => {
                let frac_secs =
                    frac_secs.expect("Assert fractionSeconds exists for Temporal::Hour");
                // a. Let fractionalMinutes be fractionalSeconds / 60 + minutes.
                let frac_minutes = frac_secs / 60f64 + result.time.minutes;
                // b. Set minutes to RoundNumberToIncrement(fractionalMinutes, increment, roundingMode).
                let rounded_minutes =
                    utils::round_number_to_increment(frac_minutes, increment, rounding_mode);
                // c. Set total to fractionalMinutes.
                // d. Set seconds, milliseconds, microseconds, and nanoseconds to 0.
                result.time =
                    TimeDuration::new(result.time.hours, rounded_minutes, 0.0, 0.0, 0.0, 0.0);

                frac_minutes
            }
            // 14. Else if unit is "second", then
            TemporalUnit::Second => {
                let frac_secs =
                    frac_secs.expect("Assert fractionSeconds exists for Temporal::Second");
                // a. Set seconds to RoundNumberToIncrement(fractionalSeconds, increment, roundingMode).
                result.time.seconds =
                    utils::round_number_to_increment(frac_secs, increment, rounding_mode);
                // b. Set total to fractionalSeconds.
                // c. Set milliseconds, microseconds, and nanoseconds to 0.
                result.time.milliseconds = 0.0;
                result.time.microseconds = 0.0;
                result.time.nanoseconds = 0.0;

                frac_secs
            }
            // 15. Else if unit is "millisecond", then
            TemporalUnit::Millisecond => {
                // a. Let fractionalMilliseconds be nanoseconds × 10-6 + microseconds × 10-3 + milliseconds.
                let fraction_millis = result.time.nanoseconds.mul_add(
                    1_000_000f64,
                    result
                        .time
                        .microseconds
                        .mul_add(1_000f64, result.time.milliseconds),
                );

                // b. Set milliseconds to RoundNumberToIncrement(fractionalMilliseconds, increment, roundingMode).
                result.time.milliseconds =
                    utils::round_number_to_increment(fraction_millis, increment, rounding_mode);

                // c. Set total to fractionalMilliseconds.
                // d. Set microseconds and nanoseconds to 0.
                result.time.microseconds = 0.0;
                result.time.nanoseconds = 0.0;
                fraction_millis
            }
            // 16. Else if unit is "microsecond", then
            TemporalUnit::Microsecond => {
                // a. Let fractionalMicroseconds be nanoseconds × 10-3 + microseconds.
                let frac_micros = result
                    .time
                    .nanoseconds
                    .mul_add(1_000f64, result.time.microseconds);

                // b. Set microseconds to RoundNumberToIncrement(fractionalMicroseconds, increment, roundingMode).
                result.time.microseconds =
                    utils::round_number_to_increment(frac_micros, increment, rounding_mode);

                // c. Set total to fractionalMicroseconds.
                // d. Set nanoseconds to 0.
                result.time.nanoseconds = 0.0;
                frac_micros
            }
            // 17. Else,
            TemporalUnit::Nanosecond => {
                // a. Assert: unit is "nanosecond".
                // b. Set total to nanoseconds.
                let total = result.time.nanoseconds;
                // c. Set nanoseconds to RoundNumberToIncrement(nanoseconds, increment, roundingMode).
                result.time.nanoseconds = utils::round_number_to_increment(
                    result.time.nanoseconds,
                    increment,
                    rounding_mode,
                );

                total
            }
            TemporalUnit::Auto => unreachable!(),
        };

        // 18. Let duration be ? CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
        // 19. Return the Record { [[DurationRecord]]: duration, [[Total]]: total }.
        Ok((result, total))
    }
}

// ==== Public Duration methods ====

impl Duration {
    /// Returns the absolute value of `Duration`.
    pub fn abs(&self) -> Self {
        Self {
            date: DateDuration::new(
                self.date.years.abs(),
                self.date.months.abs(),
                self.date.weeks.abs(),
                self.date.days.abs(),
            ),
            time: TimeDuration::new(
                self.time.hours.abs(),
                self.time.minutes.abs(),
                self.time.seconds.abs(),
                self.time.milliseconds.abs(),
                self.time.microseconds.abs(),
                self.time.nanoseconds.abs(),
            ),
        }
    }

    /// 7.5.10 `DurationSign ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )`
    ///
    /// Determines the sign for the current self.
    pub fn duration_sign(&self) -> i32 {
        // 1. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
        for v in self {
            // a. If v < 0, return -1.
            if v < 0_f64 {
                return -1;
            // b. If v > 0, return 1.
            } else if v > 0_f64 {
                return 1;
            }
        }
        // 2. Return 0.
        0
    }

    /// 7.5.12 `DefaultTemporalLargestUnit ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds )`
    pub fn default_temporal_largest_unit(&self) -> TemporalUnit {
        for (index, value) in self.into_iter().enumerate() {
            if value != 0.0 {
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
        }

        TemporalUnit::Nanosecond
    }

    /// Abstract Operation 7.5.17 `BalanceTimeDuration ( days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, largestUnit )`
    pub fn balance_time_duration(
        &self,
        largest_unit: TemporalUnit,
    ) -> TemporalResult<(f64, TimeDuration)> {
        // 1. Let balanceResult be ? BalancePossiblyInfiniteDuration(days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, largestUnit, relativeTo).
        let balance_result = self.balance_possibly_infinite_time_duration(largest_unit)?;

        // 2. If balanceResult is positive overflow or negative overflow, then
        let Some(result) = balance_result else {
            // a. Throw a RangeError exception.
            return Err(TemporalError::range()
                .with_message("duration overflowed viable range.")
                .into());
        };
        // 3. Else,
        // a. Return balanceResult.
        Ok(result)
    }
}
