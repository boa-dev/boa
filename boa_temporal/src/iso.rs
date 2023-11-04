//! An `IsoDate` that represents the `[[ISOYear]]`, `[[ISOMonth]]`, and `[[ISODay]]` internal slots.

use crate::{
    duration::DateDuration, error::TemporalError, options::ArithmeticOverflow, utils,
    TemporalResult,
};
use icu_calendar::{Date as IcuDate, Iso};
use num_bigint::BigInt;
use num_traits::cast::FromPrimitive;

/// `IsoDateTime` is the Temporal internal representation of
/// a `DateTime` record
#[derive(Debug, Default, Clone, Copy)]
pub struct IsoDateTime {
    date: IsoDate,
    time: IsoTime,
}

impl IsoDateTime {
    /// Creates a new `IsoDateTime` without any validaiton.
    pub(crate) fn new_unchecked(date: IsoDate, time: IsoTime) -> Self {
        Self { date, time }
    }

    /// Returns whether the `IsoDateTime` is within valid limits.
    pub(crate) fn is_within_limits(&self) -> bool {
        let Some(ns) = self.to_utc_epoch_nanoseconds(0f64) else {
            return false;
        };

        let max = crate::ns_max_instant() + crate::NS_PER_DAY;
        let min = crate::ns_min_instant() - crate::NS_PER_DAY;

        min < ns && max > ns
    }

    /// Returns the UTC epoch nanoseconds for this `IsoDateTime`.
    pub(crate) fn to_utc_epoch_nanoseconds(&self, offset: f64) -> Option<BigInt> {
        let day = self.date.to_epoch_days();
        let time = self.time.to_epoch_ms();
        let epoch_ms = utils::epoch_days_to_epoch_ms(day, time);

        let epoch_nanos = epoch_ms.mul_add(
            1_000_000f64,
            f64::from(self.time.microsecond).mul_add(1_000f64, f64::from(self.time.nanosecond)),
        );

        BigInt::from_f64(epoch_nanos - offset)
    }

    pub(crate) fn iso_date(&self) -> IsoDate {
        self.date
    }

    pub(crate) fn iso_time(&self) -> IsoTime {
        self.time
    }
}

// ==== `IsoDate` section ====

// TODO: Figure out `ICU4X` interop / replacement?

/// `IsoDate` serves as a record for the `[[ISOYear]]`, `[[ISOMonth]]`,
/// and `[[ISODay]]` internal fields.
///
/// These fields are used for the `Temporal.PlainDate` object, the
/// `Temporal.YearMonth` object, and the `Temporal.MonthDay` object.
#[derive(Debug, Clone, Copy, Default)]
pub struct IsoDate {
    year: i32,
    month: u8,
    day: u8,
}

