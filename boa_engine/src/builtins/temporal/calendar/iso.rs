//! Implementation of the "iso8601" `BuiltinCalendar`.

use crate::{
    builtins::temporal::{
        self,
        date_equations::mathematical_days_in_year,
        duration::DurationRecord,
        options::{ArithmeticOverflow, TemporalUnit},
        plain_date::iso::IsoDateRecord,
    },
    js_string, JsNativeError, JsResult, JsString,
};

use super::{BuiltinCalendar, FieldsType};

use icu_calendar::{
    iso::Iso,
    week::{RelativeUnit, WeekCalculator},
    Calendar, Date,
};

pub(crate) struct IsoCalendar;

impl BuiltinCalendar for IsoCalendar {
    /// Temporal 15.8.2.1 `Temporal.prototype.dateFromFields( fields [, options])` - Supercedes 12.5.4
    ///
    /// This is a basic implementation for an iso8601 calendar's `dateFromFields` method.
    fn date_from_fields(
        &self,
        fields: &mut temporal::TemporalFields,
        overflow: ArithmeticOverflow,
    ) -> JsResult<IsoDateRecord> {
        // NOTE: we are in ISO by default here.
        // a. Perform ? ISOResolveMonth(fields).
        // b. Let result be ? ISODateFromFields(fields, overflow).
        fields.iso_resolve_month()?;

        // Extra: handle reulating/overflow until implemented on `icu_calendar`
        fields.regulate(overflow)?;

        let date = Date::try_new_iso_date(
            fields.year().unwrap_or(0),
            fields.month().unwrap_or(250) as u8,
            fields.day().unwrap_or(250) as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        // 9. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
        Ok(IsoDateRecord::from_date_iso(date))
    }

    /// 12.5.5 `Temporal.Calendar.prototype.yearMonthFromFields ( fields [ , options ] )`
    ///
    /// This is a basic implementation for an iso8601 calendar's `yearMonthFromFields` method.
    fn year_month_from_fields(
        &self,
        fields: &mut temporal::TemporalFields,
        overflow: ArithmeticOverflow,
    ) -> JsResult<IsoDateRecord> {
        // 9. If calendar.[[Identifier]] is "iso8601", then
        // a. Perform ? ISOResolveMonth(fields).
        fields.iso_resolve_month()?;

        // b. Let result be ? ISOYearMonthFromFields(fields, overflow).
        fields.regulate_year_month(overflow);

        let result = Date::try_new_iso_date(
            fields.year().unwrap_or(0),
            fields.month().unwrap_or(250) as u8,
            fields.day().unwrap_or(20) as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        // 10. Return ? CreateTemporalYearMonth(result.[[Year]], result.[[Month]], "iso8601", result.[[ReferenceISODay]]).
        Ok(IsoDateRecord::from_date_iso(result))
    }

    /// 12.5.6 `Temporal.Calendar.prototype.monthDayFromFields ( fields [ , options ] )`
    ///
    /// This is a basic implementation for an iso8601 calendar's `monthDayFromFields` method.
    fn month_day_from_fields(
        &self,
        fields: &mut temporal::TemporalFields,
        overflow: ArithmeticOverflow,
    ) -> JsResult<IsoDateRecord> {
        // 8. Perform ? ISOResolveMonth(fields).
        fields.iso_resolve_month()?;

        fields.regulate(overflow)?;

        // TODO: double check error mapping is correct for specifcation/test262.
        // 9. Let result be ? ISOMonthDayFromFields(fields, overflow).
        let result = Date::try_new_iso_date(
            fields.year().unwrap_or(1972),
            fields.month().unwrap_or(250) as u8,
            fields.day().unwrap_or(250) as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        // 10. Return ? CreateTemporalMonthDay(result.[[Month]], result.[[Day]], "iso8601", result.[[ReferenceISOYear]]).
        Ok(IsoDateRecord::from_date_iso(result))
    }

    /// 12.5.7 `Temporal.Calendar.prototype.dateAdd ( date, duration [ , options ] )`
    ///
    /// Below implements the basic implementation for an iso8601 calendar's `dateAdd` method.
    fn date_add(
        &self,
        _date: &temporal::PlainDate,
        _duration: &DurationRecord,
        _overflow: ArithmeticOverflow,
    ) -> JsResult<IsoDateRecord> {
        // TODO: Not stable on `ICU4X`. Implement once completed.
        Err(JsNativeError::range()
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
        _one: &temporal::PlainDate,
        _two: &temporal::PlainDate,
        _largest_unit: TemporalUnit,
    ) -> JsResult<DurationRecord> {
        // TODO: Not stable on `ICU4X`. Implement once completed.
        Err(JsNativeError::range()
            .with_message("Feature not yet implemented.")
            .into())

        // 9. Let result be DifferenceISODate(one.[[ISOYear]], one.[[ISOMonth]], one.[[ISODay]], two.[[ISOYear]], two.[[ISOMonth]], two.[[ISODay]], largestUnit).
        // 10. Return ! CreateTemporalDuration(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], 0, 0, 0, 0, 0, 0).
    }

    /// `Temporal.Calendar.prototype.era( dateLike )` for iso8601 calendar.
    fn era(&self, _: &IsoDateRecord) -> JsResult<Option<JsString>> {
        // Returns undefined on iso8601.
        Ok(None)
    }

    /// `Temporal.Calendar.prototype.eraYear( dateLike )` for iso8601 calendar.
    fn era_year(&self, _: &IsoDateRecord) -> JsResult<Option<i32>> {
        // Returns undefined on iso8601.
        Ok(None)
    }

    /// Returns the `year` for the `Iso` calendar.
    fn year(&self, date_like: &IsoDateRecord) -> JsResult<i32> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(date.year().number)
    }

    /// Returns the `month` for the `Iso` calendar.
    fn month(&self, date_like: &IsoDateRecord) -> JsResult<i32> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(date.month().ordinal as i32)
    }

    /// Returns the `monthCode` for the `Iso` calendar.
    fn month_code(&self, date_like: &IsoDateRecord) -> JsResult<JsString> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(JsString::from(date.month().code.to_string()))
    }

    /// Returns the `day` for the `Iso` calendar.
    fn day(&self, date_like: &IsoDateRecord) -> JsResult<i32> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(date.day_of_month().0 as i32)
    }

