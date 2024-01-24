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
    components::duration::DateDuration,
    error::TemporalError,
    options::{ArithmeticOverflow, TemporalRoundingMode, TemporalUnit},
    utils, TemporalResult, NS_PER_DAY,
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

        let max = BigInt::from(crate::NS_MAX_INSTANT + i128::from(NS_PER_DAY));
        let min = BigInt::from(crate::NS_MIN_INSTANT - i128::from(NS_PER_DAY));

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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
        // 2. Set nanosecond to nanosecond modulo 1000.
        let (quotient, nanosecond) = div_mod(nanosecond, 1000f64);
        let microsecond = microsecond + quotient;

        // 3. Set millisecond to millisecond + floor(microsecond / 1000).
        // 4. Set microsecond to microsecond modulo 1000.
        let (quotient, microsecond) = div_mod(microsecond, 1000f64);
        let millisecond = millisecond + quotient;

        // 5. Set second to second + floor(millisecond / 1000).
        // 6. Set millisecond to millisecond modulo 1000.
        let (quotient, millisecond) = div_mod(millisecond, 1000f64);
        let second = second + quotient;

        // 7. Set minute to minute + floor(second / 60).
        // 8. Set second to second modulo 60.
        let (quotient, second) = div_mod(second, 60f64);
        let minute = minute + quotient;

        // 9. Set hour to hour + floor(minute / 60).
        // 10. Set minute to minute modulo 60.
        let (quotient, minute) = div_mod(minute, 60f64);
        let hour = hour + quotient;

        // 11. Let days be floor(hour / 24).
        // 12. Set hour to hour modulo 24.
        let (days, hour) = div_mod(hour, 24f64);

        let time = Self::new_unchecked(
            hour as u8,
            minute as u8,
            second as u8,
            millisecond as u16,
            microsecond as u16,
            nanosecond as u16,
        );

        (days as i32, time)
    }

    // NOTE (nekevss): Specification seemed to be off / not entirely working, so the below was adapted from the
    // temporal-polyfill
    // TODO: DayLengthNS can probably be a u64, but keep as is for now and optimize.
    /// Rounds the current `IsoTime` according to the provided settings.
    pub(crate) fn round(
        &self,
        increment: f64,
        unit: TemporalUnit,
        mode: TemporalRoundingMode,
        day_length_ns: Option<i64>,
    ) -> TemporalResult<(i32, Self)> {
        // 1. Let fractionalSecond be nanosecond × 10-9 + microsecond × 10-6 + millisecond × 10-3 + second.

        let quantity = match unit {
            // 2. If unit is "day", then
            // a. If dayLengthNs is not present, set dayLengthNs to nsPerDay.
            // b. Let quantity be (((((hour × 60 + minute) × 60 + second) × 1000 + millisecond) × 1000 + microsecond) × 1000 + nanosecond) / dayLengthNs.
            // 3. Else if unit is "hour", then
            // a. Let quantity be (fractionalSecond / 60 + minute) / 60 + hour.
            TemporalUnit::Hour | TemporalUnit::Day => {
                u64::from(self.nanosecond)
                    + u64::from(self.microsecond) * 1_000
                    + u64::from(self.millisecond) * 1_000_000
                    + u64::from(self.second) * 1_000_000_000
                    + u64::from(self.minute) * 60 * 1_000_000_000
                    + u64::from(self.hour) * 60 * 60 * 1_000_000_000
            }
            // 4. Else if unit is "minute", then
            // a. Let quantity be fractionalSecond / 60 + minute.
            TemporalUnit::Minute => {
                u64::from(self.nanosecond)
                    + u64::from(self.microsecond) * 1_000
                    + u64::from(self.millisecond) * 1_000_000
                    + u64::from(self.second) * 1_000_000_000
                    + u64::from(self.minute) * 60
            }
            // 5. Else if unit is "second", then
            // a. Let quantity be fractionalSecond.
            TemporalUnit::Second => {
                u64::from(self.nanosecond)
                    + u64::from(self.microsecond) * 1_000
                    + u64::from(self.millisecond) * 1_000_000
                    + u64::from(self.second) * 1_000_000_000
            }
            // 6. Else if unit is "millisecond", then
            // a. Let quantity be nanosecond × 10-6 + microsecond × 10-3 + millisecond.
            TemporalUnit::Millisecond => {
                u64::from(self.nanosecond)
                    + u64::from(self.microsecond) * 1_000
                    + u64::from(self.millisecond) * 1_000_000
            }
            // 7. Else if unit is "microsecond", then
            // a. Let quantity be nanosecond × 10-3 + microsecond.
            TemporalUnit::Microsecond => {
                u64::from(self.nanosecond) + 1_000 * u64::from(self.microsecond)
            }
            // 8. Else,
            // a. Assert: unit is "nanosecond".
            // b. Let quantity be nanosecond.
            TemporalUnit::Nanosecond => u64::from(self.nanosecond),
            _ => {
                return Err(TemporalError::range()
                    .with_message("Invalid temporal unit provided to Time.round."))
            }
        };

        let ns_per_unit = if unit == TemporalUnit::Day {
            day_length_ns.unwrap_or(NS_PER_DAY) as f64
        } else {
            unit.as_nanoseconds().expect("Only valid time values are ")
        };

        // TODO: Verify validity of cast or handle better.
        // 9. Let result be RoundNumberToIncrement(quantity, increment, roundingMode).
        let result =
            utils::round_number_to_increment(quantity as f64, ns_per_unit * increment, mode)
                / ns_per_unit;

        let result = match unit {
            // 10. If unit is "day", then
            // a. Return the Record { [[Days]]: result, [[Hour]]: 0, [[Minute]]: 0, [[Second]]: 0, [[Millisecond]]: 0, [[Microsecond]]: 0, [[Nanosecond]]: 0 }.
            TemporalUnit::Day => (result as i32, IsoTime::default()),
            // 11. If unit is "hour", then
            // a. Return BalanceTime(result, 0, 0, 0, 0, 0).
            TemporalUnit::Hour => IsoTime::balance(result, 0.0, 0.0, 0.0, 0.0, 0.0),
            // 12. If unit is "minute", then
            // a. Return BalanceTime(hour, result, 0, 0, 0, 0).
            TemporalUnit::Minute => {
                IsoTime::balance(f64::from(self.hour), result, 0.0, 0.0, 0.0, 0.0)
            }
            // 13. If unit is "second", then
            // a. Return BalanceTime(hour, minute, result, 0, 0, 0).
            TemporalUnit::Second => IsoTime::balance(
                f64::from(self.hour),
                f64::from(self.minute),
                result,
                0.0,
                0.0,
                0.0,
            ),
            // 14. If unit is "millisecond", then
            // a. Return BalanceTime(hour, minute, second, result, 0, 0).
            TemporalUnit::Millisecond => IsoTime::balance(
                f64::from(self.hour),
                f64::from(self.minute),
                f64::from(self.second),
                result,
                0.0,
                0.0,
            ),
            // 15. If unit is "microsecond", then
            // a. Return BalanceTime(hour, minute, second, millisecond, result, 0).
            TemporalUnit::Microsecond => IsoTime::balance(
                f64::from(self.hour),
                f64::from(self.minute),
                f64::from(self.second),
                f64::from(self.millisecond),
                result,
                0.0,
            ),
            // 16. Assert: unit is "nanosecond".
            // 17. Return BalanceTime(hour, minute, second, millisecond, microsecond, result).
            TemporalUnit::Nanosecond => IsoTime::balance(
                f64::from(self.hour),
                f64::from(self.minute),
                f64::from(self.second),
                f64::from(self.millisecond),
                f64::from(self.microsecond),
                result,
            ),
            _ => unreachable!("Error is thrown in previous match."),
        };

        Ok(result)
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

// ==== `IsoTime` specific utilities ====

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

// NOTE(nekevss): Considering the below: Balance can probably be altered from f64.
#[inline]
fn div_mod(dividend: f64, divisor: f64) -> (f64, f64) {
    (dividend.div_euclid(divisor), dividend.rem_euclid(divisor))
}
