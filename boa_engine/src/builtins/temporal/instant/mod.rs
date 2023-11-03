//! Boa's implementation of ECMAScript's `Temporal.Instant` builtin object.
#![allow(dead_code)]

use crate::{
    builtins::{
        options::{get_option, get_options_object, RoundingMode},
        temporal::{
            duration::{DateDuration, TimeDuration},
            options::{
                get_temporal_rounding_increment, get_temporal_unit, TemporalUnit, TemporalUnitGroup,
            },
        },
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, ObjectData},
    property::Attribute,
    realm::Realm,
    string::{common::StaticJsStrings, utf16},
    Context, JsArgs, JsBigInt, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

use super::{duration, ns_max_instant, ns_min_instant, MIS_PER_DAY, MS_PER_DAY, NS_PER_DAY};

const NANOSECONDS_PER_SECOND: i64 = 10_000_000_000;
const NANOSECONDS_PER_MINUTE: i64 = 600_000_000_000;
const NANOSECONDS_PER_HOUR: i64 = 36_000_000_000_000;

/// The `Temporal.Instant` object.
#[derive(Debug, Clone)]
pub struct Instant {
    pub(crate) nanoseconds: JsBigInt,
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
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("epochSeconds"),
                Some(get_seconds),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("epochMilliseconds"),
                Some(get_millis),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("epochMicroseconds"),
                Some(get_micros),
                None,
                Attribute::default(),
            )
            .accessor(
                utf16!("epochNanoseconds"),
                Some(get_nanos),
                None,
                Attribute::default(),
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
        context: &mut Context<'_>,
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
        if !is_valid_epoch_nanos(&epoch_nanos) {
            return Err(JsNativeError::range()
                .with_message("Temporal.Instant must have a valid epochNanoseconds.")
                .into());
        };
        // 4. Return ? CreateTemporalInstant(epochNanoseconds, NewTarget).
        create_temporal_instant(epoch_nanos, Some(new_target.clone()), context)
    }
}

// -- Instant method implementations --

impl Instant {
    /// 8.3.3 get Temporal.Instant.prototype.epochSeconds
    pub(crate) fn get_epoc_seconds(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;
        // 3. Let ns be instant.[[Nanoseconds]].
        let ns = &instant.nanoseconds;
        // 4. Let s be floor(ℝ(ns) / 10e9).
        let s = (ns.to_f64() / 10e9).floor();
        // 5. Return 𝔽(s).
        Ok(s.into())
    }

    /// 8.3.4 get Temporal.Instant.prototype.epochMilliseconds
    pub(crate) fn get_epoc_milliseconds(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;
        // 3. Let ns be instant.[[Nanoseconds]].
        let ns = &instant.nanoseconds;
        // 4. Let ms be floor(ℝ(ns) / 106).
        let ms = (ns.to_f64() / 10e6).floor();
        // 5. Return 𝔽(ms).
        Ok(ms.into())
    }

    /// 8.3.5 get Temporal.Instant.prototype.epochMicroseconds
    pub(crate) fn get_epoc_microseconds(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;
        // 3. Let ns be instant.[[Nanoseconds]].
        let ns = &instant.nanoseconds;
        // 4. Let µs be floor(ℝ(ns) / 103).
        let micro_s = (ns.to_f64() / 10e3).floor();
        // 5. Return ℤ(µs).
        let big_int = JsBigInt::try_from(micro_s).map_err(|_| {
            JsNativeError::typ().with_message("Could not convert microseconds to JsBigInt value")
        })?;
        Ok(big_int.into())
    }

    /// 8.3.6 get Temporal.Instant.prototype.epochNanoseconds
    pub(crate) fn get_epoc_nanoseconds(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;
        // 3. Let ns be instant.[[Nanoseconds]].
        let ns = &instant.nanoseconds;
        // 4. Return ns.
        Ok(ns.clone().into())
    }

    /// 8.3.7 `Temporal.Instant.prototype.add ( temporalDurationLike )`
    pub(crate) fn add(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;

        // 3. Return ? AddDurationToOrSubtractDurationFromInstant(add, instant, temporalDurationLike).
        let temporal_duration_like = args.get_or_undefined(0);
        add_or_subtract_duration_from_instant(true, instant, temporal_duration_like, context)
    }

