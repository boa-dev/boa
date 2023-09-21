use crate::{
    builtins::temporal::{self, TemporalFields},
    JsNativeError, JsResult, JsString,
};

use icu_calendar::{Date, Iso};

// TODO: Move ISODateRecord to a more generalized location.

// TODO: shift month and day to u8's to better align with `ICU4x`.

/// `IsoDateRecord` serves as an inner Record for the `Temporal.PlainDate`
/// object, the `Temporal.YearMonth` object, and the `Temporal.MonthDay`
/// object.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct IsoDateRecord {
    year: i32,
    month: i32,
    day: i32,
}

// TODO: determine whether the below is neccessary.
impl IsoDateRecord {
    pub(crate) const fn year(&self) -> i32 {
        self.year
    }
    pub(crate) const fn month(&self) -> i32 {
        self.month
    }
    pub(crate) const fn day(&self) -> i32 {
        self.day
    }
}

impl IsoDateRecord {
    // TODO: look into using Date<Iso> across the board...TBD.
    /// Creates `[[ISOYear]]`, `[[isoMonth]]`, `[[isoDay]]` fields from `ICU4X`'s Date<Iso> struct.
    pub(crate) fn from_date_iso(date: Date<Iso>) -> Self {
        Self {
            year: date.year().number,
            month: date.month().ordinal as i32,
            day: i32::from(date.days_in_month()),
        }
    }
}

impl IsoDateRecord {
    /// 3.5.2 `CreateISODateRecord`
    pub(crate) const fn new(year: i32, month: i32, day: i32) -> Self {
        Self { year, month, day }
    }

    /// 3.5.6 `RegulateISODate`
    pub(crate) fn from_unregulated(
        year: i32,
        month: i32,
        day: i32,
        overflow: &JsString,
    ) -> JsResult<Self> {
        match overflow.to_std_string_escaped().as_str() {
            "constrain" => {
                let m = month.clamp(1, 12);
                let days_in_month = temporal::calendar::utils::iso_days_in_month(year, month);
                let d = day.clamp(1, days_in_month);
                Ok(Self::new(year, m, d))
            }
            "reject" => {
                let date = Self::new(year, month, day);
                if !date.is_valid() {
                    return Err(JsNativeError::range()
                        .with_message("not a valid ISO date.")
                        .into());
                }
                Ok(date)
            }
            _ => unreachable!(),
        }
    }

    /// 12.2.35 `ISODateFromFields ( fields, overflow )`
    ///
    /// Note: fields.month must be resolved prior to using `from_temporal_fields`
    pub(crate) fn from_temporal_fields(
        fields: &TemporalFields,
        overflow: &JsString,
    ) -> JsResult<Self> {
        Self::from_unregulated(
            fields.year().expect("Cannot fail per spec"),
            fields.month().expect("cannot fail after resolution"),
            fields.day().expect("cannot fail per spec"),
            overflow,
        )
    }

    pub(crate) fn month_day_from_temporal_fields(
        fields: &TemporalFields,
        overflow: &JsString,
    ) -> JsResult<Self> {
        match fields.year() {
            Some(year) => Self::from_unregulated(
                year,
                fields.month().expect("month must exist."),
                fields.day().expect("cannot fail per spec"),
                overflow,
            ),
            None => Self::from_unregulated(
                1972,
                fields.month().expect("cannot fail per spec"),
                fields.day().expect("cannot fail per spec."),
                overflow,
            ),
        }
    }

    /// Within `YearMonth` valid limits
    pub(crate) const fn within_year_month_limits(&self) -> bool {
        if self.year < -271_821 || self.year > 275_760 {
            return false;
        }

        if self.year == -271_821 && self.month < 4 {
            return false;
        }

        if self.year == 275_760 && self.month > 9 {
            return true;
        }

        true
    }

