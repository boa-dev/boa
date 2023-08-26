// use DurationRecord and Duration { inner: Duration }

use crate::{
    builtins::temporal, Context, JsArgs, JsError, JsNativeError, JsNativeErrorKind, JsObject,
    JsResult, JsString, JsSymbol, JsValue,
};

use super::super::{calendar, to_integer_if_integral, zoned_date_time};

/// The `DurationRecord` is defined by Abtract Operation 7.5.1 `DurationRecords`
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DurationRecord {
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
}

// ==== Initialization methods for `DurationRecord` ====

impl DurationRecord {
    pub(crate) const fn new(
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
    ) -> Self {
        Self {
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        }
    }

    pub(crate) const fn new_partial() -> Self {
        Self {
            years: f64::NAN,
            months: f64::NAN,
            weeks: f64::NAN,
            days: f64::NAN,
            hours: f64::NAN,
            minutes: f64::NAN,
            seconds: f64::NAN,
            milliseconds: f64::NAN,
            microseconds: f64::NAN,
            nanoseconds: f64::NAN,
        }
    }

    pub(crate) const fn from_date_duration(date_duration: &Self) -> Self {
        Self {
            years: date_duration.years(),
            months: date_duration.months(),
            weeks: date_duration.weeks(),
            days: date_duration.days(),
            hours: 0_f64,
            minutes: 0_f64,
            seconds: 0_f64,
            milliseconds: 0_f64,
            microseconds: 0_f64,
            nanoseconds: 0_f64,
        }
    }

    pub(crate) const fn from_date_and_time_duration(
        date_duration: &Self,
        time_duration: &Self,
    ) -> Self {
        Self {
            years: date_duration.years(),
            months: date_duration.months(),
            weeks: date_duration.weeks(),
            days: date_duration.days(),
            hours: time_duration.hours(),
            minutes: time_duration.minutes(),
            seconds: time_duration.seconds(),
            milliseconds: time_duration.milliseconds(),
            microseconds: time_duration.microseconds(),
            nanoseconds: time_duration.nanoseconds(),
        }
    }

    pub(crate) fn with_years(mut self, y: f64) -> Self {
        self.set_years(y);
        self
    }

    pub(crate) fn with_months(mut self, mo: f64) -> Self {
        self.set_months(mo);
        self
    }

    pub(crate) fn with_weeks(mut self, w: f64) -> Self {
        self.set_weeks(w);
        self
    }

    pub(crate) fn with_days(mut self, d: f64) -> Self {
        self.set_days(d);
        self
    }

    pub(crate) fn with_hours(mut self, h: f64) -> Self {
        self.set_hours(h);
        self
    }

    pub(crate) fn with_minutes(mut self, m: f64) -> Self {
        self.set_minutes(m);
        self
    }

    pub(crate) fn with_seconds(mut self, s: f64) -> Self {
        self.set_seconds(s);
        self
    }

    pub(crate) fn with_milliseconds(mut self, ms: f64) -> Self {
        self.set_milliseconds(ms);
        self
    }

    pub(crate) fn with_microseconds(mut self, mis: f64) -> Self {
        self.set_microseconds(mis);
        self
    }

    pub(crate) fn with_nanoseconds(mut self, ns: f64) -> Self {
        self.set_nanoseconds(ns);
        self
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

    // Balance/bubble the current unit from one step down.
    fn balance_minutes(&mut self) {
        // 1. Set minutes to floor(seconds / 60).
        self.set_minutes((self.seconds() / 60_f64).floor());
        // 2. Set seconds to seconds modulo 60.
        self.set_seconds(self.seconds() % 60_f64);
    }

    // Balance/bubble the current unit from one step down.
    fn balance_seconds(&mut self) {
        // 1. Set seconds to floor(milliseconds / 1000).
        self.set_seconds((self.milliseconds() / 1_000_f64).floor());
        // 2. Set milliseconds to milliseconds modulo 1000.
        self.set_milliseconds(self.milliseconds() % 1_000_f64);
    }

    // Balance/bubble the current unit from one step down.
    fn balance_milliseconds(&mut self) {
        // c. Set milliseconds to floor(microseconds / 1000).
        self.set_milliseconds((self.microseconds() / 1_000_f64).floor());
        // d. Set microseconds to microseconds modulo 1000.
        self.set_microseconds(self.microseconds() % 1_000_f64);
    }

    // Balance/bubble the current unit from one step down.
    fn balance_microseconds(&mut self) {
        // a. Set microseconds to floor(nanoseconds / 1000).
        self.set_microseconds((self.nanoseconds() / 1_000_f64).floor());
        // b. Set nanoseconds to nanoseconds modulo 1000.
        self.set_nanoseconds(self.nanoseconds() % 1_000_f64);
    }
}

// ==== `DurationRecord` getter/setter methods ====

impl DurationRecord {
    /// Set the value for `years`.
    pub(crate) fn set_years(&mut self, y: f64) {
        self.years = y;
    }

    /// Return the value for `years`.
    pub(crate) const fn years(&self) -> f64 {
        self.years
    }

    /// Set the value for `months`.
    pub(crate) fn set_months(&mut self, mo: f64) {
        self.months = mo;
    }

    /// Return the value for `months`.
    pub(crate) const fn months(&self) -> f64 {
        self.months
    }

    /// Set the value for `weeks`.
    pub(crate) fn set_weeks(&mut self, w: f64) {
        self.weeks = w;
    }

