//! Boa's implementation of the ECMAScript `Temporal.PlainTime` builtin object.

use crate::{
    builtins::{
        options::{get_option, get_options_object},
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use temporal_rs::{
    options::{ArithmeticOverflow, TemporalRoundingMode},
    partial::PartialTime,
    PlainTime as PlainTimeInner,
};

use super::{
    create_temporal_duration,
    options::{get_difference_settings, get_temporal_unit, TemporalUnitGroup},
    to_integer_with_truncation, to_temporal_duration_record, PlainDateTime, ZonedDateTime,
};

/// The `Temporal.PlainTime` object.
#[derive(Debug, Clone, Copy, Trace, Finalize, JsData)]
// Safety: Time does not contain any traceable types.
#[boa_gc(unsafe_empty_trace)]
pub struct PlainTime {
    inner: PlainTimeInner,
}

impl BuiltInObject for PlainTime {
    const NAME: JsString = StaticJsStrings::PLAIN_TIME_NAME;
}

impl IntrinsicObject for PlainTime {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");
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

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                StaticJsStrings::PLAIN_TIME_TAG,
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
            .static_method(Self::from, js_string!("from"), 1)
            .static_method(Self::compare, js_string!("compare"), 2)
            .method(Self::add, js_string!("add"), 1)
            .method(Self::subtract, js_string!("subtract"), 1)
            .method(Self::with, js_string!("with"), 1)
            .method(Self::until, js_string!("until"), 1)
            .method(Self::since, js_string!("since"), 1)
            .method(Self::round, js_string!("round"), 1)
            .method(Self::equals, js_string!("equals"), 1)
            .method(Self::get_iso_fields, js_string!("getISOFields"), 0)
            .method(Self::value_of, js_string!("valueOf"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for PlainTime {
    const LENGTH: usize = 0;
    const P: usize = 15;
    const SP: usize = 2;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_time;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, then
        if new_target.is_undefined() {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message("NewTarget cannot be undefined.")
                .into());
        }

        // 2. If hour is undefined, set hour to 0; else set hour to ? ToIntegerWithTruncation(hour).
        let hour = args
            .first()
            .map(|v| to_integer_with_truncation(v, context))
            .transpose()?
            .unwrap_or(0);
        // 3. If minute is undefined, set minute to 0; else set minute to ? ToIntegerWithTruncation(minute).
        let minute = args
            .get(1)
            .map(|v| to_integer_with_truncation(v, context))
            .transpose()?
            .unwrap_or(0);
        // 4. If second is undefined, set second to 0; else set second to ? ToIntegerWithTruncation(second).
        let second = args
            .get(2)
            .map(|v| to_integer_with_truncation(v, context))
            .transpose()?
            .unwrap_or(0);
        // 5. If millisecond is undefined, set millisecond to 0; else set millisecond to ? ToIntegerWithTruncation(millisecond).
        let millisecond = args
            .get(3)
            .map(|v| to_integer_with_truncation(v, context))
            .transpose()?
            .unwrap_or(0);
        // 6. If microsecond is undefined, set microsecond to 0; else set microsecond to ? ToIntegerWithTruncation(microsecond).
        let microsecond = args
            .get(4)
            .map(|v| to_integer_with_truncation(v, context))
            .transpose()?
            .unwrap_or(0);
        // 7. If nanosecond is undefined, set nanosecond to 0; else set nanosecond to ? ToIntegerWithTruncation(nanosecond).
        let nanosecond = args
            .get(5)
            .map(|v| to_integer_with_truncation(v, context))
            .transpose()?
            .unwrap_or(0);

        let inner =
            PlainTimeInner::new(hour, minute, second, millisecond, microsecond, nanosecond)?;

        // 8. Return ? CreateTemporalTime(hour, minute, second, millisecond, microsecond, nanosecond, NewTarget).
        create_temporal_time(inner, Some(new_target), context).map(Into::into)
    }
}

// ==== PlainTime Accessor methods ====

impl PlainTime {
    /// 4.3.3 get `Temporal.PlainTime.prototype.hour`
    fn get_hour(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        // 3. Return 𝔽(temporalTime.[[ISOHour]]).
        Ok(time.inner.hour().into())
    }

    /// 4.3.4 get `Temporal.PlainTime.prototype.minute`
    fn get_minute(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        // 3. Return 𝔽(temporalTime.[[ISOMinute]]).
        Ok(time.inner.minute().into())
    }

    /// 4.3.5 get `Temporal.PlainTime.prototype.second`
    fn get_second(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        // 3. Return 𝔽(temporalTime.[[ISOSecond]]).
        Ok(time.inner.second().into())
    }

    /// 4.3.6 get `Temporal.PlainTime.prototype.millisecond`
    fn get_millisecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        // 3. Return 𝔽(temporalTime.[[ISOMillisecond]]).
        Ok(time.inner.millisecond().into())
    }

    /// 4.3.7 get `Temporal.PlainTime.prototype.microsecond`
    fn get_microsecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        // 3. Return 𝔽(temporalTime.[[ISOMicrosecond]]).
        Ok(time.inner.microsecond().into())
    }

    /// 4.3.8 get `Temporal.PlainTime.prototype.nanosecond`
    fn get_nanosecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        // 3. Return 𝔽(temporalTime.[[ISONanosecond]]).
        Ok(time.inner.nanosecond().into())
    }
}

