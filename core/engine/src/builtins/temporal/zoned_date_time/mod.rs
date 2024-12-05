#![allow(dead_code, unused_variables)]
use std::str::FromStr;

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject}, context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors}, js_string, object::internal_methods::get_prototype_from_constructor, property::Attribute, realm::Realm, string::StaticJsStrings, Context, JsArgs, JsBigInt, JsData, JsError, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use temporal_rs::{Duration as TemporalDuration, ZonedDateTime as InnerZdt};

/// The `Temporal.ZonedDateTime` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)]
pub struct ZonedDateTime {
    pub(crate) inner: ZonedDateTimeInner,
}

impl ZonedDateTime {
    pub(crate) fn new(inner: ZonedDateTimeInner) -> Self {
        Self { inner }
    }
}

impl BuiltInObject for ZonedDateTime {
    const NAME: JsString = StaticJsStrings::ZONED_DT_NAME;
}

impl IntrinsicObject for ZonedDateTime {
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

        let get_epoch_millisecond = BuiltInBuilder::callable(realm, Self::get_epoch_millisecond)
            .name(js_string!("get epochMillisecond"))
            .build();

        let get_epoch_nanosecond = BuiltInBuilder::callable(realm, Self::get_epoch_nanosecond)
            .name(js_string!("get epochNanosecond"))
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

        let get_hours_in_day = BuiltInBuilder::callable(realm, Self::get_hours_in_day)
            .name(js_string!("get daysInWeek"))
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
                StaticJsStrings::ZONED_DT_TAG,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("calendarId"),
                Some(get_calendar_id),
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
                js_string!("epochMillisecond"),
                Some(get_epoch_millisecond),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("epochNanosecond"),
                Some(get_epoch_nanosecond),
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
                js_string!("hoursInDay"),
                Some(get_hours_in_day),
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
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for ZonedDateTime {
    const LENGTH: usize = 2;
    const P: usize = 1;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::zoned_date_time;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, then
        if new_target.is_undefined() {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ().with_message("NewTarget cannot be undefined.").into())
        }
        //  2. Set epochNanoseconds to ? ToBigInt(epochNanoseconds).
        let epoch_nanos = args.get_or_undefined(0).to_bigint(context)?;
        //  3. If IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
        // TODO: Better primitive for handling epochNanoseconds is needed in temporal_rs
        let Some(nanos) = epoch_nanos.to_f64().to_i128() else {
            return Err(JsNativeError::range().with_message("epochNanoseconds exceeded valid range.").into())
        };

        //  4. If timeZone is not a String, throw a TypeError exception.
        let JsValue::String(timezone_str) = args.get_or_undefined(1) else {
            return Err(JsNativeError::typ().with_message("timeZone must be a string.").into())
        };

        //  5. Let timeZoneParse be ? ParseTimeZoneIdentifier(timeZone).
        //  6. If timeZoneParse.[[OffsetMinutes]] is empty, then
        // a. Let identifierRecord be GetAvailableNamezdtimeZoneIdentifier(timeZoneParse.[[Name]]).
        // b. If identifierRecord is empty, throw a RangeError exception.
        // c. Set timeZone to identifierRecord.[[Identifier]].
        //  7. Else,
        // a. Set timeZone to FormatOffsetTimeZoneIdentifier(timeZoneParse.[[OffsetMinutes]]).
        let timezone = TimeZone::try_from_str_with_provider(&timezone_str.to_std_string_escaped(), context.tz_provider())?;

        //  8. If calendar is undefined, set calendar to "iso8601".
        //  9. If calendar is not a String, throw a TypeError exception.
        //  10. Set calendar to ? CanonicalizeCalendar(calendar).
        let calendar = match args.get(2) {
            Some(JsValue::String(time_zone)) => Calendar::from_str(&time_zone.to_std_string_escaped())?,
            None=> Calendar::default(),
            _=> return Err(JsNativeError::typ().with_message("calendar must be a string.").into())
        };

        let inner = ZonedDateTimeInner::try_new(nanos, calendar, timezone)?;

        //  11. Return ? CreateTemporalZonedDateTime(epochNanoseconds, timeZone, calendar, NewTarget).
        create_temporal_zoneddatetime(inner, Some(new_target), context).map(Into::into)
    }
}

// ==== `ZonedDateTime` accessor property methods ====

