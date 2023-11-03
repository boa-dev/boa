//! The `DurationRecord` implements the internal representation of a Temporal Duration.

use crate::{
    builtins::{
        options::RoundingMode,
        temporal::{
            self,
            options::{ArithmeticOverflow, TemporalUnit},
            round_number_to_increment, to_temporal_date, NS_PER_DAY,
        },
    },
    js_string,
    string::utf16,
    Context, JsNativeError, JsObject, JsResult, JsValue,
};

use super::super::{
    calendar, plain_date, to_integer_if_integral, PlainDate, PlainDateTime, ZonedDateTime,
};

// ==== `DateDuration` ====

/// `DateDuration` represents the [date duration record][spec] of the `DurationRecord.`
///
/// These fields are laid out in the [Temporal Proposal][field spec] as 64-bit floating point numbers.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-date-duration-records
/// [field spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct DateDuration {
    years: f64,
    months: f64,
    weeks: f64,
    days: f64,
}

impl DateDuration {
    pub(crate) const fn new(years: f64, months: f64, weeks: f64, days: f64) -> Self {
        Self {
            years,
            months,
            weeks,
            days,
        }
    }

    pub(crate) const fn partial() -> Self {
        Self {
            years: f64::NAN,
            months: f64::NAN,
            weeks: f64::NAN,
            days: f64::NAN,
        }
    }

    pub(crate) const fn years(&self) -> f64 {
        self.years
    }

    pub(crate) const fn months(&self) -> f64 {
        self.months
    }

    pub(crate) const fn weeks(&self) -> f64 {
        self.weeks
    }

