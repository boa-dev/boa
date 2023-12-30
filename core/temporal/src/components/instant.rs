//! An implementation of the Temporal Instant.

use crate::{
    components::duration::TimeDuration,
    options::{DifferenceSettings, TemporalUnit},
    TemporalError, TemporalResult,
};

use num_bigint::BigInt;
use num_traits::ToPrimitive;

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
        let diff = self
            .nanos
            .to_f64()
            .expect("valid instant is representable by f64.")
            - other
                .nanos
                .to_f64()
                .expect("Valid instant nanos is representable by f64.");
        let nanos = diff.rem_euclid(1000f64);
        let micros = (diff / 1000f64).trunc().rem_euclid(1000f64);
        let millis = (diff / 1_000_000f64).trunc().rem_euclid(1000f64);
        let secs = (diff / 1_000_000_000f64).trunc();

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
        self.nanos
            .to_f64()
            .expect("A validated Instant should be within a valid f64")
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
