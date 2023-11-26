//! `YearMonth`

use crate::{
    calendar::CalendarSlot,
    iso::{IsoDate, IsoDateSlots},
    options::ArithmeticOverflow,
    TemporalResult,
};

/// The `YearMonth` struct
#[derive(Debug, Default, Clone)]
pub struct YearMonth {
    iso: IsoDate,
    calendar: CalendarSlot,
}

impl YearMonth {
    /// Creates an unvalidated `YearMonth`.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: CalendarSlot) -> Self {
        Self { iso, calendar }
    }

    /// Creates a new valid `YearMonth`.
    #[inline]
    pub fn new(
        year: i32,
        month: i32,
        reference_day: Option<i32>,
        calendar: CalendarSlot,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let day = reference_day.unwrap_or(1);
        let iso = IsoDate::new(year, month, day, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
    }

    #[inline]
    #[must_use]
    /// Returns a reference to `YearMonth`'s `CalendarSlot`
    pub fn calendar(&self) -> &CalendarSlot {
        &self.calendar
    }
}

impl IsoDateSlots for YearMonth {
    #[inline]
    /// Returns this `YearMonth`'s `IsoDate`
    fn iso_date(&self) -> IsoDate {
        self.iso
    }
}
