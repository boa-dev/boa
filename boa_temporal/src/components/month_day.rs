//! This module implements `MonthDay` and any directly related algorithms.

use crate::{
    components::calendar::CalendarSlot,
    iso::{IsoDate, IsoDateSlots},
    options::ArithmeticOverflow,
    TemporalResult,
};

/// The native Rust implementation of `Temporal.PlainMonthDay`
#[derive(Debug, Default, Clone)]
pub struct MonthDay {
    iso: IsoDate,
    calendar: CalendarSlot,
}

impl MonthDay {
    /// Creates a new unchecked `MonthDay`
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: CalendarSlot) -> Self {
        Self { iso, calendar }
    }

    #[inline]
    /// Creates a new valid `MonthDay`.
    pub fn new(
        month: i32,
        day: i32,
        calendar: CalendarSlot,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let iso = IsoDate::new(1972, month, day, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
    }

    #[inline]
    #[must_use]
    /// Returns a reference to `MonthDay`'s `CalendarSlot`
    pub fn calendar(&self) -> &CalendarSlot {
        &self.calendar
    }
}

impl IsoDateSlots for MonthDay {
    #[inline]
    /// Returns this structs `IsoDate`.
    fn iso_date(&self) -> IsoDate {
        self.iso
    }
}
