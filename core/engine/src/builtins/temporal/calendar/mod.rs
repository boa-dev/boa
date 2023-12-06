//! An implementation of the `Temporal` proposal's Calendar builtin.

use std::str::FromStr;

use super::{
    create_temporal_date, create_temporal_duration, create_temporal_month_day,
    create_temporal_year_month, fields, options::TemporalUnitGroup, PlainDate, PlainDateTime,
    PlainMonthDay, PlainYearMonth, ZonedDateTime,
};
use crate::{
    builtins::{
        iterable::IteratorHint,
        options::{get_option, get_options_object},
        temporal, Array, BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::{common::StaticJsStrings, utf16},
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use boa_temporal::{
    components::calendar::{
        AvailableCalendars, CalendarDateLike, CalendarFieldsType, CalendarSlot,
        CALENDAR_PROTOCOL_METHODS,
    },
    options::{ArithmeticOverflow, TemporalUnit},
};

mod object;

use object::CustomRuntimeCalendar;

#[cfg(feature = "experimental")]
#[cfg(test)]
mod tests;

/// The `Temporal.Calendar` object.
#[derive(Debug, Trace, Finalize, JsData)]
// SAFETY: `Calendar` doesn't contain traceable types.
#[boa_gc(unsafe_empty_trace)]
pub struct Calendar {
    slot: CalendarSlot,
}

impl Calendar {
    pub(crate) fn new(slot: CalendarSlot) -> Self {
        Self { slot }
    }
}

impl BuiltInObject for Calendar {
    const NAME: JsString = StaticJsStrings::CALENDAR;
}

impl IntrinsicObject for Calendar {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_id = BuiltInBuilder::callable(realm, Self::get_id)
            .name(js_string!("get Id"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .accessor(utf16!("id"), Some(get_id), None, Attribute::default())
            .static_method(Self::from, js_string!("from"), 1)
            .method(Self::date_from_fields, js_string!("dateFromFields"), 2)
            .method(
                Self::year_month_from_fields,
                js_string!("yearMonthFromFields"),
                2,
            )
            .method(
                Self::month_day_from_fields,
                js_string!("monthDayFromFields"),
                2,
            )
            .method(Self::date_add, js_string!("dateAdd"), 3)
            .method(Self::date_until, js_string!("dateUntil"), 3)
            .method(Self::era, js_string!("era"), 1)
            .method(Self::era_year, js_string!("eraYear"), 1)
            .method(Self::year, js_string!("year"), 1)
            .method(Self::month, js_string!("month"), 1)
            .method(Self::month_code, js_string!("monthCode"), 1)
            .method(Self::day, js_string!("day"), 1)
            .method(Self::day_of_week, js_string!("dayOfWeek"), 1)
            .method(Self::day_of_year, js_string!("dayOfYear"), 1)
            .method(Self::week_of_year, js_string!("weekOfYear"), 1)
            .method(Self::year_of_week, js_string!("yearOfWeek"), 1)
            .method(Self::days_in_week, js_string!("daysInWeek"), 1)
            .method(Self::days_in_month, js_string!("daysInMonth"), 1)
            .method(Self::days_in_year, js_string!("daysInYear"), 1)
            .method(Self::months_in_year, js_string!("monthsInYear"), 1)
            .method(Self::in_leap_year, js_string!("inLeapYear"), 1)
            .method(Self::fields, js_string!("fields"), 1)
            .method(Self::merge_fields, js_string!("mergeFields"), 2)
            .method(Self::get_id, js_string!("toString"), 0)
            .method(Self::get_id, js_string!("toJSON"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for Calendar {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::calendar;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, then
        if new_target.is_undefined() {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message(
                    "newTarget cannot be undefined when constructing a Temporal.Calendar object.",
                )
                .into());
        }

        let identifier = args.get_or_undefined(0);

        // 2. If id is not a String, throw a TypeError exception.
        let JsValue::String(id) = identifier else {
            return Err(JsNativeError::typ()
                .with_message("Calendar id must be a string.")
                .into());
        };

        // 3. If IsBuiltinCalendar(id) is false, then
        // a. Throw a RangeError exception.
        let _ = AvailableCalendars::from_str(&id.to_std_string_escaped())?;

        // 4. Return ? CreateTemporalCalendar(id, NewTarget).
        create_temporal_calendar(
            CalendarSlot::Identifier(id.to_std_string_escaped()),
            Some(new_target.clone()),
            context,
        )
    }
}

impl Calendar {
    fn from(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar_like = args.get_or_undefined(0);
        let slot = to_temporal_calendar_slot_value(calendar_like, context)?;
        create_temporal_calendar(slot, None, context)
    }

    fn get_id(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("the this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        Ok(JsString::from(protocol.identifier(context)?.as_str()).into())
    }

    /// 15.8.2.1 `Temporal.Calendar.prototype.dateFromFields ( fields [ , options ] )` - Supercedes 12.5.4
    fn date_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        // Retrieve the current CalendarProtocol.
        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        // 3. If Type(fields) is not Object, throw a TypeError exception.
        let fields = args.get_or_undefined(0);
        let fields_obj = fields.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("fields parameter must be an object.")
        })?;

        // 4. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;

        // 5. Let relevantFieldNames be « "day", "month", "monthCode", "year" ».
        let mut relevant_field_names = Vec::from([
            js_string!("day"),
            js_string!("month"),
            js_string!("monthCode"),
            js_string!("year"),
        ]);

        // 6. If calendar.[[Identifier]] is "iso8601", then
        let mut fields = if protocol.identifier(context)?.as_str() == "iso8601" {
            // a. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « "year", "day" »).
            let mut required_fields = Vec::from([js_string!("year"), js_string!("day")]);
            fields::prepare_temporal_fields(
                fields_obj,
                &mut relevant_field_names,
                &mut required_fields,
                None,
                false,
                None,
                context,
            )?
        // 7. Else,
        } else {
            // a. Let calendarRelevantFieldDescriptors be CalendarFieldDescriptors(calendar.[[Identifier]], date).
            let calendar_relevant_fields = protocol.field_descriptors(CalendarFieldsType::Date);
            // b. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « », calendarRelevantFieldDescriptors).
            fields::prepare_temporal_fields(
                fields_obj,
                &mut relevant_field_names,
                &mut Vec::new(),
                Some(calendar_relevant_fields),
                false,
                None,
                context,
            )?
        };

        // 8. Let overflow be ? ToTemporalOverflow(options).
        let overflow = get_option(&options, utf16!("overflow"), context)?
            .unwrap_or(ArithmeticOverflow::Constrain);

        // NOTE: implement the below on the calenar itself
        // 9. If calendar.[[Identifier]] is "iso8601", then
        // a. Perform ? ISOResolveMonth(fields).
        // b. Let result be ? ISODateFromFields(fields, overflow).
        // 10. Else,
        // a. Perform ? CalendarResolveFields(calendar.[[Identifier]], fields, date).
        // b. Let result be ? CalendarDateToISO(calendar.[[Identifier]], fields, overflow).

        let result = protocol.date_from_fields(&mut fields, overflow, context)?;

        create_temporal_date(result, None, context).map(Into::into)
    }

    /// 15.8.2.2 `Temporal.Calendar.prototype.yearMonthFromFields ( fields [ , options ] )` - Supercedes 12.5.5
    fn year_month_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let fields = args.get_or_undefined(0);
        let fields_obj = fields.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("fields parameter must be an object.")
        })?;

        // 5. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;

        let mut relevant_field_names = Vec::from([
            js_string!("year"),
            js_string!("month"),
            js_string!("monthCode"),
        ]);

        // 6. Set fields to ? PrepareTemporalFields(fields, « "month", "monthCode", "year" », « "year" »).
        let mut fields = if protocol.identifier(context)?.as_str() == "iso8601" {
            // a. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « "year" »).
            let mut required_fields = Vec::from([js_string!("year")]);
            fields::prepare_temporal_fields(
                fields_obj,
                &mut relevant_field_names,
                &mut required_fields,
                None,
                false,
                None,
                context,
            )?
        } else {
            // a. Let calendarRelevantFieldDescriptors be CalendarFieldDescriptors(calendar.[[Identifier]], year-month).
            // b. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « », calendarRelevantFieldDescriptors).

            let calendar_relevant_fields =
                protocol.field_descriptors(CalendarFieldsType::YearMonth);
            fields::prepare_temporal_fields(
                fields_obj,
                &mut relevant_field_names,
                &mut Vec::new(),
                Some(calendar_relevant_fields),
                false,
                None,
                context,
            )?

            // TODO: figure out the below. Maybe a method on fields?
            // c. Let firstDayIndex be the 1-based index of the first day of the month described by fields (i.e., 1 unless the month's first day is skipped by this calendar.)
            // d. Perform ! CreateDataPropertyOrThrow(fields, "day", 𝔽(firstDayIndex)).
        };

        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow = get_option::<ArithmeticOverflow>(&options, utf16!("overflow"), context)?
            .unwrap_or(ArithmeticOverflow::Constrain);

        let result = protocol.year_month_from_fields(&mut fields, overflow, context)?;

        create_temporal_year_month(result, None, context)
    }

    /// 15.8.2.3 `Temporal.Calendar.prototype.monthDayFromFields ( fields [ , options ] )` - Supercedes 12.5.6
    fn month_day_from_fields(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        // 3. If Type(fields) is not Object, throw a TypeError exception.
        let fields = args.get_or_undefined(0);
        let fields_obj = fields.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("fields parameter must be an object.")
        })?;

        // 4. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;

        // 5. Let relevantFieldNames be « "day", "month", "monthCode", "year" ».
        let mut relevant_field_names = Vec::from([
            js_string!("day"),
            js_string!("month"),
            js_string!("monthCode"),
            js_string!("year"),
        ]);

        // 6. If calendar.[[Identifier]] is "iso8601", then
        let mut fields = if protocol.identifier(context)?.as_str() == "iso8601" {
            // a. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « "day" »).
            let mut required_fields = Vec::from([js_string!("day")]);
            fields::prepare_temporal_fields(
                fields_obj,
                &mut relevant_field_names,
                &mut required_fields,
                None,
                false,
                None,
                context,
            )?
        // 7. Else,
        } else {
            // a. Let calendarRelevantFieldDescriptors be CalendarFieldDescriptors(calendar.[[Identifier]], month-day).
            let calendar_relevant_fields = protocol.field_descriptors(CalendarFieldsType::MonthDay);
            // b. Set fields to ? PrepareTemporalFields(fields, relevantFieldNames, « », calendarRelevantFieldDescriptors).
            fields::prepare_temporal_fields(
                fields_obj,
                &mut relevant_field_names,
                &mut Vec::new(),
                Some(calendar_relevant_fields),
                false,
                None,
                context,
            )?
        };

        // 8. Let overflow be ? ToTemporalOverflow(options).
        let overflow = get_option(&options, utf16!("overflow"), context)?
            .unwrap_or(ArithmeticOverflow::Constrain);

        let result = protocol.month_day_from_fields(&mut fields, overflow, context)?;

        create_temporal_month_day(result, None, context)
    }

    /// 15.8.2.4 `Temporal.Calendar.prototype.dateAdd ( date, duration [ , options ] )` - supercedes 12.5.7
    fn date_add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        // 3. Assert: calendar.[[Identifier]] is "iso8601".
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        // 4. Set date to ? ToTemporalDate(date).
        let date_like = args.get_or_undefined(0);
        let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;

        // 5. Set duration to ? ToTemporalDuration(duration).
        let duration_like = args.get_or_undefined(1);
        let duration = temporal::duration::to_temporal_duration(duration_like)?;

        // 6. Set options to ? GetOptionsObject(options).
        let options = args.get_or_undefined(2);
        let options_obj = get_options_object(options)?;

        // 7. Let overflow be ? ToTemporalOverflow(options).
        let overflow = get_option(&options_obj, utf16!("overflow"), context)?
            .unwrap_or(ArithmeticOverflow::Constrain);

        // 8. Let balanceResult be ? BalanceTimeDuration(duration.[[Days]], duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]], "day").
        duration.balance_time_duration(TemporalUnit::Day)?;

        let result = protocol.date_add(&date.inner, &duration, overflow, context)?;

        create_temporal_date(result, None, context).map(Into::into)
    }

    ///15.8.2.5 `Temporal.Calendar.prototype.dateUntil ( one, two [ , options ] )` - Supercedes 12.5.8
    fn date_until(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        // 3. Assert: calendar.[[Identifier]] is "iso8601".
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        // 4. Set one to ? ToTemporalDate(one).
        let one = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;
        // 5. Set two to ? ToTemporalDate(two).
        let two = temporal::plain_date::to_temporal_date(args.get_or_undefined(1), None, context)?;

        // 6. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(2))?;

        // 7. Let largestUnit be ? GetTemporalUnit(options, "largestUnit", date, "auto").
        // 8. If largestUnit is "auto", set largestUnit to "day".
        let largest_unit = super::options::get_temporal_unit(
            &options,
            utf16!("largestUnit"),
            TemporalUnitGroup::Date,
            None,
            context,
        )?
        .unwrap_or(TemporalUnit::Day);

        let result = protocol.date_until(&one.inner, &two.inner, largest_unit, context)?;

        create_temporal_duration(result, None, context).map(Into::into)
    }

    /// 15.8.2.6 `Temporal.Calendar.prototype.era ( temporalDateLike )`
    fn era(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let date_like = to_calendar_date_like(args.get_or_undefined(0), context)?;

        let result = protocol
            .era(&date_like, context)?
            .map_or(JsValue::undefined(), |r| JsString::from(r.as_str()).into());

        Ok(result)
    }

    /// 15.8.2.7 `Temporal.Calendar.prototype.eraYear ( temporalDateLike )`
    fn era_year(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let date_like = to_calendar_date_like(args.get_or_undefined(0), context)?;

        let result = protocol
            .era_year(&date_like, context)?
            .map_or(JsValue::undefined(), JsValue::from);

        Ok(result)
    }

    /// 15.8.2.8 `Temporal.Calendar.prototype.year ( temporalDateLike )`
    fn year(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let date_like = to_calendar_date_like(args.get_or_undefined(0), context)?;

        let result = protocol.year(&date_like, context)?;

        Ok(result.into())
    }

    /// 15.8.2.9 `Temporal.Calendar.prototype.month ( temporalDateLike )`
    fn month(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let date_like = to_calendar_date_like(args.get_or_undefined(0), context)?;

        // 3. If Type(temporalDateLike) is Object and temporalDateLike has an [[InitializedTemporalMonthDay]] internal slot, then
        // 3.a. Throw a TypeError exception.
        // 4. If Type(temporalDateLike) is not Object or temporalDateLike does not have an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]], or [[InitializedTemporalYearMonth]] internal slot, then
        // 4.a. Set temporalDateLike to ? ToTemporalDate(temporalDateLike).

        let result = protocol.month(&date_like, context)?;

        Ok(result.into())
    }

    /// 15.8.2.10 `Temporal.Calendar.prototype.monthCode ( temporalDateLike )`
    fn month_code(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let date_like = to_calendar_date_like(args.get_or_undefined(0), context)?;

        let result = protocol.month_code(&date_like, context)?;

        Ok(JsString::from(result.as_str()).into())
    }

    /// 15.8.2.11 `Temporal.Calendar.prototype.day ( temporalDateLike )`
    fn day(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let date_like = to_calendar_date_like(args.get_or_undefined(0), context)?;

        let result = protocol.day(&date_like, context)?;

        Ok(result.into())
    }

    /// 15.8.2.12 `Temporal.Calendar.prototype.dayOfWeek ( dateOrDateTime )`
    fn day_of_week(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        // 3. Let temporalDate be ? ToTemporalDate(temporalDateLike).
        let date = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;

        let result = protocol.day_of_week(&CalendarDateLike::Date(date.inner.clone()), context)?;

        Ok(result.into())
    }

    /// 15.8.2.13 `Temporal.Calendar.prototype.dayOfYear ( temporalDateLike )`
    fn day_of_year(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        // 3. Let temporalDate be ? ToTemporalDate(temporalDateLike).
        let date = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;

        let result = protocol.day_of_year(&CalendarDateLike::Date(date.inner.clone()), context)?;

        Ok(result.into())
    }

    /// 15.8.2.14 `Temporal.Calendar.prototype.weekOfYear ( temporalDateLike )`
    fn week_of_year(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        // 3. Let temporalDate be ? ToTemporalDate(temporalDateLike).
        let date = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;

        let result = protocol.week_of_year(&CalendarDateLike::Date(date.inner.clone()), context)?;

        Ok(result.into())
    }

    /// 15.8.2.15 `Temporal.Calendar.prototype.yearOfWeek ( temporalDateLike )`
    fn year_of_week(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        // 3. Let temporalDate be ? ToTemporalDate(temporalDateLike).
        let date = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;

        let result = protocol.year_of_week(&CalendarDateLike::Date(date.inner.clone()), context)?;

        Ok(result.into())
    }

    /// 15.8.2.16 `Temporal.Calendar.prototype.daysInWeek ( temporalDateLike )`
    fn days_in_week(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        // 3. Let temporalDate be ? ToTemporalDate(temporalDateLike).
        let date = temporal::plain_date::to_temporal_date(args.get_or_undefined(0), None, context)?;

        let result = protocol.days_in_week(&CalendarDateLike::Date(date.inner.clone()), context)?;

        Ok(result.into())
    }

    /// 15.8.2.17 `Temporal.Calendar.prototype.daysInMonth ( temporalDateLike )`
    fn days_in_month(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let date_like = to_calendar_date_like(args.get_or_undefined(0), context)?;

        let result = protocol.days_in_month(&date_like, context)?;

        Ok(result.into())
    }

    /// 15.8.2.18 `Temporal.Calendar.prototype.daysInYear ( temporalDateLike )`
    fn days_in_year(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let date_like = to_calendar_date_like(args.get_or_undefined(0), context)?;
        let result = protocol.days_in_year(&date_like, context)?;

        Ok(result.into())
    }

    /// 15.8.2.19 `Temporal.Calendar.prototype.monthsInYear ( temporalDateLike )`
    fn months_in_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let date_like = to_calendar_date_like(args.get_or_undefined(0), context)?;

        let result = protocol.months_in_year(&date_like, context)?;

        Ok(result.into())
    }

    /// 15.8.2.20 `Temporal.Calendar.prototype.inLeapYear ( temporalDateLike )`
    fn in_leap_year(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let date_like = to_calendar_date_like(args.get_or_undefined(0), context)?;

        let result = protocol.in_leap_year(&date_like, context)?;

        Ok(result.into())
    }

    /// 15.8.2.21 `Temporal.Calendar.prototype.fields ( fields )`
    fn fields(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        // 3. Let iteratorRecord be ? GetIterator(fields, sync).
        let mut iterator_record =
            args.get_or_undefined(0)
                .get_iterator(context, Some(IteratorHint::Sync), None)?;

        // 4. Let fieldNames be a new empty List.
        let mut fields_names = Vec::new();

        // 5. Let next be true.
        // 6. Repeat, while next is not false,
        while iterator_record.step(context)? {
            // a. Set next to ? IteratorStep(iteratorRecord).
            // b. If next is not false, then
            // i. Let nextValue be ? IteratorValue(next).
            let next_value = iterator_record.value(context)?;

            // ii. If Type(nextValue) is not String, then
            if let JsValue::String(value) = next_value {
                // iii. If fieldNames contains nextValue, then
                // 1. Let completion be ThrowCompletion(a newly created RangeError object).
                // 2. Return ? IteratorClose(iteratorRecord, completion).
                // iv. If nextValue is not one of "year", "month", "monthCode", or "day", then
                // 1. Let completion be ThrowCompletion(a newly created RangeError object).
                // 2. Return ? IteratorClose(iteratorRecord, completion).
                // v. Append nextValue to the end of the List fieldNames.
                let this_field = value.to_std_string_escaped();
                match this_field.as_str() {
                    "year" | "month" | "monthCode" | "day"
                        if !fields_names.contains(&this_field) =>
                    {
                        fields_names.push(this_field);
                    }
                    _ => {
                        let completion = Err(JsNativeError::range()
                            .with_message("Invalid field name string.")
                            .into());
                        return iterator_record.close(completion, context);
                    }
                }
            } else {
                // 1. Let completion be ThrowCompletion(a newly created TypeError object).
                let completion = Err(JsNativeError::typ()
                    .with_message("field must be of type string")
                    .into());
                // 2. Return ? IteratorClose(iteratorRecord, completion).
                return iterator_record.close(completion, context);
            }
        }

        // 7. Let result be fieldNames.
        // 8. If calendar.[[Identifier]] is not "iso8601", then
        if protocol.identifier(context)?.as_str() != "iso8601" {
            // a. NOTE: Every built-in calendar preserves all input field names in output.
            // b. Let extraFieldDescriptors be CalendarFieldDescriptors(calendar.[[Identifier]], fieldNames).
            let extended_fields =
                protocol.field_descriptors(CalendarFieldsType::from(&fields_names[..]));
            // c. For each Calendar Field Descriptor Record desc of extraFieldDescriptors, do
            for descriptor in extended_fields {
                // i. Append desc.[[Property]] to result.
                fields_names.push(descriptor.0);
            }
        }

        // 9. Return CreateArrayFromList(result).
        Ok(Array::create_array_from_list(
            fields_names
                .iter()
                .map(|s| JsString::from(s.clone()).into()),
            context,
        )
        .into())
    }

    /// 15.8.2.22 `Temporal.Calendar.prototype.mergeFields ( fields, additionalFields )`
    fn merge_fields(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let calendar be the this value.
        // 2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
        let calendar = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value of Calendar must be a Calendar object.")
            })?;

        let protocol = match &calendar.slot {
            CalendarSlot::Identifier(s) => AvailableCalendars::from_str(s)?.to_protocol(),
            CalendarSlot::Protocol(proto) => proto.clone(),
        };

        let fields = args.get_or_undefined(0).to_object(context)?;
        let additional_fields = args.get_or_undefined(1).to_object(context)?;

        // 3. Let fieldsCopy be ? SnapshotOwnProperties(? ToObject(fields), null, « », « undefined »).
        let fields_copy = temporal::snapshot_own_properties(
            &fields,
            Some(Vec::new()),
            Some(Vec::from([JsValue::undefined()])),
            context,
        )?;

        // 4. Let additionalFieldsCopy be ? SnapshotOwnProperties(? ToObject(additionalFields), null, « », « undefined »).
        let additional_fields_copy = temporal::snapshot_own_properties(
            &additional_fields,
            Some(Vec::new()),
            Some(Vec::from([JsValue::undefined()])),
            context,
        )?;

        // 5. NOTE: Every property of fieldsCopy and additionalFieldsCopy is an enumerable data property with non-undefined value, but some property keys may be Symbols.
        // 6. Let additionalKeys be ! additionalFieldsCopy.[[OwnPropertyKeys]]().
        let add_keys = additional_fields_copy
            .__own_property_keys__(context)?
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        // 7. If calendar.[[Identifier]] is "iso8601", then
        // a. Let overriddenKeys be ISOFieldKeysToIgnore(additionalKeys).
        // 8. Else,
        // a. Let overriddenKeys be CalendarFieldKeysToIgnore(calendar, additionalKeys).
        let overridden_keys = protocol.field_keys_to_ignore(add_keys);

        // 9. Let merged be OrdinaryObjectCreate(null).
        let merged = JsObject::with_null_proto();

        // 10. NOTE: The following steps ensure that property iteration order of merged
        // matches that of fields as modified by omitting overridden properties and
        // appending non-overlapping properties from additionalFields in iteration order.
        // 11. Let fieldsKeys be ! fieldsCopy.[[OwnPropertyKeys]]().
        let field_keys = fields_copy
            .__own_property_keys__(context)?
            .iter()
            .map(|k| JsString::from(k.to_string()))
            .collect::<Vec<_>>();

        // 12. For each element key of fieldsKeys, do
        for key in field_keys {
            // a. Let propValue be undefined.
            // b. If overriddenKeys contains key, then
            let prop_value = if overridden_keys.contains(&key.to_std_string_escaped()) {
                // i. Set propValue to ! Get(additionalFieldsCopy, key).
                additional_fields_copy.get(key.as_slice(), context)?
            // c. Else,
            } else {
                // i. Set propValue to ! Get(fieldsCopy, key).
                fields_copy.get(key.as_slice(), context)?
            };

            // d. If propValue is not undefined, perform ! CreateDataPropertyOrThrow(merged, key, propValue).
            if !prop_value.is_undefined() {
                merged.create_data_property_or_throw(key.as_slice(), prop_value, context)?;
            }
        }

        // 13. Perform ! CopyDataProperties(merged, additionalFieldsCopy, « »).
        temporal::copy_data_properties(
            &merged,
            &additional_fields_copy.into(),
            &Vec::new(),
            None,
            context,
        )?;

        // 14. Return merged.
        Ok(merged.into())
    }
}

