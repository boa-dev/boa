//! Boa's implementation of the ECMAScript `Temporal.PlainDateTime` builtin object.
#![allow(dead_code, unused_variables)]

use crate::{
    builtins::{
        options::{get_option, get_options_object},
        temporal::{to_partial_date_record, to_partial_time_record},
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::IntoOrUndefined,
    Context, JsArgs, JsData, JsError, JsNativeError, JsObject, JsResult, JsString, JsSymbol,
    JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

#[cfg(test)]
mod tests;

use temporal_rs::{
    options::{
        ArithmeticOverflow, DisplayCalendar, RoundingIncrement, RoundingOptions,
        TemporalRoundingMode, TemporalUnit, ToStringRoundingOptions,
    },
    partial::{PartialDate, PartialDateTime, PartialTime},
    Calendar, PlainDateTime as InnerDateTime, TinyAsciiStr,
};

use super::{
    calendar::{get_temporal_calendar_slot_value_with_default, to_temporal_calendar_slot_value},
    create_temporal_duration,
    options::{get_difference_settings, get_digits_option, get_temporal_unit, TemporalUnitGroup},
    to_temporal_duration_record, to_temporal_time, PlainDate, ZonedDateTime,
};
use crate::value::JsVariant;

/// The `Temporal.PlainDateTime` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)] // TODO: Remove this!!! `InnerDateTime` could contain `Trace` types.
pub struct PlainDateTime {
    pub(crate) inner: InnerDateTime,
}

impl PlainDateTime {
    fn new(inner: InnerDateTime) -> Self {
        Self { inner }
    }

    pub(crate) fn inner(&self) -> &InnerDateTime {
        &self.inner
    }
}

impl BuiltInObject for PlainDateTime {
    const NAME: JsString = StaticJsStrings::PLAIN_DATETIME_NAME;
}

impl IntrinsicObject for PlainDateTime {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_calendar_id = BuiltInBuilder::callable(realm, Self::get_calendar_id)
            .name(js_string!("get calendarId"))
            .build();

        let get_era = BuiltInBuilder::callable(realm, Self::get_era)
            .name(js_string!("get era"))
            .build();

        let get_era_year = BuiltInBuilder::callable(realm, Self::get_era_year)
            .name(js_string!("get eraYear"))
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
                StaticJsStrings::PLAIN_DATETIME_TAG,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("calendarId"),
                Some(get_calendar_id),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("era"),
                Some(get_era),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("eraYear"),
                Some(get_era_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("year"),
                Some(get_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("month"),
                Some(get_month),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("monthCode"),
                Some(get_month_code),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("day"),
                Some(get_day),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("hour"),
                Some(get_hour),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("minute"),
                Some(get_minute),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("second"),
                Some(get_second),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("millisecond"),
                Some(get_millisecond),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("microsecond"),
                Some(get_microsecond),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("nanosecond"),
                Some(get_nanosecond),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("dayOfWeek"),
                Some(get_day_of_week),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("dayOfYear"),
                Some(get_day_of_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("weekOfYear"),
                Some(get_week_of_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("yearOfWeek"),
                Some(get_year_of_week),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("daysInWeek"),
                Some(get_days_in_week),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("daysInMonth"),
                Some(get_days_in_month),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("daysInYear"),
                Some(get_days_in_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("monthsInYear"),
                Some(get_months_in_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("inLeapYear"),
                Some(get_in_leap_year),
                None,
                Attribute::CONFIGURABLE,
            )
            .static_method(Self::from, js_string!("from"), 1)
            .static_method(Self::compare, js_string!("compare"), 2)
            .method(Self::with, js_string!("with"), 1)
            .method(Self::with_plain_time, js_string!("withPlainTime"), 1)
            .method(Self::with_calendar, js_string!("withCalendar"), 1)
            .method(Self::add, js_string!("add"), 1)
            .method(Self::subtract, js_string!("subtract"), 1)
            .method(Self::until, js_string!("until"), 1)
            .method(Self::since, js_string!("since"), 1)
            .method(Self::round, js_string!("round"), 1)
            .method(Self::equals, js_string!("equals"), 1)
            .method(Self::to_string, js_string!("toString"), 0)
            .method(Self::to_locale_string, js_string!("toLocaleString"), 0)
            .method(Self::to_json, js_string!("toJSON"), 0)
            .method(Self::value_of, js_string!("valueOf"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for PlainDateTime {
    const LENGTH: usize = 3;
    const P: usize = 29;
    const SP: usize = 2;

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
                .with_message("NewTarget cannot be undefined when contructing PlainDatedt.")
                .into());
        };

        // 2. Set isoYear to ? ToIntegerWithTruncation(isoYear).
        let iso_year = args
            .get_or_undefined(0)
            .to_finitef64(context)?
            .as_integer_with_truncation::<i32>();
        // 3. Set isoMonth to ? ToIntegerWithTruncation(isoMonth).
        let iso_month = args
            .get_or_undefined(1)
            .to_finitef64(context)?
            .as_integer_with_truncation::<u8>();
        // 4. Set isoDay to ? ToIntegerWithTruncation(isoDay).
        let iso_day = args
            .get_or_undefined(2)
            .to_finitef64(context)?
            .as_integer_with_truncation::<u8>();
        // 5. If hour is undefined, set hour to 0; else set hour to ? ToIntegerWithTruncation(hour).
        let hour = args.get_or_undefined(3).map_or(Ok::<u8, JsError>(0), |v| {
            let finite = v.to_finitef64(context)?;
            Ok(finite.as_integer_with_truncation::<u8>())
        })?;
        // 6. If minute is undefined, set minute to 0; else set minute to ? ToIntegerWithTruncation(minute).
        let minute = args.get_or_undefined(4).map_or(Ok::<u8, JsError>(0), |v| {
            let finite = v.to_finitef64(context)?;
            Ok(finite.as_integer_with_truncation::<u8>())
        })?;

        // 7. If second is undefined, set second to 0; else set second to ? ToIntegerWithTruncation(second).
        let second = args.get_or_undefined(5).map_or(Ok::<u8, JsError>(0), |v| {
            let finite = v.to_finitef64(context)?;
            Ok(finite.as_integer_with_truncation::<u8>())
        })?;

        // 8. If millisecond is undefined, set millisecond to 0; else set millisecond to ? ToIntegerWithTruncation(millisecond).
        let millisecond = args
            .get_or_undefined(6)
            .map_or(Ok::<u16, JsError>(0), |v| {
                let finite = v.to_finitef64(context)?;
                Ok(finite.as_integer_with_truncation::<u16>())
            })?;

        // 9. If microsecond is undefined, set microsecond to 0; else set microsecond to ? ToIntegerWithTruncation(microsecond).
        let microsecond = args
            .get_or_undefined(7)
            .map_or(Ok::<u16, JsError>(0), |v| {
                let finite = v.to_finitef64(context)?;
                Ok(finite.as_integer_with_truncation::<u16>())
            })?;

        // 10. If nanosecond is undefined, set nanosecond to 0; else set nanosecond to ? ToIntegerWithTruncation(nanosecond).
        let nanosecond = args
            .get_or_undefined(8)
            .map_or(Ok::<u16, JsError>(0), |v| {
                let finite = v.to_finitef64(context)?;
                Ok(finite.as_integer_with_truncation::<u16>())
            })?;

        let calendar_slot = args
            .get_or_undefined(9)
            .map(|s| {
                s.as_string()
                    .map(JsString::to_std_string_lossy)
                    .ok_or_else(|| JsNativeError::typ().with_message("calendar must be a string."))
            })
            .transpose()?
            .map(|s| Calendar::from_utf8(s.as_bytes()))
            .transpose()?
            .unwrap_or_default();

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
    /// 5.3.3 get `Temporal.PlainDatedt.prototype.calendarId`
    fn get_calendar_id(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(JsString::from(dt.inner.calendar().identifier()).into())
    }

    /// 5.3.4 get `Temporal.PlainDatedt.prototype.year`
    fn get_era(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt
            .inner
            .era()?
            .map(|s| JsString::from(s.as_str()))
            .into_or_undefined())
    }

    /// 5.3.5 get `Temporal.PlainDatedt.prototype.eraYear`
    fn get_era_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.era_year()?.into_or_undefined())
    }

    /// 5.3.6 get `Temporal.PlainDatedt.prototype.year`
    fn get_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.year()?.into())
    }

    /// 5.3.7 get `Temporal.PlainDatedt.prototype.month`
    fn get_month(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.month()?.into())
    }

    /// 5.3.8 get Temporal.PlainDatedt.prototype.monthCode
    fn get_month_code(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(JsString::from(dt.inner.month_code()?.as_str()).into())
    }

    /// 5.3.9 get `Temporal.PlainDatedt.prototype.day`
    fn get_day(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.day()?.into())
    }

    /// 5.3.10 get `Temporal.PlainDatedt.prototype.hour`
    fn get_hour(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(datedt.[[ISOHour]]).
        Ok(dt.inner.hour().into())
    }

    /// 5.3.11 get `Temporal.PlainDatedt.prototype.minute`
    fn get_minute(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(datedt.[[ISOMinute]]).
        Ok(dt.inner.minute().into())
    }

    /// 5.3.12 get `Temporal.PlainDatedt.prototype.second`
    fn get_second(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(datedt.[[ISOSecond]]).
        Ok(dt.inner.second().into())
    }

    /// 5.3.13 get `Temporal.PlainDatedt.prototype.millisecond`
    fn get_millisecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(datedt.[[ISOMillisecond]]).
        Ok(dt.inner.millisecond().into())
    }

    /// 5.3.14 get `Temporal.PlainDatedt.prototype.microsecond`
    fn get_microsecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(datedt.[[ISOMicrosecond]]).
        Ok(dt.inner.microsecond().into())
    }

    /// 5.3.15 get `Temporal.PlainDatedt.prototype.nanosecond`
    fn get_nanosecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Return 𝔽(datedt.[[ISONanosecond]]).
        Ok(dt.inner.nanosecond().into())
    }

    /// 5.3.16 get `Temporal.PlainDatedt.prototype.dayOfWeek`
    fn get_day_of_week(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.day_of_week()?.into())
    }

    /// 5.3.17 get `Temporal.PlainDatedt.prototype.dayOfYear`
    fn get_day_of_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.day_of_year()?.into())
    }

    /// 5.3.18 get `Temporal.PlainDatedt.prototype.weekOfYear`
    fn get_week_of_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.week_of_year()?.into_or_undefined())
    }

    /// 5.3.19 get `Temporal.PlainDatedt.prototype.yearOfWeek`
    fn get_year_of_week(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.year_of_week()?.into_or_undefined())
    }

    /// 5.3.20 get `Temporal.PlainDatedt.prototype.daysInWeek`
    fn get_days_in_week(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.days_in_week()?.into())
    }

    /// 5.3.21 get `Temporal.PlainDatedt.prototype.daysInMonth`
    fn get_days_in_month(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.days_in_month()?.into())
    }

    /// 5.3.22 get `Temporal.PlainDatedt.prototype.daysInYear`
    fn get_days_in_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.days_in_year()?.into())
    }

    /// 5.3.23 get `Temporal.PlainDatedt.prototype.monthsInYear`
    fn get_months_in_year(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.months_in_year()?.into())
    }

    /// 5.3.24 get `Temporal.PlainDatedt.prototype.inLeapYear`
    fn get_in_leap_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        Ok(dt.inner.in_leap_year()?.into())
    }
}

