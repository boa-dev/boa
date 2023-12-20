//! This module implements `YearMonth` and any directly related algorithms.

use std::str::FromStr;

use crate::{
    components::calendar::CalendarSlot,
    iso::{IsoDate, IsoDateSlots},
    options::ArithmeticOverflow,
    TemporalError, TemporalResult,
};

/// The native Rust implementation of `Temporal.YearMonth`.
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

    /// Returns the `year` value for this `YearMonth`.
    #[inline]
    #[must_use]
    pub fn year(&self) -> i32 {
        self.iso.year()
    }

    /// Returns the `month` value for this `YearMonth`.
    #[inline]
    #[must_use]
    pub fn month(&self) -> u8 {
        self.iso.month()
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

impl FromStr for YearMonth {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let record = crate::parser::parse_year_month(s)?;

        let calendar = record.calendar.unwrap_or("iso8601".into());

        Self::new(
            record.date.year,
            record.date.month,
            None,
            CalendarSlot::Identifier(calendar),
            ArithmeticOverflow::Reject,
        )
    }
}
