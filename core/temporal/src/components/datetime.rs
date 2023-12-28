//! This module implements `DateTime` any directly related algorithms.

use std::str::FromStr;

use crate::{
    components::{
        calendar::{CalendarProtocol, CalendarSlot},
        Instant,
    },
    iso::{IsoDate, IsoDateSlots, IsoDateTime, IsoTime},
    options::ArithmeticOverflow,
    parser::parse_date_time,
    TemporalError, TemporalResult,
};

/// The native Rust implementation of `Temporal.PlainDateTime`
#[derive(Debug, Default, Clone)]
pub struct DateTime<C: CalendarProtocol> {
    iso: IsoDateTime,
    calendar: CalendarSlot<C>,
}

// ==== Private DateTime API ====

impl<C: CalendarProtocol> DateTime<C> {
    /// Creates a new unchecked `DateTime`.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(date: IsoDate, time: IsoTime, calendar: CalendarSlot<C>) -> Self {
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

    /// Create a new `DateTime` from an `Instant`.
    #[inline]
    pub(crate) fn from_instant(
        instant: &Instant,
        offset: f64,
        calendar: CalendarSlot<C>,
    ) -> TemporalResult<Self> {
        let iso = IsoDateTime::from_epoch_nanos(&instant.nanos, offset)?;
        Ok(Self { iso, calendar })
    }
}

// ==== Public DateTime API ====

impl<C: CalendarProtocol> DateTime<C> {
    /// Creates a new validated `DateTime`.
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
        calendar: CalendarSlot<C>,
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
        self.iso.date()
    }

    /// Returns the inner `IsoTime` value.
    #[inline]
    #[must_use]
    pub fn iso_time(&self) -> IsoTime {
        self.iso.time()
    }

    /// Returns the hour value
    #[inline]
    #[must_use]
    pub fn hours(&self) -> u8 {
        self.iso.time().hour
    }

    /// Returns the minute value
    #[inline]
    #[must_use]
    pub fn minutes(&self) -> u8 {
        self.iso.time().minute
    }

    /// Returns the second value
    #[inline]
    #[must_use]
    pub fn seconds(&self) -> u8 {
        self.iso.time().second
    }

    /// Returns the `millisecond` value
    #[inline]
    #[must_use]
    pub fn milliseconds(&self) -> u16 {
        self.iso.time().millisecond
    }

    /// Returns the `microsecond` value
    #[inline]
    #[must_use]
    pub fn microseconds(&self) -> u16 {
        self.iso.time().microsecond
    }

    /// Returns the `nanosecond` value
    #[inline]
    #[must_use]
    pub fn nanoseconds(&self) -> u16 {
        self.iso.time().nanosecond
    }

    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &CalendarSlot<C> {
        &self.calendar
    }
}

// ==== Trait impls ====

impl<C: CalendarProtocol> FromStr for DateTime<C> {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_record = parse_date_time(s)?;

        let calendar = parse_record.calendar.unwrap_or("iso8601".to_owned());

        let time = if let Some(time) = parse_record.time {
            IsoTime::from_components(
                i32::from(time.hour),
                i32::from(time.minute),
                i32::from(time.second),
                time.fraction,
            )?
        } else {
            IsoTime::default()
        };

        let date = IsoDate::new(
            parse_record.date.year,
            parse_record.date.month,
            parse_record.date.day,
            ArithmeticOverflow::Reject,
        )?;

        Ok(Self::new_unchecked(
            date,
            time,
            CalendarSlot::from_str(&calendar)?,
        ))
    }
}