// -- `Calendar` Abstract Operations --

/// 12.2.1 `CreateTemporalCalendar ( identifier [ , newTarget ] )`
pub(crate) fn create_temporal_calendar(
    identifier: CalendarSlot,
    new_target: Option<JsValue>,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. Assert: IsBuiltinCalendar(identifier) is true.
    // 2. If newTarget is not provided, set newTarget to %Temporal.Calendar%.
    let new_target = new_target.unwrap_or_else(|| {
        context
            .realm()
            .intrinsics()
            .constructors()
            .calendar()
            .constructor()
            .into()
    });

    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Calendar.prototype%", « [[InitializedTemporalCalendar]], [[Identifier]] »).
    let proto =
        get_prototype_from_constructor(&new_target, StandardConstructors::calendar, context)?;

    let obj = JsObject::from_proto_and_data(proto, Calendar::new(identifier));

    // 4. Set object.[[Identifier]] to the ASCII-lowercase of identifier.
    // 5. Return object.
    Ok(obj.into())
}

fn extract_from_temporal_type<DF, DTF, YMF, MDF, ZDTF, Ret>(
    object: &JsObject,
    date_f: DF,
    datetime_f: DTF,
    year_month_f: YMF,
    month_day_f: MDF,
    zoned_datetime_f: ZDTF,
) -> JsResult<Option<Ret>>
where
    DF: FnOnce(&PlainDate) -> JsResult<Option<Ret>>,
    DTF: FnOnce(&PlainDateTime) -> JsResult<Option<Ret>>,
    YMF: FnOnce(&PlainYearMonth) -> JsResult<Option<Ret>>,
    MDF: FnOnce(&PlainMonthDay) -> JsResult<Option<Ret>>,
    ZDTF: FnOnce(&ZonedDateTime) -> JsResult<Option<Ret>>,
{
    let o = object.borrow();

    if let Some(date) = o.downcast_ref::<PlainDate>() {
        return date_f(date);
    } else if let Some(dt) = o.downcast_ref::<PlainDateTime>() {
        return datetime_f(dt);
    } else if let Some(ym) = o.downcast_ref::<PlainYearMonth>() {
        return year_month_f(ym);
    } else if let Some(md) = o.downcast_ref::<PlainMonthDay>() {
        return month_day_f(md);
    } else if let Some(dt) = o.downcast_ref::<ZonedDateTime>() {
        return zoned_datetime_f(dt);
    }

    Ok(None)
}