    /// Returns the `dayOfWeek` for the `Iso` calendar.
    fn day_of_week(&self, date_like: &IsoDateRecord) -> JsResult<i32> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(date.day_of_week() as i32)
    }

    /// Returns the `dayOfYear` for the `Iso` calendar.
    fn day_of_year(&self, date_like: &IsoDateRecord) -> JsResult<i32> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(i32::from(date.day_of_year_info().day_of_year))
    }

    /// Returns the `weekOfYear` for the `Iso` calendar.
    fn week_of_year(&self, date_like: &IsoDateRecord) -> JsResult<i32> {
        // TODO: Determine `ICU4X` equivalent.
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        let week_calculator = WeekCalculator::default();

        let week_of = date
            .week_of_year(&week_calculator)
            .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(i32::from(week_of.week))
    }

    /// Returns the `yearOfWeek` for the `Iso` calendar.
    fn year_of_week(&self, date_like: &IsoDateRecord) -> JsResult<i32> {
        // TODO: Determine `ICU4X` equivalent.
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        let week_calculator = WeekCalculator::default();

        let week_of = date
            .week_of_year(&week_calculator)
            .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        match week_of.unit {
            RelativeUnit::Previous => Ok(date.year().number - 1),
            RelativeUnit::Current => Ok(date.year().number),
            RelativeUnit::Next => Ok(date.year().number + 1),
        }
    }

    /// Returns the `daysInWeek` value for the `Iso` calendar.
    fn days_in_week(&self, _: &IsoDateRecord) -> JsResult<i32> {
        Ok(7)
    }

    /// Returns the `daysInMonth` value for the `Iso` calendar.
    fn days_in_month(&self, date_like: &IsoDateRecord) -> JsResult<i32> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(i32::from(date.days_in_month()))
    }

    /// Returns the `daysInYear` value for the `Iso` calendar.
    fn days_in_year(&self, date_like: &IsoDateRecord) -> JsResult<i32> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(i32::from(date.days_in_year()))
    }

    /// Return the amount of months in an ISO Calendar.
    fn months_in_year(&self, _: &IsoDateRecord) -> JsResult<i32> {
        Ok(12)
    }

    /// Returns whether provided date is in a leap year according to this calendar.
    fn in_leap_year(&self, date_like: &IsoDateRecord) -> JsResult<bool> {
        // `ICU4X`'s `CalendarArithmetic` is currently private.
        if mathematical_days_in_year(date_like.year()) == 366 {
            return Ok(true);
        }
        Ok(false)
    }

    // Resolve the fields for the iso calendar.
    fn resolve_fields(&self, fields: &mut temporal::TemporalFields, _: FieldsType) -> JsResult<()> {
        fields.iso_resolve_month()?;
        Ok(())
    }

    /// Returns the ISO field descriptors, which is not called for the iso8601 calendar.
    fn field_descriptors(&self, _: FieldsType) -> Vec<(JsString, bool)> {
        // NOTE(potential improvement): look into implementing field descriptors and call
        // ISO like any other calendar?
        // Field descriptors is unused on ISO8601.
        unreachable!()
    }

    /// Returns the `CalendarFieldKeysToIgnore` implementation for ISO.
    fn field_keys_to_ignore(&self, additional_keys: Vec<JsString>) -> Vec<JsString> {
        let mut result = Vec::new();
        for key in &additional_keys {
            result.push(key.clone());
            if key.to_std_string_escaped().as_str() == "month" {
                result.push(js_string!("monthCode"));
            } else if key.to_std_string_escaped().as_str() == "monthCode" {
                result.push(js_string!("month"));
            }
        }
        result
    }

    // NOTE: This is currently not a name that is compliant with
    // the Temporal proposal. For debugging purposes only.
    /// Returns the debug name.
    fn debug_name(&self) -> &str {
        Iso.debug_name()
    }
}
