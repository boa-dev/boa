#![allow(dead_code)]
//! Boa's implementation of ECMAScript's `Temporal.Instant` object.

use crate::{
    builtins::{
        temporal::to_temporal_duration_record, BuiltInBuilder, BuiltInConstructor, BuiltInObject,
        IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    string::utf16,
    Context, JsArgs, JsBigInt, JsNativeError, JsObject, JsResult, JsSymbol, JsValue,
};
use boa_profiler::Profiler;
use num_bigint::ToBigInt;

const NS_MAX_INSTANT: f64 = 8.64e21;
const NS_PER_DAY: f64 = 8.64e13;
const NS_MIN_INSTANT: f64 = -8.64e21;

#[derive(Debug, Clone)]
pub struct Instant {
    pub(crate) nanoseconds: JsBigInt,
}

impl BuiltInObject for Instant {
    const NAME: &'static str = "Temporal.Instant";
}

impl IntrinsicObject for Instant {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let get_seconds = BuiltInBuilder::new(realm)
            .callable(Self::get_epoc_seconds)
            .name("get epochSeconds")
            .build();

        let get_millis = BuiltInBuilder::new(realm)
            .callable(Self::get_epoc_milliseconds)
            .name("get epochMilliseconds")
            .build();

        let get_micros = BuiltInBuilder::new(realm)
            .callable(Self::get_epoc_microseconds)
            .name("get epochMicroseconds")
            .build();

        let get_nanos = BuiltInBuilder::new(realm)
            .callable(Self::get_epoc_nanoseconds)
            .name("get epochNanoseconds")
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
            .method(Self::add, "add", 1)
            .method(Self::subtract, "subtract", 1)
            .method(Self::until, "until", 2)
            .method(Self::since, "since", 2)
            .method(Self::round, "round", 1)
            .method(Self::equals, "equals", 1)
            .method(Self::to_zoned_date_time, "toZonedDateTime", 1)
            .method(Self::to_zoned_date_time_iso, "toZonedDateTimeISO", 1)
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

        let epoch_nanos = args.get_or_undefined(0).to_bigint(context)?;
        // 2. Let epochNanoseconds be ? ToBigInt(epochNanoseconds).
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
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let o = o.borrow();
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
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let o = o.borrow();
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
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let o = o.borrow();
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
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let o = o.borrow();
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;
        // 3. Let ns be instant.[[Nanoseconds]].
        let ns = &instant.nanoseconds;
        // 4. Return ns.
        Ok(ns.clone().into())
    }

    /// 8.3.7 Temporal.Instant.prototype.add ( temporalDurationLike )
    pub(crate) fn add(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let instant be the this value.
        // 2. Perform ? RequireInternalSlot(instant, [[InitializedTemporalInstant]]).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("this value of Instant must be an object.")
        })?;
        let o = o.borrow();
        let instant = o.as_instant().ok_or_else(|| {
            JsNativeError::typ().with_message("the this object must be an instant object.")
        })?;
        // 3. Return ? AddDurationToOrSubtractDurationFromInstant(add, instant, temporalDurationLike).
        let temporal_duration_like = args.get_or_undefined(0);
        add_or_subtract_duration_from_instant(true, instant, temporal_duration_like, context)
    }

    /// 8.3.8 Temporal.Instant.prototype.subtract ( temporalDurationLike )
    pub(crate) fn subtract(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// 8.3.9 Temporal.Instant.prototype.until ( other [ , options ] )
    pub(crate) fn until(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// 8.3.10 Temporal.Instant.prototype.since ( other [ , options ] )
    pub(crate) fn since(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// 8.3.11 Temporal.Instant.prototype.round ( roundTo )
    pub(crate) fn round(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// 8.3.12 Temporal.Instant.prototype.equals ( other )
    pub(crate) fn equals(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// 8.3.17 Temporal.Instant.prototype.toZonedDateTime ( item )
    pub(crate) fn to_zoned_date_time(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }

    /// 8.3.18 Temporal.Instant.prototype.toZonedDateTimeISO ( timeZone )
    pub(crate) fn to_zoned_date_time_iso(
        _: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }
}

// -- Instant Abstract Operations --

/// 8.5.1 IsValidEpochNanoseconds ( epochNanoseconds )
fn is_valid_epoch_nanos(epoch_nanos: &JsBigInt) -> bool {
    // 1. Assert: Type(epochNanoseconds) is BigInt.
    // 2. If ℝ(epochNanoseconds) < nsMinInstant or ℝ(epochNanoseconds) > nsMaxInstant, then
    if epoch_nanos.to_f64() < NS_MIN_INSTANT || epoch_nanos.to_f64() > NS_MAX_INSTANT {
        // a. Return false.
        return false;
    }
    // 3. Return true.
    true
}

/// 8.5.2 CreateTemporalInstant ( epochNanoseconds [ , newTarget ] )
fn create_temporal_instant(
    epoch_nanos: JsBigInt,
    new_target: Option<JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. Assert: ! IsValidEpochNanoseconds(epochNanoseconds) is true.
    assert!(is_valid_epoch_nanos(&epoch_nanos));
    // 2. If newTarget is not present, set newTarget to %Temporal.Instant%.
    let new_target = new_target.unwrap_or(
        context
            .realm()
            .intrinsics()
            .constructors()
            .instant()
            .constructor()
            .into(),
    );
    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Instant.prototype%", « [[InitializedTemporalInstant]], [[Nanoseconds]] »).
    let new_instant =
        get_prototype_from_constructor(&new_target, StandardConstructors::instant, context)?;
    // 4. Set object.[[Nanoseconds]] to epochNanoseconds.
    new_instant
        .borrow_mut()
        .as_instant_mut()
        .expect("created object must be a `Temporal.Instant`.")
        .nanoseconds = epoch_nanos;
    // 5. Return object.
    Ok(new_instant.into())
}

/// 8.5.6 AddInstant ( epochNanoseconds, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )
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
        JsBigInt::mul(&JsBigInt::from(hours), &JsBigInt::from(36_000_000_000_000_i64)),
        JsBigInt::mul(&JsBigInt::from(minutes), &JsBigInt::from(600_000_000_000_i64)),
        JsBigInt::mul(&JsBigInt::from(seconds), &JsBigInt::from(10_000_000_000_i64)),
        JsBigInt::mul(&JsBigInt::from(millis),  &JsBigInt::from(10_000_000_i32)),
        JsBigInt::mul(&JsBigInt::from(micros), &JsBigInt::from(1000_i32)),
        JsBigInt::add(&JsBigInt::from(nanos), &epoch_nanos),
    ]);
    if !is_valid_epoch_nanos(&result) {
        return Err(JsNativeError::range().with_message("result is not a valid epoch nanosecond value.").into())
    }
    Ok(result)
}

/// 8.5.11 AddDurationToOrSubtractDurationFromInstant ( operation, instant, temporalDurationLike )
fn add_or_subtract_duration_from_instant(
    op: bool,
    instant: &Instant,
    temporal_duration_like: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. If operation is subtract, let sign be -1. Otherwise, let sign be 1.
    let sign = if op { 1 } else { -1 };
    // 2. Let duration be ? ToTemporalDurationRecord(temporalDurationLike).
    let duration = to_temporal_duration_record(temporal_duration_like)?;
    // 3. If duration.[[Days]] is not 0, throw a RangeError exception.
    if duration.days != 0.0 {}
    // 4. If duration.[[Months]] is not 0, throw a RangeError exception.
    if duration.months != 0.0 {}
    // 5. If duration.[[Weeks]] is not 0, throw a RangeError exception.
    if duration.weeks != 0.0 {}
    // 6. If duration.[[Years]] is not 0, throw a RangeError exception.
    if duration.years != 0.0 {}
    // 7. Let ns be ? AddInstant(instant.[[Nanoseconds]], sign × duration.[[Hours]],
    // sign × duration.[[Minutes]], sign × duration.[[Seconds]], sign × duration.[[Milliseconds]],
    // sign × duration.[[Microseconds]], sign × duration.[[Nanoseconds]]).
    let new = add_instant(
        &instant.nanoseconds,
        sign * duration.hours as i32,
        sign * duration.minutes as i32,
        sign * duration.seconds as i32,
        sign * duration.milliseconds as i32,
        sign * duration.microseconds as i32,
        sign * duration.nanoseconds as i32,
    )?;
    // 8. Return ! CreateTemporalInstant(ns).
    create_temporal_instant(
        new,
        None,
        context,
    )
}
