//! Boa's implementation of the ECMAScript `Temporal.PlainDateTime` builtin object.
#![allow(dead_code, unused_variables)]

use crate::{
    builtins::{
        temporal::{calendar, to_integer_with_truncation},
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
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

#[cfg(test)]
mod tests;

use temporal_rs::{
    components::{
        calendar::{CalendarSlot, GetCalendarSlot},
        DateTime as InnerDateTime,
    },
    iso::{IsoDate, IsoDateSlots},
};

/// The `Temporal.PlainDateTime` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)] // TODO: Remove this!!! `InnerDateTime` could contain `Trace` types.
pub struct PlainDateTime {
    pub(crate) inner: InnerDateTime<JsObject>,
}

impl PlainDateTime {
    fn new(inner: InnerDateTime<JsObject>) -> Self {
        Self { inner }
    }

    pub(crate) fn inner(&self) -> &InnerDateTime<JsObject> {
        &self.inner
    }
}

impl IsoDateSlots for JsObject<PlainDateTime> {
    fn iso_date(&self) -> IsoDate {
        self.borrow().data().inner.iso_date()
    }
}

impl GetCalendarSlot<JsObject> for JsObject<PlainDateTime> {
    fn get_calendar(&self) -> CalendarSlot<JsObject> {
        self.borrow().data().inner.get_calendar()
    }
}

impl BuiltInObject for PlainDateTime {
    const NAME: JsString = StaticJsStrings::PLAIN_DATETIME;
}

impl IntrinsicObject for PlainDateTime {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_calendar_id = BuiltInBuilder::callable(realm, Self::get_calendar_id)
            .name(js_string!("get calendarId"))
            .build();

        let get_year = BuiltInBuilder::callable(realm, Self::get_year)
            .name(js_string!("get year"))
            .build();

        let get_month = BuiltInBuilder::callable(realm, Self::get_month)
            .name(js_string!("get month"))
            .build();

        let get_month_code = BuiltInBuilder::callable(realm, Self::get_month_code)
            .name(js_string!("get monthCode"))
            .build();

        let get_day = BuiltInBuilder::callable(realm, Self::get_day)
            .name(js_string!("get day"))
            .build();

        let get_hour = BuiltInBuilder::callable(realm, Self::get_hour)
            .name(js_string!("get hour"))
            .build();

        let get_minute = BuiltInBuilder::callable(realm, Self::get_minute)
            .name(js_string!("get minute"))
            .build();

        let get_second = BuiltInBuilder::callable(realm, Self::get_second)
            .name(js_string!("get second"))
            .build();

        let get_millisecond = BuiltInBuilder::callable(realm, Self::get_millisecond)
            .name(js_string!("get millisecond"))
            .build();

        let get_microsecond = BuiltInBuilder::callable(realm, Self::get_microsecond)
            .name(js_string!("get microsecond"))
            .build();

        let get_nanosecond = BuiltInBuilder::callable(realm, Self::get_nanosecond)
            .name(js_string!("get nanosecond"))
            .build();

        let get_day_of_week = BuiltInBuilder::callable(realm, Self::get_day_of_week)
            .name(js_string!("get dayOfWeek"))
            .build();

        let get_day_of_year = BuiltInBuilder::callable(realm, Self::get_day_of_year)
            .name(js_string!("get dayOfYear"))
            .build();

        let get_week_of_year = BuiltInBuilder::callable(realm, Self::get_week_of_year)
            .name(js_string!("get weekOfYear"))
            .build();

        let get_year_of_week = BuiltInBuilder::callable(realm, Self::get_year_of_week)
            .name(js_string!("get yearOfWeek"))
            .build();

        let get_days_in_week = BuiltInBuilder::callable(realm, Self::get_days_in_week)
            .name(js_string!("get daysInWeek"))
            .build();

        let get_days_in_month = BuiltInBuilder::callable(realm, Self::get_days_in_month)
            .name(js_string!("get daysInMonth"))
            .build();

        let get_days_in_year = BuiltInBuilder::callable(realm, Self::get_days_in_year)
            .name(js_string!("get daysInYear"))
            .build();

        let get_months_in_year = BuiltInBuilder::callable(realm, Self::get_months_in_year)
            .name(js_string!("get monthsInYear"))
            .build();

        let get_in_leap_year = BuiltInBuilder::callable(realm, Self::get_in_leap_year)
            .name(js_string!("get inLeapYear"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("calendarId"),
                Some(get_calendar_id),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("year"),
                Some(get_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("month"),
                Some(get_month),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("monthCode"),
                Some(get_month_code),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(utf16!("day"), Some(get_day), None, Attribute::CONFIGURABLE)
            .accessor(
                utf16!("hour"),
                Some(get_hour),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("minute"),
                Some(get_minute),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("second"),
                Some(get_second),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("millisecond"),
                Some(get_millisecond),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("microsecond"),
                Some(get_microsecond),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("nanosecond"),
                Some(get_nanosecond),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("dayOfWeek"),
                Some(get_day_of_week),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("dayOfYear"),
                Some(get_day_of_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("weekOfYear"),
                Some(get_week_of_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("yearOfWeek"),
                Some(get_year_of_week),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("daysInWeek"),
                Some(get_days_in_week),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("daysInMonth"),
                Some(get_days_in_month),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("daysInYear"),
                Some(get_days_in_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("monthsInYear"),
                Some(get_months_in_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("inLeapYear"),
                Some(get_in_leap_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for PlainDateTime {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_date_time;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, then
        if new_target.is_undefined() {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message("NewTarget cannot be undefined when contructing PlainDateTime.")
                .into());
        };

        // 2. Set isoYear to ? ToIntegerWithTruncation(isoYear).
        let iso_year = to_integer_with_truncation(args.get_or_undefined(0), context)?;
        // 3. Set isoMonth to ? ToIntegerWithTruncation(isoMonth).
        let iso_month = to_integer_with_truncation(args.get_or_undefined(1), context)?;
        // 4. Set isoDay to ? ToIntegerWithTruncation(isoDay).
        let iso_day = to_integer_with_truncation(args.get_or_undefined(2), context)?;
        // 5. If hour is undefined, set hour to 0; else set hour to ? ToIntegerWithTruncation(hour).
        let hour = args
            .get(3)
            .map_or(Ok(0), |v| to_integer_with_truncation(v, context))?;
        // 6. If minute is undefined, set minute to 0; else set minute to ? ToIntegerWithTruncation(minute).
        let minute = args
            .get(4)
            .map_or(Ok(0), |v| to_integer_with_truncation(v, context))?;
        // 7. If second is undefined, set second to 0; else set second to ? ToIntegerWithTruncation(second).
        let second = args
            .get(5)
            .map_or(Ok(0), |v| to_integer_with_truncation(v, context))?;
        // 8. If millisecond is undefined, set millisecond to 0; else set millisecond to ? ToIntegerWithTruncation(millisecond).
        let millisecond = args
            .get(6)
            .map_or(Ok(0), |v| to_integer_with_truncation(v, context))?;
        // 9. If microsecond is undefined, set microsecond to 0; else set microsecond to ? ToIntegerWithTruncation(microsecond).
        let microsecond = args
            .get(7)
            .map_or(Ok(0), |v| to_integer_with_truncation(v, context))?;
        // 10. If nanosecond is undefined, set nanosecond to 0; else set nanosecond to ? ToIntegerWithTruncation(nanosecond).
        let nanosecond = args
            .get(8)
            .map_or(Ok(0), |v| to_integer_with_truncation(v, context))?;
        // 11. Let calendar be ? ToTemporalCalendarSlotValue(calendarLike, "iso8601").
        let calendar_slot =
            calendar::to_temporal_calendar_slot_value(args.get_or_undefined(9), context)?;

        let dt = InnerDateTime::new(
            iso_year,
            iso_month,
            iso_day,
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            calendar_slot,
        )?;

        // 12. Return ? CreateTemporalDateTime(isoYear, isoMonth, isoDay, hour, minute, second, millisecond, microsecond, nanosecond, calendar, NewTarget).
        create_temporal_datetime(dt, Some(new_target), context).map(Into::into)
    }
}

// ==== `PlainDateTimeTime` accessor implmentations ====

impl PlainDateTime {
    /// 5.3.3 get `Temporal.PlainDateTime.prototype.calendarId`
    fn get_calendar_id(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let date = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(JsString::from(date.inner.calendar().identifier(context)?).into())
    }

    /// 5.3.4 get `Temporal.PlainDateTime.prototype.year`
    fn get_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_year(&date, context)?.into())
    }

    /// 5.3.5 get `Temporal.PlainDateTime.prototype.month`
    fn get_month(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_month(&date, context)?.into())
    }

    /// 5.3.6 get Temporal.PlainDateTime.prototype.monthCode
    fn get_month_code(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(JsString::from(
            InnerDateTime::<JsObject>::contextual_month_code(&date, context)?.as_str(),
        )
        .into())
    }

    /// 5.3.7 get `Temporal.PlainDateTime.prototype.day`
    fn get_day(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_day(&date, context)?.into())
    }

    /// 5.3.8 get `Temporal.PlainDateTime.prototype.hour`
    fn get_hour(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(dateTime.[[ISOHour]]).
        Ok(time.inner.hour().into())
    }

    /// 5.3.9 get `Temporal.PlainDateTime.prototype.minute`
    fn get_minute(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(dateTime.[[ISOMinute]]).
        Ok(time.inner.minute().into())
    }

    /// 5.3.10 get `Temporal.PlainDateTime.prototype.second`
    fn get_second(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(dateTime.[[ISOSecond]]).
        Ok(time.inner.second().into())
    }

    /// 5.3.11 get `Temporal.PlainDateTime.prototype.millisecond`
    fn get_millisecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(dateTime.[[ISOMillisecond]]).
        Ok(time.inner.millisecond().into())
    }

    /// 5.3.12 get `Temporal.PlainDateTime.prototype.microsecond`
    fn get_microsecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(dateTime.[[ISOMicrosecond]]).
        Ok(time.inner.microsecond().into())
    }

    /// 5.3.13 get `Temporal.PlainDateTime.prototype.nanosecond`
    fn get_nanosecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(dateTime.[[ISONanosecond]]).
        Ok(time.inner.nanosecond().into())
    }

    /// 5.3.14 get `Temporal.PlainDateTime.prototype.dayOfWeek`
    fn get_day_of_week(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_day_of_week(&date, context)?.into())
    }

    /// 5.3.15 get `Temporal.PlainDateTime.prototype.dayOfYear`
    fn get_day_of_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_day_of_year(&date, context)?.into())
    }

    /// 5.3.16 get `Temporal.PlainDateTime.prototype.weekOfYear`
    fn get_week_of_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_week_of_year(&date, context)?.into())
    }

    /// 5.3.17 get `Temporal.PlainDateTime.prototype.yearOfWeek`
    fn get_year_of_week(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_year_of_week(&date, context)?.into())
    }

    /// 5.3.18 get `Temporal.PlainDateTime.prototype.daysInWeek`
    fn get_days_in_week(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_days_in_week(&date, context)?.into())
    }

    /// 5.3.19 get `Temporal.PlainDateTime.prototype.daysInMonth`
    fn get_days_in_month(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_days_in_month(&date, context)?.into())
    }

    /// 5.3.20 get `Temporal.PlainDateTime.prototype.daysInYear`
    fn get_days_in_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_days_in_year(&date, context)?.into())
    }

    /// 5.3.21 get `Temporal.PlainDateTime.prototype.monthsInYear`
    fn get_months_in_year(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_months_in_year(&date, context)?.into())
    }

    /// 5.3.22 get `Temporal.PlainDateTime.prototype.inLeapYear`
    fn get_in_leap_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("this must be an object."))?;

        let Ok(date) = obj.clone().downcast::<Self>() else {
            return Err(JsNativeError::typ()
                .with_message("the this object must be a PlainDateTime object.")
                .into());
        };

        Ok(InnerDateTime::<JsObject>::contextual_in_leap_year(&date, context)?.into())
    }
}

