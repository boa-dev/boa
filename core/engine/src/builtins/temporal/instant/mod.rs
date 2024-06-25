//! Boa's implementation of ECMAScript's `Temporal.Instant` builtin object.

use crate::{
    builtins::{
        options::{get_option, get_options_object},
        temporal::{
            duration::{create_temporal_duration, to_temporal_duration_record},
            options::{get_temporal_unit, TemporalUnitGroup},
        },
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsArgs, JsBigInt, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol,
    JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_macros::js_str;
use boa_profiler::Profiler;
use temporal_rs::{
    components::Instant as InnerInstant,
    options::{RoundingIncrement, TemporalRoundingMode, TemporalUnit},
};

/// The `Temporal.Instant` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
// SAFETY: Instant does not contain any traceable values.
#[boa_gc(unsafe_empty_trace)]
pub struct Instant {
    pub(crate) inner: InnerInstant,
}

impl BuiltInObject for Instant {
    const NAME: JsString = StaticJsStrings::INSTANT;
}

impl IntrinsicObject for Instant {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_seconds = BuiltInBuilder::callable(realm, Self::get_epoc_seconds)
            .name(js_string!("get epochSeconds"))
            .build();

        let get_millis = BuiltInBuilder::callable(realm, Self::get_epoc_milliseconds)
            .name(js_string!("get epochMilliseconds"))
            .build();

        let get_micros = BuiltInBuilder::callable(realm, Self::get_epoc_microseconds)
            .name(js_string!("get epochMicroseconds"))
            .build();

        let get_nanos = BuiltInBuilder::callable(realm, Self::get_epoc_nanoseconds)
            .name(js_string!("get epochNanoseconds"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_str!("epochSeconds"),
                Some(get_seconds),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_str!("epochMilliseconds"),
                Some(get_millis),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_str!("epochMicroseconds"),
                Some(get_micros),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_str!("epochNanoseconds"),
                Some(get_nanos),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::add, js_string!("add"), 1)
            .method(Self::subtract, js_string!("subtract"), 1)
            .method(Self::until, js_string!("until"), 2)
            .method(Self::since, js_string!("since"), 2)
            .method(Self::round, js_string!("round"), 1)
            .method(Self::equals, js_string!("equals"), 1)
            .method(Self::to_zoned_date_time, js_string!("toZonedDateTime"), 1)
            .method(
                Self::to_zoned_date_time_iso,
                js_string!("toZonedDateTimeISO"),
                1,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for Instant {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::instant;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, then
        if new_target.is_undefined() {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message("Temporal.Instant new target cannot be undefined.")
                .into());
        };

        // 2. Let epochNanoseconds be ? ToBigInt(epochNanoseconds).
        let epoch_nanos = args.get_or_undefined(0).to_bigint(context)?;

        // 3. If ! IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
        // NOTE: boa_temporal::Instant asserts that the epochNanoseconds are valid.
        let instant = InnerInstant::new(epoch_nanos.as_inner().clone())?;
        // 4. Return ? CreateTemporalInstant(epochNanoseconds, NewTarget).
        create_temporal_instant(instant, Some(new_target.clone()), context)
    }
}

// -- Instant method implementations --

impl Instant {
    /// 8.3.3 get Temporal.Instant.prototype.epochSeconds
    pub(crate) fn get_epoc_seconds(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let instant = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be an instant object.")
            })?;
        // 3. Let ns be instant.[[Nanoseconds]].
        Ok(instant.inner.epoch_seconds().into())
    }

    /// 8.3.4 get Temporal.Instant.prototype.epochMilliseconds
    pub(crate) fn get_epoc_milliseconds(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let instant = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be an instant object.")
            })?;
        // 3. Let ns be instant.[[Nanoseconds]].
        // 4. Let ms be floor(ℝ(ns) / 106).
        // 5. Return 𝔽(ms).
        Ok(instant.inner.epoch_milliseconds().into())
    }

    /// 8.3.5 get Temporal.Instant.prototype.epochMicroseconds
    pub(crate) fn get_epoc_microseconds(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let instant = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be an instant object.")
            })?;
        // 3. Let ns be instant.[[Nanoseconds]].
        // 4. Let µs be floor(ℝ(ns) / 103).
        // 5. Return ℤ(µs).
        let big_int = JsBigInt::try_from(instant.inner.epoch_microseconds())
            .expect("valid microseconds is in range of BigInt");
        Ok(big_int.into())
    }

    /// 8.3.6 get Temporal.Instant.prototype.epochNanoseconds
    pub(crate) fn get_epoc_nanoseconds(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let instant = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be an instant object.")
            })?;
        // 3. Let ns be instant.[[Nanoseconds]].
        // 4. Return ns.
        let big_int = JsBigInt::try_from(instant.inner.epoch_nanoseconds())
            .expect("valid nanoseconds is in range of BigInt");
        Ok(big_int.into())
    }

    /// 8.3.7 `Temporal.Instant.prototype.add ( temporalDurationLike )`
    pub(crate) fn add(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let instant = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be an instant object.")
            })?;

        // 3. Return ? AddDurationToOrSubtractDurationFromInstant(add, instant, temporalDurationLike).
        let temporal_duration_like =
            to_temporal_duration_record(args.get_or_undefined(0), context)?;
        let result = instant.inner.add(temporal_duration_like)?;
        create_temporal_instant(result, None, context)
    }

    /// 8.3.8 `Temporal.Instant.prototype.subtract ( temporalDurationLike )`
    pub(crate) fn subtract(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let instant = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be an instant object.")
            })?;

        // 3. Return ? AddDurationToOrSubtractDurationFromInstant(subtract, instant, temporalDurationLike).
        let temporal_duration_like =
            to_temporal_duration_record(args.get_or_undefined(0), context)?;
        let result = instant.inner.subtract(temporal_duration_like)?;
        create_temporal_instant(result, None, context)
    }

    /// 8.3.9 `Temporal.Instant.prototype.until ( other [ , options ] )`
    pub(crate) fn until(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let instant = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be an instant object.")
            })?;

        // 3. Return ? DifferenceTemporalInstant(until, instant, other, options).
        let other = to_temporal_instant(args.get_or_undefined(0))?;

        // Fetch the necessary options.
        let options = get_options_object(args.get_or_undefined(1))?;
        let mode = get_option::<TemporalRoundingMode>(&options, js_str!("roundingMode"), context)?;
        let increment =
            get_option::<RoundingIncrement>(&options, js_str!("roundingIncrement"), context)?;
        let smallest_unit = get_option::<TemporalUnit>(&options, js_str!("smallestUnit"), context)?;
        let largest_unit = get_option::<TemporalUnit>(&options, js_str!("largestUnit"), context)?;
        let result = instant
            .inner
            .until(&other, mode, increment, smallest_unit, largest_unit)?;
        create_temporal_duration(result.into(), None, context).map(Into::into)
    }

    /// 8.3.10 `Temporal.Instant.prototype.since ( other [ , options ] )`
    pub(crate) fn since(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let instant = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be an instant object.")
            })?;

        // 3. Return ? DifferenceTemporalInstant(since, instant, other, options).
        let other = to_temporal_instant(args.get_or_undefined(0))?;
        let options = get_options_object(args.get_or_undefined(1))?;
        let mode = get_option::<TemporalRoundingMode>(&options, js_str!("roundingMode"), context)?;
        let increment =
            get_option::<RoundingIncrement>(&options, js_str!("roundingIncrement"), context)?;
        let smallest_unit = get_option::<TemporalUnit>(&options, js_str!("smallestUnit"), context)?;
        let largest_unit = get_option::<TemporalUnit>(&options, js_str!("largestUnit"), context)?;
        let result = instant
            .inner
            .since(&other, mode, increment, smallest_unit, largest_unit)?;
        create_temporal_duration(result.into(), None, context).map(Into::into)
    }

    /// 8.3.11 `Temporal.Instant.prototype.round ( roundTo )`
    pub(crate) fn round(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let instant = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be an instant object.")
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
                    js_str!("smallestUnit"),
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

        // 6. NOTE: The following steps read options and perform independent validation in
        // alphabetical order (ToTemporalRoundingIncrement reads "roundingIncrement" and ToTemporalRoundingMode reads "roundingMode").
        // 7. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        let rounding_increment =
            get_option::<f64>(&round_to, js_str!("roundingIncrement"), context)?;

        // 8. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        let rounding_mode =
            get_option::<TemporalRoundingMode>(&round_to, js_str!("roundingMode"), context)?;

        // 9. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit"), time, required).
        let smallest_unit = get_temporal_unit(
            &round_to,
            js_str!("smallestUnit"),
            TemporalUnitGroup::Time,
            None,
            context,
        )?
        .ok_or_else(|| JsNativeError::range().with_message("smallestUnit cannot be undefined."))?;

        // 10. If smallestUnit is "hour"), then
        // a. Let maximum be HoursPerDay.
        // 11. Else if smallestUnit is "minute"), then
        // a. Let maximum be MinutesPerHour × HoursPerDay.
        // 12. Else if smallestUnit is "second"), then
        // a. Let maximum be SecondsPerMinute × MinutesPerHour × HoursPerDay.
        // 13. Else if smallestUnit is "millisecond"), then
        // a. Let maximum be ℝ(msPerDay).
        // 14. Else if smallestUnit is "microsecond"), then
        // a. Let maximum be 10^3 × ℝ(msPerDay).
        // 15. Else,
        // a. Assert: smallestUnit is "nanosecond".
        // b. Let maximum be nsPerDay.
        // unreachable here functions as 15.a.
        // 16. Perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, true).
        // 17. Let roundedNs be RoundTemporalInstant(instant.[[Nanoseconds]], roundingIncrement, smallestUnit, roundingMode).
        let result = instant
            .inner
            .round(rounding_increment, smallest_unit, rounding_mode)?;

        // 18. Return ! CreateTemporalInstant(roundedNs).
        create_temporal_instant(result, None, context)
    }

    /// 8.3.12 `Temporal.Instant.prototype.equals ( other )`
    pub(crate) fn equals(this: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        // 4. If instant.[[Nanoseconds]] ≠ other.[[Nanoseconds]], return false.
        // 5. Return true.
        let instant = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ().with_message("the this object must be an instant object.")
            })?;

        // 3. Set other to ? ToTemporalInstant(other).
        let other = args.get_or_undefined(0);
        let other_instant = to_temporal_instant(other)?;

        if instant.inner != other_instant {
            return Ok(false.into());
        }
        Ok(true.into())
    }

    /// 8.3.17 `Temporal.Instant.prototype.toZonedDateTime ( item )`
    pub(crate) fn to_zoned_date_time(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // TODO: Complete
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }

    /// 8.3.18 `Temporal.Instant.prototype.toZonedDateTimeISO ( timeZone )`
    pub(crate) fn to_zoned_date_time_iso(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // TODO Complete
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }
}

