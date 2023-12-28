//! This module implements the Temporal `TimeZone` and components.

use std::any::Any;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::{
    components::{calendar::CalendarSlot, DateTime, Instant},
    TemporalError, TemporalResult,
};

use super::calendar::CalendarProtocol;

/// Any object that implements the `TzProtocol` must implement the below methods/properties.
pub const TIME_ZONE_PROPERTIES: [&str; 3] =
    ["getOffsetNanosecondsFor", "getPossibleInstantsFor", "id"];

/// A clonable `TzProtocol`
pub trait TzProtocolClone {
    /// Clones the current `TimeZoneProtocol`.
    fn clone_box(&self) -> Box<dyn TzProtocol>;
}

impl<P> TzProtocolClone for P
where
    P: 'static + TzProtocol + Clone,
{
    fn clone_box(&self) -> Box<dyn TzProtocol> {
        Box::new(self.clone())
    }
}

/// The Time Zone Protocol that must be implemented for time zones.
pub trait TzProtocol: TzProtocolClone {
    /// Get the Offset nanoseconds for this `TimeZone`
    fn get_offset_nanos_for(&self, context: &mut dyn Any) -> TemporalResult<BigInt>;
    /// Get the possible Instant for this `TimeZone`
    fn get_possible_instant_for(&self, context: &mut dyn Any) -> TemporalResult<Vec<Instant>>; // TODO: Implement Instant
    /// Get the `TimeZone`'s identifier.
    fn id(&self, context: &mut dyn Any) -> String;
}

/// A Temporal `TimeZone`.
#[derive(Debug, Clone)]
#[allow(unused)]
pub struct TimeZone {
    pub(crate) iana: Option<String>, // TODO: ICU4X IANA TimeZone support.
    pub(crate) offset: Option<i16>,
}

/// The `TimeZoneSlot` represents a `[[TimeZone]]` internal slot value.
pub enum TimeZoneSlot {
    /// A native `TimeZone` representation.
    Tz(TimeZone),
    /// A Custom `TimeZone` that implements the `TzProtocol`.
    Protocol(Box<dyn TzProtocol>),
}

impl core::fmt::Debug for TimeZoneSlot {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tz(tz) => write!(f, "{tz:?}"),
            Self::Protocol(_) => write!(f, "TzProtocol"),
        }
    }
}

impl Clone for TimeZoneSlot {
    fn clone(&self) -> Self {
        match self {
            Self::Tz(tz) => Self::Tz(tz.clone()),
            Self::Protocol(p) => Self::Protocol(p.clone_box()),
        }
    }
}

impl TimeZoneSlot {
    pub(crate) fn get_datetime_for<C: CalendarProtocol>(
        &self,
        instant: &Instant,
        calendar: &CalendarSlot<C>,
        context: &mut dyn Any,
    ) -> TemporalResult<DateTime<C>> {
        let nanos = self.get_offset_nanos_for(context)?;
        DateTime::from_instant(instant, nanos.to_f64().unwrap_or(0.0), calendar.clone())
    }
}

impl TzProtocol for TimeZoneSlot {
    fn get_offset_nanos_for(&self, context: &mut dyn Any) -> TemporalResult<BigInt> {
        // 1. Let timeZone be the this value.
        // 2. Perform ? RequireInternalSlot(timeZone, [[InitializedTemporalTimeZone]]).
        // 3. Set instant to ? ToTemporalInstant(instant).
        match self {
            Self::Tz(tz) => {
                // 4. If timeZone.[[OffsetMinutes]] is not empty, return 𝔽(timeZone.[[OffsetMinutes]] × (60 × 10^9)).
                if let Some(offset) = &tz.offset {
                    return Ok(BigInt::from(i64::from(*offset) * 60_000_000_000i64));
                }
                // 5. Return 𝔽(GetNamedTimeZoneOffsetNanoseconds(timeZone.[[Identifier]], instant.[[Nanoseconds]])).
                Err(TemporalError::range().with_message("IANA TimeZone names not yet implemented."))
            }
            // Call any custom implemented TimeZone.
            Self::Protocol(p) => p.get_offset_nanos_for(context),
        }
    }

    fn get_possible_instant_for(&self, _context: &mut dyn Any) -> TemporalResult<Vec<Instant>> {
        Err(TemporalError::general("Not yet implemented."))
    }

    fn id(&self, context: &mut dyn Any) -> String {
        match self {
            Self::Tz(_) => todo!("implement tz.to_string"),
            Self::Protocol(tz) => tz.id(context),
        }
    }
}