impl IsoDate {
    /// Creates a new `IsoDate` without determining the validity.
    pub(crate) const fn new_unchecked(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub(crate) fn new(
        year: i32,
        month: i32,
        day: i32,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        match overflow {
            ArithmeticOverflow::Constrain => {
                let m = month.clamp(1, 12);
                let days_in_month = utils::iso_days_in_month(year, month);
                let d = day.clamp(1, days_in_month);
                Ok(Self::new_unchecked(year, m as u8, d as u8))
            }
            ArithmeticOverflow::Reject => {
                if !is_valid_date(year, month, day) {
                    return Err(TemporalError::range().with_message("not a valid ISO date."));
                }
                // NOTE: Values have been verified to be in a u8 range.
                Ok(Self::new_unchecked(year, month as u8, day as u8))
            }
        }
    }

    pub(crate) fn new_year_month(
        year: i32,
        month: i32,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        match overflow {
            ArithmeticOverflow::Constrain => {
                let m = month.clamp(1, 12) as u8;
                Ok(Self::new_unchecked(year, m, 1))
            }
            ArithmeticOverflow::Reject => {
                if (1..=12).contains(&month) {
                    return Err(TemporalError::range().with_message("IsoYearMonth is not valid."));
                }
                // NOTE: Values have been verified to be in a u8 range.
                Ok(Self::new_unchecked(year, month as u8, 1))
            }
        }
    }

    /// Create a balanced `IsoDate`
    ///
    /// Equivalent to `BalanceISODate`.
    fn balance(year: i32, month: i32, day: i32) -> Self {
        let epoch_days = iso_date_to_epoch_days(year, month - 1, day);
        let ms = utils::epoch_days_to_epoch_ms(epoch_days, 0f64);
        Self::new_unchecked(
            utils::epoch_time_to_epoch_year(ms),
            utils::epoch_time_to_month_in_year(ms) + 1,
            utils::epoch_time_to_date(ms),
        )
    }

    /// Returns the year field
    pub(crate) const fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month field
    pub(crate) const fn month(&self) -> u8 {
        self.month
    }

    /// Returns the day field
    pub(crate) const fn day(&self) -> u8 {
        self.day
    }

    /// Functionally the same as Date's abstract operation `MakeDay`
    ///
    /// Equivalent to `IsoDateToEpochDays`
    pub(crate) fn to_epoch_days(&self) -> i32 {
        iso_date_to_epoch_days(self.year, self.month.into(), self.day.into())
    }

    /// Returns if the current `IsoDate` is valid.
    pub(crate) fn is_valid(&self) -> bool {
        is_valid_date(self.year, self.month.into(), self.day.into())
    }

    pub(crate) fn add_iso_date(
        &self,
        duration: &DateDuration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        // 1. Assert: year, month, day, years, months, weeks, and days are integers.
        // 2. Assert: overflow is either "constrain" or "reject".
        // 3. Let intermediate be ! BalanceISOYearMonth(year + years, month + months).
        let mut intermediate_year = self.year + duration.years() as i32;
        let mut intermediate_month = i32::from(self.month) + duration.months() as i32;

        intermediate_year += (intermediate_month - 1) / 12;
        intermediate_month = (intermediate_month - 1) % 12 + 1;

        // 4. Let intermediate be ? RegulateISODate(intermediate.[[Year]], intermediate.[[Month]], day, overflow).
        let intermediate = Self::new(
            intermediate_year,
            intermediate_month,
            i32::from(self.day),
            overflow,
        )?;

        // 5. Set days to days + 7 × weeks.
        // 6. Let d be intermediate.[[Day]] + days.
        let additional_days = duration.days() as i32 + (duration.weeks() as i32 * 7);
        let d = i32::from(intermediate.day) + additional_days;

        // 7. Return BalanceISODate(intermediate.[[Year]], intermediate.[[Month]], d).
        Ok(Self::balance(
            intermediate.year,
            intermediate.month.into(),
            d,
        ))
    }
}

impl IsoDate {
    // TODO: look into using Date<Iso> across the board...TBD.
    /// Creates `[[ISOYear]]`, `[[isoMonth]]`, `[[isoDay]]` fields from `ICU4X`'s `Date<Iso>` struct.
    pub(crate) fn as_icu4x(&self) -> TemporalResult<IcuDate<Iso>> {
        IcuDate::try_new_iso_date(self.year, self.month, self.day)
            .map_err(|e| TemporalError::range().with_message(e.to_string()))
    }
}

// ==== `IsoTime` section ====

/// An `IsoTime` record that contains `Temporal`'s
/// time slots.
#[derive(Debug, Default, Clone, Copy)]
pub struct IsoTime {
    hour: i32,        // 0..=23
    minute: i32,      // 0..=59
    second: i32,      // 0..=59
    millisecond: i32, // 0..=999
    microsecond: i32, // 0..=999
    nanosecond: i32,  // 0..=999
}

impl IsoTime {
    /// Creates a new `IsoTime` without any validation.
    pub(crate) fn new_unchecked(
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: i32,
        microsecond: i32,
        nanosecond: i32,
    ) -> Self {
        Self {
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
        }
    }