    pub(crate) const fn days(&self) -> f64 {
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

pub(crate) struct DateIter<'a> {
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

/// `TimeDuration` represents the [Time Duration record][spec] of the `DurationRecord.`
///
/// These fields are laid out in the [Temporal Proposal][field spec] as 64-bit floating point numbers.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-time-duration-records
/// [field spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TimeDuration {
    hours: f64,
    minutes: f64,
    seconds: f64,
    milliseconds: f64,
    microseconds: f64,
    nanoseconds: f64,
}

impl TimeDuration {
    pub(crate) const fn new(
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

    pub(crate) const fn partial() -> Self {
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
    pub(crate) fn is_within_range(&self) -> bool {
        self.hours.abs() < 24f64
            && self.minutes.abs() < 60f64
            && self.seconds.abs() < 60f64
            && self.milliseconds.abs() < 1000f64
            && self.milliseconds.abs() < 1000f64
            && self.milliseconds.abs() < 1000f64
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

pub(crate) struct TimeIter<'a> {
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

// ==== `DurationRecord` ====

/// The `DurationRecord` is a native Rust implementation of the `Duration` builtin
/// object internal fields and is primarily defined by Abtract Operation 7.5.1-5.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DurationRecord {
    date: DateDuration,
    time: TimeDuration,
}

impl DurationRecord {
    pub(crate) const fn new(date: DateDuration, time: TimeDuration) -> Self {
        Self { date, time }
    }

    pub(crate) const fn partial() -> Self {
        Self {
            date: DateDuration::partial(),
            time: TimeDuration::partial(),
        }
    }

    pub(crate) fn from_date_duration(date: DateDuration) -> Self {
        Self {
            date,
            time: TimeDuration::default(),
        }
    }

    pub(crate) const fn from_day_and_time(day: f64, time: TimeDuration) -> Self {
        Self {
            date: DateDuration::new(0.0, 0.0, 0.0, day),
            time,
        }
    }

    /// Utility function to create a one year duration.
    pub(crate) fn one_year(year_value: f64) -> Self {
        Self::from_date_duration(DateDuration::new(year_value, 0f64, 0f64, 0f64))
    }

    /// Utility function to create a one month duration.
    pub(crate) fn one_month(month_value: f64) -> Self {
        Self::from_date_duration(DateDuration::new(0f64, month_value, 0f64, 0f64))
    }

    /// Utility function to create a one week duration.
    pub(crate) fn one_week(week_value: f64) -> Self {
        Self::from_date_duration(DateDuration::new(0f64, 0f64, week_value, 0f64))
    }

    /// Utility function to return if the Durations values are within their valid ranges.
    #[inline]
    pub(crate) fn is_time_within_range(&self) -> bool {
        self.time.is_within_range()
    }

    /// Equivalent to 7.5.13 `ToTemporalPartialDurationRecord ( temporalDurationLike )`
    ///
    /// Takes an unknown `JsObject` and attempts to create a partial duration
    pub(crate) fn from_partial_js_object(
        duration_like: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        // 1. If Type(temporalDurationLike) is not Object, then
        let JsValue::Object(unknown_object) = duration_like else {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message("temporalDurationLike must be an object.")
                .into());
        };

        // 2. Let result be a new partial Duration Record with each field set to undefined.
        let mut result = Self::partial();

        // 3. NOTE: The following steps read properties and perform independent validation in alphabetical order.
        // 4. Let days be ? Get(temporalDurationLike, "days").
        let days = unknown_object.get(utf16!("days"), context)?;
        if !days.is_undefined() {
            // 5. If days is not undefined, set result.[[Days]] to ? ToIntegerIfIntegral(days).
            result.set_days(f64::from(to_integer_if_integral(&days, context)?));
        }

        // 6. Let hours be ? Get(temporalDurationLike, "hours").
        let hours = unknown_object.get(utf16!("hours"), context)?;
        // 7. If hours is not undefined, set result.[[Hours]] to ? ToIntegerIfIntegral(hours).
        if !hours.is_undefined() {
            result.set_days(f64::from(to_integer_if_integral(&hours, context)?));
        }

        // 8. Let microseconds be ? Get(temporalDurationLike, "microseconds").
        let microseconds = unknown_object.get(utf16!("microseconds"), context)?;
        // 9. If microseconds is not undefined, set result.[[Microseconds]] to ? ToIntegerIfIntegral(microseconds).
        if !microseconds.is_undefined() {
            result.set_days(f64::from(to_integer_if_integral(&microseconds, context)?));
        }

        // 10. Let milliseconds be ? Get(temporalDurationLike, "milliseconds").
        let milliseconds = unknown_object.get(utf16!("milliseconds"), context)?;
        // 11. If milliseconds is not undefined, set result.[[Milliseconds]] to ? ToIntegerIfIntegral(milliseconds).
        if !milliseconds.is_undefined() {
            result.set_days(f64::from(to_integer_if_integral(&milliseconds, context)?));
        }

        // 12. Let minutes be ? Get(temporalDurationLike, "minutes").
        let minutes = unknown_object.get(utf16!("minutes"), context)?;
        // 13. If minutes is not undefined, set result.[[Minutes]] to ? ToIntegerIfIntegral(minutes).
        if !minutes.is_undefined() {
            result.set_days(f64::from(to_integer_if_integral(&minutes, context)?));
        }

        // 14. Let months be ? Get(temporalDurationLike, "months").
        let months = unknown_object.get(utf16!("months"), context)?;
        // 15. If months is not undefined, set result.[[Months]] to ? ToIntegerIfIntegral(months).
        if !months.is_undefined() {
            result.set_days(f64::from(to_integer_if_integral(&months, context)?));
        }

        // 16. Let nanoseconds be ? Get(temporalDurationLike, "nanoseconds").
        let nanoseconds = unknown_object.get(utf16!("nanoseconds"), context)?;
        // 17. If nanoseconds is not undefined, set result.[[Nanoseconds]] to ? ToIntegerIfIntegral(nanoseconds).
        if !nanoseconds.is_undefined() {
            result.set_days(f64::from(to_integer_if_integral(&nanoseconds, context)?));
        }

        // 18. Let seconds be ? Get(temporalDurationLike, "seconds").
        let seconds = unknown_object.get(utf16!("seconds"), context)?;
        // 19. If seconds is not undefined, set result.[[Seconds]] to ? ToIntegerIfIntegral(seconds).
        if !seconds.is_undefined() {
            result.set_days(f64::from(to_integer_if_integral(&seconds, context)?));
        }

        // 20. Let weeks be ? Get(temporalDurationLike, "weeks").
        let weeks = unknown_object.get(utf16!("weeks"), context)?;
        // 21. If weeks is not undefined, set result.[[Weeks]] to ? ToIntegerIfIntegral(weeks).
        if !weeks.is_undefined() {
            result.set_days(f64::from(to_integer_if_integral(&weeks, context)?));
        }

        // 22. Let years be ? Get(temporalDurationLike, "years").
        let years = unknown_object.get(utf16!("years"), context)?;
        // 23. If years is not undefined, set result.[[Years]] to ? ToIntegerIfIntegral(years).
        if !years.is_undefined() {
            result.set_days(f64::from(to_integer_if_integral(&years, context)?));
        }

        // 24. If years is undefined, and months is undefined, and weeks is undefined, and days is undefined, and hours is undefined, and minutes is undefined, and seconds is undefined, and milliseconds is undefined, and microseconds is undefined, and nanoseconds is undefined, throw a TypeError exception.
        if result.into_iter().all(f64::is_nan) {
            return Err(JsNativeError::typ()
                .with_message("no valid Duration fields on temporalDurationLike.")
                .into());
        }

        // 25. Return result.
        Ok(result)
    }
}

// -- `DurationRecord` bubble/balance methods --

impl DurationRecord {
    /// Balance/bubble the current unit from one step down.
    fn balance_hours(&mut self) {
        // 1. Set hours to floor(minutes / 60).
        self.set_hours((self.minutes() / 60_f64).floor());
        // 2. Set minutes to minutes modulo 60.
        self.set_minutes(self.minutes() % 60_f64);
    }

    /// Balance/bubble the current unit from one step down.
    fn balance_minutes(&mut self) {
        // 1. Set minutes to floor(seconds / 60).
        self.set_minutes((self.seconds() / 60_f64).floor());
        // 2. Set seconds to seconds modulo 60.
        self.set_seconds(self.seconds() % 60_f64);
    }

    /// Balance/bubble the current unit from one step down.
    fn balance_seconds(&mut self) {
        // 1. Set seconds to floor(milliseconds / 1000).
        self.set_seconds((self.milliseconds() / 1_000_f64).floor());
        // 2. Set milliseconds to milliseconds modulo 1000.
        self.set_milliseconds(self.milliseconds() % 1_000_f64);
    }

    /// Balance/bubble the current unit from one step down.
    fn balance_milliseconds(&mut self) {
        // c. Set milliseconds to floor(microseconds / 1000).
        self.set_milliseconds((self.microseconds() / 1_000_f64).floor());
        // d. Set microseconds to microseconds modulo 1000.
        self.set_microseconds(self.microseconds() % 1_000_f64);
    }

    /// Balance/bubble the current unit from one step down.
    fn balance_microseconds(&mut self) {
        // a. Set microseconds to floor(nanoseconds / 1000).
        self.set_microseconds((self.nanoseconds() / 1_000_f64).floor());
        // b. Set nanoseconds to nanoseconds modulo 1000.
        self.set_nanoseconds(self.nanoseconds() % 1_000_f64);
    }
}

// ==== `DurationRecord` getter/setter methods ====

impl DurationRecord {
    /// Return this `DurationRecord`'s `DateDuration`
    pub(crate) const fn date(&self) -> DateDuration {
        self.date
    }

    /// Return this `DurationRecord`'s `TimeDuration`
    pub(crate) const fn time(&self) -> TimeDuration {
        self.time
    }

    /// Set this `DurationRecord`'s `TimeDuration`.
    pub(crate) fn set_time_duration(&mut self, time: TimeDuration) {
        self.time = time;
    }

    /// Set the value for `years`.
    pub(crate) fn set_years(&mut self, y: f64) {
        self.date.years = y;
    }

    /// Return the value for `years`.
    pub(crate) const fn years(&self) -> f64 {
        self.date.years
    }

    /// Set the value for `months`.
    pub(crate) fn set_months(&mut self, mo: f64) {
        self.date.months = mo;
    }

    /// Return the value for `months`.
    pub(crate) const fn months(&self) -> f64 {
        self.date.months
    }

    /// Set the value for `weeks`.
    pub(crate) fn set_weeks(&mut self, w: f64) {
        self.date.weeks = w;
    }

    /// Return the value for `weeks`.
    pub(crate) const fn weeks(&self) -> f64 {
        self.date.weeks
    }

    /// Set the value for `days`.
    pub(crate) fn set_days(&mut self, d: f64) {
        self.date.days = d;
    }

    /// Return the value for `days`.
    pub(crate) const fn days(&self) -> f64 {
        self.date.days
    }

    /// Set the value for `hours`.
    pub(crate) fn set_hours(&mut self, h: f64) {
        self.time.hours = h;
    }

    /// Return the value for `hours`.
    pub(crate) const fn hours(&self) -> f64 {
        self.time.hours
    }

    /// Set the value for `minutes`.
    pub(crate) fn set_minutes(&mut self, m: f64) {
        self.time.minutes = m;
    }

    /// Return the value for `minutes`.
    pub(crate) const fn minutes(&self) -> f64 {
        self.time.minutes
    }

    /// Set the value for `seconds`.
    pub(crate) fn set_seconds(&mut self, s: f64) {
        self.time.seconds = s;
    }

    /// Return the value for `seconds`.
    pub(crate) const fn seconds(&self) -> f64 {
        self.time.seconds
    }

    /// Set the value for `milliseconds`.
    pub(crate) fn set_milliseconds(&mut self, ms: f64) {
        self.time.milliseconds = ms;
    }

    /// Return the value for `milliseconds`.
    pub(crate) const fn milliseconds(&self) -> f64 {
        self.time.milliseconds
    }

    /// Set the value for `microseconds`.
    pub(crate) fn set_microseconds(&mut self, mis: f64) {
        self.time.microseconds = mis;
    }

    /// Return the value for `microseconds`.
    pub(crate) const fn microseconds(&self) -> f64 {
        self.time.microseconds
    }

    /// Set the value for `nanoseconds`.
    pub(crate) fn set_nanoseconds(&mut self, ns: f64) {
        self.time.nanoseconds = ns;
    }

    /// Return the value for `nanoseconds`.
    pub(crate) const fn nanoseconds(&self) -> f64 {
        self.time.nanoseconds
    }
}

impl<'a> IntoIterator for &'a DurationRecord {
    type Item = f64;
    type IntoIter = DurationIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DurationIter {
            duration: self,
            index: 0,
        }
    }
}

pub(crate) struct DurationIter<'a> {
    duration: &'a DurationRecord,
    index: usize,
}

impl Iterator for DurationIter<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(self.duration.years()),
            1 => Some(self.duration.months()),
            2 => Some(self.duration.weeks()),
            3 => Some(self.duration.days()),
            4 => Some(self.duration.hours()),
            5 => Some(self.duration.minutes()),
            6 => Some(self.duration.seconds()),
            7 => Some(self.duration.milliseconds()),
            8 => Some(self.duration.microseconds()),
            9 => Some(self.duration.nanoseconds()),
            _ => None,
        };
        self.index += 1;
        result
    }
}

// ==== DurationRecord method ====

