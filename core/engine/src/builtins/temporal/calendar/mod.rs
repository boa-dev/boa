//! An implementation of the `Temporal` proposal's Calendar builtin.

use std::str::FromStr;

use super::extract_from_temporal_type;
use crate::{js_string, Context, JsNativeError, JsObject, JsResult, JsValue};
use temporal_rs::Calendar;

// -- `Calendar` Abstract Operations --

/// 12.2.21 `GetTemporalCalendarSlotValueWithISODefault ( item )`
#[allow(unused)]
pub(crate) fn get_temporal_calendar_slot_value_with_default(
    item: &JsObject,
    context: &mut Context,
) -> JsResult<Calendar> {
    // 1. If item has an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]], [[InitializedTemporalMonthDay]], [[InitializedTemporalYearMonth]], or [[InitializedTemporalZonedDateTime]] internal slot, then
    // a. Return item.[[Calendar]].
    if let Some(calendar) = extract_from_temporal_type(
        item,
        |d| Ok(Some(d.borrow().data().inner.calendar().clone())),
        |dt| Ok(Some(dt.borrow().data().inner.calendar().clone())),
        |ym| Ok(Some(ym.borrow().data().inner.calendar().clone())),
        |md| Ok(Some(md.borrow().data().inner.calendar().clone())),
        |zdt| {
            Err(JsNativeError::range()
                .with_message("Not yet implemented.")
                .into())
        },
    )? {
        return Ok(calendar);
    }

    // 2. Let calendarLike be ? Get(item, "calendar").
    let calendar_like = item.get(js_string!("calendar"), context)?;

    // 3. Return ? ToTemporalCalendarSlotValue(calendarLike, "iso8601").
    to_temporal_calendar_slot_value(&calendar_like)
}

/// `12.2.20 ToTemporalCalendarSlotValue ( temporalCalendarLike [ , default ] )`
pub(crate) fn to_temporal_calendar_slot_value(calendar_like: &JsValue) -> JsResult<Calendar> {
    // 1. If temporalCalendarLike is undefined and default is present, then
    // a. Assert: IsBuiltinCalendar(default) is true.
    // b. Return default.
    if calendar_like.is_undefined() {
        return Ok(Calendar::default());
    // 2. If Type(temporalCalendarLike) is Object, then
    } else if let Some(calendar_like) = calendar_like.as_object() {
        // a. If temporalCalendarLike has an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]], [[InitializedTemporalMonthDay]], [[InitializedTemporalYearMonth]], or [[InitializedTemporalZonedDateTime]] internal slot, then
        // i. Return temporalCalendarLike.[[Calendar]].
        if let Some(calendar) = extract_from_temporal_type(
            calendar_like,
            |d| Ok(Some(d.borrow().data().inner.calendar().clone())),
            |dt| Ok(Some(dt.borrow().data().inner.calendar().clone())),
            |ym| Ok(Some(ym.borrow().data().inner.calendar().clone())),
            |md| Ok(Some(md.borrow().data().inner.calendar().clone())),
            |zdt| Ok(Some(zdt.borrow().data().inner.calendar().clone())),
        )? {
            return Ok(calendar);
        }
    }

    // 3. If temporalCalendarLike is not a String, throw a TypeError exception.
    let JsValue::String(calendar_id) = calendar_like else {
        return Err(JsNativeError::typ()
            .with_message("temporalCalendarLike is not a string.")
            .into());
    };

    // 4. Let identifier be ? ParseTemporalCalendarString(temporalCalendarLike).
    // 5. If IsBuiltinCalendar(identifier) is false, throw a RangeError exception.
    // 6. Return the ASCII-lowercase of identifier.
    Ok(Calendar::from_str(&calendar_id.to_std_string_escaped())?)
}
