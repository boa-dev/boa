//! Boa's implementation of `Temporal.Now` ECMAScript Builtin object.

use crate::{
    builtins::{BuiltInBuilder, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;
use temporal_rs::{time::EpochNanoseconds, Instant, Now as NowInner, TimeZone};

use super::{
    create_temporal_date, create_temporal_datetime, create_temporal_instant, create_temporal_time,
    create_temporal_zoneddatetime, to_temporal_timezone_identifier,
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
    /// 2.2.1 `Temporal.Now.timeZoneId ( )`
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal.now.timezone
    fn time_zone_id(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return ! SystemTimeZone().
        Ok(JsString::from(system_time_zone()?).into())
    }

    /// 2.2.2 `Temporal.Now.instant()`
    fn instant(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let epoch_nanos = system_nanoseconds(context)?;
        create_temporal_instant(Instant::from(epoch_nanos), None, context).map(Into::into)
    }

    /// 2.2.3 `Temporal.Now.plainDateTimeISO ( [ temporalTimeZoneLike ] )`
    fn plain_datetime(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let (epoch_nanos, timezone) = resolve_system_values(args.get_or_undefined(0), context)?;
        let datetime = NowInner::plain_datetime_iso_with_provider(
            epoch_nanos,
            timezone,
            context.tz_provider(),
        )?;
        create_temporal_datetime(datetime, None, context).map(Into::into)
    }

    /// 2.2.4 `Temporal.Now.zonedDateTimeISO ( [ temporalTimeZoneLike ] )`
    fn zoneddatetime(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let (epoch_nanos, timezone) = resolve_system_values(args.get_or_undefined(0), context)?;
        let zdt = NowInner::zoneddatetime_iso_with_system_values(epoch_nanos, timezone)?;
        create_temporal_zoneddatetime(zdt, None, context).map(Into::into)
    }

    /// 2.2.5 `Temporal.Now.plainDateISO ( [ temporalTimeZoneLike ] )`
    fn plain_date(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let (epoch_nanos, timezone) = resolve_system_values(args.get_or_undefined(0), context)?;
        let pd =
            NowInner::plain_date_iso_with_provider(epoch_nanos, timezone, context.tz_provider())?;
        create_temporal_date(pd, None, context).map(Into::into)
    }

    /// 2.2.6 `Temporal.Now.plainTimeISO ( [ temporalTimeZoneLike ] )`
    fn plain_time(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let (epoch_nanos, timezone) = resolve_system_values(args.get_or_undefined(0), context)?;
        let pt =
            NowInner::plain_time_iso_with_provider(epoch_nanos, timezone, context.tz_provider())?;
        create_temporal_time(pt, None, context).map(Into::into)
    }
}

// `resolve_system_values` takes a `JsValue` representing a potential user provided time zone
// and returns the system time and time zone, resolving to the user time zone if provided
fn resolve_system_values(
    timezone: &JsValue,
    context: &mut Context,
) -> JsResult<(EpochNanoseconds, TimeZone)> {
    let user_timezone = timezone
        .map(|v| to_temporal_timezone_identifier(v, context))
        .transpose()?;
    let timezone =
        user_timezone.unwrap_or(TimeZone::try_from_identifier_str(&system_time_zone()?)?);
    let epoch_nanos = EpochNanoseconds::try_from(context.clock().now().nanos_since_epoch())?;
    Ok((epoch_nanos, timezone))
}

fn system_nanoseconds(context: &mut Context) -> JsResult<EpochNanoseconds> {
    Ok(EpochNanoseconds::try_from(
        context.clock().now().nanos_since_epoch(),
    )?)
}

// TODO: Move system time zone fetching to context similiar to `Clock` and `TimeZoneProvider`
fn system_time_zone() -> JsResult<String> {
    iana_time_zone::get_timezone()
        .map_err(|e| JsNativeError::range().with_message(e.to_string()).into())
}