impl DurationRecord {
    pub(crate) fn abs(&self) -> Self {
        Self {
            date: DateDuration::new(
                self.years().abs(),
                self.months().abs(),
                self.weeks().abs(),
                self.days().abs(),
            ),
            time: TimeDuration::new(
                self.hours().abs(),
                self.minutes().abs(),
                self.seconds().abs(),
                self.milliseconds().abs(),
                self.microseconds().abs(),
                self.nanoseconds().abs(),
            ),
        }
    }
}

// ==== Abstract Operations implemented on `DurationRecord` ====

impl DurationRecord {
    // TODO: look into making this destructive / Into.
    // Trace current callers and check whether the value
    // can be fed a native `DurationRecord` instead.
    /// Creates a `Duration` object from the current `DurationRecord`.
    pub(crate) fn as_object(&self, context: &mut Context<'_>) -> JsResult<JsObject> {
        super::create_temporal_duration(*self, None, context)
    }

    /// Returns the duration time values as a vec
    fn time_values(&self) -> Vec<f64> {
        self.time.into_iter().collect()
    }

    // Note(nekevss): This currently assumes that an overflow has been stored into the years
    // column as the duration is nonviable and storing it in years allows for invalidating
    // the duration the fastest.
    /// Determines if the `DurationRecord` has overflowed.
    #[inline]
    fn is_overfowed(&self) -> bool {
        self.years().is_infinite()
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn is_positive_overflow(&self) -> bool {
        self.years().is_infinite() && self.years().is_sign_positive()
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn is_negative_overflow(&self) -> bool {
        self.years().is_infinite() && self.years().is_sign_negative()
    }

    /// 7.5.10 `DurationSign ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )`
    ///
    /// Determines the sign for the current self.
    pub(crate) fn duration_sign(&self) -> i32 {
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

    /// 7.5.11 `IsValidDuration ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )`
    ///
    /// Checks if the current `DurationRecord` is a valid self.
    pub(crate) fn is_valid_duration(&self) -> bool {
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

    /// 7.5.12 `DefaultTemporalLargestUnit ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds )`
    pub(crate) fn default_temporal_largest_unit(&self) -> TemporalUnit {
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

    // TODO: implement on `DurationRecord`
    /// 7.5.17 `TotalDurationNanoseconds ( days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, offsetShift )`
    fn total_duration_nanoseconds(&self, offset_shift: f64) -> f64 {
        let nanoseconds = if self.days() == 0_f64 {
            self.nanoseconds()
        } else {
            self.nanoseconds() - offset_shift
        };

        self.days()
            .mul_add(24_f64, self.hours())
            .mul_add(60_f64, self.minutes())
            .mul_add(60_f64, self.seconds())
            .mul_add(1_000_f64, self.milliseconds())
            .mul_add(1_000_f64, self.microseconds())
            .mul_add(1_000_f64, nanoseconds)
    }

    /// Abstract Operation 7.5.18 `BalanceTimeDuration ( days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, largestUnit [ , relativeTo ] )`
    pub(crate) fn balance_time_duration(
        &mut self,
        largest_unit: TemporalUnit,
        relative_to: Option<&JsValue>,
    ) -> JsResult<()> {
        // 1. Let balanceResult be ? BalancePossiblyInfiniteDuration(days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, largestUnit, relativeTo).
        self.balance_possibly_infinite_duration(largest_unit, relative_to)?;
        // 2. If balanceResult is positive overflow or negative overflow, then
        if self.is_overfowed() {
            // a. Throw a RangeError exception.
            return Err(JsNativeError::range()
                .with_message("duration overflowed viable range.")
                .into());
        }
        // 3. Else,
        // a. Return balanceResult.
        Ok(())
    }

    /// Abstract Operation 7.5.19 `BalancePossiblyInfiniteDuration ( days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, largestUnit [ , relativeTo ] )`
    pub(crate) fn balance_possibly_infinite_duration(
        &mut self,
        largest_unit: TemporalUnit,
        relative_to: Option<&JsValue>,
    ) -> JsResult<()> {
        // 1. If relativeTo is not present, set relativeTo to undefined.
        let relative_to = if let Some(value) = relative_to {
            value.clone()
        } else {
            JsValue::undefined()
        };

        // 2. If Type(relativeTo) is Object and relativeTo has an [[InitializedTemporalZonedDateTime]] internal slot, then
        if relative_to.is_object()
            && relative_to
                .as_object()
                .expect("relative_to must be an object here.")
                .is_zoned_date_time()
        {
            // TODO
            // a. Let endNs be ? AddZonedDateTime(relativeTo.[[Nanoseconds]], relativeTo.[[TimeZone]], relativeTo.[[Calendar]], 0, 0, 0, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
            // b. Set nanoseconds to ℝ(endNs - relativeTo.[[Nanoseconds]]).
            self.set_nanoseconds(0_f64);
        // 3. Else,
        } else {
            // a. Set nanoseconds to ! TotalDurationNanoseconds(days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, 0).
            self.set_nanoseconds(self.total_duration_nanoseconds(0.0));
        }

        match largest_unit {
            // 4. If largestUnit is one of "year", "month", "week", or "day", then
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                // a. Let result be ? NanosecondsToDays(nanoseconds, relativeTo).
                let _result = temporal::zoned_date_time::nanoseconds_to_days(
                    self.nanoseconds(),
                    &relative_to,
                );
                // b. Set days to result.[[Days]].
                // c. Set nanoseconds to result.[[Nanoseconds]].
                return Err(JsNativeError::error()
                    .with_message("not yet implemented.")
                    .into());
            }
            // 5. Else,
            // a. Set days to 0.
            _ => self.set_days(0_f64),
        }

        // 6. Set hours, minutes, seconds, milliseconds, and microseconds to 0.
        let new_time = TimeDuration::new(0_f64, 0_f64, 0_f64, 0_f64, 0_f64, self.nanoseconds());
        self.time = new_time;

        // 7. If nanoseconds < 0, let sign be -1; else, let sign be 1.
        let sign = if self.nanoseconds() < 0_f64 {
            -1_f64
        } else {
            1_f64
        };
        // 8. Set nanoseconds to abs(nanoseconds).
        self.set_nanoseconds(self.nanoseconds().abs());

        match largest_unit {
            // 9. If largestUnit is "year", "month", "week", "day", or "hour", then
            TemporalUnit::Year
            | TemporalUnit::Month
            | TemporalUnit::Week
            | TemporalUnit::Day
            | TemporalUnit::Hour => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                self.balance_microseconds();

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                self.balance_milliseconds();

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                self.balance_minutes();

                // g. Set minutes to floor(seconds / 60).
                // h. Set seconds to seconds modulo 60.
                self.balance_minutes();

                // i. Set hours to floor(minutes / 60).
                // j. Set minutes to minutes modulo 60.
                self.balance_hours();
            }
            // 10. Else if largestUnit is "minute", then
            TemporalUnit::Minute => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                self.balance_microseconds();

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                self.balance_milliseconds();

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                self.balance_seconds();

                // g. Set minutes to floor(seconds / 60).
                // h. Set seconds to seconds modulo 60.
                self.balance_minutes();
            }
            // 11. Else if largestUnit is "second", then
            TemporalUnit::Second => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                self.balance_microseconds();

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                self.balance_milliseconds();

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                self.balance_seconds();
            }
            // 12. Else if largestUnit is "millisecond", then
            TemporalUnit::Millisecond => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                self.balance_microseconds();

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                self.balance_milliseconds();
            }
            // 13. Else if largestUnit is "microsecond", then
            TemporalUnit::Microsecond => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                self.balance_microseconds();
            }
            // 14. Else,
            // a. Assert: largestUnit is "nanosecond".
            _ => assert!(largest_unit == TemporalUnit::Nanosecond),
        }

