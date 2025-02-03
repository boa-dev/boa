//! Boa's implementation of `Temporal.Now` ECMAScript Builtin object.

use crate::{
    builtins::{BuiltInBuilder, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    sys::time::SystemTime,
    Context, JsBigInt, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;
use temporal_rs::{Now as NowInner, TimeZone};

use super::{
    create_temporal_date, create_temporal_datetime, create_temporal_instant, create_temporal_time,
    create_temporal_zoneddatetime, ns_max_instant, ns_min_instant, time_zone::default_time_zone,
    to_temporal_timezone_identifier,
};

/// JavaScript `Temporal.Now` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Now;

impl IntrinsicObject for Now {
    /// Initializes the `Temporal.Now` object.
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        // is an ordinary object.
        // has a [[Prototype]] internal slot whose value is %Object.prototype%.
        // is not a function object.
        // does not have a [[Construct]] internal method; it cannot be used as a constructor with the new operator.
        // does not have a [[Call]] internal method; it cannot be invoked as a function.
        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                StaticJsStrings::NOW_TAG,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_method(Self::time_zone_id, js_string!("timeZoneId"), 0)
            .static_method(Self::instant, js_string!("instant"), 0)
            .static_method(Self::plain_datetime, js_string!("plainDateTimeISO"), 0)
            .static_method(Self::zoneddatetime, js_string!("zonedDateTimeISO"), 0)
            .static_method(Self::plain_date, js_string!("plainDateISO"), 0)
            .static_method(Self::plain_time, js_string!("plainTimeISO"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().now()
    }
}

impl BuiltInObject for Now {
    const NAME: JsString = StaticJsStrings::NOW_NAME;
}

impl Now {
    /// `Temporal.Now.timeZoneId ( )`
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal.now.timezone
    #[allow(clippy::unnecessary_wraps)]
    fn time_zone_id(_: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Return ! SystemTimeZone().
        system_time_zone(context)?
            .identifier()
            .map(|s| JsValue::from(js_string!(s.as_str())))
            .map_err(Into::into)
    }

    /// `Temporal.Now.instant()`
    fn instant(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        create_temporal_instant(NowInner::instant()?, None, context).map(Into::into)
    }

    /// `Temporal.Now.plainDateTime()`
    fn plain_datetime(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let tz = args
            .first()
            .map(|v| to_temporal_timezone_identifier(v, context))
            .transpose()?;
        create_temporal_datetime(
            NowInner::plain_datetime_iso_with_provider(tz, context.tz_provider())?,
            None,
            context,
        )
        .map(Into::into)
    }

    /// `Temporal.Now.zonedDateTime`
    fn zoneddatetime(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let timezone = args
            .first()
            .map(|v| to_temporal_timezone_identifier(v, context))
            .transpose()?;
        let zdt = NowInner::zoneddatetime_iso(timezone)?;
        create_temporal_zoneddatetime(zdt, None, context).map(Into::into)
    }

    /// `Temporal.Now.plainDateISO`
    fn plain_date(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let tz = args
            .first()
            .map(|v| to_temporal_timezone_identifier(v, context))
            .transpose()?;
        let pd = NowInner::plain_date_iso_with_provider(tz, context.tz_provider())?;
        create_temporal_date(pd, None, context).map(Into::into)
    }

    fn plain_time(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let tz = args
            .first()
            .map(|v| to_temporal_timezone_identifier(v, context))
            .transpose()?;
        let pt = NowInner::plain_time_iso_with_provider(tz, context.tz_provider())?;
        create_temporal_time(pt, None, context).map(Into::into)
    }
}

// -- Temporal.Now abstract operations --

/// 2.3.1 `HostSystemUTCEpochNanoseconds ( global )`
fn host_system_utc_epoch_nanoseconds() -> JsResult<JsBigInt> {
    // TODO: Implement `SystemTime::now()` calls for `no_std`
    let epoch_nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| JsNativeError::range().with_message(e.to_string()))?
        .as_nanos();
    Ok(clamp_epoc_nanos(JsBigInt::from(epoch_nanos)))
}

fn clamp_epoc_nanos(ns: JsBigInt) -> JsBigInt {
    let max = ns_max_instant();
    let min = ns_min_instant();
    ns.clamp(min, max)
}

/// 2.3.2 `SystemUTCEpochMilliseconds`
#[allow(unused)]
fn system_utc_epoch_millis() -> JsResult<f64> {
    let now = host_system_utc_epoch_nanoseconds()?;
    Ok(now.to_f64().div_euclid(1_000_000_f64).floor())
}

/// 2.3.3 `SystemUTCEpochNanoseconds`
#[allow(unused)]
fn system_utc_epoch_nanos() -> JsResult<JsBigInt> {
    host_system_utc_epoch_nanoseconds()
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
fn system_time_zone(context: &mut Context) -> JsResult<TimeZone> {
    // 1. Let identifier be ! DefaultTimeZone().
    let identifier = default_time_zone(context);
    // 2. Return ! CreateTemporalTimeZone(identifier).

    Err(JsNativeError::error()
        .with_message("not yet implemented.")
        .into())
}
