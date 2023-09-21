//! Implementation of the "iso8601" `BuiltinCalendar`.

use crate::{
    builtins::temporal::{
        self, create_temporal_date, create_temporal_duration,
        date_equations::{
            epoch_time_for_year, mathematical_days_in_year, mathematical_in_leap_year,
        },
        get_options_object, get_temporal_unit,
        plain_date::iso::IsoDateRecord,
        to_temporal_date, to_temporal_overflow,
    },
    js_string,
    property::PropertyKey,
    string::utf16,
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
};

use super::BuiltinCalendar;

use icu_calendar::{iso::Iso, types::IsoWeekday, week::WeekCalculator, Calendar, Date};

pub(crate) struct IsoCalendar;

impl BuiltinCalendar for IsoCalendar {
    /// Temporal 15.8.2.1 `Temporal.prototype.dateFromFields( fields [, options])` - Supercedes 12.5.4
    ///
    /// This is a basic implementation for an iso8601 calendar's `dateFromFields` method.
    fn date_from_fields(
        &self,
        fields: &mut temporal::TemporalFields,
        overflow: &str,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // NOTE: we are in ISO by default here.
        // 9. If calendar.[[Identifier]] is "iso8601", then
        // a. Perform ? ISOResolveMonth(fields).
        // b. Let result be ? ISODateFromFields(fields, overflow).
        // 10. Else,
        // a. Perform ? CalendarResolveFields(calendar.[[Identifier]], fields, date).
        // b. Let result be ? CalendarDateToISO(calendar.[[Identifier]], fields, overflow).
        // NOTE: Overflow will probably have to be a work around for now for "constrained".

        // a. Perform ? ISOResolveMonth(fields).
        fields.resolve_month()?;

        // Extra: handle reulating/overflow until implemented on `icu_calendar`
        fields.regulate(overflow);

        let date = Date::try_new_iso_date(
            fields.year().unwrap_or(0),
            fields.month().unwrap_or(255) as u8,
            fields.day().unwrap_or(255) as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        // 9. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
        Ok(create_temporal_date(
            IsoDateRecord::from_date_iso(date),
            "iso8601".into(),
            None,
            context,
        )?
        .into())
    }

    /// 12.5.5 `Temporal.Calendar.prototype.yearMonthFromFields ( fields [ , options ] )`
    ///
    /// This is a basic implementation for an iso8601 calendar's `yearMonthFromFields` method.
    fn year_month_from_fields(
        &self,
        fields: &mut temporal::TemporalFields,
        overflow: &str,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 9. If calendar.[[Identifier]] is "iso8601", then
        // a. Perform ? ISOResolveMonth(fields).
        fields.resolve_month()?;

        // b. Let result be ? ISOYearMonthFromFields(fields, overflow).
        fields.regulate_year_month(overflow);

        let result = Date::try_new_iso_date(
            fields.year().unwrap_or(0),
            fields.month().unwrap_or(255) as u8,
            fields.day().unwrap_or(20) as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        // 10. Return ? CreateTemporalYearMonth(result.[[Year]], result.[[Month]], "iso8601", result.[[ReferenceISODay]]).
        temporal::create_temporal_year_month(
            IsoDateRecord::from_date_iso(result),
            "iso8601".into(),
            None,
            context,
        )
    }

    /// 12.5.6 `Temporal.Calendar.prototype.monthDayFromFields ( fields [ , options ] )`
    ///
    /// This is a basic implementation for an iso8601 calendar's `monthDayFromFields` method.
    fn month_day_from_fields(
        &self,
        fields: &mut temporal::TemporalFields,
        overflow: &str,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 8. Perform ? ISOResolveMonth(fields).
        fields.resolve_month()?;

        fields.regulate(overflow);

        // TODO: double check error mapping is correct for specifcation/test262.
        // 9. Let result be ? ISOMonthDayFromFields(fields, overflow).
        let result = Date::try_new_iso_date(
            fields.year().unwrap_or(1972),
            fields.month().unwrap_or(255) as u8,
            fields.day().unwrap_or(255) as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        // 10. Return ? CreateTemporalMonthDay(result.[[Month]], result.[[Day]], "iso8601", result.[[ReferenceISOYear]]).
        temporal::create_temporal_month_day(
            IsoDateRecord::from_date_iso(result),
            JsValue::from("iso8601"),
            None,
            context,
        )
    }

    /// 12.5.7 `Temporal.Calendar.prototype.dateAdd ( date, duration [ , options ] )`
    ///
    /// Below implements the basic implementation for an iso8601 calendar's `dateAdd` method.
    fn date_add(
        &self,
        _date: &temporal::PlainDate,
        _duration: &temporal::duration::DurationRecord,
        _overflow: &str,
        _context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
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
        _largest_unit: &str,
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // TODO: Not stable on `ICU4X`. Implement once completed.
        Err(JsNativeError::range()
            .with_message("Feature not yet implemented.")
            .into())

        // 9. Let result be DifferenceISODate(one.[[ISOYear]], one.[[ISOMonth]], one.[[ISODay]], two.[[ISOYear]], two.[[ISOMonth]], two.[[ISODay]], largestUnit).
        // 10. Return ! CreateTemporalDuration(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], 0, 0, 0, 0, 0, 0).
    }

    /// `Temporal.Calendar.prototype.era( dateLike )` for iso8601 calendar.
    fn era(&self, _: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        // Returns undefined on iso8601.
        Ok(JsValue::undefined())
    }

    /// `Temporal.Calendar.prototype.eraYear( dateLike )` for iso8601 calendar.
    fn era_year(&self, _: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        // Returns undefined on iso8601.
        Ok(JsValue::undefined())
    }

    /// Returns the `year` for the `Iso` calendar.
    fn year(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(date.year().number.into())
    }

    /// Returns the `month` for the `Iso` calendar.
    fn month(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(date.month().ordinal.into())
    }

    /// Returns the `monthCode` for the `Iso` calendar.
    fn month_code(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(date.month().code.to_string().into())
    }

    /// Returns the `day` for the `Iso` calendar.
    fn day(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(date.day_of_month().0.into())
    }

    /// Returns the `dayOfWeek` for the `Iso` calendar.
    fn day_of_week(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok((date.day_of_week() as u8).into())
    }

    /// Returns the `dayOfYear` for the `Iso` calendar.
    fn day_of_year(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok((date.day_of_year_info().day_of_year as i32).into())
    }

    /// Returns the `weekOfYear` for the `Iso` calendar.
    fn week_of_year(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        // TODO: Determine `ICU4X` equivalent.
        let record =
            super::utils::to_iso_week_of_year(date_like.year(), date_like.month(), date_like.day());

        Ok(record.0.into())
    }

    /// Returns the `yearOfWeek` for the `Iso` calendar.
    fn year_of_week(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        // TODO: Determine `ICU4X` equivalent.
        let record =
            super::utils::to_iso_week_of_year(date_like.year(), date_like.month(), date_like.day());

        Ok(record.1.into())
    }

    /// Returns the `daysInWeek` value for the `Iso` calendar.
    fn days_in_week(&self, _: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(7.into())
    }

    /// Returns the `daysInMonth` value for the `Iso` calendar.
    fn days_in_month(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(date.days_in_month().into())
    }

    /// Returns the `daysInYear` value for the `Iso` calendar.
    fn days_in_year(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        let date = Date::try_new_iso_date(
            date_like.year(),
            date_like.month() as u8,
            date_like.day() as u8,
        )
        .map_err(|err| JsNativeError::range().with_message(err.to_string()))?;

        Ok(date.days_in_year().into())
    }

    /// TODO: Docs
    fn months_in_year(&self, _: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(12.into())
    }

    /// TODO: Docs
    fn in_leap_year(&self, date_like: &IsoDateRecord, _: &mut Context<'_>) -> JsResult<JsValue> {
        // `ICU4X`'s `CalendarArithmetic` is currently private.
        if mathematical_days_in_year(date_like.year()) == 366 {
            return Ok(true.into());
        }
        Ok(false.into())
    }

    // Resolve the fields for the iso calendar.
    fn resolve_fields(&self, fields: &mut temporal::TemporalFields, _: &str) -> JsResult<()> {
        fields.resolve_month()?;
        Ok(())
    }

    /// Returns the ISO field descriptors, which is not called for the iso8601 calendar.
    fn field_descriptors(&self, _: &[String]) -> Vec<(String, bool)> {
        // NOTE(potential improvement): look into implementing field descriptors and call
        // ISO like any other calendar?
        // Field descriptors is unused on ISO8601.
        unreachable!()
    }

    /// Returns the `CalendarFieldKeysToIgnore` implementation for ISO.
    fn field_keys_to_ignore(&self, additional_keys: Vec<PropertyKey>) -> Vec<PropertyKey> {
        let mut result = Vec::new();
        for key in additional_keys {
            let key_string = key.to_string();
            result.push(key);
            if key_string.as_str() == "month" {
                result.push("monthCode".into());
            } else if key_string.as_str() == "monthCode" {
                result.push("month".into());
            }
        }
        result
    }

    fn debug_name(&self) -> &str {
        Iso.debug_name()
    }
}
