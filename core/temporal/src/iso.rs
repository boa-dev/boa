//! This module implements the internal ISO field slots.
//!
//! The three main types of slots are:
//!   - `IsoDateTime`
//!   - `IsoDate`
//!   - `IsoTime`
//!
//! An `IsoDate` represents the `[[ISOYear]]`, `[[ISOMonth]]`, and `[[ISODay]]` internal slots.
//!
//! An `IsoTime` represents the `[[ISOHour]]`, `[[ISOMinute]]`, `[[ISOsecond]]`, `[[ISOmillisecond]]`,
//! `[[ISOmicrosecond]]`, and `[[ISOnanosecond]]` internal slots.
//!
//! An `IsoDateTime` has the internal slots of both an `IsoDate` and `IsoTime`.

use crate::{
    components::duration::DateDuration, error::TemporalError, options::ArithmeticOverflow, utils,
    TemporalResult,
};
use icu_calendar::{Date as IcuDate, Iso};
use num_bigint::BigInt;
use num_traits::{cast::FromPrimitive, ToPrimitive};

/// `IsoDateTime` is the record of the `IsoDate` and `IsoTime` internal slots.
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

    // NOTE: The below assumes that nanos is from an `Instant` and thus in a valid range. -> Needs validation.
    /// Creates an `IsoDateTime` from a `BigInt` of epochNanoseconds.
    pub(crate) fn from_epoch_nanos(nanos: &BigInt, offset: f64) -> TemporalResult<Self> {
        // Skip the assert as nanos should be validated by Instant.
        // TODO: Determine whether value needs to be validated as integral.
        // Get the component ISO parts
        let mathematical_nanos = nanos.to_f64().ok_or_else(|| {
            TemporalError::range().with_message("nanos was not within a valid range.")
        })?;

        // 2. Let remainderNs be epochNanoseconds modulo 10^6.
        let remainder_nanos = mathematical_nanos % 1_000_000f64;

        // 3. Let epochMilliseconds be 𝔽((epochNanoseconds - remainderNs) / 10^6).
        let epoch_millis = ((mathematical_nanos - remainder_nanos) / 1_000_000f64).floor();

        let year = utils::epoch_time_to_epoch_year(epoch_millis);
        let month = utils::epoch_time_to_month_in_year(epoch_millis) + 1;
        let day = utils::epoch_time_to_date(epoch_millis);

        // 7. Let hour be ℝ(! HourFromTime(epochMilliseconds)).
        let hour = (epoch_millis / 3_600_000f64).floor() % 24f64;
        // 8. Let minute be ℝ(! MinFromTime(epochMilliseconds)).
        let minute = (epoch_millis / 60_000f64).floor() % 60f64;
        // 9. Let second be ℝ(! SecFromTime(epochMilliseconds)).
        let second = (epoch_millis / 1000f64).floor() % 60f64;
        // 10. Let millisecond be ℝ(! msFromTime(epochMilliseconds)).
        let millis = (epoch_millis % 1000f64).floor() % 1000f64;

        // 11. Let microsecond be floor(remainderNs / 1000).
        let micros = (remainder_nanos / 1000f64).floor();
        // 12. Assert: microsecond < 1000.
        debug_assert!(micros < 1000f64);
        // 13. Let nanosecond be remainderNs modulo 1000.
        let nanos = (remainder_nanos % 1000f64).floor();

        Ok(Self::balance(
            year,
            i32::from(month),
            i32::from(day),
            hour,
            minute,
            second,
            millis,
            micros,
            nanos + offset,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn balance(
        year: i32,
        month: i32,
        day: i32,
        hour: f64,
        minute: f64,
        second: f64,
        millisecond: f64,
        microsecond: f64,
        nanosecond: f64,
    ) -> Self {
        let (overflow_day, time) =
            IsoTime::balance(hour, minute, second, millisecond, microsecond, nanosecond);
        let date = IsoDate::balance(year, month, day + overflow_day);
        Self::new_unchecked(date, time)
    }

    /// Returns whether the `IsoDateTime` is within valid limits.
    pub(crate) fn is_within_limits(&self) -> bool {
        let Some(ns) = self.to_utc_epoch_nanoseconds(0f64) else {
            return false;
        };

        let max = BigInt::from(crate::NS_MAX_INSTANT + i128::from(crate::NS_PER_DAY));
        let min = BigInt::from(crate::NS_MIN_INSTANT - i128::from(crate::NS_PER_DAY));

        min < ns && max > ns
    }

    /// Returns the UTC epoch nanoseconds for this `IsoDateTime`.
    pub(crate) fn to_utc_epoch_nanoseconds(self, offset: f64) -> Option<BigInt> {
        let day = self.date.to_epoch_days();
        let time = self.time.to_epoch_ms();
        let epoch_ms = utils::epoch_days_to_epoch_ms(day, time);

        let epoch_nanos = epoch_ms.mul_add(
            1_000_000f64,
            f64::from(self.time.microsecond).mul_add(1_000f64, f64::from(self.time.nanosecond)),
        );

        BigInt::from_f64(epoch_nanos - offset)
    }

    pub(crate) fn date(&self) -> IsoDate {
        self.date
    }

    pub(crate) fn time(&self) -> IsoTime {
        self.time
    }
}

// ==== `IsoDate` section ====

// TODO: Figure out `ICU4X` interop / replacement?

/// A trait for accessing the `IsoDate` across the various Temporal objects
pub trait IsoDateSlots {
    /// Returns the target's internal `IsoDate`.
    fn iso_date(&self) -> IsoDate;
}

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
                // NOTE: Values are clamped in a u8 range.
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
    pub(crate) const fn year(self) -> i32 {
        self.year
    }

    /// Returns the month field
    pub(crate) const fn month(self) -> u8 {
        self.month
    }

    /// Returns the day field
    pub(crate) const fn day(self) -> u8 {
        self.day
    }

    /// Functionally the same as Date's abstract operation `MakeDay`
    ///
    /// Equivalent to `IsoDateToEpochDays`
    pub(crate) fn to_epoch_days(self) -> i32 {
        iso_date_to_epoch_days(self.year, self.month.into(), self.day.into())
    }

    /// Returns if the current `IsoDate` is valid.
    pub(crate) fn is_valid(self) -> bool {
        is_valid_date(self.year, self.month.into(), self.day.into())
    }

    /// Returns the resulting `IsoDate` from adding a provided `Duration` to this `IsoDate`
    pub(crate) fn add_iso_date(
        self,
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
    /// Creates `[[ISOYear]]`, `[[isoMonth]]`, `[[isoDay]]` fields from `ICU4X`'s `Date<Iso>` struct.
    pub(crate) fn as_icu4x(self) -> TemporalResult<IcuDate<Iso>> {
        IcuDate::try_new_iso_date(self.year, self.month, self.day)
            .map_err(|e| TemporalError::range().with_message(e.to_string()))
    }
}

// ==== `IsoTime` section ====

/// An `IsoTime` record that contains `Temporal`'s
/// time slots.
#[derive(Debug, Default, Clone, Copy)]
pub struct IsoTime {
    pub(crate) hour: u8,         // 0..=23
    pub(crate) minute: u8,       // 0..=59
    pub(crate) second: u8,       // 0..=59
    pub(crate) millisecond: u16, // 0..=999
    pub(crate) microsecond: u16, // 0..=999
    pub(crate) nanosecond: u16,  // 0..=999
}

impl IsoTime {
    /// Creates a new `IsoTime` without any validation.
    pub(crate) fn new_unchecked(
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
        microsecond: u16,
        nanosecond: u16,
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
                let h = hour.clamp(0, 23) as u8;
                let min = minute.clamp(0, 59) as u8;
                let sec = second.clamp(0, 59) as u8;
                let milli = millisecond.clamp(0, 999) as u16;
                let micro = microsecond.clamp(0, 999) as u16;
                let nano = nanosecond.clamp(0, 999) as u16;
                Ok(Self::new_unchecked(h, min, sec, milli, micro, nano))
            }
            ArithmeticOverflow::Reject => {
                if !is_valid_time(hour, minute, second, millisecond, microsecond, nanosecond) {
                    return Err(TemporalError::range().with_message("IsoTime is not valid"));
                };
                Ok(Self::new_unchecked(
                    hour as u8,
                    minute as u8,
                    second as u8,
                    millisecond as u16,
                    microsecond as u16,
                    nanosecond as u16,
                ))
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

    /// Returns an `IsoTime` based off parse components.
    pub(crate) fn from_components(
        hour: i32,
        minute: i32,
        second: i32,
        fraction: f64,
    ) -> TemporalResult<Self> {
        let millisecond = fraction * 1000f64;
        let micros = millisecond.rem_euclid(1f64) * 1000f64;
        let nanos = micros.rem_euclid(1f64).mul_add(1000f64, 0.5).floor();

        Self::new(
            hour,
            minute,
            second,
            millisecond as i32,
            micros as i32,
            nanos as i32,
            ArithmeticOverflow::Reject,
        )
    }

    // NOTE(nekevss): f64 is needed here as values could exceed i32 when input.
    /// Balances and creates a new `IsoTime` with `day` overflow from the provided values.
    pub(crate) fn balance(
        hour: f64,
        minute: f64,
        second: f64,
        millisecond: f64,
        microsecond: f64,
        nanosecond: f64,
    ) -> (i32, Self) {
        // 1. Set microsecond to microsecond + floor(nanosecond / 1000).
        let mut mis = microsecond + (nanosecond / 1000f64).floor();
        // 2. Set nanosecond to nanosecond modulo 1000.
        let ns = nanosecond % 1000f64;
        // 3. Set millisecond to millisecond + floor(microsecond / 1000).
        let mut ms = millisecond + (mis / 1000f64).floor();
        // 4. Set microsecond to microsecond modulo 1000.
        mis = mis.rem_euclid(1000f64);
        // 5. Set second to second + floor(millisecond / 1000).
        let mut secs = second + (ms / 1000f64).floor();
        // 6. Set millisecond to millisecond modulo 1000.
        ms = ms.rem_euclid(1000f64);
        // 7. Set minute to minute + floor(second / 60).
        let mut minutes = minute + (secs / 60f64).floor();
        // 8. Set second to second modulo 60.
        secs = secs.rem_euclid(60f64);
        // 9. Set hour to hour + floor(minute / 60).
        let mut hours = hour + (minutes / 60f64).floor();
        // 10. Set minute to minute modulo 60.
        minutes = minutes.rem_euclid(60f64);
        // 11. Let days be floor(hour / 24).
        let days = (hours / 24f64).floor();
        // 12. Set hour to hour modulo 24.
        hours = hours.rem_euclid(24f64);

        let time = Self::new_unchecked(
            hours as u8,
            minutes as u8,
            secs as u8,
            ms as u16,
            mis as u16,
            ns as u16,
        );

        (days as i32, time)
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
    pub(crate) fn to_epoch_ms(self) -> f64 {
        f64::from(self.hour).mul_add(
            utils::MS_PER_HOUR,
            f64::from(self.minute) * utils::MS_PER_MINUTE,
        ) + f64::from(self.second).mul_add(1000f64, f64::from(self.millisecond))
    }
}

// ==== `IsoDate` specific utiltiy functions ====

/// Returns the Epoch days based off the given year, month, and day.
#[inline]
fn iso_date_to_epoch_days(year: i32, month: i32, day: i32) -> i32 {
    // 1. Let resolvedYear be year + floor(month / 12).
    let resolved_year = year + (f64::from(month) / 12_f64).floor() as i32;
    // 2. Let resolvedMonth be month modulo 12.
    let resolved_month = month % 12;

    // 3. Find a time t such that EpochTimeToEpochYear(t) is resolvedYear, EpochTimeToMonthInYear(t) is resolvedMonth, and EpochTimeToDate(t) is 1.
    let year_t = utils::epoch_time_for_year(resolved_year);
    let month_t = utils::epoch_time_for_month_given_year(resolved_month, resolved_year);

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

#[inline]
fn is_valid_time(hour: i32, minute: i32, second: i32, ms: i32, mis: i32, ns: i32) -> bool {
    if !(0..=23).contains(&hour) {
        return false;
    }

    let min_sec = 0..=59;
    if !min_sec.contains(&minute) || !min_sec.contains(&second) {
        return false;
    }

    let sub_second = 0..=999;
    sub_second.contains(&ms) && sub_second.contains(&mis) && sub_second.contains(&ns)
}
