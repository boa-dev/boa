//! This module implements `Time` and any directly related algorithms.

use crate::{iso::IsoTime, options::ArithmeticOverflow, TemporalResult};

/// The native Rust implementation of `Temporal.PlainTime`.
#[derive(Debug, Default, Clone, Copy)]
#[allow(dead_code)]
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
}

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
}
