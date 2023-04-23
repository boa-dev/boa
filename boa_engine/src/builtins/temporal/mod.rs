//! The ECMAScript `Temporal` stage 3 built-in implementation.
//!
//! More information:
//!
//! [spec]: https://tc39.es/proposal-temporal/
#![allow(unreachable_code, unused_imports)] // Unimplemented

mod duration;
mod instant;
mod plain_date;
mod plain_date_time;
mod plain_month_day;
mod plain_time;
mod plain_year_month;
mod time_zone;

pub(crate) use self::{
    duration::*, instant::*, plain_date::*, plain_date_time::*, plain_month_day::*, plain_time::*,
    plain_year_month::*, time_zone::*,
};
use super::{BuiltInBuilder, BuiltInObject, IntrinsicObject};
use crate::{
    context::intrinsics::{Intrinsics, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, ObjectData, ObjectInitializer},
    property::Attribute,
    realm::Realm,
    value::IntegerOrInfinity,
    Context, JsObject, JsResult, JsSymbol, JsValue, NativeFunction,
};
use boa_ast::temporal::{OffsetSign, UtcOffset};
use boa_profiler::Profiler;

/// The [`Temporal`][spec] builtin object.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Temporal;

impl BuiltInObject for Temporal {
    const NAME: &'static str = "Temporal";
}

impl IntrinsicObject for Temporal {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .static_property(
                "Now",
                TemporalNow::init(realm),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().temporal()
    }
}

/// JavaScript `Temporal.Now` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TemporalNow;

impl TemporalNow {
    const NAME: &'static str = "Temporal.Now";

    /// Initializes the `Temporal.Now` object.
    fn init(realm: &Realm) -> JsValue {
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
            .function(NativeFunction::from_fn_ptr(Self::time_zone), "timeZone", 0)
            // .function(Self::instant, "instant", 0)
            // .function(Self::plain_date_time, "plainDateTime", 2)
            // .function(Self::plain_date_time_iso, "plainDateTimeISO", 1)
            // .function(Self::zoned_date_time, "zonedDateTime", 2)
            // .function(Self::zoned_date_time_iso, "zonedDateTimeISO", 1)
            // .function(Self::plain_date, "plainDate", 2)
            // .function(Self::plain_date_iso, "plainDateISO", 1)
            // .function(Self::plain_time_iso, "plainTimeISO", 1)
            .build()
            .into()
    }

    /// `Temporal.Now.timeZone ( )`
    ///
    /// More information:
    ///  - [ECMAScript specififcation][spec]
    ///
    /// [spec]: https://tc39.es/proposal-temporal/#sec-temporal.now.timezone
    #[allow(clippy::unnecessary_wraps)]
    fn time_zone(_: &JsValue, _args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Return ! SystemTimeZone().
        Ok(system_time_zone(context).expect("retrieving the system timezone must not fail"))
    }
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
    let identifier = default_time_zone();
    // 2. Return ! CreateTemporalTimeZone(identifier).
    create_temporal_time_zone(identifier, None, context)
}

/// Abstract operation `DefaultTimeZone ( )`
///
/// The abstract operation `DefaultTimeZone` takes no arguments. It returns a String value
/// representing the host environment's current time zone, which is either a valid (11.1.1) and
/// canonicalized (11.1.2) time zone name, or an offset conforming to the syntax of a
/// `TimeZoneNumericUTCOffset`.
///
/// An ECMAScript implementation that includes the ECMA-402 Internationalization API must implement
/// the `DefaultTimeZone` abstract operation as specified in the ECMA-402 specification.
///
/// More information:
///  - [ECMAScript specififcation][spec]
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-defaulttimezone
#[allow(unused)]
fn default_time_zone() -> String {
    // The minimum implementation of DefaultTimeZone for ECMAScript implementations that do not
    // include the ECMA-402 API, supporting only the "UTC" time zone, performs the following steps
    // when called:

    // 1. Return "UTC".
    "UTC".to_owned()

    // TODO: full, system-aware implementation
}

/// Abstract operation `CreateTemporalTimeZone ( identifier [ , newTarget ] )`
///
/// More information:
///  - [ECMAScript specififcation][spec]
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-createtemporaltimezone
#[allow(clippy::needless_pass_by_value, unused)]
fn create_temporal_time_zone(
    identifier: String,
    new_target: Option<JsValue>,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. If newTarget is not present, set newTarget to %Temporal.TimeZone%.
    let new_target = new_target.unwrap_or_else(|| todo!("%Temporal.TimeZone%"));

    // 2. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.TimeZone.prototype%", « [[InitializedTemporalTimeZone]], [[Identifier]], [[OffsetNanoseconds]] »).
    let prototype =
        get_prototype_from_constructor(&new_target, StandardConstructors::time_zone, context)?;
    let object = JsObject::from_proto_and_data(prototype, ObjectData::time_zone(TimeZone {}));

    // 3. Let offsetNanosecondsResult be Completion(ParseTimeZoneOffsetString(identifier)).
    let offset_nanoseconds_result = parse_timezone_offset_string(&identifier, context);

    // 4. If offsetNanosecondsResult is an abrupt completion, then
    if offset_nanoseconds_result.is_err() {
        // a. Assert: ! CanonicalizeTimeZoneName(identifier) is identifier.
        assert_eq!(canonicalize_time_zone_name(&identifier), identifier);

        // b. Set object.[[Identifier]] to identifier.
        todo!();

        // c. Set object.[[OffsetNanoseconds]] to undefined.
        todo!()
    } else {
        // 5. Else,
        // a. Set object.[[Identifier]] to ! FormatTimeZoneOffsetString(offsetNanosecondsResult.[[Value]]).
        todo!();

        // b. Set object.[[OffsetNanoseconds]] to offsetNanosecondsResult.[[Value]].
        todo!()
    }

    // 6. Return object.
    Ok(object.into())
}