impl ZonedDateTime {
    /// 6.3.3 get `Temporal.PlainDatezdt.prototype.calendarId`
    fn get_calendar_id(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDate object.")
            })?;

        Ok(JsString::from(zdt.inner.calendar().identifier()).into())
    }

    /// 6.3.4 get `Temporal.PlainDatezdt.prototype.timeZoneId`
    fn get_timezone_id(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let _zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDate object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
    }

    /// 6.3.5 get `Temporal.PlainDatezdt.prototype.era`
    fn get_era(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let _zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDate object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
    }

    /// 6.3.6 get `Temporal.PlainDatezdt.prototype.eraYear`
    fn get_era_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let _zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainDate object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
    }

    /// 6.3.7 get `Temporal.PlainDatezdt.prototype.year`
    fn get_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.year_with_provider(context.tz_provider())?.into())
    }

    /// 6.3.8 get `Temporal.PlainDatezdt.prototype.month`
    fn get_month(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.month_with_provider(context.tz_provider())?.into())
    }

    /// 6.3.9 get Temporal.PlainDatezdt.prototype.monthCode
    fn get_month_code(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(JsString::from(zdt.inner.month_code_with_provider(context.tz_provider())?.as_str()).into())
    }

    /// 6.3.10 get `Temporal.PlainDatezdt.prototype.day`
    fn get_day(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.day_with_provider(context.tz_provider())?.into())
    }

    /// 6.3.11 get `Temporal.PlainDatezdt.prototype.hour`
    fn get_hour(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.hour_with_provider(context.tz_provider())?.into())
    }

    /// 6.3.12 get `Temporal.PlainDatezdt.prototype.minute`
    fn get_minute(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.minute_with_provider(context.tz_provider())?.into())
    }

    /// 6.3.13 get `Temporal.PlainDatezdt.prototype.second`
    fn get_second(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.second_with_provider(context.tz_provider())?.into())
    }

    /// 6.3.14 get `Temporal.PlainDatezdt.prototype.millisecond`
    fn get_millisecond(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.millisecond_with_provider(context.tz_provider())?.into())
    }

    /// 6.3.15 get `Temporal.PlainDatezdt.prototype.microsecond`
    fn get_microsecond(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.microsecond_with_provider(context.tz_provider())?.into())
    }

    /// 6.3.16 get `Temporal.PlainDatezdt.prototype.nanosecond`
    fn get_nanosecond(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.nanosecond_with_provider(context.tz_provider())?.into())
    }

    /// 6.3.17 get `Temporal.PlainDatezdt.prototype.epochMilliseconds`
    fn get_epoch_millisecond(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.epoch_milliseconds().into())
    }

    /// 6.3.18 get `Temporal.PlainDatezdt.prototype.epochNanosecond`
    fn get_epoch_nanosecond(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Ok(zdt.inner.epoch_nanoseconds().into())
    }

    /// 6.3.19 get `Temporal.PlainDatezdt.prototype.dayOfWeek`
    fn get_day_of_week(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
        // Ok(zdt.inner.day_of_week()?.into())
    }

    /// 6.3.20 get `Temporal.PlainDatezdt.prototype.dayOfYear`
    fn get_day_of_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
        // Ok(zdt.inner.day_of_year()?.into())
    }

    /// 6.3.21 get `Temporal.PlainDatezdt.prototype.weekOfYear`
    fn get_week_of_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
        // Ok(zdt.inner.week_of_year()?.into_or_undefined())
    }

    /// 6.3.22 get `Temporal.PlainDatezdt.prototype.yearOfWeek`
    fn get_year_of_week(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
        // Ok(zdt.inner.year_of_week()?.into_or_undefined())
    }

    /// 6.3.23 get `Temporal.PlainDatezdt.prototype.hoursInDay`
    fn get_hours_in_day(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
    }

    /// 6.3.24 get `Temporal.PlainDatezdt.prototype.daysInWeek`
    fn get_days_in_week(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
        // Ok(zdt.inner.days_in_week()?.into())
    }

    /// 6.3.25 get `Temporal.PlainDatezdt.prototype.daysInMonth`
    fn get_days_in_month(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
        // Ok(zdt.inner.days_in_month()?.into())
    }

    /// 6.3.26 get `Temporal.PlainDatezdt.prototype.daysInYear`
    fn get_days_in_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
        // Ok(zdt.inner.days_in_year()?.into())
    }

    /// 6.3.27 get `Temporal.PlainDatezdt.prototype.monthsInYear`
    fn get_months_in_year(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
        // Ok(zdt.inner.months_in_year()?.into())
    }

    /// 6.3.28 get `Temporal.PlainDatezdt.prototype.inLeapYear`
    fn get_in_leap_year(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let zdt = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a ZonedDateTime object.")
            })?;

        Err(JsNativeError::error().with_message("Not yet implemented.").into())
        // Ok(zdt.inner.in_leap_year()?.into())
    }
}

// -- ZonedDateTime Abstract Operations --

pub(crate) fn create_temporal_zoneddatetime(
    inner: ZonedDateTimeInner,
    new_target: Option<&JsValue>,
    context: &mut Context,
) -> JsResult<JsObject> {
    // 1. Assert: IsValidEpochNanoseconds(epochNanoseconds) is true.
    // 2. If newTarget is not present, set newTarget to %Temporal.ZonedDateTime%.
    let new_target = new_target.cloned().unwrap_or(
        context
            .realm()
            .intrinsics()
            .constructors()
            .zoned_date_time()
            .constructor()
            .into()
    );
    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.ZonedDateTime.prototype%", « [[InitializezdtemporalZonedDateTime]], [[EpochNanoseconds]], [[TimeZone]], [[Calendar]] »).
    let prototype = 
        get_prototype_from_constructor(&new_target, StandardConstructors::zoned_date_time, context)?;
    // 4. Set object.[[EpochNanoseconds]] to epochNanoseconds.
    // 5. Set object.[[TimeZone]] to timeZone.
    // 6. Set object.[[Calendar]] to calendar.
    let obj = JsObject::from_proto_and_data(prototype, ZonedDateTime::new(inner));

    // 7. Return object.
    Ok(obj)
}