    /// Return the value for `weeks`.
    pub(crate) const fn weeks(&self) -> f64 {
        self.weeks
    }

    /// Set the value for `days`.
    pub(crate) fn set_days(&mut self, d: f64) {
        self.days = d;
    }

    /// Return the value for `days`.
    pub(crate) const fn days(&self) -> f64 {
        self.days
    }

    /// Set the value for `hours`.
    pub(crate) fn set_hours(&mut self, h: f64) {
        self.hours = h;
    }

    /// Return the value for `hours`.
    pub(crate) const fn hours(&self) -> f64 {
        self.hours
    }

    /// Set the value for `minutes`.
    pub(crate) fn set_minutes(&mut self, m: f64) {
        self.minutes = m;
    }

    /// Return the value for `minutes`.
    pub(crate) const fn minutes(&self) -> f64 {
        self.minutes
    }

    /// Set the value for `seconds`.
    pub(crate) fn set_seconds(&mut self, s: f64) {
        self.seconds = s;
    }

    /// Return the value for `seconds`.
    pub(crate) const fn seconds(&self) -> f64 {
        self.seconds
    }

    /// Set the value for `milliseconds`.
    pub(crate) fn set_milliseconds(&mut self, ms: f64) {
        self.milliseconds = ms;
    }

    /// Return the value for `milliseconds`.
    pub(crate) const fn milliseconds(&self) -> f64 {
        self.milliseconds
    }

    /// Set the value for `microseconds`.
    pub(crate) fn set_microseconds(&mut self, mis: f64) {
        self.microseconds = mis;
    }

    /// Return the value for `microseconds`.
    pub(crate) const fn microseconds(&self) -> f64 {
        self.microseconds
    }

    /// Set the value for `nanoseconds`.
    pub(crate) fn set_nanoseconds(&mut self, ns: f64) {
        self.nanoseconds = ns;
    }

    /// Return the value for `nanoseconds`.
    pub(crate) const fn nanoseconds(&self) -> f64 {
        self.nanoseconds
    }
}

// -- Abstract Operations implemented on `DurationRecord`
impl DurationRecord {
    /// Returns the values of the current duration record as a vec.
    fn values(&self) -> Vec<f64> {
        vec![
            self.years(),
            self.months(),
            self.weeks(),
            self.days(),
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.milliseconds(),
            self.microseconds(),
            self.nanoseconds(),
        ]
    }

    /// Returns the duration time values as a vec
    fn time_values(&self) -> Vec<f64> {
        vec![
            self.days(),
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.milliseconds(),
            self.microseconds(),
            self.nanoseconds(),
        ]
    }

    /// Determines if the `DurationRecord` has overflowed.
    // Note(nekevss): This currently assumes that an overflow has been stored into the years
    // column as the duration is nonviable and storing it in years allows for invalidating
    // the duration the fastest.
    #[inline]
    fn is_overfowed(&self) -> bool {
        self.years().is_infinite()
    }

    #[inline]
    pub(crate) fn is_positive_overflow(&self) -> bool {
        self.years().is_infinite() && self.years().is_sign_positive()
    }

    #[inline]
    pub(crate) fn is_negative_overflow(&self) -> bool {
        !self.is_positive_overflow()
    }

    /// 7.5.2 Date Duration Records
    ///
    /// Checks if current `DurationRecord` is a Date Duration record.
    pub(crate) fn is_date_duration(&self) -> bool {
        self.hours == 0_f64
            && self.minutes == 0_f64
            && self.seconds == 0_f64
            && self.milliseconds == 0_f64
            && self.microseconds == 0_f64
            && self.nanoseconds == 0_f64
    }

    /// 7.5.3 Time Duration Records
    ///
    /// Checks if current `DurationRecord` is a Time Duration record.
    pub(crate) fn is_time_duration(&self) -> bool {
        self.years == 0_f64 && self.months == 0_f64 && self.weeks == 0_f64
    }

