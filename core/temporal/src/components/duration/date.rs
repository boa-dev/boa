//! Implementation of a `DateDuration`

use crate::{
    components::{
        calendar::CalendarProtocol, duration::TimeDuration, tz::TzProtocol, Date, DateTime,
        Duration, ZonedDateTime,
    },
    options::{ArithmeticOverflow, TemporalRoundingMode, TemporalUnit},
    utils, TemporalError, TemporalResult, NS_PER_DAY,
};

/// `DateDuration` represents the [date duration record][spec] of the `Duration.`
///
/// These fields are laid out in the [Temporal Proposal][field spec] as 64-bit floating point numbers.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-date-duration-records
/// [field spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[derive(Debug, Default, Clone, Copy)]
pub struct DateDuration {
    pub(crate) years: f64,
    pub(crate) months: f64,
    pub(crate) weeks: f64,
    pub(crate) days: f64,
}

impl DateDuration {
    /// Creates a new, non-validated `DateDuration`.
    #[inline]
    #[must_use]
    pub(crate) const fn new_unchecked(years: f64, months: f64, weeks: f64, days: f64) -> Self {
        Self {
            years,
            months,
            weeks,
            days,
        }
    }
}

impl DateDuration {
    /// Creates a new `DateDuration` with provided values.
    pub fn new(years: f64, months: f64, weeks: f64, days: f64) -> TemporalResult<Self> {
        let result = Self::new_unchecked(years, months, weeks, days);
        if !super::is_valid_duration(&result.into_iter().collect()) {
            return Err(TemporalError::range().with_message("Invalid DateDuration."));
        }
        Ok(result)
    }

    /// Returns a `PartialDateDuration` with all fields set to `NaN`.
    #[must_use]
    pub const fn partial() -> Self {
        Self {
            years: f64::NAN,
            months: f64::NAN,
            weeks: f64::NAN,
            days: f64::NAN,
        }
    }

    /// Creates a `DateDuration` from a provided partial `DateDuration`.
    #[must_use]
    pub fn from_partial(partial: &DateDuration) -> Self {
        Self {
            years: if partial.years.is_nan() {
                0.0
            } else {
                partial.years
            },
            months: if partial.months.is_nan() {
                0.0
            } else {
                partial.months
            },
            weeks: if partial.weeks.is_nan() {
                0.0
            } else {
                partial.weeks
            },
            days: if partial.days.is_nan() {
                0.0
            } else {
                partial.days
            },
        }
    }

    /// Returns a new `DateDuration` representing the absolute value of the current.
    #[inline]
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            years: self.years.abs(),
            months: self.months.abs(),
            weeks: self.weeks.abs(),
            days: self.days.abs(),
        }
    }

    /// Returns the `[[years]]` value.
    #[must_use]
    pub const fn years(&self) -> f64 {
        self.years
    }

    /// Returns the `[[months]]` value.
    #[must_use]
    pub const fn months(&self) -> f64 {
        self.months
    }

    /// Returns the `[[weeks]]` value.
    #[must_use]
    pub const fn weeks(&self) -> f64 {
        self.weeks
    }

    /// Returns the `[[days]]` value.
    #[must_use]
    pub const fn days(&self) -> f64 {
        self.days
    }

    /// Returns the iterator for `DateDuration`
    #[must_use]
    pub fn iter(&self) -> DateIter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

// ==== DateDuration Operations ====