// ==== PlainDateTime method implemenations ====

impl PlainDateTime {
    /// 5.2.2 Temporal.PlainDateTime.from ( item [ , options ] )
    fn from(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let item = args.get_or_undefined(0);
        // 1. Set options to ? GetOptionsObject(options).
        let options = args.get(1);
        // 2. If item is an Object and item has an [[InitializedTemporalDateTime]] internal slot, then
        let dt = if let Some(pdt) = item.as_object().and_then(JsObject::downcast_ref::<Self>) {
            // a. Perform ? GetTemporalOverflowOption(options).
            let options = get_options_object(args.get_or_undefined(1))?;
            let _ = get_option::<ArithmeticOverflow>(&options, js_string!("overflow"), context)?;
            // b. Return ! CreateTemporalDateTime(item.[[ISOYear]], item.[[ISOMonth]],
            // item.[[ISODay]], item.[[ISOHour]], item.[[ISOMinute]], item.[[ISOSecond]],
            // item.[[ISOMillisecond]], item.[[ISOMicrosecond]], item.[[ISONanosecond]],
            // item.[[Calendar]]).
            pdt.inner.clone()
        } else {
            to_temporal_datetime(item, options.cloned(), context)?
        };

        // 3. Return ? ToTemporalDateTime(item, options).
        create_temporal_datetime(dt, None, context).map(Into::into)
    }

