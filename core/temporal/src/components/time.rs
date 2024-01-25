//! This module implements `Time` and any directly related algorithms.

use crate::{
    components::{duration::TimeDuration, Duration},
    iso::IsoTime,
    options::{ArithmeticOverflow, TemporalRoundingMode, TemporalUnit},
    utils, TemporalError, TemporalResult,
};

/// The native Rust implementation of `Temporal.PlainTime`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    iso: IsoTime,
}

// ==== Private API ====

impl Time {
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoTime) -> Self {
        Self { iso }
    }

    /// Returns true if a valid `Time`.
    #[allow(dead_code)]
    pub(crate) fn is_valid(&self) -> bool {
        self.iso.is_valid()
    }

    /// Adds a `TimeDuration` to the current `Time`.
    ///
    /// Spec Equivalent: `AddDurationToOrSubtractDurationFromPlainTime` AND `AddTime`.
    pub(crate) fn add_to_time(&self, duration: &TimeDuration) -> Self {
        let (_, result) = IsoTime::balance(
            f64::from(self.hour()) + duration.hours(),
            f64::from(self.minute()) + duration.minutes(),
            f64::from(self.second()) + duration.seconds(),
            f64::from(self.millisecond()) + duration.milliseconds(),
            f64::from(self.microsecond()) + duration.microseconds(),
            f64::from(self.nanosecond()) + duration.nanoseconds(),
        );

        // NOTE (nekevss): IsoTime::balance should never return an invalid `IsoTime`

        Self::new_unchecked(result)
    }
}

// ==== Public API ====

impl Time {
    /// Creates a new `IsoTime` value.
    pub fn new(
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: i32,
        microsecond: i32,
        nanosecond: i32,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let time = IsoTime::new(
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            overflow,
        )?;
        Ok(Self::new_unchecked(time))
    }

    /// Returns the internal `hour` field.
    #[inline]
    #[must_use]
    pub const fn hour(&self) -> u8 {
        self.iso.hour
    }

    /// Returns the internal `minute` field.
    #[inline]
    #[must_use]
    pub const fn minute(&self) -> u8 {
        self.iso.minute
    }

    /// Returns the internal `second` field.
    #[inline]
    #[must_use]
    pub const fn second(&self) -> u8 {
        self.iso.second
    }

    /// Returns the internal `millisecond` field.
    #[inline]
    #[must_use]
    pub const fn millisecond(&self) -> u16 {
        self.iso.millisecond
    }

    /// Returns the internal `microsecond` field.
    #[inline]
    #[must_use]
    pub const fn microsecond(&self) -> u16 {
        self.iso.microsecond
    }

    /// Returns the internal `nanosecond` field.
    #[inline]
    #[must_use]
    pub const fn nanosecond(&self) -> u16 {
        self.iso.nanosecond
    }

    /// Add a `Duration` to the current `Time`.
    pub fn add(&self, duration: &Duration) -> TemporalResult<Self> {
        if !duration.is_time_duration() {
            return Err(TemporalError::range()
                .with_message("DateDuration values cannot be added to `Time`."));
        }
        Ok(self.add_time_duration(duration.time()))
    }

    /// Adds a `TimeDuration` to the current `Time`.
    #[inline]
    #[must_use]
    pub fn add_time_duration(&self, duration: &TimeDuration) -> Self {
        self.add_to_time(duration)
    }

    /// Subtract a `Duration` to the current `Time`.
    pub fn subtract(&self, duration: &Duration) -> TemporalResult<Self> {
        if !duration.is_time_duration() {
            return Err(TemporalError::range()
                .with_message("DateDuration values cannot be added to `Time` component."));
        }
        Ok(self.add_time_duration(duration.time()))
    }

    /// Adds a `TimeDuration` to the current `Time`.
    #[inline]
    #[must_use]
    pub fn subtract_time_duration(&self, duration: &TimeDuration) -> Self {
        self.add_to_time(&duration.neg())
    }

