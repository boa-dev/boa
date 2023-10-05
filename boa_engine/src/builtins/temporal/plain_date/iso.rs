//! An `IsoDateRecord` that represents the `[[ISOYear]]`, `[[ISOMonth]]`, and `[[ISODay]]` internal slots.

use crate::{
    builtins::temporal::{self, TemporalFields},
    JsNativeError, JsResult, JsString,
};

use icu_calendar::{Date, Iso};

// TODO: Move ISODateRecord to a more generalized location.

// TODO: Determine whether month/day should be u8 or i32.

/// `IsoDateRecord` serves as an record for the `[[ISOYear]]`, `[[ISOMonth]]`,
/// and `[[ISODay]]` internal fields.
///
/// These fields are used for the `Temporal.PlainDate` object, the
/// `Temporal.YearMonth` object, and the `Temporal.MonthDay` object.
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
    /// Creates `[[ISOYear]]`, `[[isoMonth]]`, `[[isoDay]]` fields from `ICU4X`'s `Date<Iso>` struct.
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

    /// Create a Month-Day record from a `TemporalFields` object.
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
        // TODO: Implement on `ICU4X`.

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