    /// 5.2.3 Temporal.PlainDateTime.compare ( one, two )
    fn compare(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Set one to ? ToTemporalDateTime(one).
        let one = to_temporal_datetime(args.get_or_undefined(0), None, context)?;
        // 2. Set two to ? ToTemporalDateTime(two).
        let two = to_temporal_datetime(args.get_or_undefined(1), None, context)?;

        // 3. Return 𝔽(CompareISODateTime(one.[[ISOYear]], one.[[ISOMonth]], one.[[ISODay]],
        // one.[[ISOHour]], one.[[ISOMinute]], one.[[ISOSecond]], one.[[ISOMillisecond]],
        // one.[[ISOMicrosecond]], one.[[ISONanosecond]], two.[[ISOYear]], two.[[ISOMonth]],
        // two.[[ISODay]], two.[[ISOHour]], two.[[ISOMinute]], two.[[ISOSecond]],
        // two.[[ISOMillisecond]], two.[[ISOMicrosecond]], two.[[ISONanosecond]])).
        Ok((one.compare_iso(&two) as i8).into())
    }
}

// ==== PlainDateTime.prototype method implementations ====

impl PlainDateTime {
    ///  5.3.25 Temporal.PlainDateTime.prototype.with ( temporalDateTimeLike [ , options ] )
    fn with(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        let Some(partial_object) =
            super::is_partial_temporal_object(args.get_or_undefined(0), context)?
        else {
            return Err(JsNativeError::typ()
                .with_message("with object was not a PartialTemporalObject.")
                .into());
        };

        let date = to_partial_date_record(partial_object, context)?;
        let time = to_partial_time_record(partial_object, context)?;

        let partial_dt = PartialDateTime { date, time };

        let options = get_options_object(args.get_or_undefined(1))?;
        let overflow = get_option::<ArithmeticOverflow>(&options, js_string!("overflow"), context)?;

        create_temporal_datetime(dt.inner.with(partial_dt, overflow)?, None, context)
            .map(Into::into)
    }

