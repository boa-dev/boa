//! This module implements `ZonedDateTime` and any directly related algorithms.

use num_bigint::BigInt;
use tinystr::TinyStr4;

use crate::{
    components::{
        calendar::{CalendarDateLike, CalendarProtocol, CalendarSlot},
        tz::TimeZoneSlot,
        Instant,
    },
    TemporalResult,
};

use core::any::Any;

/// The native Rust implementation of `Temporal.ZonedDateTime`.
#[derive(Debug, Clone)]
pub struct ZonedDateTime<C: CalendarProtocol> {
    instant: Instant,
    calendar: CalendarSlot<C>,
    tz: TimeZoneSlot,
}

// ==== Private API ====

impl<C: CalendarProtocol> ZonedDateTime<C> {
    /// Creates a `ZonedDateTime` without validating the input.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(
        instant: Instant,
        calendar: CalendarSlot<C>,
        tz: TimeZoneSlot,
    ) -> Self {
        Self {
            instant,
            calendar,
            tz,
        }
    }
}

// ==== Public API ====

impl<C: CalendarProtocol> ZonedDateTime<C> {
    /// Creates a new valid `ZonedDateTime`.
    #[inline]
    pub fn new(nanos: BigInt, calendar: CalendarSlot<C>, tz: TimeZoneSlot) -> TemporalResult<Self> {
        let instant = Instant::new(nanos)?;
        Ok(Self::new_unchecked(instant, calendar, tz))
    }

    /// Returns the `ZonedDateTime`'s Calendar identifier.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &CalendarSlot<C> {
        &self.calendar
    }

    /// Returns the `epochSeconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_seconds(&self) -> f64 {
        self.instant.epoch_seconds()
    }

    /// Returns the `epochMilliseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_milliseconds(&self) -> f64 {
        self.instant.epoch_milliseconds()
    }

    /// Returns the `epochMicroseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_microseconds(&self) -> f64 {
        self.instant.epoch_microseconds()
    }

    /// Returns the `epochNanoseconds` value of this `ZonedDateTime`.
    #[must_use]
    pub fn epoch_nanoseconds(&self) -> f64 {
        self.instant.epoch_nanoseconds()
    }
}

// ==== Context based API ====

impl<C: CalendarProtocol> ZonedDateTime<C> {
    /// Returns the `year` value for this `ZonedDateTime`.
    #[inline]
    pub fn contextual_year(&self, context: &mut dyn Any) -> TemporalResult<i32> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar.year(&CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `year` value for this `ZonedDateTime`.
    #[inline]
    pub fn year(&self) -> TemporalResult<i32> {
        self.contextual_year(&mut ())
    }

    /// Returns the `month` value for this `ZonedDateTime`.
    pub fn contextual_month(&self, context: &mut dyn Any) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar
            .month(&CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `month` value for this `ZonedDateTime`.
    #[inline]
    pub fn month(&self) -> TemporalResult<u8> {
        self.contextual_month(&mut ())
    }

    /// Returns the `monthCode` value for this `ZonedDateTime`.
    pub fn contextual_month_code(&self, context: &mut dyn Any) -> TemporalResult<TinyStr4> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar
            .month_code(&CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `monthCode` value for this `ZonedDateTime`.
    #[inline]
    pub fn month_code(&self) -> TemporalResult<TinyStr4> {
        self.contextual_month_code(&mut ())
    }

    /// Returns the `day` value for this `ZonedDateTime`.
    pub fn contextual_day(&self, context: &mut dyn Any) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar.day(&CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `day` value for this `ZonedDateTime`.
    pub fn day(&self) -> TemporalResult<u8> {
        self.contextual_day(&mut ())
    }

    /// Returns the `hour` value for this `ZonedDateTime`.
    pub fn contextual_hour(&self, context: &mut dyn Any) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.hours())
    }

    /// Returns the `hour` value for this `ZonedDateTime`.
    pub fn hour(&self) -> TemporalResult<u8> {
        self.contextual_hour(&mut ())
    }

    /// Returns the `minute` value for this `ZonedDateTime`.
    pub fn contextual_minute(&self, context: &mut dyn Any) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.minutes())
    }

    /// Returns the `minute` value for this `ZonedDateTime`.
    pub fn minute(&self) -> TemporalResult<u8> {
        self.contextual_minute(&mut ())
    }

    /// Returns the `second` value for this `ZonedDateTime`.
    pub fn contextual_second(&self, context: &mut dyn Any) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.seconds())
    }

    /// Returns the `second` value for this `ZonedDateTime`.
    pub fn second(&self) -> TemporalResult<u8> {
        self.contextual_second(&mut ())
    }

    /// Returns the `millisecond` value for this `ZonedDateTime`.
    pub fn contextual_millisecond(&self, context: &mut dyn Any) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.milliseconds())
    }

    /// Returns the `millisecond` value for this `ZonedDateTime`.
    pub fn millisecond(&self) -> TemporalResult<u16> {
        self.contextual_millisecond(&mut ())
    }

    /// Returns the `microsecond` value for this `ZonedDateTime`.
    pub fn contextual_microsecond(&self, context: &mut dyn Any) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.milliseconds())
    }

    /// Returns the `microsecond` value for this `ZonedDateTime`.
    pub fn microsecond(&self) -> TemporalResult<u16> {
        self.contextual_microsecond(&mut ())
    }

    /// Returns the `nanosecond` value for this `ZonedDateTime`.
    pub fn contextual_nanosecond(&self, context: &mut dyn Any) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.nanoseconds())
    }

    /// Returns the `nanosecond` value for this `ZonedDateTime`.
    pub fn nanosecond(&self) -> TemporalResult<u16> {
        self.contextual_nanosecond(&mut ())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::components::tz::TimeZone;
    use num_bigint::BigInt;

    use super::{CalendarSlot, TimeZoneSlot, ZonedDateTime};

    #[test]
    fn basic_zdt_test() {
        let nov_30_2023_utc = BigInt::from(1_701_308_952_000_000_000i64);

        let zdt = ZonedDateTime::<()>::new(
            nov_30_2023_utc.clone(),
            CalendarSlot::from_str("iso8601").unwrap(),
            TimeZoneSlot::Tz(TimeZone {
                iana: None,
                offset: Some(0),
            }),
        )
        .unwrap();

        assert_eq!(zdt.year().unwrap(), 2023);
        assert_eq!(zdt.month().unwrap(), 11);
        assert_eq!(zdt.day().unwrap(), 30);
        assert_eq!(zdt.hour().unwrap(), 1);
        assert_eq!(zdt.minute().unwrap(), 49);
        assert_eq!(zdt.second().unwrap(), 12);

        let zdt_minus_five = ZonedDateTime::<()>::new(
            nov_30_2023_utc,
            CalendarSlot::from_str("iso8601").unwrap(),
            TimeZoneSlot::Tz(TimeZone {
                iana: None,
                offset: Some(-300),
            }),
        )
        .unwrap();

        assert_eq!(zdt_minus_five.year().unwrap(), 2023);
        assert_eq!(zdt_minus_five.month().unwrap(), 11);
        assert_eq!(zdt_minus_five.day().unwrap(), 29);
        assert_eq!(zdt_minus_five.hour().unwrap(), 20);
        assert_eq!(zdt_minus_five.minute().unwrap(), 49);
        assert_eq!(zdt_minus_five.second().unwrap(), 12);
    }
}