// ==== PlainTime method implementations ====

impl PlainTime {
    /// 4.2.2 Temporal.PlainTime.from ( item [ , options ] )
    fn from(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let item = args.get_or_undefined(0);
        // 1. Set options to ? GetOptionsObject(options).
        // 2. Let overflow be ? GetTemporalOverflowOption(options).
        let overflow = get_option::<ArithmeticOverflow>(
            &get_options_object(args.get_or_undefined(1))?,
            js_string!("overflow"),
            context,
        )?;
        // 3. If item is an Object and item has an [[InitializedTemporalTime]] internal slot, then
        let time = if let Some(time) = item
            .as_object()
            .and_then(JsObject::downcast_ref::<PlainTime>)
        {
            // a. Return ! CreateTemporalTime(item.[[ISOHour]], item.[[ISOMinute]],
            // item.[[ISOSecond]], item.[[ISOMillisecond]], item.[[ISOMicrosecond]],
            // item.[[ISONanosecond]]).
            time.inner
        } else {
            to_temporal_time(item, overflow, context)?
        };

        // 4. Return ? ToTemporalTime(item, overflow).
        create_temporal_time(time, None, context).map(Into::into)
    }

    /// 4.2.3 Temporal.PlainTime.compare ( one, two )
    fn compare(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Set one to ? ToTemporalTime(one).
        let one = to_temporal_time(args.get_or_undefined(0), None, context)?;
        // 2. Set two to ? ToTemporalTime(two).
        let two = to_temporal_time(args.get_or_undefined(1), None, context)?;
        // 3. Return 𝔽(CompareTemporalTime(one.[[ISOHour]], one.[[ISOMinute]], one.[[ISOSecond]],
        // one.[[ISOMillisecond]], one.[[ISOMicrosecond]], one.[[ISONanosecond]], two.[[ISOHour]],
        // two.[[ISOMinute]], two.[[ISOSecond]], two.[[ISOMillisecond]], two.[[ISOMicrosecond]],
        // two.[[ISONanosecond]])).
        Ok((one.cmp(&two) as i8).into())
    }
}

// ==== PlainTime.prototype method implementations ====

impl PlainTime {
    /// 4.3.9 Temporal.PlainTime.prototype.add ( temporalDurationLike )
    fn add(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        let temporal_duration_like = args.get_or_undefined(0);
        let duration = to_temporal_duration_record(temporal_duration_like, context)?;

        // 3. Return ? AddDurationToOrSubtractDurationFromPlainTime(add, temporalTime, temporalDurationLike).
        create_temporal_time(time.inner.add(&duration)?, None, context).map(Into::into)
    }

    /// 4.3.10 Temporal.PlainTime.prototype.subtract ( temporalDurationLike )
    fn subtract(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        let temporal_duration_like = args.get_or_undefined(0);
        let duration = to_temporal_duration_record(temporal_duration_like, context)?;

        // 3. Return ? AddDurationToOrSubtractDurationFromPlainTime(subtract, temporalTime, temporalDurationLike).
        create_temporal_time(time.inner.subtract(&duration)?, None, context).map(Into::into)
    }

    fn with(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1.Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        // 3. If ? IsPartialTemporalObject(temporalTimeLike) is false, throw a TypeError exception.
        // 4. Set options to ? GetOptionsObject(options).
        let Some(partial_object) =
            super::is_partial_temporal_object(args.get_or_undefined(0), context)?
        else {
            return Err(JsNativeError::typ()
                .with_message("with object was not a PartialTemporalObject.")
                .into());
        };

        let options = get_options_object(args.get_or_undefined(1))?;
        let overflow = get_option::<ArithmeticOverflow>(&options, js_string!("overflow"), context)?;
        let partial = to_partial_time_record(partial_object, context)?;

        create_temporal_time(time.inner.with(partial, overflow)?, None, context).map(Into::into)
    }