    /// 5.3.26 Temporal.PlainDateTime.prototype.withPlainTime ( `[ plainTimeLike ]` )
    fn with_plain_time(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        let time = to_temporal_time(args.get_or_undefined(0), None, context)?;

        create_temporal_datetime(dt.inner.with_time(time)?, None, context).map(Into::into)
    }

    /// 5.3.27 Temporal.PlainDateTime.prototype.withCalendar ( calendarLike )
    fn with_calendar(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        let calendar = to_temporal_calendar_slot_value(args.get_or_undefined(0))?;

        create_temporal_datetime(dt.inner.with_calendar(calendar)?, None, context).map(Into::into)
    }

    /// 5.3.28 Temporal.PlainDateTime.prototype.add ( temporalDurationLike [ , options ] )
    fn add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalDate be the this value.
        // 2. Perform ? RequireInternalSlot(temporalDate, [[InitializedTemporalDate]]).
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Let duration be ? ToTemporalDuration(temporalDurationLike).
        let duration = to_temporal_duration_record(args.get_or_undefined(0), context)?;

        // 4. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;
        let overflow = get_option::<ArithmeticOverflow>(&options, js_string!("overflow"), context)?;

        // 5. Let calendarRec be ? CreateCalendarMethodsRecord(temporalDate.[[Calendar]], « date-add »).
        // 6. Return ? AddDate(calendarRec, temporalDate, duration, options).
        create_temporal_datetime(dt.inner.add(&duration, overflow)?, None, context).map(Into::into)
    }

    /// 5.3.29 Temporal.PlainDateTime.prototype.subtract ( temporalDurationLike [ , options ] )
    fn subtract(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalDate be the this value.
        // 2. Perform ? RequireInternalSlot(temporalDate, [[InitializedTemporalDate]]).
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Let duration be ? ToTemporalDuration(temporalDurationLike).
        let duration = to_temporal_duration_record(args.get_or_undefined(0), context)?;

        // 4. Set options to ? GetOptionsObject(options).
        let options = get_options_object(args.get_or_undefined(1))?;
        let overflow = get_option::<ArithmeticOverflow>(&options, js_string!("overflow"), context)?;

        // 5. Let negatedDuration be CreateNegatedTemporalDuration(duration).
        // 6. Let calendarRec be ? CreateCalendarMethodsRecord(temporalDate.[[Calendar]], « date-add »).
        // 7. Return ? AddDate(calendarRec, temporalDate, negatedDuration, options).
        create_temporal_datetime(dt.inner.subtract(&duration, overflow)?, None, context)
            .map(Into::into)
    }

    /// 5.3.30 Temporal.PlainDateTime.prototype.until ( other [ , options ] )
    fn until(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        let other = to_temporal_datetime(args.get_or_undefined(0), None, context)?;

        let options = get_options_object(args.get_or_undefined(1))?;
        let settings = get_difference_settings(&options, context)?;

        create_temporal_duration(dt.inner.until(&other, settings)?, None, context).map(Into::into)
    }

    /// 5.3.31 Temporal.PlainDateTime.prototype.since ( other [ , options ] )
    fn since(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        let other = to_temporal_datetime(args.get_or_undefined(0), None, context)?;

        let options = get_options_object(args.get_or_undefined(1))?;
        let settings = get_difference_settings(&options, context)?;

        create_temporal_duration(dt.inner.since(&other, settings)?, None, context).map(Into::into)
    }

    /// 5.3.32 Temporal.PlainDateTime.prototype.round ( roundTo )
    fn round(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        let round_to = match args.first().map(JsValue::variant) {
            // 3. If roundTo is undefined, then
            None | Some(JsVariant::Undefined) => {
                return Err(JsNativeError::typ()
                    .with_message("roundTo cannot be undefined.")
                    .into())
            }
            // 4. If Type(roundTo) is String, then
            Some(JsVariant::String(rt)) => {
                // a. Let paramString be roundTo.
                let param_string = rt.clone();
                // b. Set roundTo to OrdinaryObjectCreate(null).
                let new_round_to = JsObject::with_null_proto();
                // c. Perform ! CreateDataPropertyOrThrow(roundTo, "smallestUnit", paramString).
                new_round_to.create_data_property_or_throw(
                    js_string!("smallestUnit"),
                    param_string,
                    context,
                )?;
                new_round_to
            }
            // 5. Else,
            Some(round_to) => {
                // a. Set roundTo to ? GetOptionsObject(roundTo).
                get_options_object(&JsValue::from(round_to))?
            }
        };

        let (plain_relative_to, zoned_relative_to) =
            super::to_relative_temporal_object(&round_to, context)?;

        let mut options = RoundingOptions::default();

        options.increment =
            get_option::<RoundingIncrement>(&round_to, js_string!("roundingIncrement"), context)?;

        // 8. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        options.rounding_mode =
            get_option::<TemporalRoundingMode>(&round_to, js_string!("roundingMode"), context)?;

        // 9. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit", TIME, REQUIRED, undefined).
        options.smallest_unit = get_temporal_unit(
            &round_to,
            js_string!("smallestUnit"),
            TemporalUnitGroup::Time,
            Some(vec![TemporalUnit::Day]),
            context,
        )?;

        create_temporal_datetime(dt.inner().round(options)?, None, context).map(Into::into)
    }

    /// 5.3.33 Temporal.PlainDateTime.prototype.equals ( other )
    fn equals(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let dateTime be the this value.
        // 2. Perform ? RequireInternalSlot(dateTime, [[InitializedTemporalDateTime]]).
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        // 3. Set other to ? ToTemporalDateTime(other).
        let other = to_temporal_datetime(args.get_or_undefined(0), None, context)?;

        // 4. Let result be CompareISODateTime(dateTime.[[ISOYear]], dateTime.[[ISOMonth]],
        // dateTime.[[ISODay]], dateTime.[[ISOHour]], dateTime.[[ISOMinute]],
        // dateTime.[[ISOSecond]], dateTime.[[ISOMillisecond]], dateTime.[[ISOMicrosecond]],
        // dateTime.[[ISONanosecond]], other.[[ISOYear]], other.[[ISOMonth]], other.[[ISODay]],
        // other.[[ISOHour]], other.[[ISOMinute]], other.[[ISOSecond]], other.[[ISOMillisecond]],
        // other.[[ISOMicrosecond]], other.[[ISONanosecond]]).
        // 5. If result is not 0, return false.
        // 6. Return ? CalendarEquals(dateTime.[[Calendar]], other.[[Calendar]]).
        Ok((dt.inner == other).into())
    }

    fn to_string(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        let options = get_options_object(args.get_or_undefined(0))?;

        let show_calendar =
            get_option::<DisplayCalendar>(&options, js_string!("calendarName"), context)?
                .unwrap_or(DisplayCalendar::Auto);
        let precision = get_digits_option(&options, context)?;
        let rounding_mode =
            get_option::<TemporalRoundingMode>(&options, js_string!("roundingMode"), context)?;
        let smallest_unit =
            get_option::<TemporalUnit>(&options, js_string!("smallestUnit"), context)?;

        let ixdtf = dt.inner.to_ixdtf_string(
            ToStringRoundingOptions {
                precision,
                smallest_unit,
                rounding_mode,
            },
            show_calendar,
        )?;
        Ok(JsString::from(ixdtf).into())
    }

    /// 5.3.35 `Temporal.PlainDateTime.prototype.toLocaleString ( [ locales [ , options ] ] )`
    fn to_locale_string(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // TODO: Update for ECMA-402 compliance
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        let ixdtf = dt
            .inner
            .to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto)?;
        Ok(JsString::from(ixdtf).into())
    }

    /// 5.3.36 `Temporal.PlainDateTime.prototype.toJSON ( )`
    fn to_json(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDateTime object.")
            })?;

        let ixdtf = dt
            .inner
            .to_ixdtf_string(ToStringRoundingOptions::default(), DisplayCalendar::Auto)?;
        Ok(JsString::from(ixdtf).into())
    }

    /// 5.3.37 `Temporal.PlainDateTime.prototype.valueOf ( )`
    pub(crate) fn value_of(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("`valueOf` not supported by Temporal built-ins. See 'compare', 'equals', or `toString`")
            .into())
    }
}

