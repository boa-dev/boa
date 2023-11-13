//! Temporal implementation of `DateTime`

use crate::{
    calendar::CalendarSlot,
    iso::{IsoDate, IsoDateSlots, IsoDateTime, IsoTime},
    options::ArithmeticOverflow,
    TemporalResult,
};

/// The `TemporalDateTime` struct.
#[derive(Debug, Default, Clone)]
pub struct TemporalDateTime {
    iso: IsoDateTime,
    calendar: CalendarSlot,
}

// ==== Private DateTime API ====

impl TemporalDateTime {
    /// Creates a new unchecked `TemporalDateTime`.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(date: IsoDate, time: IsoTime, calendar: CalendarSlot) -> Self {
        Self {
            iso: IsoDateTime::new_unchecked(date, time),
            calendar,
        }
    }

    #[inline]
    #[must_use]
    /// Utility function for validating `IsoDate`s
    fn validate_iso(iso: IsoDate) -> bool {
        IsoDateTime::new_unchecked(iso, IsoTime::noon()).is_within_limits()
    }
}

// ==== Public DateTime API ====

impl TemporalDateTime {
    /// Creates a new validated `TemporalDateTime`.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: i32,
        microsecond: i32,
        nanosecond: i32,
        calendar: CalendarSlot,
    ) -> TemporalResult<Self> {
        let iso_date = IsoDate::new(year, month, day, ArithmeticOverflow::Reject)?;
        let iso_time = IsoTime::new(
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            ArithmeticOverflow::Reject,
        )?;
        Ok(Self::new_unchecked(iso_date, iso_time, calendar))
    }

    /// Validates whether ISO date slots are within iso limits at noon.
    #[inline]
    pub fn validate<T: IsoDateSlots>(target: &T) -> bool {
        Self::validate_iso(target.iso_date())
    }

    /// Returns the inner `IsoDate` value.
    #[inline]
    #[must_use]
    pub fn iso_date(&self) -> IsoDate {
        self.iso.iso_date()
    }

    /// Returns the inner `IsoTime` value.
    #[inline]
    #[must_use]
    pub fn iso_time(&self) -> IsoTime {
        self.iso.iso_time()
    }

    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &CalendarSlot {
        &self.calendar
    }
}