impl DateDuration {
    /// Rounds the current `DateDuration` returning a tuple of the rounded `DateDuration` and
    /// the `total` value of the smallest unit prior to rounding.
    #[allow(clippy::type_complexity, clippy::let_and_return)]
    pub fn round<C: CalendarProtocol, Z: TzProtocol>(
        &self,
        additional_time: Option<TimeDuration>,
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
        // 1. If plainRelativeTo is not present, set plainRelativeTo to undefined.
        let plain_relative_to = relative_targets.0;
        // 2. If zonedRelativeTo is not present, set zonedRelativeTo to undefined.
        let zoned_relative_to = relative_targets.1;
        // 3. If precalculatedPlainDateTime is not present, set precalculatedPlainDateTime to undefined.
        let _ = relative_targets.2;

        let mut fractional_days = match unit {
            // 4. If unit is "year", "month", or "week", and plainRelativeTo is undefined, then
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week
                if plain_relative_to.is_none() =>
            {
                // a. Throw a RangeError exception.
                return Err(TemporalError::range()
                    .with_message("plainRelativeTo canot be undefined with given TemporalUnit"));
            }
            // 5. If unit is one of "year", "month", "week", or "day", then
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                // a. Let nanoseconds be TotalDurationNanoseconds(hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
                let nanoseconds = additional_time.unwrap_or_default().as_nanos();

                // b. If zonedRelativeTo is not undefined, then
                // i. Let intermediate be ? MoveRelativeZonedDateTime(zonedRelativeTo, years, months, weeks, days, precalculatedPlainDateTime).
                // ii. Let result be ? NanosecondsToDays(nanoseconds, intermediate).
                // iii. Let fractionalDays be days + result.[[Days]] + result.[[Nanoseconds]] / result.[[DayLength]].
                // c. Else,
                // i. Let fractionalDays be days + nanoseconds / nsPerDay.
                // d. Set days, hours, minutes, seconds, milliseconds, microseconds, and nanoseconds to 0.
                // e. Assert: fractionalSeconds is not used below.
                if zoned_relative_to.is_none() {
                    self.days + nanoseconds / NS_PER_DAY as f64
                } else {
                    // implementation of b: i-iii needed.
                    return Err(TemporalError::range().with_message("Not yet implemented."));
                }
            }
            _ => {
                return Err(TemporalError::range()
                    .with_message("Invalid TemporalUnit provided to DateDuration.round"))
            }
        };