/// Abstract operation `ParseTimeZoneOffsetString ( offsetString )`
///
/// The abstract operation `ParseTimeZoneOffsetString` takes argument `offsetString` (a String). It
/// parses the argument as a numeric UTC offset string and returns a signed integer representing
/// that offset as a number of nanoseconds.
///
/// More information:
///  - [ECMAScript specififcation][spec]
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-parsetimezoneoffsetstring
#[allow(clippy::unnecessary_wraps, unused)]
fn parse_timezone_offset_string(_offset_string: &str, context: &mut Context<'_>) -> JsResult<i64> {
    // // 1. Let parseResult be ParseText(StringToCodePoints(offsetString), TimeZoneNumericUTCOffset).
    // let parse_result = TimeZoneNumericUTCOffset.parse(
    //     &mut Cursor::new(offset_string.as_bytes()),
    //     context.interner_mut(),
    // );

    // 2. If parseResult is a List of errors, throw a RangeError exception.
    // let parse_result = parse_result.map_err(|e| {
    //     return Err(JsNativeError::range().with_message(format!("invalid timezone offset string: {e}")).into());
    // })?;

    // 3. Let each of sign, hours, minutes, seconds, and fSeconds be the source text matched by the
    // respective TimeZoneUTCOffsetSign, TimeZoneUTCOffsetHour, TimeZoneUTCOffsetMinute,
    // TimeZoneUTCOffsetSecond, and TimeZoneUTCOffsetFraction Parse Node contained within
    // parseResult, or an empty sequence of code points if not present.
    let UtcOffset {
        sign,
        hour: hours,
        minute: minutes,
        second: seconds,
        fraction: f_seconds,
    } = UtcOffset {
        sign: OffsetSign::Negative,
        hour: String::new(),
        minute: String::new(),
        second: String::new(),
        fraction: String::new(),
    }; //parse_result;

    // 4. Assert: sign is not empty.
    // It cannot be empty, because it's checked at type level.

    // 5. If sign contains the code point U+002D (HYPHEN-MINUS) or U+2212 (MINUS SIGN), then
    let _factor = if matches!(sign, OffsetSign::Negative) {
        // a. Let factor be -1.
        -1
    } else {
        // 6. Else,
        // a. Let factor be 1.
        1
    };

    // 7. Assert: hours is not empty.
    // It cannot be empty, because it's checked at type level.

    // 8. Let hoursMV be ! ToIntegerOrInfinity(CodePointsToString(hours)).
    let _hours_mv = JsValue::String(hours.into())
        .to_integer_or_infinity(context)
        .expect("cannot fail");

    // 9. Let minutesMV be ! ToIntegerOrInfinity(CodePointsToString(minutes)).
    let _minutes_mv = JsValue::String(minutes.into())
        .to_integer_or_infinity(context)
        .expect("cannot fail");

    // 10. Let secondsMV be ! ToIntegerOrInfinity(CodePointsToString(seconds)).
    let _seconds_mv = JsValue::String(seconds.into())
        .to_integer_or_infinity(context)
        .expect("cannot fail");

    // 11. If fSeconds is not empty, then
    #[allow(clippy::if_not_else)]
    let _nanoseconds_mv = if !f_seconds.is_empty() {
        // a. Let fSecondsDigits be the substring of CodePointsToString(fSeconds) from 1.
        let f_seconds_digits = &f_seconds[1..];

        // b. Let fSecondsDigitsExtended be the string-concatenation of fSecondsDigits and "000000000".
        let f_seconds_digits_extended = format!("{f_seconds_digits}000000000");

        // c. Let nanosecondsDigits be the substring of fSecondsDigitsExtended from 0 to 9.
        let nanoseconds_digits = &f_seconds_digits_extended[0..=9];

        // d. Let nanosecondsMV be ! ToIntegerOrInfinity(nanosecondsDigits).
        JsValue::String(nanoseconds_digits.into())
            .to_integer_or_infinity(context)
            .expect("cannot fail")
    } else {
        // 12. Else,
        // a. Let nanosecondsMV be 0.
        IntegerOrInfinity::Integer(0)
    };

    // 13. Return factor × (((hoursMV × 60 + minutesMV) × 60 + secondsMV) × 10^9 + nanosecondsMV).
    // Ok(factor
    //     * (((hours_mv * IntegerOrInfinity::Integer(60) + minutes_mv) * 60 + seconds_mv)
    //         * 10.pow(9)
    //         + nanoseconds_mv))
    Ok(0)
}

/// Abstract operation `CanonicalizeTimeZoneName ( timeZone )`
///
/// The abstract operation `CanonicalizeTimeZoneName` takes argument `timeZone` (a String that is a
/// valid time zone name as verified by `IsAvailableTimeZoneName`). It returns the canonical and
/// case-regularized form of `timeZone`.
fn canonicalize_time_zone_name(time_zone: &str) -> String {
    // The minimum implementation of CanonicalizeTimeZoneName for ECMAScript implementations that
    // do not include local political rules for any time zones performs the following steps when
    // called:
    // 1. Assert: timeZone is an ASCII-case-insensitive match for "UTC".
    assert!(time_zone.to_ascii_uppercase() == "UTC");
    // 2. Return "UTC".
    "UTC".to_owned()
}