    /// 3.5.5 `DifferenceISODate`
    pub(crate) fn diff_iso_date(
        &self,
        o: &Self,
        largest_unit: &JsString,
    ) -> JsResult<temporal::duration::DurationRecord> {
        debug_assert!(self.is_valid());
        // 1. Assert: IsValidISODate(y1, m1, d1) is true.
        // 2. Assert: IsValidISODate(y2, m2, d2) is true.
        // 3. If largestUnit is "year" or "month", then
        // a. Let sign be -(! CompareISODate(y1, m1, d1, y2, m2, d2)).
        // b. If sign is 0, return ! CreateDateDurationRecord(0, 0, 0, 0).
        // c. Let start be the Record { [[Year]]: y1, [[Month]]: m1, [[Day]]: d1 }.
        // d. Let end be the Record { [[Year]]: y2, [[Month]]: m2, [[Day]]: d2 }.
        // e. Let years be end.[[Year]] - start.[[Year]].
        // f. Let mid be ! AddISODate(y1, m1, d1, years, 0, 0, 0, "constrain").
        // g. Let midSign be -(! CompareISODate(mid.[[Year]], mid.[[Month]], mid.[[Day]], y2, m2, d2)).
        // h. If midSign is 0, then
        // i. If largestUnit is "year", return ! CreateDateDurationRecord(years, 0, 0, 0).
        // ii. Return ! CreateDateDurationRecord(0, years × 12, 0, 0).
        // i. Let months be end.[[Month]] - start.[[Month]].
        // j. If midSign is not equal to sign, then
        // i. Set years to years - sign.
        // ii. Set months to months + sign × 12.
        // k. Set mid to ! AddISODate(y1, m1, d1, years, months, 0, 0, "constrain").
        // l. Set midSign to -(! CompareISODate(mid.[[Year]], mid.[[Month]], mid.[[Day]], y2, m2, d2)).
        // m. If midSign is 0, then
        // i. If largestUnit is "year", return ! CreateDateDurationRecord(years, months, 0, 0).
        // ii. Return ! CreateDateDurationRecord(0, months + years × 12, 0, 0).
        // n. If midSign is not equal to sign, then
        // i. Set months to months - sign.
        // ii. Set mid to ! AddISODate(y1, m1, d1, years, months, 0, 0, "constrain").
        // o. If mid.[[Month]] = end.[[Month]], then
        // i. Assert: mid.[[Year]] = end.[[Year]].
        // ii. Let days be end.[[Day]] - mid.[[Day]].
        // p. Else,
        // i. If sign < 0, let days be -mid.[[Day]] - (ISODaysInMonth(end.[[Year]], end.[[Month]]) - end.[[Day]]).
        // q. Else, let days be end.[[Day]] + (ISODaysInMonth(mid.[[Year]], mid.[[Month]]) - mid.[[Day]]).
        // r. If largestUnit is "month", then
        // i. Set months to months + years × 12.
        // ii. Set years to 0.
        // s. Return ! CreateDateDurationRecord(years, months, 0, days).
        // 4. Else,
        // a. Assert: largestUnit is "day" or "week".
        // b. Let epochDays1 be ISODateToEpochDays(y1, m1 - 1, d1).
        // c. Let epochDays2 be ISODateToEpochDays(y2, m2 - 1, d2).
        // d. Let days be epochDays2 - epochDays1.
        // e. Let weeks be 0.
        // f. If largestUnit is "week", then
        // i. Set weeks to truncate(days / 7).
        // ii. Set days to remainder(days, 7).
        // g. Return ! CreateDateDurationRecord(0, 0, weeks, days).

        Err(JsNativeError::range()
            .with_message("not yet implemented.")
            .into())
    }

    /// 3.5.7 `IsValidISODate`
    pub(crate) fn is_valid(&self) -> bool {
        if self.month < 1 || self.month > 12 {
            return false;
        }

        let days_in_month = temporal::calendar::utils::iso_days_in_month(self.year, self.month);

        if self.day < 1 || self.day > days_in_month {
            return false;
        }
        true
    }

    /// 13.2 `IsoDateToEpochDays`
    pub(crate) fn as_epoch_days(&self) -> i32 {
        // 1. Let resolvedYear be year + floor(month / 12).
        let resolved_year = self.year + (f64::from(self.month) / 12_f64).floor() as i32;
        // 2. Let resolvedMonth be month modulo 12.
        let resolved_month = self.month % 12;

        // 3. Find a time t such that EpochTimeToEpochYear(t) is resolvedYear, EpochTimeToMonthInYear(t) is resolvedMonth, and EpochTimeToDate(t) is 1.
        let year_t = temporal::date_equations::epoch_time_for_year(resolved_year);
        let month_t = temporal::date_equations::epoch_time_for_month_given_year(
            resolved_month,
            resolved_year,
        );

        // 4. Return EpochTimeToDayNumber(t) + date - 1.
        temporal::date_equations::epoch_time_to_day_number(year_t + month_t) + self.day - 1
    }

    // NOTE: Implementing as mut self so balance is applied to self, but TBD.
    /// 3.5.8 `BalanceIsoDate`
    pub(crate) fn balance(&mut self) {
        let epoch_days = self.as_epoch_days();
        let ms = temporal::epoch_days_to_epoch_ms(epoch_days, 0);

        // Balance current values
        self.year = temporal::date_equations::epoch_time_to_epoch_year(ms);
        self.month = temporal::date_equations::epoch_time_to_month_in_year(ms);
        self.day = temporal::date_equations::epoch_time_to_date(ms);
    }

    // NOTE: Used in AddISODate only, so could possibly be deleted in the future.
    /// 9.5.4 `BalanceISOYearMonth ( year, month )`
    pub(crate) fn balance_year_month(&mut self) {
        self.year += (self.month - 1) / 12;
        self.month = ((self.month - 1) % 12) + 1;
    }

    /// 3.5.11 `AddISODate ( year, month, day, years, months, weeks, days, overflow )`
    pub(crate) fn add_iso_date(
        &self,
        years: i32,
        months: i32,
        weeks: i32,
        days: i32,
        overflow: &JsString,
    ) -> JsResult<Self> {
        // 1. Assert: year, month, day, years, months, weeks, and days are integers.
        // 2. Assert: overflow is either "constrain" or "reject".
        let mut intermediate = Self::new(self.year + years, self.month + months, 0);

        // 3. Let intermediate be ! BalanceISOYearMonth(year + years, month + months).
        intermediate.balance_year_month();

        // 4. Let intermediate be ? RegulateISODate(intermediate.[[Year]], intermediate.[[Month]], day, overflow).
        let mut new_date = Self::from_unregulated(
            intermediate.year(),
            intermediate.month(),
            self.day,
            overflow,
        )?;

        // 5. Set days to days + 7 × weeks.
        // 6. Let d be intermediate.[[Day]] + days.
        let additional_days = days + (weeks * 7);
        new_date.day += additional_days;

        // 7. Return BalanceISODate(intermediate.[[Year]], intermediate.[[Month]], d).
        new_date.balance();

        Ok(new_date)
    }
}
