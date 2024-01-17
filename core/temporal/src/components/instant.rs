//! An implementation of the Temporal Instant.

use crate::{
    components::{Duration, duration::TimeDuration},
    options::{DifferenceSettings, TemporalUnit, TemporalRoundingMode},
    TemporalError, TemporalResult,
};

use num_bigint::BigInt;
use num_traits::{ToPrimitive, FromPrimitive};

const NANOSECONDS_PER_SECOND: f64 = 1e9;
const NANOSECONDS_PER_MINUTE: f64 = 60f64 * NANOSECONDS_PER_SECOND;
const NANOSECONDS_PER_HOUR: f64 = 60f64 * NANOSECONDS_PER_MINUTE;

/// The native Rust implementation of `Temporal.Instant`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    pub(crate) nanos: BigInt,
}

// ==== Private API ====

impl Instant {
    // TODO: Add test for `diff_instant`.
    // NOTE(nekevss): As the below is internal, op will be left as a boolean
    // with a `since` op being true and `until` being false.
    /// Internal operation to handle `since` and `until` difference ops.
    #[allow(unused)]
    pub(crate) fn diff_instant(
        &self,
        op: bool,
        other: &Self,
        settings: DifferenceSettings, // TODO: Determine DifferenceSettings fate -> is there a better way to approach this.
    ) -> TemporalResult<TimeDuration> {
        // Diff the Instant and determine its component values.
        let diff = self.to_f64() - other.to_f64();
        let nanos = diff.rem_euclid(1000f64);
        let micros = (diff / 1000f64).trunc().rem_euclid(1000f64);
        let millis = (diff / 1_000_000f64).trunc().rem_euclid(1000f64);
        let secs = (diff / NANOSECONDS_PER_SECOND).trunc();

        // Handle the settings provided to `diff_instant`
        if settings.smallest_unit == TemporalUnit::Nanosecond {
            let (_, result) = TimeDuration::new_unchecked(0f64, 0f64, secs, millis, micros, nanos)
                .balance(0f64, settings.largest_unit)?;
            return Ok(result);
        }

        let (round_result, _) = TimeDuration::new(0f64, 0f64, secs, millis, micros, nanos)?.round(
            settings.rounding_increment,
            settings.smallest_unit,
            settings.rounding_mode,
        )?;
        let (_, result) = round_result.balance(0f64, settings.largest_unit)?;
        Ok(result)
    }

    /// Adds a `TimeDuration` to the current `Instant`.
    pub(crate) fn add_to_instant(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        let result = self.epoch_nanoseconds() + duration.nanoseconds + (duration.microseconds * 1000f64) + (duration.milliseconds * 1_000_000f64) + (duration.seconds * NANOSECONDS_PER_SECOND)+ (duration.minutes * NANOSECONDS_PER_MINUTE) + (duration.hours * NANOSECONDS_PER_HOUR);
        let nanos = BigInt::from_f64(result).ok_or_else(|| TemporalError::range().with_message("Duration added to instant exceeded valid range."))?;
        Self::new(nanos)
    }

    /// Utility for converting `Instant` to f64.
    ///
    /// # Panics
    ///
    /// This function will panic if called on an invalid `Instant`.
    pub(crate) fn to_f64(&self) -> f64 {
        self.nanos.to_f64().expect("A valid instant is representable by f64.")
    }
}

// ==== Public API ====

impl Instant {
    /// Create a new validated `Instant`.
    #[inline]
    pub fn new(nanos: BigInt) -> TemporalResult<Self> {
        if !is_valid_epoch_nanos(&nanos) {
            return Err(TemporalError::range()
                .with_message("Instant nanoseconds are not within a valid epoch range."));
        }
        Ok(Self { nanos })
    }

    /// Adds a `Duration` to the current `Instant`, returning an error if the `Duration`
    /// contains a `DateDuration`.
    #[inline]
    pub fn add(&self, duration: Duration) -> TemporalResult<Self> {
        if !duration.is_time_duration() {
            return Err(TemporalError::range().with_message("DateDuration values cannot be added to instant."))
        }
        self.add_time_duration(duration.time())
    }