        // 15. For each value v of « days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
        for value in self.time_values() {
            // a. If 𝔽(v) is not finite, then
            if !value.is_finite() {
                // i. If sign = 1, then
                if sign as i32 == 1 {
                    // 1. Return positive overflow.
                    self.set_years(f64::INFINITY);
                    return Ok(());
                }
                // ii. Else if sign = -1, then
                // 1. Return negative overflow.
                self.set_years(f64::NEG_INFINITY);
                return Ok(());
            }
        }

        // NOTE (nekevss): diviate from spec here as the current implementation with `DurationRecord` means that we create the record and than mutate values.
        // 16. Return ? CreateTimeDurationRecord(days, hours × sign, minutes × sign, seconds × sign, milliseconds × sign, microseconds × sign, nanoseconds × sign).
        self.set_hours(self.hours() * sign);
        self.set_minutes(self.minutes() * sign);
        self.set_seconds(self.seconds() * sign);
        self.set_milliseconds(self.milliseconds() * sign);
        self.set_microseconds(self.microseconds() * sign);
        self.set_nanoseconds(self.nanoseconds() * sign);

        // `CreateTimeDurationRecord` validates that the record that would be created is a valid duration, so validate here
        if !self.is_valid_duration() {
            return Err(JsNativeError::range()
                .with_message("TimeDurationRecord was not a valid duration.")
                .into());
        }

