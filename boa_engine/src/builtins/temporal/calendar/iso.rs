/// Implementation of the "iso8601" calendar.

use crate::{
    Context, JsValue, JsResult,
};

use super::BuiltinCalendar;

use icu_calendar::{iso::Iso, Calendar};

struct IsoCalendar;

impl BuiltinCalendar for IsoCalendar {
    fn identifier(&self) -> &str {
        "iso8601"
    }

    /// Temporal Proposal 12.5.4 `Temporal.prototype.dateFromFields( fields [, options])`
    fn date_from_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        // 3. Assert: calendar.[[Identifier]] is "iso8601".

        // 4. If Type(fields) is not Object, throw a TypeError exception.
        // 5. Set options to ? GetOptionsObject(options).
        // 6. Set fields to ? PrepareTemporalFields(fields, « "day", "month", "monthCode", "year" », « "year", "day" »).
        // 7. Let overflow be ? ToTemporalOverflow(options).
        // 8. Let result be ? ISODateFromFields(fields, overflow).
        // ISODateFromFields is equivalent to iso.date_from_codes("default", )
        let iso = Iso::new();
        // let result = iso.date_from_codes("default", year, month_code, day)
        // 9. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], "iso8601").
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn year_month_from_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {

        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn month_day_from_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {

        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn date_add(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {

        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn date_until(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {

        Ok(JsValue::Undefined)
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
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn month_code(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn day(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn day_of_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn day_of_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn week_of_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn year_of_week(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn days_in_month(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn months_in_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn in_leap_year(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }

    /// TODO: Docs
    fn merge_fields(&self, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::Undefined)
    }
}