    /// Adds a `TimeDuration` to `Instant`.
    #[inline]
    pub fn add_time_duration(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        self.add_to_instant(duration)
    }

    /// Subtract a `Duration` to the current `Instant`, returning an error if the `Duration`
    /// contains a `DateDuration`.
    #[inline]
    pub fn subtract(&self, duration: Duration) -> TemporalResult<Self> {
        if !duration.is_time_duration() {
            return Err(TemporalError::range().with_message("DateDuration values cannot be added to instant."))
        }
        self.subtract_time_duration(duration.time())
    }

    /// Subtracts a `TimeDuration` to `Instant`.
    #[inline]
    pub fn subtract_time_duration(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        self.add_to_instant(&duration.neg())
    }

    // TODO: Address DifferenceSettings.
    /// Returns a `TimeDuration` representing the duration since provided `Instant`
    pub fn since(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<TimeDuration> {
        self.diff_instant(true, other, settings)
    }

    // TODO: Address DifferenceSettings.
    /// Returns a `TimeDuration` representing the duration until provided `Instant`
    pub fn until(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<TimeDuration> {
        self.diff_instant(false, other, settings)
    }

    /// Returns an `Instant` by rounding the current `Instant` according to the provided settings.
    pub fn round(&self, _rounding_increment: u64, _smallest_unit: TemporalUnit, _rounding_mode: TemporalRoundingMode) -> TemporalResult<Self> {
        Err(TemporalError::range().with_message("Instant.round is not yet implemented."))
    }

    /// Returns the `epochSeconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_seconds(&self) -> f64 {
        (&self.nanos / BigInt::from(1_000_000_000))
            .to_f64()
            .expect("A validated Instant should be within a valid f64")
            .floor()
    }

    /// Returns the `epochMilliseconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_milliseconds(&self) -> f64 {
        (&self.nanos / BigInt::from(1_000_000))
            .to_f64()
            .expect("A validated Instant should be within a valid f64")
            .floor()
    }

    /// Returns the `epochMicroseconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_microseconds(&self) -> f64 {
        (&self.nanos / BigInt::from(1_000))
            .to_f64()
            .expect("A validated Instant should be within a valid f64")
            .floor()
    }

    /// Returns the `epochNanoseconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_nanoseconds(&self) -> f64 {
        self.to_f64()
    }
}

// ==== Utility Functions ====

/// Utility for determining if the nanos are within a valid range.
#[inline]
#[must_use]
pub(crate) fn is_valid_epoch_nanos(nanos: &BigInt) -> bool {
    nanos <= &BigInt::from(crate::NS_MAX_INSTANT) && nanos >= &BigInt::from(crate::NS_MIN_INSTANT)
}

// ==== Instant Tests ====

#[cfg(test)]
mod tests {
    use crate::{components::Instant, NS_MAX_INSTANT, NS_MIN_INSTANT};
    use num_bigint::BigInt;
    use num_traits::ToPrimitive;

    #[test]
    #[allow(clippy::float_cmp)]
    fn max_and_minimum_instant_bounds() {
        // This test is primarily to assert that the `expect` in the epoch methods is
        // valid, i.e., a valid instant is within the range of an f64.
        let max = BigInt::from(NS_MAX_INSTANT);
        let min = BigInt::from(NS_MIN_INSTANT);
        let max_instant = Instant::new(max.clone()).unwrap();
        let min_instant = Instant::new(min.clone()).unwrap();

        assert_eq!(max_instant.epoch_nanoseconds(), max.to_f64().unwrap());
        assert_eq!(min_instant.epoch_nanoseconds(), min.to_f64().unwrap());

        let max_plus_one = BigInt::from(NS_MAX_INSTANT + 1);
        let min_minus_one = BigInt::from(NS_MIN_INSTANT - 1);

        assert!(Instant::new(max_plus_one).is_err());
        assert!(Instant::new(min_minus_one).is_err());
    }
}