        // 7. let total be unset.
        // We begin matching against unit and return the remainder value.
        match unit {
            // 8. If unit is "year", then
            TemporalUnit::Year => {
                let plain_relative_to = plain_relative_to.expect("this must exist.");
                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let calendar = plain_relative_to.calendar();

                // b. Let yearsDuration be ! CreateTemporalDuration(years, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let years = DateDuration::new_unchecked(self.years, 0.0, 0.0, 0.0);
                let years_duration = Duration::new_unchecked(years, TimeDuration::default());

                // c. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // d. Else,
                // i. Let dateAdd be unused.

                // e. Let yearsLater be ? AddDate(calendar, plainRelativeTo, yearsDuration, undefined, dateAdd).
                let years_later = plain_relative_to.add_date(
                    &years_duration,
                    ArithmeticOverflow::Constrain,
                    context,
                )?;

                // f. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = Duration::new_unchecked(
                    Self::new_unchecked(self.years, self.months, self.weeks, 0.0),
                    TimeDuration::default(),
                );

                // g. Let yearsMonthsWeeksLater be ? AddDate(calendar, plainRelativeTo, yearsMonthsWeeks, undefined, dateAdd).
                let years_months_weeks_later = plain_relative_to.add_date(
                    &years_months_weeks,
                    ArithmeticOverflow::Constrain,
                    context,
                )?;

                // h. Let monthsWeeksInDays be DaysUntil(yearsLater, yearsMonthsWeeksLater).
                let months_weeks_in_days = years_later.days_until(&years_months_weeks_later);

                // i. Set plainRelativeTo to yearsLater.
                let plain_relative_to = years_later;

                // j. Set fractionalDays to fractionalDays + monthsWeeksInDays.
                fractional_days += f64::from(months_weeks_in_days);

                // k. Let isoResult be ! AddISODate(plainRelativeTo.[[ISOYear]]. plainRelativeTo.[[ISOMonth]], plainRelativeTo.[[ISODay]], 0, 0, 0, truncate(fractionalDays), "constrain").
                let iso_result = plain_relative_to.iso().add_iso_date(
                    &DateDuration::new_unchecked(0.0, 0.0, 0.0, fractional_days.trunc()),
                    ArithmeticOverflow::Constrain,
                )?;

                // l. Let wholeDaysLater be ? CreateDate(isoResult.[[Year]], isoResult.[[Month]], isoResult.[[Day]], calendar).
                let whole_days_later = Date::new_unchecked(iso_result, calendar.clone());

                // m. Let untilOptions be OrdinaryObjectCreate(null).
                // n. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "year").
                // o. Let timePassed be ? DifferenceDate(calendar, plainRelativeTo, wholeDaysLater, untilOptions).
                let time_passed = plain_relative_to.difference_date(
                    &whole_days_later,
                    TemporalUnit::Year,
                    context,
                )?;

                // p. Let yearsPassed be timePassed.[[Years]].
                let years_passed = time_passed.date.years();

                // q. Set years to years + yearsPassed.
                let years = self.years() + years_passed;

                // r. Let yearsDuration be ! CreateTemporalDuration(yearsPassed, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_duration = Duration::one_year(years_passed);

                // s. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, yearsDuration, dateAdd).
                // t. Set plainRelativeTo to moveResult.[[RelativeTo]].
                // u. Let daysPassed be moveResult.[[Days]].
                let (plain_relative_to, days_passed) =
                    plain_relative_to.move_relative_date(&years_duration, context)?;

                // v. Set fractionalDays to fractionalDays - daysPassed.
                fractional_days -= days_passed;

                // w. If fractionalDays < 0, let sign be -1; else, let sign be 1.
                let sign = if fractional_days < 0.0 { -1 } else { 1 };

                // x. Let oneYear be ! CreateTemporalDuration(sign, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let one_year = Duration::one_year(f64::from(sign));

                // y. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneYear, dateAdd).
                // z. Let oneYearDays be moveResult.[[Days]].
                let (_, one_year_days) =
                    plain_relative_to.move_relative_date(&one_year, context)?;

                // aa. Let fractionalYears be years + fractionalDays / abs(oneYearDays).
                let frac_years = years + (fractional_days / one_year_days.abs());

                // ab. Set years to RoundNumberToIncrement(fractionalYears, increment, roundingMode).
                let rounded_years =
                    utils::round_number_to_increment(frac_years, increment, rounding_mode);

                // ac. Set total to fractionalYears.
                // ad. Set months and weeks to 0.
                let result = Self::new(rounded_years, 0f64, 0f64, 0f64)?;
                Ok((result, frac_years))
            }
            // 9. Else if unit is "month", then
            TemporalUnit::Month => {
                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let plain_relative_to = plain_relative_to.expect("this must exist.");

                // b. Let yearsMonths be ! CreateTemporalDuration(years, months, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_months = Duration::from_date_duration(DateDuration::new_unchecked(
                    self.years(),
                    self.months(),
                    0.0,
                    0.0,
                ));

                // c. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // d. Else,
                // i. Let dateAdd be unused.

                // e. Let yearsMonthsLater be ? AddDate(calendar, plainRelativeTo, yearsMonths, undefined, dateAdd).
                let years_months_later = plain_relative_to.add_date(
                    &years_months,
                    ArithmeticOverflow::Constrain,
                    context,
                )?;

                // f. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = Duration::from_date_duration(DateDuration::new_unchecked(
                    self.years(),
                    self.months(),
                    self.weeks(),
                    0.0,
                ));

                // g. Let yearsMonthsWeeksLater be ? AddDate(calendar, plainRelativeTo, yearsMonthsWeeks, undefined, dateAdd).
                let years_months_weeks_later = plain_relative_to.add_date(
                    &years_months_weeks,
                    ArithmeticOverflow::Constrain,
                    context,
                )?;

                // h. Let weeksInDays be DaysUntil(yearsMonthsLater, yearsMonthsWeeksLater).
                let weeks_in_days = years_months_later.days_until(&years_months_weeks_later);

                // i. Set plainRelativeTo to yearsMonthsLater.
                let plain_relative_to = years_months_later;

                // j. Set fractionalDays to fractionalDays + weeksInDays.
                fractional_days += f64::from(weeks_in_days);

                // k. If fractionalDays < 0, let sign be -1; else, let sign be 1.
                let sign = if fractional_days < 0.0 { -1f64 } else { 1f64 };

                // l. Let oneMonth be ! CreateTemporalDuration(0, sign, 0, 0, 0, 0, 0, 0, 0, 0).
                let one_month = Duration::one_month(sign);

                // m. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
                // n. Set plainRelativeTo to moveResult.[[RelativeTo]].
                // o. Let oneMonthDays be moveResult.[[Days]].
                let (mut plain_relative_to, mut one_month_days) =
                    plain_relative_to.move_relative_date(&one_month, context)?;

                let mut months = self.months;
                // p. Repeat, while abs(fractionalDays) ≥ abs(oneMonthDays),
                while fractional_days.abs() >= one_month_days.abs() {
                    // i. Set months to months + sign.
                    months += sign;

                    // ii. Set fractionalDays to fractionalDays - oneMonthDays.
                    fractional_days -= one_month_days;

                    // iii. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
                    let move_result = plain_relative_to.move_relative_date(&one_month, context)?;

                    // iv. Set plainRelativeTo to moveResult.[[RelativeTo]].
                    plain_relative_to = move_result.0;
                    // v. Set oneMonthDays to moveResult.[[Days]].
                    one_month_days = move_result.1;
                }

                // q. Let fractionalMonths be months + fractionalDays / abs(oneMonthDays).
                let frac_months = months + fractional_days / one_month_days.abs();

                // r. Set months to RoundNumberToIncrement(fractionalMonths, increment, roundingMode).
                let rounded_months =
                    utils::round_number_to_increment(frac_months, increment, rounding_mode);

                // s. Set total to fractionalMonths.
                // t. Set weeks to 0.
                let result = Self::new(self.years, rounded_months, 0f64, 0f64)?;
                Ok((result, frac_months))
            }
            // 10. Else if unit is "week", then
            TemporalUnit::Week => {
                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let plain_relative_to = plain_relative_to.expect("date must exist given Week");

                // b. If fractionalDays < 0, let sign be -1; else, let sign be 1.
                let sign = if fractional_days < 0.0 { -1f64 } else { 1f64 };

                // c. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
                let one_week = Duration::one_week(sign);

                // d. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // e. Else,
                // i. Let dateAdd be unused.

                // f. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneWeek, dateAdd).
                // g. Set plainRelativeTo to moveResult.[[RelativeTo]].
                // h. Let oneWeekDays be moveResult.[[Days]].
                let (mut plain_relative_to, mut one_week_days) =
                    plain_relative_to.move_relative_date(&one_week, context)?;

                let mut weeks = self.weeks;
                // i. Repeat, while abs(fractionalDays) ≥ abs(oneWeekDays),
                while fractional_days.abs() >= one_week_days.abs() {
                    // i. Set weeks to weeks + sign.
                    weeks += sign;

                    // ii. Set fractionalDays to fractionalDays - oneWeekDays.
                    fractional_days -= one_week_days;

                    // iii. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneWeek, dateAdd).
                    let move_result = plain_relative_to.move_relative_date(&one_week, context)?;

                    // iv. Set plainRelativeTo to moveResult.[[RelativeTo]].
                    plain_relative_to = move_result.0;
                    // v. Set oneWeekDays to moveResult.[[Days]].
                    one_week_days = move_result.1;
                }

                // j. Let fractionalWeeks be weeks + fractionalDays / abs(oneWeekDays).
                let frac_weeks = weeks + fractional_days / one_week_days.abs();

                // k. Set weeks to RoundNumberToIncrement(fractionalWeeks, increment, roundingMode).
                let rounded_weeks =
                    utils::round_number_to_increment(frac_weeks, increment, rounding_mode);
                // l. Set total to fractionalWeeks.
                let result = Self::new(self.years, self.months, rounded_weeks, 0f64)?;
                Ok((result, frac_weeks))
            }
            // 11. Else if unit is "day", then
            TemporalUnit::Day => {
                // a. Set days to RoundNumberToIncrement(fractionalDays, increment, roundingMode).
                let rounded_days =
                    utils::round_number_to_increment(fractional_days, increment, rounding_mode);
                // b. Set total to fractionalDays.
                let result = Self::new(self.years, self.months, self.weeks, rounded_days)?;
                Ok((result, fractional_days))
            }
            _ => unreachable!("All other TemporalUnits were returned early as invalid."),
        }
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
