use std::env::temp_dir;

/// Implementation of the "iso8601" calendar.
use crate::{
    builtins::temporal::{self, plain_date::iso::IsoDateRecord, IsoYearMonthRecord},
    js_string,
    string::utf16,
    Context, JsArgs, JsNativeError, JsResult, JsValue,
};

use super::BuiltinCalendar;

use icu_calendar::{iso::Iso, Calendar};

pub(crate) struct IsoCalendar;

impl BuiltinCalendar for IsoCalendar {
    fn identifier(&self) -> &str {
        "iso8601"
    }

    /// Temporal Proposal 15.8.2.1 `Temporal.prototype.dateFromFields( fields [, options])` - Supercedes 12.5.4
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
        let options = temporal::get_options_object(args.get_or_undefined(1))?;

        // 6. Set fields to ? PrepareTemporalFields(fields, « "day", "month", "monthCode", "year" », « "year", "day" »).
        let mut fields = temporal::TemporalFields::from_js_object(
            fields_obj,
            &vec![
                js_string!("day"),
                js_string!("month"),
                js_string!("monthCode"),
            ],
            Some(&vec![js_string!("year"), js_string!("day")]),
            None,
            context,
        )?;

        // NOTE: Overflow will probably have to be a work around for now for "constrained".
        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow = temporal::to_temporal_overflow(&options, context)?;

        fields.resolve_month()?;

        // 8. Let result be ? ISODateFromFields(fields, overflow).
        let result = IsoDateRecord::from_temporal_fields(fields, &overflow)?;

        // 9. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
        temporal::create_temporal_date(result, JsValue::from("iso8601"), None, context)
    }

    /// 12.5.5 Temporal.Calendar.prototype.yearMonthFromFields ( fields [ , options ] )
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
        let options = temporal::get_options_object(args.get_or_undefined(1))?;

        // 6. Set fields to ? PrepareTemporalFields(fields, « "month", "monthCode", "year" », « "year" »).
        let mut fields = temporal::TemporalFields::from_js_object(
            fields_obj,
            &vec![
                js_string!("year"),
                js_string!("month"),
                js_string!("monthCode"),
            ],
            Some(&vec![js_string!("year")]),
            None,
            context,
        )?;

        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow = temporal::to_temporal_overflow(&options, context)?;

        // 8. Perform ? ISOResolveMonth(fields).
        fields.resolve_month()?;

        // 9. Let result be ? ISOYearMonthFromFields(fields, overflow).
        let result = IsoYearMonthRecord::from_temporal_fields(&mut fields, &overflow)?;

        // 10. Return ? CreateTemporalYearMonth(result.[[Year]], result.[[Month]], "iso8601", result.[[ReferenceISODay]]).
        temporal::create_temporal_year_month(result, JsValue::from("iso8601"), None, context)
    }

    /// TODO: Docs
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
        let options = temporal::get_options_object(args.get_or_undefined(1))?;

        // 6. Set fields to ? PrepareTemporalFields(fields, « "day", "month", "monthCode", "year" », « "day" »).
        let mut fields = temporal::TemporalFields::from_js_object(
            fields_obj,
            &vec![
                js_string!("day"),
                js_string!("month"),
                js_string!("monthCode"),
                js_string!("year"),
            ],
            Some(&vec![js_string!("year")]),
            None,
            context,
        )?;

        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow = temporal::to_temporal_overflow(&options, context)?;

        // 8. Perform ? ISOResolveMonth(fields).
        fields.resolve_month()?;

        // 9. Let result be ? ISOMonthDayFromFields(fields, overflow).
        let result = IsoDateRecord::month_day_from_temporal_fields(fields, &overflow)?;

        // 10. Return ? CreateTemporalMonthDay(result.[[Month]], result.[[Day]], "iso8601", result.[[ReferenceISOYear]]).
        temporal::create_temporal_month_day(result, JsValue::from("iso8601"), None, context)
    }

    /// TODO: Docs
    fn date_add(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// TODO: Docs
    fn date_until(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
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
