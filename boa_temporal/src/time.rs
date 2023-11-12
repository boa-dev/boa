//! Temporal Time Representation.

use crate::iso::IsoTime;

/// The Temporal `PlainTime` object.
#[derive(Debug, Default, Clone, Copy)]
pub struct TemporalTime {
    iso: IsoTime,
}

// ==== Private API ====

impl TemporalTime {
    pub(crate) fn new_unchecked(
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: i32,
        microsecond: i32,
        nanosecond: i32,
    ) -> Self {
        Self {
            iso: IsoTime::new_unchecked(hour, minute, second, millisecond, microsecond, nanosecond),
        }
    }

    /// Checks if the time is a valid `TemporalTime`
    pub(crate) fn is_valid(&self) -> bool {
        self.iso.is_valid()
    }
}