/// 12.2.21 `GetTemporalCalendarSlotValueWithISODefault ( item )`
#[allow(unused)]
pub(crate) fn get_temporal_calendar_slot_value_with_default(
    item: &JsObject,
    context: &mut Context,
) -> JsResult<CalendarSlot> {
    // 1. If item has an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]], [[InitializedTemporalMonthDay]], [[InitializedTemporalYearMonth]], or [[InitializedTemporalZonedDateTime]] internal slot, then
    // a. Return item.[[Calendar]].
    if let Some(calendar) = extract_from_temporal_type(
        item,
        |d| Ok(Some(d.inner.calendar().clone())),
        |dt| Ok(Some(dt.inner.calendar().clone())),
        |ym| Ok(Some(ym.inner.calendar().clone())),
        |md| Ok(Some(md.inner.calendar().clone())),
        |zdt| {
            Err(JsNativeError::range()
                .with_message("Not yet implemented.")
                .into())
        },
    )? {
        return Ok(calendar);
    }

    // 2. Let calendarLike be ? Get(item, "calendar").
    let calendar_like = item.get(utf16!("calendar"), context)?;

    // 3. Return ? ToTemporalCalendarSlotValue(calendarLike, "iso8601").
    to_temporal_calendar_slot_value(&calendar_like, context)
}