    /// 8.3.8 `Temporal.Instant.prototype.subtract ( temporalDurationLike )`
    pub(crate) fn subtract(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;

        // 3. Return ? AddDurationToOrSubtractDurationFromInstant(subtract, instant, temporalDurationLike).
        let temporal_duration_like = args.get_or_undefined(0);
        add_or_subtract_duration_from_instant(false, instant, temporal_duration_like, context)
    }

    /// 8.3.9 `Temporal.Instant.prototype.until ( other [ , options ] )`
    pub(crate) fn until(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;

        // 3. Return ? DifferenceTemporalInstant(until, instant, other, options).
        let other = args.get_or_undefined(0);
        let option = args.get_or_undefined(1);
        diff_temporal_instant(true, instant, other, option, context)
    }

    /// 8.3.10 `Temporal.Instant.prototype.since ( other [ , options ] )`
    pub(crate) fn since(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;

        // 3. Return ? DifferenceTemporalInstant(since, instant, other, options).
        let other = args.get_or_undefined(0);
        let option = args.get_or_undefined(1);
        diff_temporal_instant(false, instant, other, option, context)
    }

    /// 8.3.11 `Temporal.Instant.prototype.round ( roundTo )`
    pub(crate) fn round(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;

        let round_to = args.get_or_undefined(0);
        // 3. If roundTo is undefined, then
        if round_to.is_undefined() {
            // a. Throw a TypeError exception.
            return Err(JsNativeError::typ()
                .with_message("roundTo cannot be undefined.")
                .into());
        };
        // 4. If Type(roundTo) is String, then
        let round_to = if round_to.is_string() {
            // a. Let paramString be roundTo.
            let param_string = round_to
                .as_string()
                .expect("roundTo is confirmed to be a string here.");
            // b. Set roundTo to OrdinaryObjectCreate(null).
            let new_round_to = JsObject::with_null_proto();
            // c. Perform ! CreateDataPropertyOrThrow(roundTo, "smallestUnit"), paramString).
            new_round_to.create_data_property_or_throw(
                utf16!("smallestUnit"),
                param_string.clone(),
                context,
            )?;
            new_round_to
        // 5. Else,
        } else {
            // a. Set roundTo to ? GetOptionsObject(roundTo).
            get_options_object(round_to)?
        };

        // 6. NOTE: The following steps read options and perform independent validation in
        // alphabetical order (ToTemporalRoundingIncrement reads "roundingIncrement" and ToTemporalRoundingMode reads "roundingMode").
        // 7. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        let rounding_increment = get_temporal_rounding_increment(&round_to, context)?;

        // 8. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        let rounding_mode =
            get_option(&round_to, utf16!("roundingMode"), context)?.unwrap_or_default();

        // 9. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit"), time, required).
        let smallest_unit = get_temporal_unit(
            &round_to,
            utf16!("smallestUnit"),
            TemporalUnitGroup::Time,
            None,
            context,
        )?
        .ok_or_else(|| JsNativeError::range().with_message("smallestUnit cannot be undefined."))?;

        let maximum = match smallest_unit {
            // 10. If smallestUnit is "hour"), then
            // a. Let maximum be HoursPerDay.
            TemporalUnit::Hour => 24,
            // 11. Else if smallestUnit is "minute"), then
            // a. Let maximum be MinutesPerHour × HoursPerDay.
            TemporalUnit::Minute => 14400,
            // 12. Else if smallestUnit is "second"), then
            // a. Let maximum be SecondsPerMinute × MinutesPerHour × HoursPerDay.
            TemporalUnit::Second => 86400,
            // 13. Else if smallestUnit is "millisecond"), then
            // a. Let maximum be ℝ(msPerDay).
            TemporalUnit::Millisecond => i64::from(MS_PER_DAY),
            // 14. Else if smallestUnit is "microsecond"), then
            // a. Let maximum be 10^3 × ℝ(msPerDay).
            TemporalUnit::Microsecond => MIS_PER_DAY,
            // 15. Else,
            // a. Assert: smallestUnit is "nanosecond".
            // b. Let maximum be nsPerDay.
            TemporalUnit::Nanosecond => NS_PER_DAY,
            // unreachable here functions as 15.a.
            _ => unreachable!(),
        };

        // 16. Perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, true).
        super::validate_temporal_rounding_increment(rounding_increment, maximum as f64, true)?;

        // 17. Let roundedNs be RoundTemporalInstant(instant.[[Nanoseconds]], roundingIncrement, smallestUnit, roundingMode).
        let rounded_ns = round_temporal_instant(
            &instant.nanoseconds,
            rounding_increment,
            smallest_unit,
            rounding_mode,
        )?;

        // 18. Return ! CreateTemporalInstant(roundedNs).
        create_temporal_instant(rounded_ns, None, context)
    }

