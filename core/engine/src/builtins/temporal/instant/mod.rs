//! Boa's implementation of ECMAScript's `Temporal.Instant` builtin object.

use crate::{
    builtins::{
        options::{get_option, get_options_object},
        temporal::{
            duration::{create_temporal_duration, to_temporal_duration_record},
            options::{get_temporal_unit, TemporalUnitGroup},
            ZonedDateTime,
        },
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::PreferredType,
    Context, JsArgs, JsBigInt, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol,
    JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use num_traits::ToPrimitive;
use temporal_rs::{
    options::{RoundingIncrement, RoundingOptions, TemporalRoundingMode},
    Instant as InnerInstant,
};

use super::options::get_difference_settings;

/// The `Temporal.Instant` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
// SAFETY: Instant does not contain any traceable values.
#[boa_gc(unsafe_empty_trace)]
pub struct Instant {
    pub(crate) inner: InnerInstant,
}

impl BuiltInObject for Instant {
    const NAME: JsString = StaticJsStrings::INSTANT_NAME;
}

impl IntrinsicObject for Instant {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let get_seconds = BuiltInBuilder::callable(realm, Self::get_epoch_seconds)
            .name(js_string!("get epochSeconds"))
            .build();

        let get_millis = BuiltInBuilder::callable(realm, Self::get_epoch_milliseconds)
            .name(js_string!("get epochMilliseconds"))
            .build();

        let get_micros = BuiltInBuilder::callable(realm, Self::get_epoch_microseconds)
            .name(js_string!("get epochMicroseconds"))
            .build();

        let get_nanos = BuiltInBuilder::callable(realm, Self::get_epoch_nanoseconds)
            .name(js_string!("get epochNanoseconds"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                JsSymbol::to_string_tag(),
                StaticJsStrings::INSTANT_TAG,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("epochSeconds"),
                Some(get_seconds),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("epochMilliseconds"),
                Some(get_millis),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("epochMicroseconds"),
                Some(get_micros),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("epochNanoseconds"),
                Some(get_nanos),
                None,
                Attribute::CONFIGURABLE,
            )
            .static_method(Self::from, js_string!("from"), 1)
            .static_method(
                Self::from_epoch_milliseconds,
                js_string!("fromEpochMilliseconds"),
                1,
            )
            .static_method(
                Self::from_epoch_nanoseconds,
                js_string!("fromEpochNanoseconds"),
                1,
            )
            .static_method(Self::compare, js_string!("compare"), 1)
            .method(Self::add, js_string!("add"), 1)
            .method(Self::subtract, js_string!("subtract"), 1)
            .method(Self::until, js_string!("until"), 1)
            .method(Self::since, js_string!("since"), 1)
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
    const P: usize = 13;
    const SP: usize = 4;

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
        // NOTE: temporal_rs::Instant asserts that the epochNanoseconds are valid.
        let instant = InnerInstant::try_new(epoch_nanos.as_inner().to_i128().unwrap_or(i128::MAX))?;
        // 4. Return ? CreateTemporalInstant(epochNanoseconds, NewTarget).
        create_temporal_instant(instant, Some(new_target.clone()), context)
    }
}

// ==== Instant Static method implementations ====

impl Instant {
    pub(crate) fn from(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. If item is an Object and item has an [[InitializedTemporalInstant]] internal slot, then
        // a. Return ! CreateTemporalInstant(item.[[Nanoseconds]]).
        // 2. Return ? ToTemporalInstant(item).
        create_temporal_instant(
            to_temporal_instant(args.get_or_undefined(0), context)?,
            None,
            context,
        )
        .map(Into::into)
    }

    pub(crate) fn from_epoch_milliseconds(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Set epochMilliseconds to ? ToNumber(epochMilliseconds).
        let epoch_millis = args.get_or_undefined(0).to_number(context)?;
        // 2. Set epochMilliseconds to ? NumberToBigInt(epochMilliseconds).
        // 3. Let epochNanoseconds be epochMilliseconds × ℤ(10**6).
        // 4. If IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
        // 5. Return ! CreateTemporalInstant(epochNanoseconds).
        create_temporal_instant(
            InnerInstant::from_epoch_milliseconds(epoch_millis.to_i128().unwrap_or(i128::MAX))?,
            None,
            context,
        )
        .map(Into::into)
    }

    pub(crate) fn from_epoch_nanoseconds(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Set epochNanoseconds to ? ToBigInt(epochNanoseconds).
        let epoch_nanos = args.get_or_undefined(0).to_bigint(context)?;
        // 2. If IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
        // 3. Return ! CreateTemporalInstant(epochNanoseconds).
        let nanos = epoch_nanos.as_inner().to_i128();
        create_temporal_instant(
            InnerInstant::try_new(nanos.unwrap_or(i128::MAX))?,
            None,
            context,
        )
        .map(Into::into)
    }

    pub(crate) fn compare(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Set one to ? ToTemporalInstant(one).
        let one = to_temporal_instant(args.get_or_undefined(0), context)?;
        // 2. Set two to ? ToTemporalInstant(two).
        let two = to_temporal_instant(args.get_or_undefined(1), context)?;
        // 3. Return 𝔽(CompareEpochNanoseconds(one.[[Nanoseconds]], two.[[Nanoseconds]])).
        Ok((one.cmp(&two) as i8).into())
    }
}

// ==== Instant method implementations ====

impl Instant {
    /// 8.3.3 get Temporal.Instant.prototype.epochSeconds
    pub(crate) fn get_epoch_seconds(
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
        Ok(JsBigInt::from(instant.inner.epoch_seconds()).into())
    }

    /// 8.3.4 get Temporal.Instant.prototype.epochMilliseconds
    pub(crate) fn get_epoch_milliseconds(
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
        Ok(JsBigInt::from(instant.inner.epoch_milliseconds()).into())
    }

    /// 8.3.5 get Temporal.Instant.prototype.epochMicroseconds
    pub(crate) fn get_epoch_microseconds(
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
        Ok(JsBigInt::from(instant.inner.epoch_microseconds()).into())
    }

    /// 8.3.6 get Temporal.Instant.prototype.epochNanoseconds
    pub(crate) fn get_epoch_nanoseconds(
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
        Ok(JsBigInt::from(instant.inner.epoch_nanoseconds()).into())
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
        let other = to_temporal_instant(args.get_or_undefined(0), context)?;

        // Fetch the necessary options.
        let settings =
            get_difference_settings(&get_options_object(args.get_or_undefined(1))?, context)?;
        let result = instant.inner.until(&other, settings)?;
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
        let other = to_temporal_instant(args.get_or_undefined(0), context)?;
        let settings =
            get_difference_settings(&get_options_object(args.get_or_undefined(1))?, context)?;
        let result = instant.inner.since(&other, settings)?;
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

        // 6. NOTE: The following steps read options and perform independent validation in
        // alphabetical order (ToTemporalRoundingIncrement reads "roundingIncrement" and ToTemporalRoundingMode reads "roundingMode").
        let mut options = RoundingOptions::default();
        // 7. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        options.increment =
            get_option::<RoundingIncrement>(&round_to, js_string!("roundingIncrement"), context)?;

        // 8. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        options.rounding_mode =
            get_option::<TemporalRoundingMode>(&round_to, js_string!("roundingMode"), context)?;

        // 9. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit"), time, required).
        let smallest_unit = get_temporal_unit(
            &round_to,
            js_string!("smallestUnit"),
            TemporalUnitGroup::Time,
            None,
            context,
        )?
        .ok_or_else(|| JsNativeError::range().with_message("smallestUnit cannot be undefined."))?;

        options.smallest_unit = Some(smallest_unit);

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
        let result = instant.inner.round(options)?;

        // 18. Return ! CreateTemporalInstant(roundedNs).
        create_temporal_instant(result, None, context)
    }

    /// 8.3.12 `Temporal.Instant.prototype.equals ( other )`
    pub(crate) fn equals(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
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
        let other_instant = to_temporal_instant(other, context)?;

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
fn to_temporal_instant(item: &JsValue, context: &mut Context) -> JsResult<InnerInstant> {
    // 1.If item is an Object, then
    let item = if let Some(obj) = item.as_object() {
        // a. If item has an [[InitializedTemporalInstant]] internal slot, then
        //     i. Return item.
        // b. If item has an [[InitializedTemporalZonedDateTime]] internal slot, then
        //     i. Return ! CreateTemporalInstant(item.[[Nanoseconds]]).
        // c. NOTE: This use of ToPrimitive allows Instant-like objects to be converted.
        // d. Set item to ? ToPrimitive(item, string).
        if let Some(instant) = obj.downcast_ref::<Instant>() {
            return Ok(instant.inner.clone());
        } else if let Some(_zdt) = obj.downcast_ref::<ZonedDateTime>() {
            return Err(JsNativeError::error()
                .with_message("Not yet implemented.")
                .into());
        }
        item.to_primitive(context, PreferredType::String)?
    } else {
        item.clone()
    };

    let Some(string_to_parse) = item.as_string() else {
        return Err(JsNativeError::typ()
            .with_message("Invalid type to convert to a Temporal.Instant.")
            .into());
    };

    // 3. Let parsed be ? ParseTemporalInstantString(item).
    // 4. If parsed.[[TimeZone]].[[Z]] is true, let offsetNanoseconds be 0; otherwise, let offsetNanoseconds be ! ParseDateTimeUTCOffset(parsed.[[TimeZone]].[[OffsetString]]).
    // 5. If abs(ISODateToEpochDays(parsed.[[Year]], parsed.[[Month]] - 1, parsed.[[Day]])) > 10**8, throw a RangeError exception.
    // 6. Let epochNanoseconds be GetUTCEpochNanoseconds(parsed.[[Year]], parsed.[[Month]], parsed.[[Day]], parsed.[[Hour]], parsed.[[Minute]], parsed.[[Second]], parsed.[[Millisecond]], parsed.[[Microsecond]], parsed.[[Nanosecond]], offsetNanoseconds).
    // 7. If IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
    // 8. Return ! CreateTemporalInstant(epochNanoseconds).
    // 2. If item is not a String, throw a TypeError exception.
    string_to_parse
        .to_std_string_escaped()
        .parse::<InnerInstant>()
        .map_err(Into::into)
}
