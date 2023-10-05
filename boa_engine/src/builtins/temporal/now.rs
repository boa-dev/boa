//! Boa's implementation of `Temporal.Now` `EcmaScript` object.

use crate::{
    builtins::temporal::{create_temporal_time_zone, default_time_zone},
    context::intrinsics::{Intrinsics, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, ObjectData, ObjectInitializer},
    property::Attribute,
    realm::Realm,
    value::IntegerOrInfinity,
    Context, JsBigInt, JsNativeError, JsObject, JsResult, JsSymbol, JsValue, NativeFunction,
};
use boa_profiler::Profiler;

use super::{NS_MAX_INSTANT, NS_MIN_INSTANT};
use std::time::SystemTime;

/// JavaScript `Temporal.Now` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Now;

impl Now {
    const NAME: &'static str = "Temporal.Now";

    /// Initializes the `Temporal.Now` object.
    pub(crate) fn init(realm: &Realm) -> JsValue {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        // is an ordinary object.
        // has a [[Prototype]] internal slot whose value is %Object.prototype%.
        // is not a function object.
        // does not have a [[Construct]] internal method; it cannot be used as a constructor with the new operator.
        // does not have a [[Call]] internal method; it cannot be invoked as a function.
        ObjectInitializer::new(realm.clone())
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::time_zone_id),
                "timeZoneId",
                0,
            )
            .function(NativeFunction::from_fn_ptr(Self::instant), "instant", 0)
            .function(
                NativeFunction::from_fn_ptr(Self::plain_date_time),
                "plainDateTime",
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::plain_date_time_iso),
                "plainDateTimeISO",
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::zoned_date_time),
                "zonedDateTime",
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::zoned_date_time_iso),
                "zonedDateTimeISO",
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::plain_date),
                "plainDate",
                2,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::plain_date_iso),
                "plainDateISO",
                1,
            )
            .build()
            .into()
    }

    /// `Temporal.Now.timeZoneId ( )`
    ///
    /// More information:
    ///  - [ECMAScript specififcation][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal.now.timezone
    #[allow(clippy::unnecessary_wraps)]
    fn time_zone_id(
        _: &JsValue,
        _args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Return ! SystemTimeZone().
        Ok(system_time_zone(context).expect("retrieving the system timezone must not fail"))
    }

    /// `Temporal.Now.instant()`
    fn instant(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// `Temporal.Now.plainDateTime()`
    fn plain_date_time(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// `Temporal.Now.plainDateTimeISO`
    fn plain_date_time_iso(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// `Temporal.Now.zonedDateTime`
    fn zoned_date_time(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// `Temporal.Now.zonedDateTimeISO`
    fn zoned_date_time_iso(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// `Temporal.Now.plainDate()`
    fn plain_date(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }

    /// `Temporal.Now.plainDateISO`
    fn plain_date_iso(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        todo!()
    }
}

// -- Temporal.Now abstract operations --

/// 2.3.1 `HostSystemUTCEpochNanoseconds ( global )`
fn host_system_utc_epoch_nanoseconds() -> JsResult<JsBigInt> {
    // TODO: Implement `SystemTime::now()` calls manually. Needed for `no_std`
    let epoch_nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| JsNativeError::range().with_message(e.to_string()))?
        .as_nanos();
    Ok(clamp_epoc_nanos(JsBigInt::from(epoch_nanos)))
}

fn clamp_epoc_nanos(ns: JsBigInt) -> JsBigInt {
    let max = JsBigInt::from(NS_MAX_INSTANT);
    let min = JsBigInt::from(NS_MIN_INSTANT);
    ns.clamp(min, max)
}

/// 2.3.2 `SystemUTCEpochNanoseconds`
#[allow(unused)]
fn system_utc_epoch_nanos() -> JsBigInt {
    todo!()
}

/// `SystemInstant`
#[allow(unused)]
fn system_instant() {
    todo!()
}

/// `SystemDateTime`
#[allow(unused)]
fn system_date_time() {
    todo!()
}

/// `SystemZonedDateTime`
#[allow(unused)]
fn system_zoned_date_time() {
    todo!()
}

/// Abstract operation `SystemTimeZone ( )`
///
/// More information:
///  - [ECMAScript specififcation][spec]
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-systemtimezone
#[allow(unused)]
fn system_time_zone(context: &mut Context<'_>) -> JsResult<JsValue> {
    // 1. Let identifier be ! DefaultTimeZone().
    let identifier = default_time_zone(context);
    // 2. Return ! CreateTemporalTimeZone(identifier).
    create_temporal_time_zone(identifier, None, context)
}