/// `12.2.20 ToTemporalCalendarSlotValue ( temporalCalendarLike [ , default ] )`
pub(crate) fn to_temporal_calendar_slot_value(
    calendar_like: &JsValue,
    context: &mut Context,
) -> JsResult<CalendarSlot> {
    // 1. If temporalCalendarLike is undefined and default is present, then
    // a. Assert: IsBuiltinCalendar(default) is true.
    // b. Return default.
    if calendar_like.is_undefined() {
        return Ok(CalendarSlot::Identifier("iso8601".to_owned()));
    // 2. If Type(temporalCalendarLike) is Object, then
    } else if let Some(calendar_like) = calendar_like.as_object() {
        // a. If temporalCalendarLike has an [[InitializedTemporalDate]], [[InitializedTemporalDateTime]], [[InitializedTemporalMonthDay]], [[InitializedTemporalYearMonth]], or [[InitializedTemporalZonedDateTime]] internal slot, then
        // i. Return temporalCalendarLike.[[Calendar]].
        if let Some(calendar) = extract_from_temporal_type(
            calendar_like,
            |d| Ok(Some(d.inner.calendar().clone())),
            |_dt| {
                Err(JsNativeError::range()
                    .with_message("Not yet implemented.")
                    .into())
            },
            |_ym| {
                Err(JsNativeError::range()
                    .with_message("Not yet implemented.")
                    .into())
            },
            |_md| {
                Err(JsNativeError::range()
                    .with_message("Not yet implemented.")
                    .into())
            },
            |_zdt| {
                Err(JsNativeError::range()
                    .with_message("Not yet implemented.")
                    .into())
            },
        )? {
            return Ok(calendar);
        }

        // TODO: implement ObjectImplementsTemporalCalendarProtocol
        // b. If ? ObjectImplementsTemporalCalendarProtocol(temporalCalendarLike) is false, throw a TypeError exception.
        if !object_implements_calendar_protocol(calendar_like, context) {
            return Err(JsNativeError::typ()
                .with_message("CalendarLike does not implement the CalendarProtocol.")
                .into());
        }

        // Types: Box<dyn CalendarProtocol> <- UserCalendar
        let protocol = Box::new(CustomRuntimeCalendar::new(calendar_like));
        // c. Return temporalCalendarLike.
        return Ok(CalendarSlot::Protocol(protocol));
    }

    // 3. If temporalCalendarLike is not a String, throw a TypeError exception.
    if !calendar_like.is_string() {
        return Err(JsNativeError::typ()
            .with_message("temporalCalendarLike is not a string.")
            .into());
    }

    // TODO: 4-6
    // 4. Let identifier be ? ParseTemporalCalendarString(temporalCalendarLike).
    // 5. If IsBuiltinCalendar(identifier) is false, throw a RangeError exception.
    // 6. Return the ASCII-lowercase of identifier.
    Ok(CalendarSlot::Identifier("iso8601".to_owned()))
}

