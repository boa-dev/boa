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

use super::tz::TzProtocol;

/// The native Rust implementation of `Temporal.ZonedDateTime`.
#[derive(Debug, Clone)]
pub struct ZonedDateTime<C: CalendarProtocol, Z: TzProtocol> {
    instant: Instant,
    calendar: CalendarSlot<C>,
    tz: TimeZoneSlot<Z>,
}

// ==== Private API ====

impl<C: CalendarProtocol, Z: TzProtocol> ZonedDateTime<C, Z> {
    /// Creates a `ZonedDateTime` without validating the input.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(
        instant: Instant,
        calendar: CalendarSlot<C>,
        tz: TimeZoneSlot<Z>,
    ) -> Self {
        Self {
            instant,
            calendar,
            tz,
        }
    }
}

// ==== Public API ====

impl<C: CalendarProtocol, Z: TzProtocol> ZonedDateTime<C, Z> {
    /// Creates a new valid `ZonedDateTime`.
    #[inline]
    pub fn new(
        nanos: BigInt,
        calendar: CalendarSlot<C>,
        tz: TimeZoneSlot<Z>,
    ) -> TemporalResult<Self> {
        let instant = Instant::new(nanos)?;
        Ok(Self::new_unchecked(instant, calendar, tz))
    }

    /// Returns `ZonedDateTime`'s Calendar.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &CalendarSlot<C> {
        &self.calendar
    }

    /// Returns `ZonedDateTime`'s `TimeZone` slot.
    #[inline]
    #[must_use]
    pub fn tz(&self) -> &TimeZoneSlot<Z> {
        &self.tz
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

impl<C, Z: TzProtocol> ZonedDateTime<C, Z>
where
    C: CalendarProtocol<Context = Z::Context>,
{
    /// Returns the `year` value for this `ZonedDateTime`.
    #[inline]
    pub fn contextual_year(&self, context: &mut C::Context) -> TemporalResult<i32> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar.year(&CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `month` value for this `ZonedDateTime`.
    pub fn contextual_month(&self, context: &mut C::Context) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar
            .month(&CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `monthCode` value for this `ZonedDateTime`.
    pub fn contextual_month_code(&self, context: &mut C::Context) -> TemporalResult<TinyStr4> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar
            .month_code(&CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `day` value for this `ZonedDateTime`.
    pub fn contextual_day(&self, context: &mut C::Context) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar.day(&CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `hour` value for this `ZonedDateTime`.
    pub fn contextual_hour(&self, context: &mut C::Context) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.hour())
    }

    /// Returns the `minute` value for this `ZonedDateTime`.
    pub fn contextual_minute(&self, context: &mut C::Context) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.minute())
    }

    /// Returns the `second` value for this `ZonedDateTime`.
    pub fn contextual_second(&self, context: &mut C::Context) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.second())
    }

    /// Returns the `millisecond` value for this `ZonedDateTime`.
    pub fn contextual_millisecond(&self, context: &mut C::Context) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.millisecond())
    }

    /// Returns the `microsecond` value for this `ZonedDateTime`.
    pub fn contextual_microsecond(&self, context: &mut C::Context) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.millisecond())
    }

    /// Returns the `nanosecond` value for this `ZonedDateTime`.
    pub fn contextual_nanosecond(&self, context: &mut C::Context) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.nanosecond())
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

        let zdt = ZonedDateTime::<(), ()>::new(
            nov_30_2023_utc.clone(),
            CalendarSlot::from_str("iso8601").unwrap(),
            TimeZoneSlot::Tz(TimeZone {
                iana: None,
                offset: Some(0),
            }),
        )
        .unwrap();

        assert_eq!(zdt.contextual_year(&mut ()).unwrap(), 2023);
        assert_eq!(zdt.contextual_month(&mut ()).unwrap(), 11);
        assert_eq!(zdt.contextual_day(&mut ()).unwrap(), 30);
        assert_eq!(zdt.contextual_hour(&mut ()).unwrap(), 1);
        assert_eq!(zdt.contextual_minute(&mut ()).unwrap(), 49);
        assert_eq!(zdt.contextual_second(&mut ()).unwrap(), 12);

        let zdt_minus_five = ZonedDateTime::<(), ()>::new(
            nov_30_2023_utc,
            CalendarSlot::from_str("iso8601").unwrap(),
            TimeZoneSlot::Tz(TimeZone {
                iana: None,
                offset: Some(-300),
            }),
        )
        .unwrap();

        assert_eq!(zdt_minus_five.contextual_year(&mut ()).unwrap(), 2023);
        assert_eq!(zdt_minus_five.contextual_month(&mut ()).unwrap(), 11);
        assert_eq!(zdt_minus_five.contextual_day(&mut ()).unwrap(), 29);
        assert_eq!(zdt_minus_five.contextual_hour(&mut ()).unwrap(), 20);
        assert_eq!(zdt_minus_five.contextual_minute(&mut ()).unwrap(), 49);
        assert_eq!(zdt_minus_five.contextual_second(&mut ()).unwrap(), 12);
    }
}