    /// 4.3.12 Temporal.PlainTime.prototype.until ( other [ , options ] )
    fn until(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        let other = to_temporal_time(args.get_or_undefined(0), None, context)?;

        let settings =
            get_difference_settings(&get_options_object(args.get_or_undefined(1))?, context)?;

        let result = time.inner.until(&other, settings)?;

        create_temporal_duration(result, None, context).map(Into::into)
    }

    /// 4.3.13 Temporal.PlainTime.prototype.since ( other [ , options ] )
    fn since(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        let other = to_temporal_time(args.get_or_undefined(0), None, context)?;

        let settings =
            get_difference_settings(&get_options_object(args.get_or_undefined(1))?, context)?;

        let result = time.inner.since(&other, settings)?;

        create_temporal_duration(result, None, context).map(Into::into)
    }

    /// 4.3.14 Temporal.PlainTime.prototype.round ( roundTo )
    fn round(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        let round_to = match args.first() {
            // 3. If roundTo is undefined, then
            None | Some(JsValue::Undefined) => {
                return Err(JsNativeError::typ()
                    .with_message("roundTo cannot be undefined.")
                    .into())
            }
            // 4. If Type(roundTo) is String, then
            Some(JsValue::String(rt)) => {
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
                get_options_object(round_to)?
            }
        };

        // 6. NOTE: The following steps read options and perform independent validation in alphabetical order (ToTemporalRoundingIncrement reads "roundingIncrement" and ToTemporalRoundingMode reads "roundingMode").
        // 7. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        let rounding_increment =
            get_option::<f64>(&round_to, js_string!("roundingIncrement"), context)?;

        // 8. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        let rounding_mode =
            get_option::<TemporalRoundingMode>(&round_to, js_string!("roundingMode"), context)?;

        // 9. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit", time, required).
        let smallest_unit = get_temporal_unit(
            &round_to,
            js_string!("smallestUnit"),
            TemporalUnitGroup::Time,
            None,
            context,
        )?
        .ok_or_else(|| JsNativeError::range().with_message("smallestUnit cannot be undefined."))?;

        // 10. Let maximum be MaximumTemporalDurationRoundingIncrement(smallestUnit).
        // 11. Assert: maximum is not undefined.
        // 12. Perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
        // 13. Let result be RoundTime(temporalTime.[[ISOHour]], temporalTime.[[ISOMinute]], temporalTime.[[ISOSecond]], temporalTime.[[ISOMillisecond]], temporalTime.[[ISOMicrosecond]], temporalTime.[[ISONanosecond]], roundingIncrement, smallestUnit, roundingMode).
        let result = time
            .inner
            .round(smallest_unit, rounding_increment, rounding_mode)?;

        // 14. Return ! CreateTemporalTime(result.[[Hour]], result.[[Minute]], result.[[Second]], result.[[Millisecond]], result.[[Microsecond]], result.[[Nanosecond]]).
        create_temporal_time(result, None, context).map(Into::into)
    }

    /// 4.3.15 Temporal.PlainTime.prototype.equals ( other )
    fn equals(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        // 3. Set other to ? ToTemporalTime(other).
        let other = to_temporal_time(args.get_or_undefined(0), None, context)?;
        // 4. If temporalTime.[[ISOHour]] ≠ other.[[ISOHour]], return false.
        // 5. If temporalTime.[[ISOMinute]] ≠ other.[[ISOMinute]], return false.
        // 6. If temporalTime.[[ISOSecond]] ≠ other.[[ISOSecond]], return false.
        // 7. If temporalTime.[[ISOMillisecond]] ≠ other.[[ISOMillisecond]], return false.
        // 8. If temporalTime.[[ISOMicrosecond]] ≠ other.[[ISOMicrosecond]], return false.
        // 9. If temporalTime.[[ISONanosecond]] ≠ other.[[ISONanosecond]], return false.
        // 10. Return true.
        Ok((time.inner == other).into())
    }