fn object_implements_calendar_protocol(calendar_like: &JsObject, context: &mut Context) -> bool {
    CALENDAR_PROTOCOL_METHODS.into_iter().all(|method| {
        calendar_like
            .__has_property__(&JsString::from(method).into(), &mut context.into())
            .unwrap_or(false)
    })
}

/// Utility function for taking a `JsValue` and converting it to a temporal library `CalendarDateLike` enum.
fn to_calendar_date_like(date_like: &JsValue, context: &mut Context) -> JsResult<CalendarDateLike> {
    let Some(obj) = date_like.as_object() else {
        let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;

        return Ok(CalendarDateLike::Date(date.inner.clone()));
    };

    let Some(date_like) = extract_from_temporal_type(
        obj,
        |d| Ok(Some(CalendarDateLike::Date(d.inner.clone()))),
        |dt| Ok(Some(CalendarDateLike::DateTime(dt.inner.clone()))),
        |ym| Ok(Some(CalendarDateLike::YearMonth(ym.inner.clone()))),
        |_| Ok(None),
        |_| Ok(None),
    )?
    else {
        let date = temporal::plain_date::to_temporal_date(date_like, None, context)?;

        return Ok(CalendarDateLike::Date(date.inner.clone()));
    };

    Ok(date_like)
}