    /// 8.3.12 `Temporal.Instant.prototype.equals ( other )`
    pub(crate) fn equals(
        this: &JsValue,
        args: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        // 4. If instant.[[Nanoseconds]] ≠ other.[[Nanoseconds]], return false.
        // 5. Return true.
        let o = this.as_object().map(JsObject::borrow).ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;

        // 3. Set other to ? ToTemporalInstant(other).
        let other = args.get_or_undefined(0);
        let other_instant = to_temporal_instant(other)?;

        if instant.nanoseconds != other_instant.nanoseconds {
            return Ok(false.into());
        }
        Ok(true.into())
    }

    /// 8.3.17 `Temporal.Instant.prototype.toZonedDateTime ( item )`
    pub(crate) fn to_zoned_date_time(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
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
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // TODO Complete
        Err(JsNativeError::error()
            .with_message("not yet implemented.")
            .into())
    }
}

// -- Instant Abstract Operations --

/// 8.5.1 `IsValidEpochNanoseconds ( epochNanoseconds )`
#[inline]
fn is_valid_epoch_nanos(epoch_nanos: &JsBigInt) -> bool {
    // 1. Assert: Type(epochNanoseconds) is BigInt.
    // 2. If ℝ(epochNanoseconds) < nsMinInstant or ℝ(epochNanoseconds) > nsMaxInstant, then
    if epoch_nanos.to_f64() < ns_min_instant().to_f64()
        || epoch_nanos.to_f64() > ns_max_instant().to_f64()
    {
        // a. Return false.
        return false;
    }
    // 3. Return true.
    true
}

/// 8.5.2 `CreateTemporalInstant ( epochNanoseconds [ , newTarget ] )`
#[inline]
fn create_temporal_instant(
    epoch_nanos: JsBigInt,
    new_target: Option<JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. Assert: ! IsValidEpochNanoseconds(epochNanoseconds) is true.
    assert!(is_valid_epoch_nanos(&epoch_nanos));
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
    let obj = JsObject::from_proto_and_data(
        proto,
        ObjectData::instant(Instant {
            nanoseconds: epoch_nanos,
        }),
    );

    // 5. Return object.
    Ok(obj.into())
}

/// 8.5.3 `ToTemporalInstant ( item )`
#[inline]
fn to_temporal_instant(_: &JsValue) -> JsResult<Instant> {
    // TODO: Need to implement parsing.
    Err(JsNativeError::error()
        .with_message("Instant parsing is not yet implemented.")
        .into())
}

/// 8.5.6 `AddInstant ( epochNanoseconds, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )`
#[inline]
fn add_instant(
    epoch_nanos: &JsBigInt,
    hours: i32,
    minutes: i32,
    seconds: i32,
    millis: i32,
    micros: i32,
    nanos: i32,
) -> JsResult<JsBigInt> {
    let result = JsBigInt::add_n(&[
        JsBigInt::mul(
            &JsBigInt::from(hours),
            &JsBigInt::from(NANOSECONDS_PER_HOUR),
        ),
        JsBigInt::mul(
            &JsBigInt::from(minutes),
            &JsBigInt::from(NANOSECONDS_PER_MINUTE),
        ),
        JsBigInt::mul(
            &JsBigInt::from(seconds),
            &JsBigInt::from(NANOSECONDS_PER_SECOND),
        ),
        JsBigInt::mul(&JsBigInt::from(millis), &JsBigInt::from(10_000_000_i32)),
        JsBigInt::mul(&JsBigInt::from(micros), &JsBigInt::from(1000_i32)),
        JsBigInt::add(&JsBigInt::from(nanos), epoch_nanos),
    ]);
    if !is_valid_epoch_nanos(&result) {
        return Err(JsNativeError::range()
            .with_message("result is not a valid epoch nanosecond value.")
            .into());
    }
    Ok(result)
}

