//! `TemporalMonthDay`

use crate::{
    calendar::CalendarSlot,
    iso::{IsoDate, IsoDateSlots},
    options::ArithmeticOverflow,
    TemporalResult,
};

/// The `TemporalMonthDay` struct
#[derive(Debug, Default, Clone)]
pub struct TemporalMonthDay {
    iso: IsoDate,
    calendar: CalendarSlot,
}

impl TemporalMonthDay {
    #[inline]
    /// Creates a new unchecked `TemporalMonthDay`
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: CalendarSlot) -> Self {
        Self { iso, calendar }
    }

    #[inline]
    #[must_use]
    /// Creates a new valid `TemporalMonthDay`.
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

impl IsoDateSlots for TemporalMonthDay {
    #[inline]
    /// Returns this structs `IsoDate`.
    fn iso_date(&self) -> IsoDate {
        self.iso
    }
}