    // TODO (nekevss): optimize and test rounding_increment type (f64 vs. u64).
    /// Rounds the current `Time` according to provided options.
    pub fn round(
        &self,
        smallest_unit: TemporalUnit,
        rounding_increment: Option<f64>,
        rounding_mode: Option<TemporalRoundingMode>,
    ) -> TemporalResult<Self> {
        let increment = utils::to_rounding_increment(rounding_increment)?;
        let mode = rounding_mode.unwrap_or(TemporalRoundingMode::HalfExpand);

        let max = smallest_unit
            .to_maximum_rounding_increment()
            .ok_or_else(|| {
                TemporalError::range().with_message("smallestUnit must be a time value.")
            })?;

        // Safety (nekevss): to_rounding_increment returns a value in the range of a u32.
        utils::validate_temporal_rounding_increment(increment as u32, u64::from(max), false)?;

        let (_, result) = self.iso.round(increment, smallest_unit, mode, None)?;

        Ok(Self::new_unchecked(result))
    }
}

// ==== Test land ====

#[cfg(test)]
mod tests {
    use crate::{components::Duration, iso::IsoTime, options::TemporalUnit};

    use super::Time;

    fn assert_time(result: Time, values: (u8, u8, u8, u16, u16, u16)) {
        assert!(result.hour() == values.0);
        assert!(result.minute() == values.1);
        assert!(result.second() == values.2);
        assert!(result.millisecond() == values.3);
        assert!(result.microsecond() == values.4);
        assert!(result.nanosecond() == values.5);
    }

    #[test]
    fn time_round_millisecond() {
        let base = Time::new_unchecked(IsoTime::new_unchecked(3, 34, 56, 987, 654, 321));

        let result_1 = base
            .round(TemporalUnit::Millisecond, Some(1.0), None)
            .unwrap();
        assert_time(result_1, (3, 34, 56, 988, 0, 0));

        let result_2 = base
            .round(TemporalUnit::Millisecond, Some(2.0), None)
            .unwrap();
        assert_time(result_2, (3, 34, 56, 988, 0, 0));

        let result_3 = base
            .round(TemporalUnit::Millisecond, Some(4.0), None)
            .unwrap();
        assert_time(result_3, (3, 34, 56, 988, 0, 0));

        let result_4 = base
            .round(TemporalUnit::Millisecond, Some(5.0), None)
            .unwrap();
        assert_time(result_4, (3, 34, 56, 990, 0, 0));
    }

    #[test]
    fn time_round_microsecond() {
        let base = Time::new_unchecked(IsoTime::new_unchecked(3, 34, 56, 987, 654, 321));

        let result_1 = base
            .round(TemporalUnit::Microsecond, Some(1.0), None)
            .unwrap();
        assert_time(result_1, (3, 34, 56, 987, 654, 0));

        let result_2 = base
            .round(TemporalUnit::Microsecond, Some(2.0), None)
            .unwrap();
        assert_time(result_2, (3, 34, 56, 987, 654, 0));

        let result_3 = base
            .round(TemporalUnit::Microsecond, Some(4.0), None)
            .unwrap();
        assert_time(result_3, (3, 34, 56, 987, 656, 0));

        let result_4 = base
            .round(TemporalUnit::Microsecond, Some(5.0), None)
            .unwrap();
        assert_time(result_4, (3, 34, 56, 987, 655, 0));
    }

    #[test]
    fn time_round_nanoseconds() {
        let base = Time::new_unchecked(IsoTime::new_unchecked(3, 34, 56, 987, 654, 321));

        let result_1 = base
            .round(TemporalUnit::Nanosecond, Some(1.0), None)
            .unwrap();
        assert_time(result_1, (3, 34, 56, 987, 654, 321));

        let result_2 = base
            .round(TemporalUnit::Nanosecond, Some(2.0), None)
            .unwrap();
        assert_time(result_2, (3, 34, 56, 987, 654, 322));

        let result_3 = base
            .round(TemporalUnit::Nanosecond, Some(4.0), None)
            .unwrap();
        assert_time(result_3, (3, 34, 56, 987, 654, 320));

        let result_4 = base
            .round(TemporalUnit::Nanosecond, Some(5.0), None)
            .unwrap();
        assert_time(result_4, (3, 34, 56, 987, 654, 320));
    }

    #[test]
    fn add_duration_basic() {
        let base = Time::new_unchecked(IsoTime::new_unchecked(15, 23, 30, 123, 456, 789));
        let result = base.add(&"PT16H".parse::<Duration>().unwrap()).unwrap();

        assert_time(result, (7, 23, 30, 123, 456, 789));
    }
}
