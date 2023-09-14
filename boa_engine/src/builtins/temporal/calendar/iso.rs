use std::env::temp_dir;

/// Implementation of the "iso8601" calendar.
use crate::{
    builtins::temporal::{
        self, create_temporal_date, create_temporal_duration, get_options_object,
        get_temporal_unit, plain_date::iso::IsoDateRecord, to_temporal_date, to_temporal_overflow,
        IsoYearMonthRecord,
    },
    js_string,
    string::utf16,
    Context, JsArgs, JsNativeError, JsResult, JsString, JsValue,
};

use super::BuiltinCalendar;

use icu_calendar::{iso::Iso, Calendar};

pub(crate) struct IsoCalendar;

impl BuiltinCalendar for IsoCalendar {
    /// Temporal 15.8.2.1 `Temporal.prototype.dateFromFields( fields [, options])` - Supercedes 12.5.4
    ///
    /// This is a basic implementation for an iso8601 calendar's `dateFromFields` method.
    fn date_from_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        // 3. Assert: calendar.[[Identifier]] is "iso8601".
        // 4. If Type(fields) is not Object, throw a TypeError exception.
        let fields = args.get_or_undefined(0);
        let fields_obj = fields.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("fields parameter must be an object.")
        })?;

        // 5. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;

        // 6. Set fields to ? PrepareTemporalFields(fields, « "day", "month", "monthCode", "year" », « "year", "day" »).
        let mut fields = temporal::TemporalFields::from_js_object(
            fields_obj,
            &[
                js_string!("day"),
                js_string!("month"),
                js_string!("monthCode"),
            ],
            Some(&[js_string!("year"), js_string!("day")]),
            None,
            context,
        )?;

        // NOTE: Overflow will probably have to be a work around for now for "constrained".
        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow = to_temporal_overflow(&options, context)?;

        fields.resolve_month()?;

        // 8. Let result be ? ISODateFromFields(fields, overflow).
        let result = IsoDateRecord::from_temporal_fields(&fields, &overflow)?;

        // 9. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
        Ok(create_temporal_date(result, "iso8601".into(), None, context)?.into())
    }

    /// 12.5.5 `Temporal.Calendar.prototype.yearMonthFromFields ( fields [ , options ] )`
    ///
    /// This is a basic implementation for an iso8601 calendar's `yearMonthFromFields` method.
    fn year_month_from_fields(
        &self,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        // 3. Assert: calendar.[[Identifier]] is "iso8601".
        // 4. If Type(fields) is not Object, throw a TypeError exception.
        let fields = args.get_or_undefined(0);
        let fields_obj = fields.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("fields parameter must be an object.")
        })?;

        // 5. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;

        // 6. Set fields to ? PrepareTemporalFields(fields, « "month", "monthCode", "year" », « "year" »).
        let mut fields = temporal::TemporalFields::from_js_object(
            fields_obj,
            &[
                js_string!("year"),
                js_string!("month"),
                js_string!("monthCode"),
            ],
            Some(&[js_string!("year")]),
            None,
            context,
        )?;

        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow = to_temporal_overflow(&options, context)?;

        // 8. Perform ? ISOResolveMonth(fields).
        fields.resolve_month()?;

        // 9. Let result be ? ISOYearMonthFromFields(fields, overflow).
        let result = IsoYearMonthRecord::from_temporal_fields(&mut fields, &overflow)?;

        // 10. Return ? CreateTemporalYearMonth(result.[[Year]], result.[[Month]], "iso8601", result.[[ReferenceISODay]]).
        temporal::create_temporal_year_month(result, JsValue::from("iso8601"), None, context)
    }

    /// 12.5.6 `Temporal.Calendar.prototype.monthDayFromFields ( fields [ , options ] )`
    ///
    /// This is a basic implementation for an iso8601 calendar's `monthDayFromFields` method.
    fn month_day_from_fields(
        &self,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        // 3. Assert: calendar.[[Identifier]] is "iso8601".
        // 4. If Type(fields) is not Object, throw a TypeError exception.
        let fields = args.get_or_undefined(0);
        let fields_obj = fields.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("fields parameter must be an object.")
        })?;

        // 5. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;

        // 6. Set fields to ? PrepareTemporalFields(fields, « "day", "month", "monthCode", "year" », « "day" »).
        let mut fields = temporal::TemporalFields::from_js_object(
            fields_obj,
            &[
                js_string!("day"),
                js_string!("month"),
                js_string!("monthCode"),
                js_string!("year"),
            ],
            Some(&[js_string!("year")]),
            None,
            context,
        )?;

        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow = to_temporal_overflow(&options, context)?;

        // 8. Perform ? ISOResolveMonth(fields).
        fields.resolve_month()?;

        // 9. Let result be ? ISOMonthDayFromFields(fields, overflow).
        let result = IsoDateRecord::month_day_from_temporal_fields(&fields, &overflow)?;

        // 10. Return ? CreateTemporalMonthDay(result.[[Month]], result.[[Day]], "iso8601", result.[[ReferenceISOYear]]).
        temporal::create_temporal_month_day(result, JsValue::from("iso8601"), None, context)
    }

    /// 12.5.7 `Temporal.Calendar.prototype.dateAdd ( date, duration [ , options ] )`
    ///
    /// Below implements the basic implementation for an iso8601 calendar's `dateAdd` method.
    fn date_add(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        // 3. Assert: calendar.[[Identifier]] is "iso8601".

        // 4. Set date to ? ToTemporalDate(date).
        let date_like = args.get_or_undefined(0);
        let date = to_temporal_date(date_like, None, context)?;

        // 5. Set duration to ? ToTemporalDuration(duration).
        let duration_like = args.get_or_undefined(1);
        let mut duration = temporal::duration::to_temporal_duration(duration_like, context)?;

        // 6. Set options to ? GetOptionsObject(options).
        let options = args.get_or_undefined(2);
        let options_obj = get_options_object(options)?;

        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow = to_temporal_overflow(&options_obj, context)?;

        // 8. Let balanceResult be ? BalanceTimeDuration(duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]], "day").
        duration
            .inner
            .balance_time_duration(&JsString::from("day"), None)?;

        // 9. Let result be ? AddISODate(date.[[ISOYear]], date.[[ISOMonth]], date.[[ISODay]], duration.[[Years]], duration.[[Months]], duration.[[Weeks]], balanceResult.[[Days]], overflow).
        let result = date.inner.add_iso_date(
            duration.inner.years() as i32,
            duration.inner.months() as i32,
            duration.inner.weeks() as i32,
            duration.inner.days() as i32,
            &overflow,
        )?;

        // 10. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
        Ok(create_temporal_date(result, "iso8601".into(), None, context)?.into())
    }

    /// 12.5.8 `Temporal.Calendar.prototype.dateUntil ( one, two [ , options ] )`
    ///
    ///  Below implements the basic implementation for an iso8601 calendar's `dateUntil` method.
    fn date_until(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        // 3. Assert: calendar.[[Identifier]] is "iso8601".

        // 4. Set one to ? ToTemporalDate(one).
        let one = to_temporal_date(args.get_or_undefined(0), None, context)?;
        // 5. Set two to ? ToTemporalDate(two).
        let two = to_temporal_date(args.get_or_undefined(1), None, context)?;

        // 6. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(2))?;

        let auto: JsValue = "auto".into();
        // 7. Let largestUnit be ? GetTemporalUnit(options, "largestUnit", date, "auto").
        let retrieved_unit = get_temporal_unit(
            &options,
            "largestUnit".into(),
            &JsString::from("date"),
            Some(&auto),
            None,
            context,
        )?
        .expect("Return must be a string.");

        // 8. If largestUnit is "auto", set largestUnit to "day".
        let largest_unit = match retrieved_unit.to_std_string_escaped().as_str() {
            "auto" => JsString::from("day"),
            _ => retrieved_unit,
        };

        // 9. Let result be DifferenceISODate(one.[[ISOYear]], one.[[ISOMonth]], one.[[ISODay]], two.[[ISOYear]], two.[[ISOMonth]], two.[[ISODay]], largestUnit).
        let result = one.inner.diff_iso_date(&two.inner, &largest_unit)?;

        // 10. Return ! CreateTemporalDuration(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], 0, 0, 0, 0, 0, 0).
        Ok(create_temporal_duration(result, None, context)?.into())
    }

    /// TODO: Docs
    fn era(&self, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }

    /// TODO: Docs
    fn era_year(&self, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }

    /// TODO: Docs
    fn year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn month(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn month_code(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn day(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn day_of_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn day_of_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn week_of_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn year_of_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn days_in_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }
    /// TODO: Docs
    fn days_in_month(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn days_in_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn months_in_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn in_leap_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn merge_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }
}