    /// 7.5.10 `DurationSign ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )`
    ///
    /// Determines the sign for the current self.
    pub(crate) fn duration_sign(&self) -> i32 {
        // 1. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
        for v in self.values() {
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
        for v in self.values() {
            // a. If 𝔽(v) is not finite, return false.
            if v.is_finite() {
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
    pub(crate) fn default_temporal_largest_unit(&self) -> JsString {
        // TODO: is there a better way to implement the below?
        if self.years() != 0_f64 {
            return JsString::from("year");
        }
        if self.months() != 0_f64 {
            return JsString::from("month");
        }
        if self.weeks() != 0_f64 {
            return JsString::from("week");
        }
        if self.days() != 0_f64 {
            return JsString::from("day");
        }
        if self.hours() != 0_f64 {
            return JsString::from("hour");
        }
        if self.minutes() != 0_f64 {
            return JsString::from("minute");
        }
        if self.seconds() != 0_f64 {
            return JsString::from("second");
        }
        if self.milliseconds() != 0_f64 {
            return JsString::from("millisecond");
        }
        if self.microseconds() != 0_f64 {
            return JsString::from("microsecond");
        }
        JsString::from("nanosecond")
    }

    /// Abstract Operation 7.5.18 `BalanceDuration ( days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, largestUnit [ , relativeTo ] )`
    pub(crate) fn balance_duration(
        &mut self,
        largest_unit: &JsString,
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
        largest_unit: &JsString,
        relative_to: Option<&JsValue>,
    ) -> JsResult<()> {
        assert!(self.is_time_duration());
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
            // a. Let endNs be ? AddZonedDateTime(relativeTo.[[Nanoseconds]], relativeTo.[[TimeZone]], relativeTo.[[Calendar]], 0, 0, 0, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
            // b. Set nanoseconds to ℝ(endNs - relativeTo.[[Nanoseconds]]).
            todo!()
        // 3. Else,
        } else {
            // a. Set nanoseconds to ! TotalDurationNanoseconds(days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, 0).
            self.set_nanoseconds(super::total_duration_nanoseconds(
                self.days,
                self.hours,
                self.minutes,
                self.seconds,
                self.milliseconds,
                self.microseconds,
                self.nanoseconds,
                0_f64,
            ));
        }

        match largest_unit.as_slice() {
            // 4. If largestUnit is one of "year", "month", "week", or "day", then
            temporal::YEAR | temporal::MONTH | temporal::WEEK | temporal::DAY => {
                // a. Let result be ? NanosecondsToDays(nanoseconds, relativeTo).
                let result = temporal::zoned_date_time::nanoseconds_to_days(
                    self.nanoseconds(),
                    &relative_to,
                );
                // b. Set days to result.[[Days]].
                // c. Set nanoseconds to result.[[Nanoseconds]].
                todo!()
            }
            // 5. Else,
            // a. Set days to 0.
            _ => self.set_days(0_f64),
        }

        // 6. Set hours, minutes, seconds, milliseconds, and microseconds to 0.
        self.set_hours(0_f64);
        self.set_minutes(0_f64);
        self.set_seconds(0_f64);
        self.set_milliseconds(0_f64);
        self.set_microseconds(0_f64);

        // 7. If nanoseconds < 0, let sign be -1; else, let sign be 1.
        let sign = if self.nanoseconds() < 0_f64 {
            -1_f64
        } else {
            1_f64
        };
        // 8. Set nanoseconds to abs(nanoseconds).
        self.set_nanoseconds(self.nanoseconds().abs());

        match largest_unit.to_std_string_escaped().as_str() {
            // 9. If largestUnit is "year", "month", "week", "day", or "hour", then
            "year" | "month" | "week" | "day" | "hour" => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                self.balance_microseconds();

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.              self.balance_milliseconds();
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
            "minute" => {
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
            "second" => {
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
            "millisecond" => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                self.balance_microseconds();

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                self.balance_milliseconds();
            }
            // 13. Else if largestUnit is "microsecond", then
            "microsecond" => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                self.balance_microseconds();
            }
            // 14. Else,
            // a. Assert: largestUnit is "nanosecond".
        _ => assert!(largest_unit.to_std_string_escaped().as_str() == "nanosecond"),
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
        //
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

    /// 7.5.20 `UnbalanceDurationRelative ( years, months, weeks, days, largestUnit, relativeTo )`
    pub(crate) fn unbalance_duration_relative(
        &mut self,
        largest_unit: &JsString,
        relative_to: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        // 1. Let allZero be false.
        // 2. If years = 0, and months = 0, and weeks = 0, and days = 0, set allZero to true.
        let all_zero = self.years() == 0_f64
            && self.months() == 0_f64
            && self.weeks() == 0_f64
            && self.days() == 0_f64;

        // 3. If largestUnit is "year" or allZero is true, then
        if largest_unit.as_slice() == temporal::YEAR || all_zero {
            // a. Return ! CreateDateDurationRecord(years, months, weeks, days).
            return Ok(());
        };

        // 4. Let sign be ! DurationSign(years, months, weeks, days, 0, 0, 0, 0, 0, 0).
        let sign = self.duration_sign();
        // 5. Assert: sign ≠ 0.
        assert!(sign != 0);

        // 6. Let oneYear be ! CreateTemporalDuration(sign, 0, 0, 0, 0, 0, 0, 0, 0, 0).
        let one_year = super::create_temporal_duration(
            DurationRecord::default().with_years(self.years()),
            None,
            context,
        )?;
        // 7. Let oneMonth be ! CreateTemporalDuration(0, sign, 0, 0, 0, 0, 0, 0, 0, 0).
        let one_month = super::create_temporal_duration(
            DurationRecord::default().with_months(self.months()),
            None,
            context,
        )?;
        // 8. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
        let one_week = super::create_temporal_duration(
            DurationRecord::default().with_weeks(self.weeks()),
            None,
            context,
        )?;

        // 9. If relativeTo is not undefined, then
        let calendar = if relative_to.is_undefined() {
            // 10. Else
            // a. Let calendar be undefined.
            None
        } else {
            // a. Set relativeTo to ? ToTemporalDate(relativeTo).
            let relative_to = temporal::plain_date::to_temporal_date(
                &relative_to
                    .as_object()
                    .expect("relative_to must be an object")
                    .clone()
                    .into(),
                None,
                context,
            )
            .and_then(|date| {
                Ok(date
                    .as_object()
                    .ok_or::<JsError>(
                        JsNativeError::typ()
                            .with_message("object did not return TemporalPlainDate.")
                            .into(),
                    )?
                    .clone())
            })?;

            // b. Let calendar be relativeTo.[[Calendar]].
            let obj = relative_to.borrow_mut();
            let date = obj.as_plain_date().ok_or::<JsError>(
                JsNativeError::typ()
                    .with_message("relativeTo was not a PlainDate.")
                    .into(),
            )?;
            let calendar = date.calendar.clone();

            drop(obj);
            Some(calendar)
        };

        // 11. If largestUnit is "month", then
        // a. If calendar is undefined, then
        // i. Throw a RangeError exception.
        // b. If calendar is an Object, then
        // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
        // ii. Let dateUntil be ? GetMethod(calendar, "dateUntil").
        // c. Else,
        // i. Let dateAdd be unused.
        // ii. Let dateUntil be unused.
        // d. Repeat, while years ≠ 0,
        // i. Let newRelativeTo be ? CalendarDateAdd(calendar, relativeTo, oneYear, undefined, dateAdd).
        // ii. Let untilOptions be OrdinaryObjectCreate(null).
        // iii. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "month").
        // iv. Let untilResult be ? CalendarDateUntil(calendar, relativeTo, newRelativeTo, untilOptions, dateUntil).
        // v. Let oneYearMonths be untilResult.[[Months]].
        // vi. Set relativeTo to newRelativeTo.
        // vii. Set years to years - sign.
        // viii. Set months to months + oneYearMonths.
        // 12. Else if largestUnit is "week", then
        // a. If calendar is undefined, then
        // i. Throw a RangeError exception.
        // b. If calendar is an Object, then
        // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
        // c. Else,
        // i. Let dateAdd be unused.
        // d. Repeat, while years ≠ 0,
        // i. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneYear, dateAdd).
        // ii. Set relativeTo to moveResult.[[RelativeTo]].
        // iii. Set days to days + moveResult.[[Days]].
        // iv. Set years to years - sign.
        // e. Repeat, while months ≠ 0,
        // i. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
        // ii. Set relativeTo to moveResult.[[RelativeTo]].
        // iii. Set days to days + moveResult.[[Days]].
        // iv. Set months to months - sign.
        // 13. Else,
        // a. If years ≠ 0, or months ≠ 0, or weeks ≠ zero, then
        // i. If calendar is undefined, then
        // 1. Throw a RangeError exception.
        // ii. If calendar is an Object, then
        // 1. Let dateAdd be ? GetMethod(calendar, "dateAdd").
        // iii. Else,
        // 1. Let dateAdd be unused.
        // iv. Repeat, while years ≠ 0,
        // 1. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneYear, dateAdd).
        // 2. Set relativeTo to moveResult.[[RelativeTo]].
        // 3. Set days to days + moveResult.[[Days]].
        // 4. Set years to years - sign.
        // v. Repeat, while months ≠ 0,
        // 1. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
        // 2. Set relativeTo to moveResult.[[RelativeTo]].
        // 3. Set days to days +moveResult.[[Days]].
        // 4. Set months to months - sign.
        // vi. Repeat, while weeks ≠ 0,
        // 1. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneWeek, dateAdd).
        // 2. Set relativeTo to moveResult.[[RelativeTo]].
        // 3. Set days to days + moveResult.[[Days]].
        // 4. Set weeks to weeks - sign.
        // 14. Return ? CreateDateDurationRecord(years, months, weeks, days).
        Ok(())
    }

    /// Abstract Operation 7.5.26 `RoundDuration ( years, months, weeks, days, hours, minutes,
    ///   seconds, milliseconds, microseconds, nanoseconds, increment, unit,
    ///   roundingMode [ , relativeTo ] )`
    pub(crate) fn round_duration(
        &mut self,
        increment: f64,
        unit: &JsString,
        rounding_mode: &JsString,
        relative_to: Option<&JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<f64> {
        // 1. If relativeTo is not present, set relativeTo to undefined.
        let relative_to = if let Some(val) = relative_to {
            val.clone()
        } else {
            JsValue::undefined()
        };

        let unit_string = unit.to_std_string_escaped();

        // 2. If unit is "year", "month", or "week", and relativeTo is undefined, then
        if relative_to.is_undefined()
            && (unit_string.as_str() == "year"
                || unit_string.as_str() == "month"
                || unit_string.as_str() == "week")
        {
            // a. Throw a RangeError exception.
            return Err(JsNativeError::range()
                .with_message("relativeTo was out of range while rounding self.")
                .into());
        }

        // 3. Let zonedRelativeTo be undefined.
        let mut zoned_relative_to = JsValue::undefined();

        // 4. If relativeTo is not undefined, then
        let (calendar, relative_to) = if !relative_to.is_undefined() {
            let relative_to_obj = relative_to.as_object().expect(
                "relativeTo must be a Temporal.ZonedDateTime or Temporal.PlainDate object if defined.",
            );
            // a. If relativeTo has an [[InitializedTemporalZonedDateTime]] internal slot, then
            let relative_to_obj = if relative_to_obj.is_zoned_date_time() {
                // i. Set zonedRelativeTo to relativeTo.
                zoned_relative_to = relative_to.clone();

                // TODO: ii. Set relativeTo to ? ToTemporalDate(relativeTo).
                relative_to_obj.clone()
            // b. Else,
            } else {
                // i. Assert: relativeTo has an [[InitializedTemporalDate]] internal slot.
                relative_to_obj.clone()
            };

            let obj = relative_to_obj.borrow();
            let plain_date = obj.as_plain_date().expect("object must be a PlainDate");

            // c. Let calendar be relativeTo.[[Calendar]].
            let calendar = plain_date.calendar.clone();

            drop(obj);

            (Some(calendar), Some(relative_to_obj))
        // 5. Else,
        } else {
            // a. NOTE: calendar will not be used below.
            (None, None)
        };

        // 6. If unit is one of "year", "month", "week", or "day", then
        let fractional_secs = match unit_string.as_str() {
            "year" | "month" | "week" | "day" => {
                // a. Let nanoseconds be ! TotalDurationNanoseconds(0, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, 0).
                // NOTE: consider switching duration nanos to f64 based.
                let nanoseconds = super::total_duration_nanoseconds(
                    0_f64,
                    self.hours(),
                    self.minutes(),
                    self.seconds(),
                    self.milliseconds(),
                    self.microseconds(),
                    self.nanoseconds(),
                    0_f64,
                );
                // b. Let intermediate be undefined.
                let intermediate = JsValue::undefined();
                // c. If zonedRelativeTo is not undefined, then
                if zoned_relative_to.is_undefined() {
                    // i. Let intermediate be ? MoveRelativeZonedDateTime(zonedRelativeTo, years, months, weeks, days).
                    todo!()
                }
                // d. Let result be ? NanosecondsToDays(nanoseconds, intermediate).
                let result = zoned_date_time::nanoseconds_to_days(nanoseconds, &intermediate)?;

                // e. Set days to days + result.[[Days]] + result.[[Nanoseconds]] / result.[[DayLength]].
                let days = self.days() as i32;
                self.set_days(f64::from(days + result.0 + result.1 / result.2));

                // f. Set hours, minutes, seconds, milliseconds, microseconds, and nanoseconds to 0.
                self.set_hours(0_f64);
                self.set_hours(0_f64);
                self.set_minutes(0_f64);
                self.set_seconds(0_f64);
                self.set_milliseconds(0_f64);
                self.set_microseconds(0_f64);
                self.set_nanoseconds(0_f64);

                0_f64
            },
            // 7. Else,
            _=> {
                // a. Let fractionalSeconds be nanoseconds × 10-9 + microseconds × 10-6 + milliseconds × 10-3 + seconds.
                self.seconds().mul_add(
                    1000_f64,
                    self.nanoseconds()
                        .mul_add(1_000_000_000_f64, self.microseconds() * 1_000_000_f64),
                )
            }
        };

        // 8. Let remainder be undefined.
        // We begin matching against unit and return the remainder value.
        let remainder = match unit_string.as_str() {
            // 9. If unit is "year", then
            "year" => {
                // This should be safe as we throw a range error if relative_to does not exist.
                assert!(calendar.is_some() && relative_to.is_some());
                let calendar_obj = calendar.expect("calendar must exist at this point.");
                let relative_to = relative_to.expect("relative_to must exist at this point.");

                // a. Let yearsDuration be ! CreateTemporalDuration(years, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_duration = super::create_temporal_duration(
                    DurationRecord::default().with_years(self.years()),
                    None,
                    context,
                )?;

                // b. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // c. Else,
                // i. Let dateAdd be unused.
                let date_add: JsValue = calendar_obj
                    .get_method("dateAdd", context)?
                    .expect("dateAdd must exist on a calendar prototype")
                    .into();

                // d. Let yearsLater be ? CalendarDateAdd(calendar, relativeTo, yearsDuration, undefined, dateAdd).

                let years_later = calendar::calendar_date_add(
                    &calendar_obj,
                    &relative_to,
                    &years_duration,
                    &JsValue::undefined(),
                    Some(&date_add),
                )?;

                // e. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = super::create_temporal_duration(
                    DurationRecord::default()
                        .with_years(self.years())
                        .with_months(self.months())
                        .with_weeks(self.weeks()),
                    None,
                    context,
                )?;

                // f. Let yearsMonthsWeeksLater be ? CalendarDateAdd(calendar, relativeTo, yearsMonthsWeeks, undefined, dateAdd).
                let years_months_weeks_later = calendar::calendar_date_add(
                    &calendar_obj,
                    &relative_to,
                    &years_months_weeks,
                    &JsValue::undefined(),
                    Some(&date_add),
                )?;

                // g. Let monthsWeeksInDays be DaysUntil(yearsLater, yearsMonthsWeeksLater).
                let months_weeks_in_days =
                    super::days_until(&years_later, &years_months_weeks_later);

                // h. Set relativeTo to yearsLater.
                let relative_to = years_later;

                // i. Let days be days + monthsWeeksInDays.
                self.set_days(self.days() + f64::from(months_weeks_in_days));

                // j. Let wholeDaysDuration be ? CreateTemporalDuration(0, 0, 0, truncate(days), 0, 0, 0, 0, 0, 0).
                let whole_days_duration = super::create_temporal_duration(
                    DurationRecord::default().with_days(self.days()),
                    None,
                    context,
                )?;

                // k. Let wholeDaysLater be ? CalendarDateAdd(calendar, relativeTo, wholeDaysDuration, undefined, dateAdd).
                let whole_days_later = calendar::calendar_date_add(
                    &calendar_obj,
                    &relative_to,
                    &whole_days_duration,
                    &JsValue::undefined(),
                    Some(&date_add),
                )?;

                // l. Let untilOptions be OrdinaryObjectCreate(null).
                let until_options = JsObject::with_null_proto();
                // m. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "year").
                until_options.create_data_property_or_throw("largestUnit", "year", context)?;

                // n. Let timePassed be ? CalendarDateUntil(calendar, relativeTo, wholeDaysLater, untilOptions).
                let time_passed = calendar::calendar_date_until(
                    &calendar_obj,
                    &relative_to,
                    &whole_days_later,
                    &until_options.into(),
                    None,
                )?;

                // o. Let yearsPassed be timePassed.[[Years]].
                let years_passed = time_passed.years();
                // p. Set years to years + yearsPassed.
                self.set_years(self.years() + years_passed);

                // q. Let oldRelativeTo be relativeTo.
                let old_relative_to = relative_to.clone();

                // r. Let yearsDuration be ! CreateTemporalDuration(yearsPassed, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_duration = super::create_temporal_duration(
                    DurationRecord::default().with_years(years_passed),
                    None,
                    context,
                )?;

                // s. Set relativeTo to ? CalendarDateAdd(calendar, relativeTo, yearsDuration, undefined, dateAdd).
                let relative_to = calendar::calendar_date_add(
                    &calendar_obj,
                    &relative_to,
                    &years_duration,
                    &JsValue::undefined(),
                    Some(&date_add),
                )?;

                // t. Let daysPassed be DaysUntil(oldRelativeTo, relativeTo).
                let days_passed = super::days_until(&old_relative_to, &relative_to);

                // u. Set days to days - daysPassed.
                self.set_days(self.days() - f64::from(days_passed));

                // v. If days < 0, let sign be -1; else, let sign be 1.
                let sign = if self.days() < 0_f64 { -1 } else { 1 };

                // w. Let oneYear be ! CreateTemporalDuration(sign, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let one_year = super::create_temporal_duration(
                    DurationRecord::default().with_years(f64::from(sign)),
                    None,
                    context,
                )?;

                // x. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneYear, dateAdd).
                let move_result = super::move_relative_date(
                    &calendar_obj,
                    &relative_to,
                    one_year,
                    Some(&date_add),
                )?;

                // y. Let oneYearDays be moveResult.[[Days]].
                let one_year_days = move_result.1;
                // z. Let fractionalYears be years + days / abs(oneYearDays).
                let fractional_years = self.years() + self.days() / one_year_days.abs();

                // ?. Set years to RoundNumberToIncrement(fractionalYears, increment, roundingMode).
                self.set_years(temporal::round_number_to_increment(
                    fractional_years,
                    increment,
                    rounding_mode,
                ));

                // ?. Set months, weeks, and days to 0.
                self.set_months(0_f64);
                self.set_weeks(0_f64);
                self.set_days(0_f64);

                fractional_years - self.years()
            }
            // 10. Else if unit is "month", then
            "month" => {
                let mut relative_to =
                    relative_to.expect("relative_to must exist if unit is a month");
                let calendar_obj = calendar.expect("calendar must exist at this point.");

                // a. Let yearsMonths be ! CreateTemporalDuration(years, months, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_month = super::create_temporal_duration(
                    DurationRecord::default()
                        .with_years(self.years())
                        .with_months(self.months()),
                    None,
                    context,
                )?;

                // b. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // c. Else,
                // i. Let dateAdd be unused.
                let date_add: JsValue = calendar_obj
                    .get_method("dateAdd", context)?
                    .expect("dateAdd must exist on a calendar prototype")
                    .into();

                // d. Let yearsMonthsLater be ? CalendarDateAdd(calendar, relativeTo, yearsMonths, undefined, dateAdd).
                let years_months_later = calendar::calendar_date_add(
                    &calendar_obj,
                    &relative_to,
                    &years_month,
                    &JsValue::undefined(),
                    Some(&date_add),
                )?;

                // e. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = super::create_temporal_duration(
                    DurationRecord::default()
                        .with_years(self.years())
                        .with_months(self.months())
                        .with_weeks(self.weeks()),
                    None,
                    context,
                )?;

                // f. Let yearsMonthsWeeksLater be ? CalendarDateAdd(calendar, relativeTo, yearsMonthsWeeks, undefined, dateAdd).
                let years_months_weeks_later = calendar::calendar_date_add(
                    &calendar_obj,
                    &relative_to,
                    &years_months_weeks,
                    &JsValue::undefined(),
                    Some(&date_add),
                )?;
                // g. Let weeksInDays be DaysUntil(yearsMonthsLater, yearsMonthsWeeksLater).
                let weeks_in_days =
                    super::days_until(&years_months_later, &years_months_weeks_later);

                // h. Set relativeTo to yearsMonthsLater.
                relative_to = years_months_later;

                // i. Let days be days + weeksInDays.
                self.set_days(self.days() + f64::from(weeks_in_days));

                // j. If days < 0, let sign be -1; else, let sign be 1.
                let sign = if self.days() < 0_f64 { -1_f64 } else { 1_f64 };

                // k. Let oneMonth be ! CreateTemporalDuration(0, sign, 0, 0, 0, 0, 0, 0, 0, 0).
                let one_month = super::create_temporal_duration(
                    DurationRecord::default().with_months(sign),
                    None,
                    context,
                )?;

                // l. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
                let move_result = super::move_relative_date(
                    &calendar_obj,
                    &relative_to,
                    one_month.clone(),
                    Some(&date_add),
                )?;

                // m. Set relativeTo to moveResult.[[RelativeTo]].
                relative_to = move_result.0;
                // n. Let oneMonthDays be moveResult.[[Days]].
                let mut one_month_days = move_result.1;

                // o. Repeat, while abs(days) ≥ abs(oneMonthDays),
                while self.days().abs() >= one_month_days.abs() {
                    // i. Set months to months + sign.
                    self.set_months(self.months() + sign);
                    // ii. Set days to days - oneMonthDays.
                    self.set_days(self.days() - one_month_days);
                    // iii. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneMonth, dateAdd).
                    let move_result = super::move_relative_date(
                        &calendar_obj,
                        &relative_to,
                        one_month.clone(),
                        Some(&date_add),
                    )?;

                    // iv. Set relativeTo to moveResult.[[RelativeTo]].
                    relative_to = move_result.0;
                    // v. Set oneMonthDays to moveResult.[[Days]].
                    one_month_days = move_result.1;
                }

                // p. Let fractionalMonths be months + days / abs(oneMonthDays).
                let fractional_months = self.months() + (self.days() / one_month_days.abs());
                // q. Set months to RoundNumberToIncrement(fractionalMonths, increment, roundingMode).
                self.set_months(temporal::round_number_to_increment(
                    fractional_months,
                    increment,
                    rounding_mode,
                ));
                // r. Set remainder to fractionalMonths - months.
                // s. Set weeks and days to 0.
                self.set_weeks(0_f64);
                self.set_days(0_f64);
                fractional_months - self.months()
            }
            // 11. Else if unit is "week", then
            "week" => {
                let mut relative_to =
                    relative_to.expect("relative_to must exist if unit is a month");
                let calendar_obj = calendar.expect("calendar must exist at this point.");
                // a. If days < 0, let sign be -1; else, let sign be 1.
                let sign = if self.days() < 0_f64 { -1_f64 } else { 1_f64 };
                // b. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
                let one_week = super::create_temporal_duration(
                    DurationRecord::default().with_weeks(sign),
                    None,
                    context,
                )?;

                // c. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // d. Else,
                // i. Let dateAdd be unused.
                let date_add: JsValue = calendar_obj
                    .get_method("dateAdd", context)?
                    .expect("dateAdd must exist on a calendar prototype")
                    .into();

                // e. Let moveResult be ? MoveRelativeDate(calendar, relativeTo, oneWeek, dateAdd).
                let move_result = super::move_relative_date(
                    &calendar_obj,
                    &relative_to,
                    one_week.clone(),
                    Some(&date_add),
                )?;

                // f. Set relativeTo to moveResult.[[RelativeTo]].
                relative_to = move_result.0;
                // g. Let oneWeekDays be moveResult.[[Days]].
                let mut one_week_days = move_result.1;

                // h. Repeat, while abs(days) ≥ abs(oneWeekDays),
                while one_week_days.abs() <= self.days().abs() {
                    // i. Set weeks to weeks + sign.
                    self.set_weeks(self.weeks() + sign);
                    // ii. Set days to days - oneWeekDays.
                    self.set_days(self.days() - one_week_days);
                    // iii. Set moveResult to ? MoveRelativeDate(calendar, relativeTo, oneWeek, dateAdd).
                    let move_result = super::move_relative_date(
                        &calendar_obj,
                        &relative_to,
                        one_week.clone(),
                        Some(&date_add),
                    )?;
                    // iv. Set relativeTo to moveResult.[[RelativeTo]].
                    relative_to = move_result.0;
                    // v. Set oneWeekDays to moveResult.[[Days]].
                    one_week_days = move_result.1;
                }

                // i. Let fractionalWeeks be weeks + days / abs(oneWeekDays).
                let fractional_weeks = self.weeks() + (self.days() / one_week_days.abs());

                // j. Set weeks to RoundNumberToIncrement(fractionalWeeks, increment, roundingMode).
                self.set_weeks(temporal::round_number_to_increment(
                    fractional_weeks,
                    increment,
                    rounding_mode,
                ));
                // k. Set remainder to fractionalWeeks - weeks.
                // l. Set days to 0.
                self.set_days(0_f64);
                fractional_weeks - self.weeks()
            }
            // 12. Else if unit is "day", then
            "day" => {
                // a. Let fractionalDays be days.
                let fractional_days = self.days();
                // b. Set days to RoundNumberToIncrement(days, increment, roundingMode).
                self.set_days(temporal::round_number_to_increment(
                    self.days(),
                    increment,
                    rounding_mode,
                ));
                // c. Set remainder to fractionalDays - days.
                fractional_days - self.days()
            }
            // 13. Else if unit is "hour", then
            "hour" => {
                // a. Let fractionalHours be (fractionalSeconds / 60 + minutes) / 60 + hours.
                let fractional_hours =
                    (fractional_secs / (60_f64 + self.minutes())) / 60_f64 + self.hours();
                // b. Set hours to RoundNumberToIncrement(fractionalHours, increment, roundingMode).
                self.set_hours(temporal::round_number_to_increment(
                    fractional_hours,
                    increment,
                    rounding_mode,
                ));
                // d. Set minutes, seconds, milliseconds, microseconds, and nanoseconds to 0.
                self.set_minutes(0_f64);
                self.set_seconds(0_f64);
                self.set_milliseconds(0_f64);
                self.set_microseconds(0_f64);
                self.set_nanoseconds(0_f64);
                // c. Set remainder to fractionalHours - hours.
                fractional_hours - self.hours()
            }
            // 14. Else if unit is "minute", then
            "minute" => {
                // a. Let fractionalMinutes be fractionalSeconds / 60 + minutes.
                let fraction_minutes = fractional_secs / 60_f64 + self.minutes();
                // b. Set minutes to RoundNumberToIncrement(fractionalMinutes, increment, roundingMode).
                self.set_minutes(temporal::round_number_to_increment(
                    fraction_minutes,
                    increment,
                    rounding_mode,
                ));
                // d. Set seconds, milliseconds, microseconds, and nanoseconds to 0.
                self.set_seconds(0_f64);
                self.set_milliseconds(0_f64);
                self.set_microseconds(0_f64);
                self.set_nanoseconds(0_f64);
                // c. Set remainder to fractionalMinutes - minutes.
                fraction_minutes - self.minutes()
            }
            // 15. Else if unit is "second", then
            "second" => {
                // a. Set seconds to RoundNumberToIncrement(fractionalSeconds, increment, roundingMode).
                self.set_seconds(temporal::round_number_to_increment(
                    fractional_secs,
                    increment,
                    rounding_mode,
                ));
                // c. Set milliseconds, microseconds, and nanoseconds to 0.
                self.set_milliseconds(0_f64);
                self.set_microseconds(0_f64);
                self.set_nanoseconds(0_f64);
                // b. Set remainder to fractionalSeconds - seconds.
                fractional_secs - self.seconds()
            }
            // 16. Else if unit is "millisecond", then
            "millisecond" => {
                // a. Let fractionalMilliseconds be nanoseconds × 10-6 + microseconds × 10-3 + milliseconds.
                let fractional_millis = self
                    .nanoseconds()
                    .mul_add(1_000_000_f64, self.microseconds() * 1_000_f64)
                    + self.milliseconds();
                // b. Set milliseconds to RoundNumberToIncrement(fractionalMilliseconds, increment, roundingMode).
                self.set_milliseconds(temporal::round_number_to_increment(
                    fractional_millis,
                    increment,
                    rounding_mode,
                ));
                // d. Set microseconds and nanoseconds to 0.
                self.set_microseconds(0_f64);
                self.set_nanoseconds(0_f64);
                // c. Set remainder to fractionalMilliseconds - milliseconds.
                fractional_millis - self.milliseconds()
            }
            // 17. Else if unit is "microsecond", then
            "microsecond" => {
                // a. Let fractionalMicroseconds be nanoseconds × 10-3 + microseconds.
                let fractional_micros = self.nanoseconds().mul_add(1_000_f64, self.microseconds());
                // b. Set microseconds to RoundNumberToIncrement(fractionalMicroseconds, increment, roundingMode).
                self.set_microseconds(temporal::round_number_to_increment(
                    fractional_micros,
                    increment,
                    rounding_mode,
                ));
                // d. Set nanoseconds to 0.
                self.set_nanoseconds(0_f64);
                // c. Set remainder to fractionalMicroseconds - microseconds.
                fractional_micros - self.microseconds()
            }
            // 18. Else,
            "nanosecond" => {
                // a. Assert: unit is "nanosecond".
                // b. Set remainder to nanoseconds.
                let remainder = self.nanoseconds();
                // c. Set nanoseconds to RoundNumberToIncrement(nanoseconds, increment, roundingMode).
                self.set_nanoseconds(temporal::round_number_to_increment(
                    self.nanoseconds(),
                    increment,
                    rounding_mode,
                ));
                // d. Set remainder to remainder - nanoseconds.
                remainder - self.nanoseconds()
            }
            _ => unreachable!(),
        };

        // 19. Assert: days is an integer.
        assert!(self.days().fract() == 0.0);

        // 20. Let duration be ? CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
        // 21. Return the Record { [[DurationRecord]]: duration, [[Remainder]]: remainder }.
        Ok(remainder)
    }