// ==== `PlainDateTime` Abstract Operations` ====

pub(crate) fn create_temporal_datetime(
    inner: InnerDateTime<JsObject>,
    new_target: Option<&JsValue>,
    context: &mut Context,
) -> JsResult<JsObject> {
    // NOTE(nekevss): The below validations should be upheld with the creation of `InnerDateTime`.
    // 1. If IsValidISODate(isoYear, isoMonth, isoDay) is false, throw a RangeError exception.
    // 2. If IsValidTime(hour, minute, second, millisecond, microsecond, nanosecond) is false, throw a RangeError exception.
    // 3. If ISODateTimeWithinLimits(isoYear, isoMonth, isoDay, hour, minute, second, millisecond, microsecond, nanosecond) is false, then
    // a. Throw a RangeError exception.

    // 4. If newTarget is not present, set newTarget to %Temporal.PlainDateTime%.
    let new_target = if let Some(new_target) = new_target {
        new_target.clone()
    } else {
        context
            .realm()
            .intrinsics()
            .constructors()
            .plain_date_time()
            .constructor()
            .into()
    };

    // 5. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.PlainDateTime.prototype%", « [[InitializedTemporalDateTime]], [[ISOYear]], [[ISOMonth]], [[ISODay]], [[ISOHour]], [[ISOMinute]], [[ISOSecond]], [[ISOMillisecond]], [[ISOMicrosecond]], [[ISONanosecond]], [[Calendar]] »).
    let prototype = get_prototype_from_constructor(
        &new_target,
        StandardConstructors::plain_date_time,
        context,
    )?;

    // 6. Set object.[[ISOYear]] to isoYear.
    // 7. Set object.[[ISOMonth]] to isoMonth.
    // 8. Set object.[[ISODay]] to isoDay.
    // 9. Set object.[[ISOHour]] to hour.
    // 10. Set object.[[ISOMinute]] to minute.
    // 11. Set object.[[ISOSecond]] to second.
    // 12. Set object.[[ISOMillisecond]] to millisecond.
    // 13. Set object.[[ISOMicrosecond]] to microsecond.
    // 14. Set object.[[ISONanosecond]] to nanosecond.
    // 15. Set object.[[Calendar]] to calendar.
    let obj = JsObject::from_proto_and_data(prototype, PlainDateTime::new(inner));

    // 16. Return object.
    Ok(obj)
}