    /// Creates a new regulated `IsoTime`.
    pub fn new(
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: i32,
        microsecond: i32,
        nanosecond: i32,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<IsoTime> {
        match overflow {
            ArithmeticOverflow::Constrain => {
                let h = hour.clamp(0, 23);
                let min = minute.clamp(0, 59);
                let sec = second.clamp(0, 59);
                let milli = millisecond.clamp(0, 999);
                let micro = microsecond.clamp(0, 999);
                let nano = nanosecond.clamp(0, 999);
                Ok(Self::new_unchecked(h, min, sec, milli, micro, nano))
            }
            ArithmeticOverflow::Reject => {
                // TODO: Invert structure validation and update fields to u16.
                let time =
                    Self::new_unchecked(hour, minute, second, millisecond, microsecond, nanosecond);
                if !time.is_valid() {
                    return Err(TemporalError::range().with_message("IsoTime is not valid"));
                }
                Ok(time)
            }
        }
    }

    /// Returns an `IsoTime` set to 12:00:00
    pub(crate) const fn noon() -> Self {
        Self {
            hour: 12,
            minute: 0,
            second: 0,
            millisecond: 0,
            microsecond: 0,
            nanosecond: 0,
        }
    }

    /// Checks if the time is a valid `IsoTime`
    pub(crate) fn is_valid(&self) -> bool {
        if !(0..=23).contains(&self.hour) {
            return false;
        }

        let min_sec = 0..=59;
        if !min_sec.contains(&self.minute) || !min_sec.contains(&self.second) {
            return false;
        }

        let sub_second = 0..=999;
        sub_second.contains(&self.millisecond)
            && sub_second.contains(&self.microsecond)
            && sub_second.contains(&self.nanosecond)
    }

    /// `IsoTimeToEpochMs`
    ///
    /// Note: This method is library specific and not in spec
    ///
    /// Functionally the same as Date's `MakeTime`
    pub(crate) fn to_epoch_ms(&self) -> f64 {
        f64::from(self.hour).mul_add(
            utils::MS_PER_HOUR,
            f64::from(self.minute) * utils::MS_PER_MINUTE,
        ) + f64::from(self.second).mul_add(1000f64, f64::from(self.millisecond))
    }
}

// ==== `IsoDate` specific utiltiy functions ====

/// Returns the EpochDays based off the given year, month, and day.
#[inline]
fn iso_date_to_epoch_days(year: i32, month: i32, day: i32) -> i32 {
    // 1. Let resolvedYear be year + floor(month / 12).
    let resolved_year = year + (f64::from(month) / 12_f64).floor() as i32;
    // 2. Let resolvedMonth be month modulo 12.
    let resolved_month = month % 12;

    // 3. Find a time t such that EpochTimeToEpochYear(t) is resolvedYear, EpochTimeToMonthInYear(t) is resolvedMonth, and EpochTimeToDate(t) is 1.
    let year_t = utils::epoch_time_for_year(resolved_year);
    let month_t = utils::epoch_time_for_month_given_year(resolved_month.into(), resolved_year);

    // 4. Return EpochTimeToDayNumber(t) + date - 1.
    utils::epoch_time_to_day_number(year_t + month_t) + day - 1
}

#[inline]
// Determines if the month and day are valid for the given year.
fn is_valid_date(year: i32, month: i32, day: i32) -> bool {
    if !(1..=12).contains(&month) {
        return false;
    }

    let days_in_month = utils::iso_days_in_month(year, month);
    (1..=days_in_month).contains(&day)
}

/*
impl IsoDate {
    /// 3.5.6 `RegulateISODate`
    pub(crate) fn from_unregulated(
        year: i32,
        month: i32,
        day: i32,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        match overflow {
            ArithmeticOverflow::Constrain => {
                let m = month.clamp(1, 12);
                let days_in_month = utils::iso_days_in_month(year, month);
                let d = day.clamp(1, days_in_month);
                Ok(Self::new_unchecked(year, m, d))
            }
            ArithmeticOverflow::Reject => {
                let date = Self::new_unchecked(year, month, day);
                if !date.is_valid() {
                    return Err(TemporalError::range()
                        .with_message("not a valid ISO date."));
                }
                Ok(date)
            }
        }
    }

    /// 12.2.35 `ISODateFromFields ( fields, overflow )`
    ///
    /// Note: fields.month must be resolved prior to using `from_temporal_fields`
    pub(crate) fn from_temporal_fields(
        fields: &TemporalFields,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
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
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
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
        largest_unit: TemporalUnit,
    ) -> TemporalResult<DurationRecord> {
        debug_assert!(self.is_valid());
        // TODO: Implement on `ICU4X`.

        Err(TemporalError::range()
            .with_message("not yet implemented."))
    }

    /// 3.5.7 `IsValidISODate`
    pub(crate) fn is_valid(&self) -> bool {
        if self.month < 1 || self.month > 12 {
            return false;
        }

        let days_in_month = utils::iso_days_in_month(self.year, self.month);

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
        let year_t = utils::epoch_time_for_year(resolved_year);
        let month_t = utils::epoch_time_for_month_given_year(
            resolved_month,
            resolved_year,
        );

        // 4. Return EpochTimeToDayNumber(t) + date - 1.
        utils::epoch_time_to_day_number(year_t + month_t) + self.day - 1
    }

    // NOTE: Implementing as mut self so balance is applied to self, but TBD.
    /// 3.5.8 `BalanceIsoDate`
    pub(crate) fn balance(&mut self) {
        let epoch_days = self.as_epoch_days();
        let ms = utils::epoch_days_to_epoch_ms(epoch_days, 0);

        // Balance current values
        self.year = utils::epoch_time_to_epoch_year(ms);
        self.month = utils::epoch_time_to_month_in_year(ms);
        self.day = utils::epoch_time_to_date(ms);
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
        date_duration: DateDuration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        // 1. Assert: year, month, day, years, months, weeks, and days are integers.
        // 2. Assert: overflow is either "constrain" or "reject".
        let mut intermediate = Self::new_unchecked(
            self.year + date_duration.years() as i32,
            self.month + date_duration.months() as i32,
            0,
        );

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
        let additional_days = date_duration.days() as i32 + (date_duration.weeks() as i32 * 7);
        new_date.day += additional_days;

        // 7. Return BalanceISODate(intermediate.[[Year]], intermediate.[[Month]], d).
        new_date.balance();

        Ok(new_date)
    }
}
*/