    /// 7.5.27 `AdjustRoundedDurationDays ( years, months, weeks, days, hours, minutes, seconds, milliseconds,
    /// microseconds, nanoseconds, increment, unit, roundingMode, relativeTo )`
    pub(crate) fn adjust_rounded_duration_days(
        &mut self,
        increment: f64,
        unit: &JsString,
        rounding_mode: &JsString,
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
                obj.as_zoned_date_time()
                    .expect("Object must be a ZonedDateTime.")
                    .clone()
            }
            // a. Return ! CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
            _ => return Ok(()),
        };

        match unit.to_std_string_escaped().as_str() {
            // a. Return ! CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
            "year" | "month" | "week" | "day" => return Ok(()),
            // a. Return ! CreateDurationRecord(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
            "nanosecond" if (increment - 1_f64).abs() < f64::EPSILON => return Ok(()),
            _ => {}
        }

        // 2. Let timeRemainderNs be ! TotalDurationNanoseconds(0, hours, minutes, seconds, milliseconds, microseconds, nanoseconds, 0).
        let time_remainder_ns = super::total_duration_nanoseconds(
            0_f64,
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.milliseconds(),
            self.microseconds(),
            self.nanoseconds(),
            0_f64,
        );

        // 3. If timeRemainderNs = 0, let direction be 0.
        let direction = if time_remainder_ns == 0_f64 {
            0
        // 4. Else if timeRemainderNs < 0, let direction be -1.
        } else if time_remainder_ns < 0_f64 {
            -1
        // 5. Else, let direction be 1.
        } else {
            1
        };

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
        Ok(())
    }
}
