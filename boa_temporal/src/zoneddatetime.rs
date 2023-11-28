//! The `ZonedDateTime` module.

use num_bigint::BigInt;
use tinystr::TinyStr4;

use crate::{calendar::CalendarSlot, instant::Instant, tz::TimeZoneSlot, TemporalResult};

#[cfg(feature = "context")]
use core::any::Any;

/// Temporal's `ZonedDateTime` object.
#[derive(Debug, Clone)]
pub struct ZonedDateTime {
    instant: Instant,
    calendar: CalendarSlot,
    tz: TimeZoneSlot,
}

// ==== Private API ====

impl ZonedDateTime {
    /// Creates a `ZonedDateTime` without validating the input.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(
        instant: Instant,
        calendar: CalendarSlot,
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

impl ZonedDateTime {
    /// Creates a new valid `ZonedDateTime`.
    #[inline]
    pub fn new(nanos: BigInt, calendar: CalendarSlot, tz: TimeZoneSlot) -> TemporalResult<Self> {
        let instant = Instant::new(nanos)?;
        Ok(Self::new_unchecked(instant, calendar, tz))
    }

    /// Returns the `ZonedDateTime`'s Calendar identifier.
    #[inline]
    #[must_use]
    pub fn calendar_id(&self) -> String {
        // TODO: Implement Identifier method on `CalendarSlot`
        String::from("Not yet implemented.")
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

impl ZonedDateTime {
    /// Returns the `year` value for this `ZonedDateTime`.
    #[inline]
    #[cfg(feature = "context")]
    pub fn year(&self, context: &mut dyn Any) -> TemporalResult<i32> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar
            .year(&crate::calendar::CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `year` value for this `ZonedDateTime`.
    #[inline]
    #[cfg(not(feature = "context"))]
    pub fn year(&self) -> TemporalResult<i32> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, &mut ())?;
        self.calendar
            .year(&crate::calendar::CalendarDateLike::DateTime(dt), &mut ())
    }

    /// Returns the `month` value for this `ZonedDateTime`.
    #[cfg(feature = "context")]
    pub fn month(&self, context: &mut dyn Any) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar
            .month(&crate::calendar::CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `month` value for this `ZonedDateTime`.
    #[inline]
    #[cfg(not(feature = "context"))]
    pub fn month(&self) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, &mut ())?;
        self.calendar
            .month(&crate::calendar::CalendarDateLike::DateTime(dt), &mut ())
    }

    /// Returns the `monthCode` value for this `ZonedDateTime`.
    #[cfg(feature = "context")]
    pub fn month_code(&self, context: &mut dyn Any) -> TemporalResult<TinyStr4> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar
            .month_code(&crate::calendar::CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `monthCode` value for this `ZonedDateTime`.
    #[inline]
    #[cfg(not(feature = "context"))]
    pub fn month_code(&self) -> TemporalResult<TinyStr4> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, &mut ())?;
        self.calendar
            .month_code(&crate::calendar::CalendarDateLike::DateTime(dt), &mut ())
    }

    /// Returns the `day` value for this `ZonedDateTime`.
    #[cfg(feature = "context")]
    pub fn day(&self, context: &mut dyn Any) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        self.calendar
            .day(&crate::calendar::CalendarDateLike::DateTime(dt), context)
    }

    /// Returns the `day` value for this `ZonedDateTime`.
    #[cfg(not(feature = "context"))]
    pub fn day(&self) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, &mut ())?;
        self.calendar
            .day(&crate::calendar::CalendarDateLike::DateTime(dt), &mut ())
    }

    /// Returns the `hour` value for this `ZonedDateTime`.
    #[cfg(feature = "context")]
    pub fn hour(&self, context: &mut dyn Any) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.hours())
    }

    /// Returns the `hour` value for this `ZonedDateTime`.
    #[cfg(not(feature = "context"))]
    pub fn hour(&self) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, &mut ())?;
        Ok(dt.hours())
    }

    /// Returns the `minute` value for this `ZonedDateTime`.
    #[cfg(feature = "context")]
    pub fn minute(&self, context: &mut dyn Any) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.minutes())
    }

    /// Returns the `minute` value for this `ZonedDateTime`.
    #[cfg(not(feature = "context"))]
    pub fn minute(&self) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, &mut ())?;
        Ok(dt.minutes())
    }

    /// Returns the `second` value for this `ZonedDateTime`.
    #[cfg(feature = "context")]
    pub fn second(&self, context: &mut dyn Any) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.seconds())
    }

    /// Returns the `second` value for this `ZonedDateTime`.
    #[cfg(not(feature = "context"))]
    pub fn second(&self) -> TemporalResult<u8> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, &mut ())?;
        Ok(dt.seconds())
    }

    /// Returns the `millisecond` value for this `ZonedDateTime`.
    #[cfg(feature = "context")]
    pub fn millisecond(&self, context: &mut dyn Any) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.milliseconds())
    }

    /// Returns the `millisecond` value for this `ZonedDateTime`.
    #[cfg(not(feature = "context"))]
    pub fn millisecond(&self) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, &mut ())?;
        Ok(dt.milliseconds())
    }

    /// Returns the `microsecond` value for this `ZonedDateTime`.
    #[cfg(feature = "context")]
    pub fn microsecond(&self, context: &mut dyn Any) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.milliseconds())
    }

    /// Returns the `microsecond` value for this `ZonedDateTime`.
    #[cfg(not(feature = "context"))]
    pub fn microsecond(&self) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, &mut ())?;
        Ok(dt.milliseconds())
    }

    /// Returns the `nanosecond` value for this `ZonedDateTime`.
    #[cfg(feature = "context")]
    pub fn nanosecond(&self, context: &mut dyn Any) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, context)?;
        Ok(dt.nanoseconds())
    }

    /// Returns the `nanosecond` value for this `ZonedDateTime`.
    #[cfg(not(feature = "context"))]
    pub fn nanosecond(&self) -> TemporalResult<u16> {
        let dt = self
            .tz
            .get_datetime_for(&self.instant, &self.calendar, &mut ())?;
        Ok(dt.nanoseconds())
    }
}

#[cfg(test)]
#[cfg(not(feature = "context"))]
mod tests {
    use crate::tz::TimeZone;
    use num_bigint::BigInt;

    use super::{CalendarSlot, TimeZoneSlot, ZonedDateTime};

    #[test]
    fn basic_zdt_test() {
        let nov_30_2023_utc = BigInt::from(1701308952000000000i64);

        let zdt = ZonedDateTime::new(
            nov_30_2023_utc.clone(),
            CalendarSlot::Identifier("iso8601".to_owned()),
            TimeZoneSlot::Tz(TimeZone { offset: Some(0) }),
        )
        .unwrap();

        assert_eq!(zdt.year().unwrap(), 2023);
        assert_eq!(zdt.month().unwrap(), 11);
        assert_eq!(zdt.day().unwrap(), 30);
        assert_eq!(zdt.hour().unwrap(), 1);
        assert_eq!(zdt.minute().unwrap(), 49);
        assert_eq!(zdt.second().unwrap(), 12);

        let zdt_minus_five = ZonedDateTime::new(
            nov_30_2023_utc,
            CalendarSlot::Identifier("iso8601".to_owned()),
            TimeZoneSlot::Tz(TimeZone { offset: Some(-300) }),
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
