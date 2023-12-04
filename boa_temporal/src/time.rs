//! Temporal Time Representation.

use crate::iso::IsoTime;

/// The Temporal `PlainTime` object.
#[derive(Debug, Default, Clone, Copy)]
#[allow(dead_code)]
pub struct Time {
    iso: IsoTime,
}

// ==== Private API ====

impl Time {
    #[allow(dead_code)]
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

    /// Returns true if a valid `Time`.
    #[allow(dead_code)]
    pub(crate) fn is_valid(&self) -> bool {
        self.iso.is_valid()
    }
}
