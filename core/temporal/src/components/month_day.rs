//! This module implements `MonthDay` and any directly related algorithms.

use std::str::FromStr;

use crate::{
    components::calendar::CalendarSlot,
    iso::{IsoDate, IsoDateSlots},
    options::ArithmeticOverflow,
    TemporalError, TemporalResult,
};

use super::calendar::CalendarProtocol;

/// The native Rust implementation of `Temporal.PlainMonthDay`
#[derive(Debug, Default, Clone)]
pub struct MonthDay<C: CalendarProtocol> {
    iso: IsoDate,
    calendar: CalendarSlot<C>,
}

impl<C: CalendarProtocol> MonthDay<C> {
    /// Creates a new unchecked `MonthDay`
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: CalendarSlot<C>) -> Self {
        Self { iso, calendar }
    }

    /// Creates a new valid `MonthDay`.
    #[inline]
    pub fn new(
        month: i32,
        day: i32,
        calendar: CalendarSlot<C>,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let iso = IsoDate::new(1972, month, day, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
    }

    /// Returns the `month` value of `MonthDay`.
    #[inline]
    #[must_use]
    pub fn month(&self) -> u8 {
        self.iso.month
    }

    /// Returns the `day` value of `MonthDay`.
    #[inline]
    #[must_use]
    pub fn day(&self) -> u8 {
        self.iso.day
    }

    /// Returns a reference to `MonthDay`'s `CalendarSlot`
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &CalendarSlot<C> {
        &self.calendar
    }
}

impl<C: CalendarProtocol> IsoDateSlots for MonthDay<C> {
    #[inline]
    /// Returns this structs `IsoDate`.
    fn iso_date(&self) -> IsoDate {
        self.iso
    }
}

impl<C: CalendarProtocol> FromStr for MonthDay<C> {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let record = crate::parser::parse_month_day(s)?;

        let calendar = record.calendar.unwrap_or("iso8601".into());

        Self::new(
            record.date.month,
            record.date.day,
            CalendarSlot::from_str(&calendar)?,
            ArithmeticOverflow::Reject,
        )
    }
}