// -- Instant Abstract Operations --

// 8.5.1 `IsValidEpochNanoseconds ( epochNanoseconds )`
// Implemented in `boa_temporal`

/// 8.5.2 `CreateTemporalInstant ( epochNanoseconds [ , newTarget ] )`
#[inline]
fn create_temporal_instant(
    instant: InnerInstant,
    new_target: Option<JsValue>,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. Assert: ! IsValidEpochNanoseconds(epochNanoseconds) is true.
    // 2. If newTarget is not present, set newTarget to %Temporal.Instant%.
    let new_target = new_target.unwrap_or_else(|| {
        context
            .realm()
            .intrinsics()
            .constructors()
            .instant()
            .constructor()
            .into()
    });
    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Instant.prototype%"), « [[InitializedTemporalInstant]], [[Nanoseconds]] »).
    let proto =
        get_prototype_from_constructor(&new_target, StandardConstructors::instant, context)?;

    // 4. Set object.[[Nanoseconds]] to epochNanoseconds.
    let obj = JsObject::from_proto_and_data(proto, Instant { inner: instant });

    // 5. Return object.
    Ok(obj.into())
}

/// 8.5.3 `ToTemporalInstant ( item )`
#[inline]
fn to_temporal_instant(_: &JsValue) -> JsResult<InnerInstant> {
    // TODO: Need to implement parsing.
    Err(JsNativeError::error()
        .with_message("Instant parsing is not yet implemented.")
        .into())
}
