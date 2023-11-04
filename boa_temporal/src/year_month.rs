//! TemporalYearMonth

use crate::{calendar::CalendarSlot, iso::IsoDate, options::ArithmeticOverflow, TemporalResult};

/// The `TemporalYearMonth` struct
#[derive(Debug, Default, Clone)]
pub struct TemporalYearMonth {
    iso: IsoDate,
    calendar: CalendarSlot,
}

impl TemporalYearMonth {
    #[inline]
    /// Creates an unvalidated `TemporalYearMonth`.
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: CalendarSlot) -> Self {
        Self { iso, calendar }
    }

    #[inline]
    /// Creates a new valid `TemporalYearMonth`.
    pub fn new(
        year: i32,
        month: i32,
        calendar: CalendarSlot,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let iso = IsoDate::new_year_month(year, month, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
    }

    #[inline]
    /// Returns a reference to this `YearMonth`'s `IsoDate`
    pub fn iso_date(&self) -> IsoDate {
        self.iso
    }

    #[inline]
    /// Returns a reference to `YearMonth`'s `CalendarSlot`
    pub fn calendar(&self) -> &CalendarSlot {
        &self.calendar
    }
}