// ==== `PlainDateTime` Abstract Operations` ====

pub(crate) fn create_temporal_datetime(
    inner: InnerDateTime,
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

    // 5. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.PlainDatedt.prototype%", « [[InitializedTemporalDateTime]], [[ISOYear]], [[ISOMonth]], [[ISODay]], [[ISOHour]], [[ISOMinute]], [[ISOSecond]], [[ISOMillisecond]], [[ISOMicrosecond]], [[ISONanosecond]], [[Calendar]] »).
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

pub(crate) fn to_temporal_datetime(
    value: &JsValue,
    options: Option<JsValue>,
    context: &mut Context,
) -> JsResult<InnerDateTime> {
    // 1. If options is not present, set options to undefined.
    // 2. Let resolvedOptions be ? SnapshotOwnProperties(! GetOptionsObject(options), null).
    // 3. If item is an Object, then
    if let Some(object) = value.as_object() {
        // a. If item has an [[InitializedTemporalDateTime]] internal slot, then
        if let Some(dt) = object.downcast_ref::<PlainDateTime>() {
            // i. Return item.
            return Ok(dt.inner.clone());
        // b. If item has an [[InitializedTemporalZonedDateTime]] internal slot, then
        } else if let Some(zdt) = object.downcast_ref::<ZonedDateTime>() {
            // i. Perform ? GetTemporalOverflowOption(resolvedOptions).
            let options = get_options_object(&options.unwrap_or(JsValue::undefined()))?;
            let _ = get_option::<ArithmeticOverflow>(&options, js_string!("overflow"), context)?;
            // ii. Let instant be ! CreateTemporalInstant(item.[[Nanoseconds]]).
            // iii. Let timeZoneRec be ? CreateTimeZoneMethodsRecord(item.[[TimeZone]], « get-offset-nanoseconds-for »).
            // iv. Return ? GetPlainDateTimeFor(timeZoneRec, instant, item.[[Calendar]]).
            return zdt
                .inner
                .to_plain_datetime_with_provider(context.tz_provider())
                .map_err(Into::into);
        // c. If item has an [[InitializedTemporalDate]] internal slot, then
        } else if let Some(date) = object.downcast_ref::<PlainDate>() {
            // i. Perform ? GetTemporalOverflowOption(resolvedOptions).
            let options = get_options_object(&options.unwrap_or(JsValue::undefined()))?;
            let _ = get_option::<ArithmeticOverflow>(&options, js_string!("overflow"), context)?;
            // ii. Return ? CreateTemporalDateTime(item.[[ISOYear]], item.[[ISOMonth]], item.[[ISODay]], 0, 0, 0, 0, 0, 0, item.[[Calendar]]).
            return Ok(InnerDateTime::new(
                date.inner.iso_year(),
                date.inner.iso_month(),
                date.inner.iso_day(),
                0,
                0,
                0,
                0,
                0,
                0,
                date.inner.calendar().clone(),
            )?);
        }

        // d. Let calendar be ? GetTemporalCalendarSlotValueWithISODefault(item).
        // e. Let calendarRec be ? CreateCalendarMethodsRecord(calendar, « date-from-fields, fields »).
        // f. Let fields be ? PrepareCalendarFields(calendarRec, item, « "day", "month",
        // "monthCode", "year" », « "hour", "microsecond", "millisecond", "minute",
        // "nanosecond", "second" », «»)
        // TODO: Move validation to `temporal_rs`.
        let partial_dt = to_partial_datetime(object, context)?;
        let resolved_options = get_options_object(&options.unwrap_or(JsValue::undefined()))?;
        // g. Let result be ? InterpretTemporalDateTimeFields(calendarRec, fields, resolvedOptions).
        let overflow =
            get_option::<ArithmeticOverflow>(&resolved_options, js_string!("overflow"), context)?;
        return InnerDateTime::from_partial(partial_dt, overflow).map_err(Into::into);
    }
    // 4. Else,
    //     a. If item is not a String, throw a TypeError exception.
    let Some(string) = value.as_string() else {
        return Err(JsNativeError::typ()
            .with_message("Cannot convert unrecognized value to PlainDateTime.")
            .into());
    };
    // b. Let result be ? ParseTemporalDateTimeString(item).
    // c. Assert: IsValidISODate(result.[[Year]], result.[[Month]], result.[[Day]]) is true.
    // d. Assert: IsValidTime(result.[[Hour]], result.[[Minute]], result.[[Second]],
    // result.[[Millisecond]], result.[[Microsecond]], result.[[Nanosecond]]) is true.
    // e. Let calendar be result.[[Calendar]].
    // f. If calendar is empty, set calendar to "iso8601".
    // g. If IsBuiltinCalendar(calendar) is false, throw a RangeError exception.
    // h. Set calendar to CanonicalizeUValue("ca", calendar).
    let date = string.to_std_string_escaped().parse::<InnerDateTime>()?;
    // i. Perform ? GetTemporalOverflowOption(resolvedOptions).
    let resolved_options = get_options_object(&options.unwrap_or(JsValue::undefined()))?;
    let _ = get_option::<ArithmeticOverflow>(&resolved_options, js_string!("overflow"), context)?;
    // 5. Return ? CreateTemporalDateTime(result.[[Year]], result.[[Month]], result.[[Day]],
    // result.[[Hour]], result.[[Minute]], result.[[Second]], result.[[Millisecond]],
    // result.[[Microsecond]], result.[[Nanosecond]], calendar).
    Ok(date)
}

fn to_partial_datetime(
    partial_object: &JsObject,
    context: &mut Context,
) -> JsResult<PartialDateTime> {
    let calendar = get_temporal_calendar_slot_value_with_default(partial_object, context)?;
    let day = partial_object
        .get(js_string!("day"), context)?
        .map(|v| {
            let finite = v.to_finitef64(context)?;
            finite
                .as_positive_integer_with_truncation()
                .map_err(JsError::from)
        })
        .transpose()?;
    let hour = partial_object
        .get(js_string!("hour"), context)?
        .map(|v| {
            let finite = v.to_finitef64(context)?;
            Ok::<u8, JsError>(finite.as_integer_with_truncation::<u8>())
        })
        .transpose()?;
    // TODO: `temporal_rs` needs a `has_era` method
    let (era, era_year) = if calendar == Calendar::default() {
        (None, None)
    } else {
        let era = partial_object
            .get(js_string!("era"), context)?
            .map(|v| {
                let v = v.to_primitive(context, crate::value::PreferredType::String)?;
                let Some(era) = v.as_string() else {
                    return Err(JsError::from(
                        JsNativeError::typ()
                            .with_message("The monthCode field value must be a string."),
                    ));
                };
                // TODO: double check if an invalid monthCode is a range or type error.
                TinyAsciiStr::<19>::try_from_str(&era.to_std_string_escaped())
                    .map_err(|e| JsError::from(JsNativeError::range().with_message(e.to_string())))
            })
            .transpose()?;
        let era_year = partial_object
            .get(js_string!("eraYear"), context)?
            .map(|v| {
                let finite = v.to_finitef64(context)?;
                Ok::<i32, JsError>(finite.as_integer_with_truncation::<i32>())
            })
            .transpose()?;
        (era, era_year)
    };
    let microsecond = partial_object
        .get(js_string!("microsecond"), context)?
        .map(|v| {
            let finite = v.to_finitef64(context)?;
            Ok::<u16, JsError>(finite.as_integer_with_truncation::<u16>())
        })
        .transpose()?;

    let millisecond = partial_object
        .get(js_string!("millisecond"), context)?
        .map(|v| {
            let finite = v.to_finitef64(context)?;
            Ok::<u16, JsError>(finite.as_integer_with_truncation::<u16>())
        })
        .transpose()?;

    let minute = partial_object
        .get(js_string!("minute"), context)?
        .map(|v| {
            let finite = v.to_finitef64(context)?;
            Ok::<u8, JsError>(finite.as_integer_with_truncation::<u8>())
        })
        .transpose()?;

    let month = partial_object
        .get(js_string!("month"), context)?
        .map(|v| {
            let finite = v.to_finitef64(context)?;
            finite
                .as_positive_integer_with_truncation()
                .map_err(JsError::from)
        })
        .transpose()?;

    let month_code = partial_object
        .get(js_string!("monthCode"), context)?
        .map(|v| {
            let v = v.to_primitive(context, crate::value::PreferredType::String)?;
            let Some(month_code) = v.as_string() else {
                return Err(JsNativeError::typ()
                    .with_message("The monthCode field value must be a string.")
                    .into());
            };
            TinyAsciiStr::<4>::try_from_str(&month_code.to_std_string_escaped())
                .map_err(|e| JsError::from(JsNativeError::typ().with_message(e.to_string())))
        })
        .transpose()?;

    let nanosecond = partial_object
        .get(js_string!("nanosecond"), context)?
        .map(|v| {
            let finite = v.to_finitef64(context)?;
            Ok::<u16, JsError>(finite.as_integer_with_truncation::<u16>())
        })
        .transpose()?;

    let second = partial_object
        .get(js_string!("second"), context)?
        .map(|v| {
            let finite = v.to_finitef64(context)?;
            Ok::<u8, JsError>(finite.as_integer_with_truncation::<u8>())
        })
        .transpose()?;

    let year = partial_object
        .get(js_string!("year"), context)?
        .map(|v| {
            let finite = v.to_finitef64(context)?;
            Ok::<i32, JsError>(finite.as_integer_with_truncation::<i32>())
        })
        .transpose()?;

    let date = PartialDate {
        year,
        month,
        month_code,
        day,
        era,
        era_year,
        calendar,
    };

    let time = PartialTime {
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
    };

    Ok(PartialDateTime { date, time })
}