/// 8.5.7 `DifferenceInstant ( ns1, ns2, roundingIncrement, smallestUnit, largestUnit, roundingMode )`
#[inline]
fn diff_instant(
    ns1: &JsBigInt,
    ns2: &JsBigInt,
    rounding_increment: f64,
    smallest_unit: TemporalUnit,
    largest_unit: TemporalUnit,
    rounding_mode: RoundingMode,
    context: &mut Context<'_>,
) -> JsResult<duration::DurationRecord> {
    // 1. Let difference be ℝ(ns2) - ℝ(ns1).
    let difference = JsBigInt::sub(ns1, ns2);
    // 2. Let nanoseconds be remainder(difference, 1000).
    let nanoseconds = JsBigInt::rem(&difference, &JsBigInt::from(1000));
    // 3. Let microseconds be remainder(truncate(difference / 1000), 1000).
    let truncated_micro = JsBigInt::try_from((&difference.to_f64() / 1000_f64).trunc())
        .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
    let microseconds = JsBigInt::rem(&truncated_micro, &JsBigInt::from(1000));

    // 4. Let milliseconds be remainder(truncate(difference / 106), 1000).
    let truncated_milli = JsBigInt::try_from((&difference.to_f64() / 1_000_000_f64).trunc())
        .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
    let milliseconds = JsBigInt::rem(&truncated_milli, &JsBigInt::from(1000));

    // 5. Let seconds be truncate(difference / 109).
    let seconds = (&difference.to_f64() / 1_000_000_000_f64).trunc();

    // 6. Let roundResult be ! RoundDuration(0, 0, 0, 0, 0, 0, seconds, milliseconds, microseconds, nanoseconds, roundingIncrement, smallestUnit, largestUnit, roundingMode).
    let mut roundable_duration = duration::DurationRecord::new(
        DateDuration::default(),
        TimeDuration::new(
            0.0,
            0.0,
            seconds,
            milliseconds.to_f64(),
            microseconds.to_f64(),
            nanoseconds.to_f64(),
        ),
    );
    let _total = roundable_duration.round_duration(
        rounding_increment,
        smallest_unit,
        rounding_mode,
        None,
        None,
        None,
        context,
    )?;

    // 7. Assert: roundResult.[[Days]] is 0.
    assert_eq!(roundable_duration.days() as i32, 0);

    // 8. Return ! BalanceTimeDuration(0, roundResult.[[Hours]], roundResult.[[Minutes]],
    //    roundResult.[[Seconds]], roundResult.[[Milliseconds]], roundResult.[[Microseconds]],
    //    roundResult.[[Nanoseconds]], largestUnit).
    roundable_duration.balance_time_duration(largest_unit, None)?;

    Ok(roundable_duration)
}

/// 8.5.8 `RoundTemporalInstant ( ns, increment, unit, roundingMode )`
#[inline]
fn round_temporal_instant(
    ns: &JsBigInt,
    increment: f64,
    unit: TemporalUnit,
    rounding_mode: RoundingMode,
) -> JsResult<JsBigInt> {
    let increment_ns = match unit {
        // 1. If unit is "hour"), then
        TemporalUnit::Hour => {
            // a. Let incrementNs be increment × 3.6 × 10^12.
            increment as i64 * NANOSECONDS_PER_HOUR
        }
        // 2. Else if unit is "minute"), then
        TemporalUnit::Minute => {
            // a. Let incrementNs be increment × 6 × 10^10.
            increment as i64 * NANOSECONDS_PER_MINUTE
        }
        // 3. Else if unit is "second"), then
        TemporalUnit::Second => {
            // a. Let incrementNs be increment × 10^9.
            increment as i64 * NANOSECONDS_PER_SECOND
        }
        // 4. Else if unit is "millisecond"), then
        TemporalUnit::Millisecond => {
            // a. Let incrementNs be increment × 10^6.
            increment as i64 * 1_000_000
        }
        // 5. Else if unit is "microsecond"), then
        TemporalUnit::Microsecond => {
            // a. Let incrementNs be increment × 10^3.
            increment as i64 * 1000
        }
        // 6. Else,
        TemporalUnit::Nanosecond => {
            // NOTE: We shouldn't have to assert here as `unreachable` asserts instead.
            // a. Assert: unit is "nanosecond".
            // b. Let incrementNs be increment.
            increment as i64
        }
        _ => unreachable!(),
    };

    // 7. Return ℤ(RoundNumberToIncrementAsIfPositive(ℝ(ns), incrementNs, roundingMode)).
    super::round_to_increment_as_if_positive(ns, increment_ns, rounding_mode)
}

