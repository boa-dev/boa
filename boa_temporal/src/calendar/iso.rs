//! Implementation of the "iso8601" calendar.

use crate::{
    date::TemporalDate,
    duration::Duration,
    error::TemporalError,
    fields::TemporalFields,
    month_day::TemporalMonthDay,
    options::{ArithmeticOverflow, TemporalUnit},
    utils,
    year_month::TemporalYearMonth,
    TemporalResult,
};
use std::any::Any;

use tinystr::{tinystr, TinyStr4, TinyStr8};

use super::{CalendarDateLike, CalendarFieldsType, CalendarProtocol, CalendarSlot};

use icu_calendar::week::{RelativeUnit, WeekCalculator};

/// This represents the implementation of the `ISO8601`
/// calendar for Temporal.
#[derive(Debug, Clone, Copy)]
pub struct IsoCalendar;

impl CalendarProtocol for IsoCalendar {
    /// Temporal 15.8.2.1 `Temporal.prototype.dateFromFields( fields [, options])` - Supercedes 12.5.4
    ///
    /// This is a basic implementation for an iso8601 calendar's `dateFromFields` method.
    fn date_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        _: &mut dyn Any,
    ) -> TemporalResult<TemporalDate> {
        // NOTE: we are in ISO by default here.
        // a. Perform ? ISOResolveMonth(fields).
        // b. Let result be ? ISODateFromFields(fields, overflow).
        fields.iso_resolve_month()?;

        // 9. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
        TemporalDate::new(
            fields.year().unwrap_or(0),
            fields.month().unwrap_or(0),
            fields.day().unwrap_or(0),
            CalendarSlot::Identifier("iso8601".to_string()),
            overflow,
        )
    }

    /// 12.5.5 `Temporal.Calendar.prototype.yearMonthFromFields ( fields [ , options ] )`
    ///
    /// This is a basic implementation for an iso8601 calendar's `yearMonthFromFields` method.
    fn year_month_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        _: &mut dyn Any,
    ) -> TemporalResult<TemporalYearMonth> {
        // 9. If calendar.[[Identifier]] is "iso8601", then
        // a. Perform ? ISOResolveMonth(fields).
        fields.iso_resolve_month()?;

        // TODO: Do we even need ISOYearMonthFromFields? YearMonth would should pass as a valid date
        // b. Let result be ? ISOYearMonthFromFields(fields, overflow).
        // 10. Return ? CreateTemporalYearMonth(result.[[Year]], result.[[Month]], "iso8601", result.[[ReferenceISODay]]).
        TemporalYearMonth::new(
            fields.year().unwrap_or(0),
            fields.month().unwrap_or(0),
            CalendarSlot::Identifier("iso8601".to_string()),
            overflow,
        )
    }

    /// 12.5.6 `Temporal.Calendar.prototype.monthDayFromFields ( fields [ , options ] )`
    ///
    /// This is a basic implementation for an iso8601 calendar's `monthDayFromFields` method.
    fn month_day_from_fields(
        &self,
        fields: &mut TemporalFields,
        overflow: ArithmeticOverflow,
        _: &mut dyn Any,
    ) -> TemporalResult<TemporalMonthDay> {
        // 8. Perform ? ISOResolveMonth(fields).
        fields.iso_resolve_month()?;

        // TODO: double check error mapping is correct for specifcation/test262.
        // 9. Let result be ? ISOMonthDayFromFields(fields, overflow).
        // 10. Return ? CreateTemporalMonthDay(result.[[Month]], result.[[Day]], "iso8601", result.[[ReferenceISOYear]]).
        TemporalMonthDay::new(
            fields.month().unwrap_or(0),
            fields.month().unwrap_or(0),
            CalendarSlot::Identifier("iso8601".to_string()),
            overflow,
        )
    }

    /// 12.5.7 `Temporal.Calendar.prototype.dateAdd ( date, duration [ , options ] )`
    ///
    /// Below implements the basic implementation for an iso8601 calendar's `dateAdd` method.
    fn date_add(
        &self,
        _date: &TemporalDate,
        _duration: &Duration,
        _overflow: ArithmeticOverflow,
        _: &mut dyn Any,
    ) -> TemporalResult<TemporalDate> {
        // TODO: Not stable on `ICU4X`. Implement once completed.
        Err(TemporalError::range()
            .with_message("feature not implemented.")
            .into())

        // 9. Let result be ? AddISODate(date.[[ISOYear]], date.[[ISOMonth]], date.[[ISODay]], duration.[[Years]], duration.[[Months]], duration.[[Weeks]], balanceResult.[[Days]], overflow).
        // 10. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
    }

    /// 12.5.8 `Temporal.Calendar.prototype.dateUntil ( one, two [ , options ] )`
    ///
    ///  Below implements the basic implementation for an iso8601 calendar's `dateUntil` method.
    fn date_until(
        &self,
        _one: &TemporalDate,
        _two: &TemporalDate,
        _largest_unit: TemporalUnit,
        _: &mut dyn Any,
    ) -> TemporalResult<Duration> {
        // TODO: Not stable on `ICU4X`. Implement once completed.
        Err(TemporalError::range()
            .with_message("Feature not yet implemented.")
            .into())

        // 9. Let result be DifferenceISODate(one.[[ISOYear]], one.[[ISOMonth]], one.[[ISODay]], two.[[ISOYear]], two.[[ISOMonth]], two.[[ISODay]], largestUnit).
        // 10. Return ! CreateTemporalDuration(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], 0, 0, 0, 0, 0, 0).
    }

    /// `Temporal.Calendar.prototype.era( dateLike )` for iso8601 calendar.
    fn era(&self, _: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<Option<TinyStr8>> {
        // Returns undefined on iso8601.
        Ok(None)
    }

    /// `Temporal.Calendar.prototype.eraYear( dateLike )` for iso8601 calendar.
    fn era_year(&self, _: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<Option<i32>> {
        // Returns undefined on iso8601.
        Ok(None)
    }

    /// Returns the `year` for the `Iso` calendar.
    fn year(&self, date_like: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<i32> {
        Ok(date_like.as_iso_date().year())
    }

    /// Returns the `month` for the `Iso` calendar.
    fn month(&self, date_like: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<u8> {
        Ok(date_like.as_iso_date().month())
    }

    /// Returns the `monthCode` for the `Iso` calendar.
    fn month_code(
        &self,
        date_like: &CalendarDateLike,
        _: &mut dyn Any,
    ) -> TemporalResult<TinyStr4> {
        let date = date_like.as_iso_date().as_icu4x()?;
        Ok(date.month().code.0)
    }

    /// Returns the `day` for the `Iso` calendar.
    fn day(&self, date_like: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<u8> {
        Ok(date_like.as_iso_date().day())
    }

    /// Returns the `dayOfWeek` for the `Iso` calendar.
    fn day_of_week(&self, date_like: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<i32> {
        let date = date_like.as_iso_date().as_icu4x()?;
        Ok(date.day_of_week() as i32)
    }

    /// Returns the `dayOfYear` for the `Iso` calendar.
    fn day_of_year(&self, date_like: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<i32> {
        let date = date_like.as_iso_date().as_icu4x()?;
        Ok(i32::from(date.day_of_year_info().day_of_year))
    }

    /// Returns the `weekOfYear` for the `Iso` calendar.
    fn week_of_year(&self, date_like: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<i32> {
        let date = date_like.as_iso_date().as_icu4x()?;

        let week_calculator = WeekCalculator::default();

        let week_of = date
            .week_of_year(&week_calculator)
            .map_err(|err| TemporalError::range().with_message(err.to_string()))?;

        Ok(i32::from(week_of.week))
    }

    /// Returns the `yearOfWeek` for the `Iso` calendar.
    fn year_of_week(&self, date_like: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<i32> {
        let date = date_like.as_iso_date().as_icu4x()?;

        let week_calculator = WeekCalculator::default();

        let week_of = date
            .week_of_year(&week_calculator)
            .map_err(|err| TemporalError::range().with_message(err.to_string()))?;

        // TODO: Reach out and see about RelativeUnit starting at -1
        // Ok(date.year().number - week_of.unit)
        match week_of.unit {
            RelativeUnit::Previous => Ok(date.year().number - 1),
            RelativeUnit::Current => Ok(date.year().number),
            RelativeUnit::Next => Ok(date.year().number + 1),
        }
    }

    /// Returns the `daysInWeek` value for the `Iso` calendar.
    fn days_in_week(&self, _: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<i32> {
        Ok(7)
    }

    /// Returns the `daysInMonth` value for the `Iso` calendar.
    fn days_in_month(&self, date_like: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<i32> {
        let date = date_like.as_iso_date().as_icu4x()?;
        Ok(i32::from(date.days_in_month()))
    }

    /// Returns the `daysInYear` value for the `Iso` calendar.
    fn days_in_year(&self, date_like: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<i32> {
        let date = date_like.as_iso_date().as_icu4x()?;
        Ok(i32::from(date.days_in_year()))
    }

    /// Return the amount of months in an ISO Calendar.
    fn months_in_year(&self, _: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<i32> {
        Ok(12)
    }

    /// Returns whether provided date is in a leap year according to this calendar.
    fn in_leap_year(&self, date_like: &CalendarDateLike, _: &mut dyn Any) -> TemporalResult<bool> {
        // `ICU4X`'s `CalendarArithmetic` is currently private.
        Ok(utils::mathematical_days_in_year(date_like.as_iso_date().year()) == 366)
    }

    // Resolve the fields for the iso calendar.
    fn resolve_fields(
        &self,
        fields: &mut TemporalFields,
        _: CalendarFieldsType,
    ) -> TemporalResult<()> {
        fields.iso_resolve_month()?;
        Ok(())
    }

    /// Returns the ISO field descriptors, which is not called for the iso8601 calendar.
    fn field_descriptors(&self, _: CalendarFieldsType) -> Vec<(String, bool)> {
        // NOTE(potential improvement): look into implementing field descriptors and call
        // ISO like any other calendar?
        // Field descriptors is unused on ISO8601.
        unreachable!()
    }

    /// Returns the `CalendarFieldKeysToIgnore` implementation for ISO.
    fn field_keys_to_ignore(&self, additional_keys: Vec<String>) -> Vec<String> {
        let mut result = Vec::new();
        for key in &additional_keys {
            result.push(key.clone());
            if key.as_str() == "month" {
                result.push("monthCode".to_string());
            } else if key.as_str() == "monthCode" {
                result.push("month".to_string());
            }
        }
        result
    }

    // NOTE: This is currently not a name that is compliant with
    // the Temporal proposal. For debugging purposes only.
    /// Returns the debug name.
    fn identifier(&self, _: &mut dyn Any) -> TemporalResult<String> {
        Ok("iso8601".to_string())
    }
}