    /// 4.3.18 Temporal.PlainTime.prototype.getISOFields ( )
    fn get_iso_fields(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let temporalTime be the this value.
        // 2. Perform ? RequireInternalSlot(temporalTime, [[InitializedTemporalTime]]).
        let time = this
            .as_object()
            .and_then(JsObject::downcast_ref::<PlainTime>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be a PlainTime object.")
            })?;

        // 3. Let fields be OrdinaryObjectCreate(%Object.prototype%).
        let fields = JsObject::with_object_proto(context.intrinsics());

        // 4. Perform ! CreateDataPropertyOrThrow(fields, "isoHour", 𝔽(temporalTime.[[ISOHour]])).
        fields.create_data_property_or_throw(js_string!("isoHour"), time.inner.hour(), context)?;
        // 5. Perform ! CreateDataPropertyOrThrow(fields, "isoMicrosecond", 𝔽(temporalTime.[[ISOMicrosecond]])).
        fields.create_data_property_or_throw(
            js_string!("isoMicrosecond"),
            time.inner.microsecond(),
            context,
        )?;
        // 6. Perform ! CreateDataPropertyOrThrow(fields, "isoMillisecond", 𝔽(temporalTime.[[ISOMillisecond]])).
        fields.create_data_property_or_throw(
            js_string!("isoMillisecond"),
            time.inner.millisecond(),
            context,
        )?;
        // 7. Perform ! CreateDataPropertyOrThrow(fields, "isoMinute", 𝔽(temporalTime.[[ISOMinute]])).
        fields.create_data_property_or_throw(
            js_string!("isoMinute"),
            time.inner.minute(),
            context,
        )?;
        // 8. Perform ! CreateDataPropertyOrThrow(fields, "isoNanosecond", 𝔽(temporalTime.[[ISONanosecond]])).
        fields.create_data_property_or_throw(
            js_string!("isoNanosecond"),
            time.inner.nanosecond(),
            context,
        )?;
        // 9. Perform ! CreateDataPropertyOrThrow(fields, "isoSecond", 𝔽(temporalTime.[[ISOSecond]])).
        fields.create_data_property_or_throw(
            js_string!("isoSecond"),
            time.inner.second(),
            context,
        )?;

        // 10. Return fields.
        Ok(fields.into())
    }

    /// 4.3.22 Temporal.PlainTime.prototype.valueOf ( )
    fn value_of(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Throw a TypeError exception.
        Err(JsNativeError::typ()
            .with_message("valueOf cannot be called on PlainTime.")
            .into())
    }
}

// ==== PlainTime Abstract Operations ====

pub(crate) fn create_temporal_time(
    inner: PlainTimeInner,
    new_target: Option<&JsValue>,
    context: &mut Context,
) -> JsResult<JsObject> {
    // Note: IsValidTime is enforced by Time.
    // 1. If IsValidTime(hour, minute, second, millisecond, microsecond, nanosecond) is false, throw a RangeError exception.

    // 2. If newTarget is not present, set newTarget to %Temporal.PlainTime%.
    let new_target = if let Some(new_target) = new_target {
        new_target.clone()
    } else {
        context
            .realm()
            .intrinsics()
            .constructors()
            .plain_time()
            .constructor()
            .into()
    };

    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.PlainTime.prototype%", « [[InitializedTemporalTime]], [[ISOHour]], [[ISOMinute]], [[ISOSecond]], [[ISOMillisecond]], [[ISOMicrosecond]], [[ISONanosecond]] »).
    let prototype =
        get_prototype_from_constructor(&new_target, StandardConstructors::plain_time, context)?;

    // 4. Set object.[[ISOHour]] to hour.
    // 5. Set object.[[ISOMinute]] to minute.
    // 6. Set object.[[ISOSecond]] to second.
    // 7. Set object.[[ISOMillisecond]] to millisecond.
    // 8. Set object.[[ISOMicrosecond]] to microsecond.
    // 9. Set object.[[ISONanosecond]] to nanosecond.
    let obj = JsObject::from_proto_and_data(prototype, PlainTime { inner });

    // 10. Return object.
    Ok(obj)
}