        Ok(())
    }

    /// 7.5.21 `UnbalanceDateDurationRelative ( years, months, weeks, days, largestUnit, plainRelativeTo )`
    #[allow(dead_code)]
    pub(crate) fn unbalance_duration_relative(
        &self,
        largest_unit: TemporalUnit,
        plain_relative_to: Option<&PlainDate>,
        context: &mut Context<'_>,
    ) -> JsResult<DateDuration> {
        // 1. Let allZero be false.
        // 2. If years = 0, and months = 0, and weeks = 0, and days = 0, set allZero to true.
        let all_zero = self.years() == 0_f64
            && self.months() == 0_f64
            && self.weeks() == 0_f64
            && self.days() == 0_f64;

        // 3. If largestUnit is "year" or allZero is true, then
        if largest_unit == TemporalUnit::Year || all_zero {
            // a. Return ! CreateDateDurationRecord(years, months, weeks, days).
            return Ok(self.date());
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
            if self.years() == 0f64 {
                return Ok(DateDuration::new(
                    0f64,
                    self.months(),
                    self.weeks(),
                    self.days(),
                ));
            }

            // b. If calendar is undefined, then
            let (mut plain_relative_to, calendar) =
                if let Some(plain_relative_to) = plain_relative_to {
                    (
                        PlainDate::new(plain_relative_to.inner, plain_relative_to.calendar.clone()),
                        plain_relative_to.calendar.clone(),
                    )
                } else {
                    // i. Throw a RangeError exception.
                    return Err(JsNativeError::range()
                        .with_message("Calendar cannot be undefined.")
                        .into());
                };

            // c. If calendar is an Object, then
            // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
            // ii. Let dateUntil be ? GetMethod(calendar, "dateUntil").
            // d. Else,
            // i. Let dateAdd be unused.
            // ii. Let dateUntil be unused.

            let mut years = self.years();
            let mut months = self.months();
            // e. Repeat, while years ≠ 0,
            while years != 0f64 {
                // i. Let newRelativeTo be ? CalendarDateAdd(calendar, plainRelativeTo, oneYear, undefined, dateAdd).
                let new_relative_to = calendar::calendar_date_add(
                    &calendar,
                    &plain_relative_to,
                    &one_year,
                    &JsValue::undefined(),
                    context,
                )?;

                // ii. Let untilOptions be OrdinaryObjectCreate(null).
                let until_options = JsObject::with_null_proto();
                // iii. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "month").
                until_options.create_data_property_or_throw(
                    utf16!("largestUnit"),
                    js_string!("month"),
                    context,
                )?;

                // iv. Let untilResult be ? CalendarDateUntil(calendar, plainRelativeTo, newRelativeTo, untilOptions, dateUntil).
                let until_result = calendar::calendar_date_until(
                    &calendar,
                    &plain_relative_to,
                    &new_relative_to,
                    &until_options.into(),
                    context,
                )?;

                // v. Let oneYearMonths be untilResult.[[Months]].
                let one_year_months = until_result.months();

                // vi. Set plainRelativeTo to newRelativeTo.
                plain_relative_to = new_relative_to;

                // vii. Set years to years - sign.
                years -= sign;
                // viii. Set months to months + oneYearMonths.
                months += one_year_months;
            }
            // f. Return ? CreateDateDurationRecord(0, months, weeks, days).
            return Ok(DateDuration::new(years, months, self.weeks(), self.days()));

        // 12. If largestUnit is "week", then
        } else if largest_unit == TemporalUnit::Week {
            // a. If years = 0 and months = 0, return ! CreateDateDurationRecord(0, 0, weeks, days).
            if self.years() == 0f64 && self.months() == 0f64 {
                return Ok(DateDuration::new(0f64, 0f64, self.weeks(), self.days()));
            }

            // b. If calendar is undefined, then
            let (mut plain_relative_to, calendar) =
                if let Some(plain_relative_to) = plain_relative_to {
                    (
                        PlainDate::new(plain_relative_to.inner, plain_relative_to.calendar.clone()),
                        plain_relative_to.calendar.clone(),
                    )
                } else {
                    // i. Throw a RangeError exception.
                    return Err(JsNativeError::range()
                        .with_message("Calendar cannot be undefined.")
                        .into());
                };

            // c. If calendar is an Object, then
            // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
            // d. Else,
            // i. Let dateAdd be unused.

            let mut years = self.years();
            let mut days = self.days();
            // e. Repeat, while years ≠ 0,
            while years != 0f64 {
                // i. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneYear, dateAdd).
                let move_result =
                    super::move_relative_date(&calendar, &plain_relative_to, &one_year, context)?;

                // ii. Set plainRelativeTo to moveResult.[[RelativeTo]].
                plain_relative_to = move_result.0;
                // iii. Set days to days + moveResult.[[Days]].
                days += move_result.1;
                // iv. Set years to years - sign.
                years -= sign;
            }

            let mut months = self.months();
            // f. Repeat, while months ≠ 0,
            while months != 0f64 {
                // i. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
                let move_result =
                    super::move_relative_date(&calendar, &plain_relative_to, &one_month, context)?;
                // ii. Set plainRelativeTo to moveResult.[[RelativeTo]].
                plain_relative_to = move_result.0;
                // iii. Set days to days + moveResult.[[Days]].
                days += move_result.1;
                // iv. Set months to months - sign.
                months -= sign;
            }
            // g. Return ? CreateDateDurationRecord(0, 0, weeks, days).
            return Ok(DateDuration::new(0f64, 0f64, self.weeks(), days));
        }

        // 13. If years = 0, and months = 0, and weeks = 0, return ! CreateDateDurationRecord(0, 0, 0, days).
        if self.years() == 0f64 && self.months() == 0f64 && self.weeks() == 0f64 {
            return Ok(DateDuration::new(0f64, 0f64, 0f64, self.days()));
        }

        // NOTE: Move 8 down to past 13 as we only use one_week after making it past 13.
        // 8. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
        let one_week = Self::one_week(sign);

        // 14. If calendar is undefined, then
        let (mut plain_relative_to, calendar) = if let Some(plain_relative_to) = plain_relative_to {
            (
                PlainDate::new(plain_relative_to.inner, plain_relative_to.calendar.clone()),
                plain_relative_to.calendar.clone(),
            )
        } else {
            // a. Throw a RangeError exception.
            return Err(JsNativeError::range()
                .with_message("Calendar cannot be undefined.")
                .into());
        };

        // 15. If calendar is an Object, then
        // a. Let dateAdd be ? GetMethod(calendar, "dateAdd").
        // 16. Else,
        // a. Let dateAdd be unused.

        let mut years = self.years();
        let mut days = self.days();
        // a. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneYear, dateAdd).
        while years != 0f64 {
            // a. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneYear, dateAdd).
            let move_result =
                super::move_relative_date(&calendar, &plain_relative_to, &one_year, context)?;

            // b. Set plainRelativeTo to moveResult.[[RelativeTo]].
            plain_relative_to = move_result.0;
            // c. Set days to days + moveResult.[[Days]].
            days += move_result.1;
            // d. Set years to years - sign.
            years -= sign;
        }

        let mut months = self.months();
        // 18. Repeat, while months ≠ 0,
        while months != 0f64 {
            // a. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
            let move_result =
                super::move_relative_date(&calendar, &plain_relative_to, &one_month, context)?;
            // b. Set plainRelativeTo to moveResult.[[RelativeTo]].
            plain_relative_to = move_result.0;
            // c. Set days to days +moveResult.[[Days]].
            days += move_result.1;
            // d. Set months to months - sign.
            months -= sign;
        }

        let mut weeks = self.weeks();
        // 19. Repeat, while weeks ≠ 0,
        while weeks != 0f64 {
            // a. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneWeek, dateAdd).
            let move_result =
                super::move_relative_date(&calendar, &plain_relative_to, &one_week, context)?;
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

    /// `BalanceDateDurationRelative`
    #[allow(unused)]
    pub(crate) fn balance_date_duration_relative(
        &mut self,
        largest_unit: TemporalUnit,
        relative_to: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        // 1. Let allZero be false.
        // 2. If years = 0, and months = 0, and weeks = 0, and days = 0, set allZero to true.
        let all_zero = self.years() == 0.0
            && self.months() == 0.0
            && self.weeks() == 0.0
            && self.days() == 0.0;

        // 3. If largestUnit is not one of "year", "month", or "week", or allZero is true, then
        match largest_unit {
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week if !all_zero => {}
            _ => {
                // a. Return ! CreateDateDurationRecord(years, months, weeks, days).
                return Ok(());
            }
        }

        // 4. If relativeTo is undefined, then
        if relative_to.is_undefined() {
            // a. Throw a RangeError exception.
            return Err(JsNativeError::range()
                .with_message("relativeTo cannot be undefined.")
                .into());
        }

        // 5. Let sign be ! DurationSign(years, months, weeks, days, 0, 0, 0, 0, 0, 0).
        // 6. Assert: sign ≠ 0.
        let sign = f64::from(self.duration_sign());

        // 7. Let oneYear be ! CreateTemporalDuration(sign, 0, 0, 0, 0, 0, 0, 0, 0, 0).
        let one_year = Self::one_year(sign);
        // 8. Let oneMonth be ! CreateTemporalDuration(0, sign, 0, 0, 0, 0, 0, 0, 0, 0).
        let one_month = Self::one_month(sign);
        // 9. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
        let one_week = Self::one_week(sign);

        // 10. Set relativeTo to ? ToTemporalDate(relativeTo).
        let mut relative_to = to_temporal_date(relative_to, None, context)?;

        // 11. Let calendar be relativeTo.[[Calendar]].
        let calendar = &relative_to.calendar.clone();

        match largest_unit {
            // 12. If largestUnit is "year", then
            TemporalUnit::Year => {
                // a. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // b. Else,
                // i. Let dateAdd be unused.
                // c. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneYear, dateAdd).
                let move_result =
                    super::move_relative_date(calendar, &relative_to, &one_year, context)?;

                // d. Let newRelativeTo be moveResult.[[RelativeTo]].
                let mut new_relative = move_result.0;
                // e. Let oneYearDays be moveResult.[[Days]].
                let mut one_year_days = move_result.1;

                // f. Repeat, while abs(days) ≥ abs(oneYearDays),
                while self.days().abs() >= one_year_days.abs() {
                    // i. Set days to days - oneYearDays.
                    self.set_days(self.days() - one_year_days);

                    // ii. Set years to years + sign.
                    self.set_years(self.years() + sign);

                    // iii. Set relativeTo to newRelativeTo.
                    let relative_to = new_relative;
                    // iv. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneYear, dateAdd).
                    let move_result =
                        super::move_relative_date(calendar, &relative_to, &one_year, context)?;

                    // v. Set newRelativeTo to moveResult.[[RelativeTo]].
                    new_relative = move_result.0;
                    // vi. Set oneYearDays to moveResult.[[Days]].
                    one_year_days = move_result.1;
                }

                // g. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
                // h. Set newRelativeTo to moveResult.[[RelativeTo]].
                // i. Let oneMonthDays be moveResult.[[Days]].
                let (mut new_relative_to, mut one_month_days) =
                    super::move_relative_date(calendar, &relative_to, &one_month, context)?;

                // j. Repeat, while abs(days) ≥ abs(oneMonthDays),
                while self.days().abs() >= one_month_days.abs() {
                    // i. Set days to days - oneMonthDays.
                    self.set_days(self.days() - one_month_days);
                    // ii. Set months to months + sign.
                    self.set_months(self.months() + sign);
                    // iii. Set relativeTo to newRelativeTo.

                    let relative_to = new_relative.clone();
                    // iv. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
                    let move_result =
                        super::move_relative_date(calendar, &relative_to, &one_month, context)?;

                    // v. Set newRelativeTo to moveResult.[[RelativeTo]].
                    new_relative_to = move_result.0;
                    // vi. Set oneMonthDays to moveResult.[[Days]].
                    one_month_days = move_result.1;
                }

                // k. Set newRelativeTo to ? CalendarDateAdd(calendar, relativeTo, oneYear, undefined, dateAdd).
                new_relative_to = calendar::calendar_date_add(
                    calendar,
                    &relative_to,
                    &one_year,
                    &JsValue::undefined(),
                    context,
                )?;

                // l. If calendar is an Object, then
                // i. Let dateUntil be ? GetMethod(calendar, "dateUntil").
                // m. Else,
                // i. Let dateUntil be unused.

                // n. Let untilOptions be OrdinaryObjectCreate(null).
                let until_options = JsObject::with_null_proto();
                // o. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "month").
                until_options.create_data_property_or_throw(
                    utf16!("largestUnit"),
                    js_string!("month"),
                    context,
                )?;

                // p. Let untilResult be ? CalendarDateUntil(calendar, relativeTo, newRelativeTo, untilOptions, dateUntil).
                let until_result = calendar::calendar_date_until(
                    calendar,
                    &relative_to,
                    &new_relative_to,
                    &until_options.into(),
                    context,
                )?;

                // q. Let oneYearMonths be untilResult.[[Months]].
                let mut one_year_months = until_result.months();

                // r. Repeat, while abs(months) ≥ abs(oneYearMonths),
                while self.months().abs() >= one_year_months.abs() {
                    // i. Set months to months - oneYearMonths.
                    self.set_months(self.months() - one_year_months);
                    // ii. Set years to years + sign.
                    self.set_years(self.years() + sign);

                    // iii. Set relativeTo to newRelativeTo.
                    relative_to = new_relative_to;

                    // iv. Set newRelativeTo to ? CalendarDateAdd(calendar, relativeTo, oneYear, undefined, dateAdd).
                    new_relative_to = calendar::calendar_date_add(
                        calendar,
                        &relative_to,
                        &one_year,
                        &JsValue::undefined(),
                        context,
                    )?;

                    // v. Set untilOptions to OrdinaryObjectCreate(null).
                    let until_options = JsObject::with_null_proto();
                    // vi. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "month").
                    until_options.create_data_property_or_throw(
                        utf16!("largestUnit"),
                        js_string!("month"),
                        context,
                    )?;
                    // vii. Set untilResult to ? CalendarDateUntil(calendar, relativeTo, newRelativeTo, untilOptions, dateUntil).
                    let until_result = calendar::calendar_date_until(
                        calendar,
                        &relative_to,
                        &new_relative_to,
                        &until_options.into(),
                        context,
                    )?;
                    // viii. Set oneYearMonths to untilResult.[[Months]].
                    one_year_months = until_result.months();
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
                    super::move_relative_date(calendar, &relative_to, &one_month, context)?;

                // f. Repeat, while abs(days) ≥ abs(oneMonthDays),
                while self.days().abs() >= one_month_days.abs() {
                    // i. Set days to days - oneMonthDays.
                    self.set_days(self.days() - one_month_days);

                    // ii. Set months to months + sign.
                    self.set_months(self.months() + sign);

                    // iii. Set relativeTo to newRelativeTo.
                    relative_to = new_relative_to;

                    // iv. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
                    let move_result =
                        super::move_relative_date(calendar, &relative_to, &one_month, context)?;
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
                    super::move_relative_date(calendar, &relative_to, &one_week, context)?;

                // g. Repeat, while abs(days) ≥ abs(oneWeekDays),
                while self.days().abs() >= one_week_days.abs() {
                    // i. Set days to days - oneWeekDays.
                    self.set_days(self.days() - one_week_days);
                    // ii. Set weeks to weeks + sign.
                    self.set_weeks(self.weeks() + sign);
                    // iii. Set relativeTo to newRelativeTo.
                    relative_to = new_relative_to;
                    // iv. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneWeek, dateAdd).
                    let move_result =
                        super::move_relative_date(calendar, &relative_to, &one_week, context)?;
                    // v. Set newRelativeTo to moveResult.[[RelativeTo]].
                    new_relative_to = move_result.0;
                    // vi. Set oneWeekDays to moveResult.[[Days]].
                    one_week_days = move_result.1;
                }
            }
            _ => unreachable!(),
        }

        // 15. Return ! CreateDateDurationRecord(years, months, weeks, days).
        Ok(())
    }

    // TODO: Refactor relative_to's into a RelativeTo struct?
    /// Abstract Operation 7.5.26 `RoundDuration ( years, months, weeks, days, hours, minutes,
    ///   seconds, milliseconds, microseconds, nanoseconds, increment, unit,
    ///   roundingMode [ , plainRelativeTo [, zonedRelativeTo [, precalculatedDateTime]]] )`
    pub(crate) fn round_duration(
        &self,
        unbalance_date_duration: DateDuration,
        increment: f64,
        unit: TemporalUnit,
        rounding_mode: RoundingMode,
        relative_targets: (
            Option<&PlainDate>,
            Option<&ZonedDateTime>,
            Option<&PlainDateTime>,
        ),
        context: &mut Context<'_>,
    ) -> JsResult<(Self, f64)> {
        let mut result = DurationRecord::new(unbalance_date_duration, self.time());

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
                return Err(JsNativeError::range()
                    .with_message("plainRelativeTo canot be undefined with given TemporalUnit")
                    .into());
            }
            // 5. If unit is one of "year", "month", "week", or "day", then
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                // a. Let nanoseconds be TotalDurationNanoseconds(hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
                let nanos =
                    Self::from_day_and_time(0.0, result.time()).total_duration_nanoseconds(0.0);

                // b. If zonedRelativeTo is not undefined, then
                // i. Let intermediate be ? MoveRelativeZonedDateTime(zonedRelativeTo, years, months, weeks, days, precalculatedPlainDateTime).
                // ii. Let result be ? NanosecondsToDays(nanoseconds, intermediate).
                // iii. Let fractionalDays be days + result.[[Days]] + result.[[Nanoseconds]] / result.[[DayLength]].
                // c. Else,
                // i. Let fractionalDays be days + nanoseconds / nsPerDay.
                let frac_days = if zoned_relative_to.is_none() {
                    result.days() + nanos / NS_PER_DAY as f64
                } else {
                    // implementation of b: i-iii needed.
                    return Err(JsNativeError::range()
                        .with_message("Not yet implemented.")
                        .into());
                };
                // d. Set days, hours, minutes, seconds, milliseconds, microseconds, and nanoseconds to 0.
                result.set_days(0.0);
                result.set_time_duration(TimeDuration::default());
                // e. Assert: fractionalSeconds is not used below.
                (Some(frac_days), None)
            }
            // 6. Else,
            _ => {
                // a. Let fractionalSeconds be nanoseconds × 10-9 + microseconds × 10-6 + milliseconds × 10-3 + seconds.
                let frac_secs = result.nanoseconds().mul_add(
                    1_000_000_000f64,
                    result.microseconds().mul_add(
                        1_000_000f64,
                        result.milliseconds().mul_add(1_000f64, result.seconds()),
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
                let calendar = &plain_relative_to.calendar;

                // b. Let yearsDuration be ! CreateTemporalDuration(years, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let years = DateDuration::new(result.years(), 0.0, 0.0, 0.0);
                let years_duration = DurationRecord::new(years, TimeDuration::default());

                // c. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // d. Else,
                // i. Let dateAdd be unused.

                // e. Let yearsLater be ? AddDate(calendar, plainRelativeTo, yearsDuration, undefined, dateAdd).
                let years_later = plain_date::add_date(
                    calendar,
                    plain_relative_to,
                    &years_duration,
                    &JsValue::undefined(),
                    context,
                )?;

                // f. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = Self::new(
                    DateDuration::new(result.years(), result.months(), result.weeks(), 0.0),
                    TimeDuration::default(),
                );

                // g. Let yearsMonthsWeeksLater be ? AddDate(calendar, plainRelativeTo, yearsMonthsWeeks, undefined, dateAdd).
                let years_months_weeks_later = plain_date::add_date(
                    calendar,
                    plain_relative_to,
                    &years_months_weeks,
                    &JsValue::undefined(),
                    context,
                )?;

                // h. Let monthsWeeksInDays be DaysUntil(yearsLater, yearsMonthsWeeksLater).
                let months_weeks_in_days =
                    super::days_until(&years_later, &years_months_weeks_later);

                // i. Set plainRelativeTo to yearsLater.
                let plain_relative_to = years_later;

                // j. Set fractionalDays to fractionalDays + monthsWeeksInDays.
                frac_days += f64::from(months_weeks_in_days);

                // k. Let isoResult be ! AddISODate(plainRelativeTo.[[ISOYear]]. plainRelativeTo.[[ISOMonth]], plainRelativeTo.[[ISODay]], 0, 0, 0, truncate(fractionalDays), "constrain").
                let iso_result = plain_relative_to.inner.add_iso_date(
                    DateDuration::new(0.0, 0.0, 0.0, frac_days.trunc()),
                    ArithmeticOverflow::Constrain,
                )?;

                // l. Let wholeDaysLater be ? CreateTemporalDate(isoResult.[[Year]], isoResult.[[Month]], isoResult.[[Day]], calendar).
                let whole_days_later = PlainDate::new(iso_result, calendar.clone());

                // m. Let untilOptions be OrdinaryObjectCreate(null).
                // n. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "year").
                // o. Let timePassed be ? DifferenceDate(calendar, plainRelativeTo, wholeDaysLater, untilOptions).
                let time_passed = plain_date::difference_date(
                    calendar,
                    &plain_relative_to,
                    &whole_days_later,
                    TemporalUnit::Year,
                    context,
                )?;

                // p. Let yearsPassed be timePassed.[[Years]].
                let years_passed = time_passed.years();

                // q. Set years to years + yearsPassed.
                result.set_years(result.years() + years_passed);

                // r. Let yearsDuration be ! CreateTemporalDuration(yearsPassed, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_duration = Self::one_year(years_passed);

                // s. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, yearsDuration, dateAdd).
                // t. Set plainRelativeTo to moveResult.[[RelativeTo]].
                // u. Let daysPassed be moveResult.[[Days]].
                let (plain_relative_to, days_passed) = super::move_relative_date(
                    calendar,
                    &plain_relative_to,
                    &years_duration,
                    context,
                )?;

                // v. Set fractionalDays to fractionalDays - daysPassed.
                frac_days -= days_passed;

                // w. If fractionalDays < 0, let sign be -1; else, let sign be 1.
                let sign = if frac_days < 0.0 { -1 } else { 1 };

                // x. Let oneYear be ! CreateTemporalDuration(sign, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let one_year = Self::one_year(f64::from(sign));

                // y. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneYear, dateAdd).
                // z. Let oneYearDays be moveResult.[[Days]].
                let (_, one_year_days) =
                    super::move_relative_date(calendar, &plain_relative_to, &one_year, context)?;

                // aa. Let fractionalYears be years + fractionalDays / abs(oneYearDays).
                let frac_years = result.years() + (frac_days / one_year_days.abs());

                // ab. Set years to RoundNumberToIncrement(fractionalYears, increment, roundingMode).
                result.set_years(round_number_to_increment(
                    frac_years,
                    increment,
                    rounding_mode,
                ));

                // ac. Set total to fractionalYears.
                // ad. Set months and weeks to 0.
                result.set_months(0.0);
                result.set_weeks(0.0);

                frac_years
            }
            // 9. Else if unit is "month", then
            TemporalUnit::Month => {
                let mut frac_days =
                    frac_days.expect("assert that fractionalDays exists for TemporalUnit::Month");

                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let plain_relative_to = plain_relative_to.expect("this must exist.");
                let calendar = &plain_relative_to.calendar;

                // b. Let yearsMonths be ! CreateTemporalDuration(years, months, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_months = Self::from_date_duration(DateDuration::new(
                    result.years(),
                    result.months(),
                    0.0,
                    0.0,
                ));

                // c. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // d. Else,
                // i. Let dateAdd be unused.

                // e. Let yearsMonthsLater be ? AddDate(calendar, plainRelativeTo, yearsMonths, undefined, dateAdd).
                let years_months_later = plain_date::add_date(
                    calendar,
                    plain_relative_to,
                    &years_months,
                    &JsValue::undefined(),
                    context,
                )?;

                // f. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = Self::from_date_duration(DateDuration::new(
                    result.years(),
                    result.months(),
                    result.weeks(),
                    0.0,
                ));

                // g. Let yearsMonthsWeeksLater be ? AddDate(calendar, plainRelativeTo, yearsMonthsWeeks, undefined, dateAdd).
                let years_months_weeks_later = plain_date::add_date(
                    calendar,
                    plain_relative_to,
                    &years_months_weeks,
                    &JsValue::undefined(),
                    context,
                )?;

                // h. Let weeksInDays be DaysUntil(yearsMonthsLater, yearsMonthsWeeksLater).
                let weeks_in_days =
                    super::days_until(&years_months_later, &years_months_weeks_later);

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
                    super::move_relative_date(calendar, &plain_relative_to, &one_month, context)?;

                // p. Repeat, while abs(fractionalDays) ≥ abs(oneMonthDays),
                while frac_days.abs() >= one_month_days.abs() {
                    // i. Set months to months + sign.
                    result.set_months(result.months() + sign);

                    // ii. Set fractionalDays to fractionalDays - oneMonthDays.
                    frac_days -= one_month_days;

                    // iii. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
                    let move_result = super::move_relative_date(
                        calendar,
                        &plain_relative_to,
                        &one_month,
                        context,
                    )?;
                    // iv. Set plainRelativeTo to moveResult.[[RelativeTo]].
                    plain_relative_to = move_result.0;
                    // v. Set oneMonthDays to moveResult.[[Days]].
                    one_month_days = move_result.1;
                }

                // q. Let fractionalMonths be months + fractionalDays / abs(oneMonthDays).
                let frac_months = result.months() + frac_days / one_month_days.abs();

                // r. Set months to RoundNumberToIncrement(fractionalMonths, increment, roundingMode).
                result.set_months(round_number_to_increment(
                    frac_months,
                    increment,
                    rounding_mode,
                ));

                // s. Set total to fractionalMonths.
                // t. Set weeks to 0.
                result.set_weeks(0.0);
                frac_months
            }
            // 10. Else if unit is "week", then
            TemporalUnit::Week => {
                let mut frac_days =
                    frac_days.expect("assert that fractionalDays exists for TemporalUnit::Month");
                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let plain_relative_to = plain_relative_to.expect("date must exist given Week");
                let calendar = &plain_relative_to.calendar;

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
                    super::move_relative_date(calendar, plain_relative_to, &one_week, context)?;

                // i. Repeat, while abs(fractionalDays) ≥ abs(oneWeekDays),
                while frac_days.abs() >= one_week_days.abs() {
                    // i. Set weeks to weeks + sign.
                    result.set_weeks(result.weeks() + sign);

                    // ii. Set fractionalDays to fractionalDays - oneWeekDays.
                    frac_days -= one_week_days;

                    // iii. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneWeek, dateAdd).
                    let move_result = super::move_relative_date(
                        calendar,
                        &plain_relative_to,
                        &one_week,
                        context,
                    )?;

                    // iv. Set plainRelativeTo to moveResult.[[RelativeTo]].
                    plain_relative_to = move_result.0;
                    // v. Set oneWeekDays to moveResult.[[Days]].
                    one_week_days = move_result.1;
                }

                // j. Let fractionalWeeks be weeks + fractionalDays / abs(oneWeekDays).
                let frac_weeks = result.weeks() + frac_days / one_week_days.abs();

                // k. Set weeks to RoundNumberToIncrement(fractionalWeeks, increment, roundingMode).
                result.set_weeks(round_number_to_increment(
                    frac_weeks,
                    increment,
                    rounding_mode,
                ));
                // l. Set total to fractionalWeeks.
                frac_weeks
            }
            // 11. Else if unit is "day", then
            TemporalUnit::Day => {
                let frac_days =
                    frac_days.expect("assert that fractionalDays exists for TemporalUnit::Day");

                // a. Set days to RoundNumberToIncrement(fractionalDays, increment, roundingMode).
                result.set_days(round_number_to_increment(
                    frac_days,
                    increment,
                    rounding_mode,
                ));
                // b. Set total to fractionalDays.
                frac_days
            }
            // 12. Else if unit is "hour", then
            TemporalUnit::Hour => {
                let frac_secs =
                    frac_secs.expect("Assert fractionSeconds exists for Temporal::Hour");
                // a. Let fractionalHours be (fractionalSeconds / 60 + minutes) / 60 + hours.
                let frac_hours = (frac_secs / 60f64 + result.minutes()) / 60f64 + result.hours();
                // b. Set hours to RoundNumberToIncrement(fractionalHours, increment, roundingMode).
                let rounded_hours = round_number_to_increment(frac_hours, increment, rounding_mode);
                // c. Set total to fractionalHours.
                // d. Set minutes, seconds, milliseconds, microseconds, and nanoseconds to 0.
                result.set_time_duration(TimeDuration::new(rounded_hours, 0.0, 0.0, 0.0, 0.0, 0.0));
                frac_hours
            }
            // 13. Else if unit is "minute", then
            TemporalUnit::Minute => {
                let frac_secs =
                    frac_secs.expect("Assert fractionSeconds exists for Temporal::Hour");
                // a. Let fractionalMinutes be fractionalSeconds / 60 + minutes.
                let frac_minutes = frac_secs / 60f64 + result.minutes();
                // b. Set minutes to RoundNumberToIncrement(fractionalMinutes, increment, roundingMode).
                let rounded_minutes =
                    round_number_to_increment(frac_minutes, increment, rounding_mode);
                // c. Set total to fractionalMinutes.
                // d. Set seconds, milliseconds, microseconds, and nanoseconds to 0.
                result.set_time_duration(TimeDuration::new(
                    result.hours(),
                    rounded_minutes,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                ));

                frac_minutes
            }
            // 14. Else if unit is "second", then
            TemporalUnit::Second => {
                let frac_secs =
                    frac_secs.expect("Assert fractionSeconds exists for Temporal::Second");
                // a. Set seconds to RoundNumberToIncrement(fractionalSeconds, increment, roundingMode).
                result.set_seconds(round_number_to_increment(
                    frac_secs,
                    increment,
                    rounding_mode,
                ));
                // b. Set total to fractionalSeconds.
                // c. Set milliseconds, microseconds, and nanoseconds to 0.
                result.set_milliseconds(0.0);
                result.set_microseconds(0.0);
                result.set_nanoseconds(0.0);

                frac_secs
            }
            // 15. Else if unit is "millisecond", then
            TemporalUnit::Millisecond => {
                // a. Let fractionalMilliseconds be nanoseconds × 10-6 + microseconds × 10-3 + milliseconds.
                let fraction_millis = result.nanoseconds().mul_add(
                    1_000_000f64,
                    result
                        .microseconds()
                        .mul_add(1_000f64, result.milliseconds()),
                );

                // b. Set milliseconds to RoundNumberToIncrement(fractionalMilliseconds, increment, roundingMode).
                result.set_milliseconds(round_number_to_increment(
                    fraction_millis,
                    increment,
                    rounding_mode,
                ));

                // c. Set total to fractionalMilliseconds.
                // d. Set microseconds and nanoseconds to 0.
                result.set_microseconds(0.0);
                result.set_nanoseconds(0.0);
                fraction_millis
            }
            // 16. Else if unit is "microsecond", then
            TemporalUnit::Microsecond => {
                // a. Let fractionalMicroseconds be nanoseconds × 10-3 + microseconds.
                let frac_micros = result
                    .nanoseconds()
                    .mul_add(1_000f64, result.microseconds());

                // b. Set microseconds to RoundNumberToIncrement(fractionalMicroseconds, increment, roundingMode).
                result.set_microseconds(round_number_to_increment(
                    frac_micros,
                    increment,
                    rounding_mode,
                ));

                // c. Set total to fractionalMicroseconds.
                // d. Set nanoseconds to 0.
                result.set_nanoseconds(0.0);
                frac_micros
            }
            // 17. Else,
            TemporalUnit::Nanosecond => {
                // a. Assert: unit is "nanosecond".
                // b. Set total to nanoseconds.
                let total = result.nanoseconds();
                // c. Set nanoseconds to RoundNumberToIncrement(nanoseconds, increment, roundingMode).
                result.set_nanoseconds(round_number_to_increment(
                    result.nanoseconds(),
                    increment,
                    rounding_mode,
                ));

                total
            }
            TemporalUnit::Auto => unreachable!(),
        };

        // 18. Let duration be ? CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
        // 19. Return the Record { [[DurationRecord]]: duration, [[Total]]: total }.
        Ok((result, total))
    }

    /// 7.5.27 `AdjustRoundedDurationDays ( years, months, weeks, days, hours, minutes, seconds, milliseconds,
    /// microseconds, nanoseconds, increment, unit, roundingMode, relativeTo )`
    #[allow(unused)]
    pub(crate) fn adjust_rounded_duration_days(
        &mut self,
        increment: f64,
        unit: TemporalUnit,
        rounding_mode: RoundingMode,
        relative_to: Option<&JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        // 1. If Type(relativeTo) is not Object; or relativeTo does not have an [[InitializedTemporalZonedDateTime]]
        // internal slot; or unit is one of "year", "month", "week", or "day"; or unit is "nanosecond" and increment is 1, then
        let relative_to = match relative_to {
            Some(rt)
                if rt.is_object()
                    && rt.as_object().expect("must be object").is_zoned_date_time() =>
            {
                let obj = rt.as_object().expect("This must be an object.");
                let obj = obj.borrow();
                // a. Return ! CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
                obj.as_zoned_date_time()
                    .expect("Object must be a ZonedDateTime.")
                    .clone()
            }
            _ => return Ok(()),
        };

        match unit {
            // a. Return ! CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                return Ok(())
            }
            // a. Return ! CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
            TemporalUnit::Nanosecond if (increment - 1_f64).abs() < f64::EPSILON => return Ok(()),
            _ => {}
        }

        // 2. Let timeRemainderNs be ! TotalDurationNanoseconds(0, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, 0).
        let time_remainder_ns =
            Self::from_day_and_time(0.0, self.time()).total_duration_nanoseconds(0.0);

        // 3. If timeRemainderNs = 0, let direction be 0.
        let _direction = if time_remainder_ns == 0_f64 {
            0
        // 4. Else if timeRemainderNs < 0, let direction be -1.
        } else if time_remainder_ns < 0_f64 {
            -1
        // 5. Else, let direction be 1.
        } else {
            1
        };

        // TODO: 6.5.5 AddZonedDateTime
        // 6. Let dayStart be ? AddZonedDateTime(relativeTo.[[Nanoseconds]], relativeTo.[[TimeZone]], relativeTo.[[Calendar]], years, months, weeks, days, 0, 0, 0, 0, 0, 0).
        // 7. Let dayEnd be ? AddZonedDateTime(dayStart, relativeTo.[[TimeZone]], relativeTo.[[Calendar]], 0, 0, 0, direction, 0, 0, 0, 0, 0, 0).
        // 8. Let dayLengthNs be ℝ(dayEnd - dayStart).
        // 9. If (timeRemainderNs - dayLengthNs) × direction < 0, then
        // a. Return ! CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
        // 10. Set timeRemainderNs to ℝ(RoundTemporalInstant(ℤ(timeRemainderNs - dayLengthNs), increment, unit, roundingMode)).
        // 11. Let adjustedDateDuration be ? AddDuration(years, months, weeks, days, 0, 0, 0, 0, 0, 0, 0, 0, 0, direction, 0, 0, 0, 0, 0, 0, relativeTo).
        // 12. Let adjustedTimeDuration be ? BalanceDuration(0, 0, 0, 0, 0, 0, timeRemainderNs, "hour").
        // 13. Return ! CreateDurationRecord(adjustedDateDuration.[[Years]], adjustedDateDuration.[[Months]], adjustedDateDuration.[[Weeks]],
        // adjustedDateDuration.[[Days]], adjustedTimeDuration.[[Hours]], adjustedTimeDuration.[[Minutes]], adjustedTimeDuration.[[Seconds]],
        // adjustedTimeDuration.[[Milliseconds]], adjustedTimeDuration.[[Microseconds]], adjustedTimeDuration.[[Nanoseconds]]).
        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }
}
