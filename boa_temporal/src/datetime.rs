//! Temporal implementation of `DateTime`

use crate::{
    calendar::CalendarSlot,
    date::TemporalDate,
    iso::{IsoDate, IsoDateTime, IsoTime},
    month_day::TemporalMonthDay,
    options::ArithmeticOverflow,
    TemporalResult,
};

/// The `TemporalDateTime` struct.
#[derive(Debug, Default, Clone)]
pub struct TemporalDateTime {
    iso: IsoDateTime,
    calendar: CalendarSlot,
}

impl TemporalDateTime {
    /// Creates a new unchecked `TemporalDateTime`.
    pub(crate) fn new_unchecked(date: IsoDate, time: IsoTime, calendar: CalendarSlot) -> Self {
        Self {
            iso: IsoDateTime::new_unchecked(date, time),
            calendar,
        }
    }

    /// Creates a new validated `TemporalDateTime`.
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

    #[inline]
    /// Utility function for validating `IsoDate`s
    fn validate_iso(iso: IsoDate) -> bool {
        IsoDateTime::new_unchecked(iso, IsoTime::noon()).is_within_limits()
    }

    #[inline]
    /// Validates that the provide `TemporalDate` is within iso limits at noon.
    pub fn validate_date(date: &TemporalDate) -> bool {
        Self::validate_iso(date.iso_date())
    }

    #[inline]
    /// Validates that the provided `TemporalMonthDay` is within limits.
    pub fn validate_month_day(month_day: &TemporalMonthDay) -> bool {
        Self::validate_iso(month_day.iso_date())
    }

    /// Returns the inner `IsoDate` value.
    pub fn iso_date(&self) -> IsoDate {
        self.iso.iso_date()
    }

    /// Returns the inner `IsoTime` value.
    pub fn iso_time(&self) -> IsoTime {
        self.iso.iso_time()
    }

    /// Returns the Calendar value.
    pub fn calendar(&self) -> &CalendarSlot {
        &self.calendar
    }
}