/// 4.5.3 `ToTemporalTime ( item [ , overflow ] )`
pub(crate) fn to_temporal_time(
    value: &JsValue,
    overflow: Option<ArithmeticOverflow>,
    context: &mut Context,
) -> JsResult<PlainTimeInner> {
    // 1.If overflow is not present, set overflow to "constrain".
    let resolved_overflow = overflow.unwrap_or(ArithmeticOverflow::Constrain);
    // 2. If item is an Object, then
    match value {
        JsValue::Object(object) => {
            // a. If item has an [[InitializedTemporalTime]] internal slot, then
            if let Some(time) = object.downcast_ref::<PlainTime>() {
                // i. Return item.
                return Ok(time.inner);
            // b. If item has an [[InitializedTemporalZonedDateTime]] internal slot, then
            } else if let Some(_zdt) = object.downcast_ref::<ZonedDateTime>() {
                // i. Let instant be ! CreateTemporalInstant(item.[[Nanoseconds]]).
                // ii. Let timeZoneRec be ? CreateTimeZoneMethodsRecord(item.[[TimeZone]], « get-offset-nanoseconds-for »).
                // iii. Let plainDateTime be ? GetPlainDateTimeFor(timeZoneRec, instant, item.[[Calendar]]).
                // iv. Return ! CreateTemporalTime(plainDateTime.[[ISOHour]], plainDateTime.[[ISOMinute]],
                // plainDateTime.[[ISOSecond]], plainDateTime.[[ISOMillisecond]], plainDateTime.[[ISOMicrosecond]],
                // plainDateTime.[[ISONanosecond]]).
                return Err(JsNativeError::range()
                    .with_message("Not yet implemented.")
                    .into());
            // c. If item has an [[InitializedTemporalDateTime]] internal slot, then
            } else if let Some(dt) = object.downcast_ref::<PlainDateTime>() {
                // i. Return ! CreateTemporalTime(item.[[ISOHour]], item.[[ISOMinute]],
                // item.[[ISOSecond]], item.[[ISOMillisecond]], item.[[ISOMicrosecond]],
                // item.[[ISONanosecond]]).
                return Ok(PlainTimeInner::from(dt.inner.clone()));
            }
            // d. Let result be ? ToTemporalTimeRecord(item).
            // e. Set result to ? RegulateTime(result.[[Hour]], result.[[Minute]],
            // result.[[Second]], result.[[Millisecond]], result.[[Microsecond]],
            // result.[[Nanosecond]], overflow).
            let partial = to_partial_time_record(object, context)?;

            PlainTimeInner::new_with_overflow(
                partial.hour.unwrap_or(0),
                partial.minute.unwrap_or(0),
                partial.second.unwrap_or(0),
                partial.millisecond.unwrap_or(0),
                partial.microsecond.unwrap_or(0),
                partial.nanosecond.unwrap_or(0),
                resolved_overflow,
            )
            .map_err(Into::into)
        }
        // 3. Else,
        JsValue::String(str) => {
            // b. Let result be ? ParseTemporalTimeString(item).
            // c. Assert: IsValidTime(result.[[Hour]], result.[[Minute]], result.[[Second]], result.[[Millisecond]], result.[[Microsecond]], result.[[Nanosecond]]) is true.
            str.to_std_string_escaped()
                .parse::<PlainTimeInner>()
                .map_err(Into::into)
        }
        // a. If item is not a String, throw a TypeError exception.
        _ => Err(JsNativeError::typ()
            .with_message("Invalid value for converting to PlainTime.")
            .into()),
    }

    // 4. Return ! CreateTemporalTime(result.[[Hour]], result.[[Minute]], result.[[Second]], result.[[Millisecond]], result.[[Microsecond]], result.[[Nanosecond]]).
}

pub(crate) fn to_partial_time_record(
    partial_object: &JsObject,
    context: &mut Context,
) -> JsResult<PartialTime> {
    let hour = partial_object
        .get(js_string!("hour"), context)?
        .map(|v| super::to_integer_if_integral(v, context))
        .transpose()?;

    let minute = partial_object
        .get(js_string!("minute"), context)?
        .map(|v| super::to_integer_if_integral(v, context))
        .transpose()?;

    let second = partial_object
        .get(js_string!("second"), context)?
        .map(|v| super::to_integer_if_integral(v, context))
        .transpose()?;

    let millisecond = partial_object
        .get(js_string!("millisecond"), context)?
        .map(|v| super::to_integer_if_integral(v, context))
        .transpose()?;

    let microsecond = partial_object
        .get(js_string!("microsecond"), context)?
        .map(|v| super::to_integer_if_integral(v, context))
        .transpose()?;

    let nanosecond = partial_object
        .get(js_string!("nanosecond"), context)?
        .map(|v| super::to_integer_if_integral(v, context))
        .transpose()?;

    Ok(PartialTime {
        hour,
        minute,
        second,
        millisecond,
        microsecond,
        nanosecond,
    })
}