/// 8.5.10 `DifferenceTemporalInstant ( operation, instant, other, options )`
#[inline]
fn diff_temporal_instant(
    op: bool,
    instant: &Instant,
    other: &JsValue,
    options: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. If operation is since, let sign be -1. Otherwise, let sign be 1.
    let sign = if op { 1_f64 } else { -1_f64 };
    // 2. Set other to ? ToTemporalInstant(other).
    let other = to_temporal_instant(other)?;
    // 3. Let resolvedOptions be ? CopyOptions(options).
    let resolved_options =
        super::snapshot_own_properties(&get_options_object(options)?, None, None, context)?;

    // 4. Let settings be ? GetDifferenceSettings(operation, resolvedOptions, time, « », "nanosecond"), "second").
    let settings = super::get_diff_settings(
        op,
        &resolved_options,
        TemporalUnitGroup::Time,
        &[],
        TemporalUnit::Nanosecond,
        TemporalUnit::Second,
        context,
    )?;

    // 5. Let result be DifferenceInstant(instant.[[Nanoseconds]], other.[[Nanoseconds]], settings.[[RoundingIncrement]], settings.[[SmallestUnit]], settings.[[LargestUnit]], settings.[[RoundingMode]]).
    let result = diff_instant(
        &instant.nanoseconds,
        &other.nanoseconds,
        settings.3,
        settings.0,
        settings.1,
        settings.2,
        context,
    )?;

    // 6. Return ! CreateTemporalDuration(0, 0, 0, 0, sign × result.[[Hours]], sign × result.[[Minutes]], sign × result.[[Seconds]], sign × result.[[Milliseconds]], sign × result.[[Microseconds]], sign × result.[[Nanoseconds]]).
    Ok(duration::create_temporal_duration(
        duration::DurationRecord::new(
            DateDuration::default(),
            TimeDuration::new(
                sign * result.hours(),
                sign * result.minutes(),
                sign * result.seconds(),
                sign * result.milliseconds(),
                sign * result.microseconds(),
                sign * result.nanoseconds(),
            ),
        ),
        None,
        context,
    )?
    .into())
}

/// 8.5.11 `AddDurationToOrSubtractDurationFromInstant ( operation, instant, temporalDurationLike )`
#[inline]
fn add_or_subtract_duration_from_instant(
    op: bool,
    instant: &Instant,
    temporal_duration_like: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. If operation is subtract, let sign be -1. Otherwise, let sign be 1.
    let sign = if op { 1 } else { -1 };
    // 2. Let duration be ? ToTemporalDurationRecord(temporalDurationLike).
    let duration = super::to_temporal_duration_record(temporal_duration_like)?;
    // 3. If duration.[[Days]] is not 0, throw a RangeError exception.
    if duration.days() != 0_f64 {
        return Err(JsNativeError::range()
            .with_message("DurationDays cannot be 0")
            .into());
    }
    // 4. If duration.[[Months]] is not 0, throw a RangeError exception.
    if duration.months() != 0_f64 {
        return Err(JsNativeError::range()
            .with_message("DurationMonths cannot be 0")
            .into());
    }
    // 5. If duration.[[Weeks]] is not 0, throw a RangeError exception.
    if duration.weeks() != 0_f64 {
        return Err(JsNativeError::range()
            .with_message("DurationWeeks cannot be 0")
            .into());
    }
    // 6. If duration.[[Years]] is not 0, throw a RangeError exception.
    if duration.years() != 0_f64 {
        return Err(JsNativeError::range()
            .with_message("DurationYears cannot be 0")
            .into());
    }
    // 7. Let ns be ? AddInstant(instant.[[Nanoseconds]], sign × duration.[[Hours]],
    // sign × duration.[[Minutes]], sign × duration.[[Seconds]], sign × duration.[[Milliseconds]],
    // sign × duration.[[Microseconds]], sign × duration.[[Nanoseconds]]).
    let new = add_instant(
        &instant.nanoseconds,
        sign * duration.hours() as i32,
        sign * duration.minutes() as i32,
        sign * duration.seconds() as i32,
        sign * duration.milliseconds() as i32,
        sign * duration.microseconds() as i32,
        sign * duration.nanoseconds() as i32,
    )?;
    // 8. Return ! CreateTemporalInstant(ns).
    create_temporal_instant(new, None, context)
}
