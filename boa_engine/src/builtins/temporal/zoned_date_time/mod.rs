#![allow(dead_code, unused_variables)]
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::Attribute,
    realm::Realm,
    string::common::StaticJsStrings,
    Context, JsBigInt, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

/// The `Temporal.ZonedDateTime` object.
#[derive(Debug, Clone)]
pub struct ZonedDateTime {
    nanoseconds: JsBigInt,
    time_zone: JsObject,
    calendar: JsObject,
}

impl BuiltInObject for ZonedDateTime {
    const NAME: JsString = StaticJsStrings::ZONED_DT;
}

impl IntrinsicObject for ZonedDateTime {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for ZonedDateTime {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::zoned_date_time;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // TODO: Implement ZonedDateTime.
        Err(JsNativeError::error()
            .with_message("%ZonedDateTime% not yet implemented.")
            .into())
    }
}

// -- ZonedDateTime Abstract Operations --

///6.5.5 `AddZonedDateTime ( epochNanoseconds, timeZone, calendar, years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds [ , options ] )`
pub(crate) fn add_zoned_date_time(
    epoch_nanos: &JsBigInt,
    time_zone: &JsObject,
    calendar: &JsObject,
    duration: super::duration::DurationRecord,
    options: Option<&JsObject>,
) -> JsResult<JsBigInt> {
    // 1. If options is not present, set options to undefined.
    // 2. Assert: Type(options) is Object or Undefined.
    // 3. If years = 0, months = 0, weeks = 0, and days = 0, then
    // a. Return ? AddInstant(epochNanoseconds, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
    // 4. Let instant be ! CreateTemporalInstant(epochNanoseconds).
    // 5. Let temporalDateTime be ? GetPlainDateTimeFor(timeZone, instant, calendar).
    // 6. Let datePart be ! CreateTemporalDate(temporalDateTime.[[ISOYear]], temporalDateTime.[[ISOMonth]], temporalDateTime.[[ISODay]], calendar).
    // 7. Let dateDuration be ! CreateTemporalDuration(years, months, weeks, days, 0, 0, 0, 0, 0, 0).
    // 8. Let addedDate be ? CalendarDateAdd(calendar, datePart, dateDuration, options).
    // 9. Let intermediateDateTime be ? CreateTemporalDateTime(addedDate.[[ISOYear]], addedDate.[[ISOMonth]], addedDate.[[ISODay]], temporalDateTime.[[ISOHour]], temporalDateTime.[[ISOMinute]], temporalDateTime.[[ISOSecond]], temporalDateTime.[[ISOMillisecond]], temporalDateTime.[[ISOMicrosecond]], temporalDateTime.[[ISONanosecond]], calendar).
    // 10. Let intermediateInstant be ? GetInstantFor(timeZone, intermediateDateTime, "compatible").
    // 11. Return ? AddInstant(intermediateInstant.[[Nanoseconds]], hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
    Err(JsNativeError::error()
        .with_message("%ZonedDateTime% not yet implemented.")
        .into())
}

/// 6.5.7 `NanosecondsToDays ( nanoseconds, relativeTo )`
pub(crate) fn nanoseconds_to_days(
    nanoseconds: f64,
    relative_to: &JsValue,
) -> JsResult<(i32, i32, i32)> {
    // 1. Let dayLengthNs be nsPerDay.
    // 2. If nanoseconds = 0, then
    // a. Return the Record { [[Days]]: 0, [[Nanoseconds]]: 0, [[DayLength]]: dayLengthNs }.
    // 3. If nanoseconds < 0, let sign be -1; else, let sign be 1.
    // 4. If Type(relativeTo) is not Object or relativeTo does not have an [[InitializedTemporalZonedDateTime]] internal slot, then
    // a. Return the Record { [[Days]]: truncate(nanoseconds / dayLengthNs), [[Nanoseconds]]: (abs(nanoseconds) modulo dayLengthNs) × sign, [[DayLength]]: dayLengthNs }.
    // 5. Let startNs be ℝ(relativeTo.[[Nanoseconds]]).
    // 6. Let startInstant be ! CreateTemporalInstant(ℤ(startNs)).
    // 7. Let startDateTime be ? GetPlainDateTimeFor(relativeTo.[[TimeZone]], startInstant, relativeTo.[[Calendar]]).
    // 8. Let endNs be startNs + nanoseconds.
    // 9. If ! IsValidEpochNanoseconds(ℤ(endNs)) is false, throw a RangeError exception.
    // 10. Let endInstant be ! CreateTemporalInstant(ℤ(endNs)).
    // 11. Let endDateTime be ? GetPlainDateTimeFor(relativeTo.[[TimeZone]], endInstant, relativeTo.[[Calendar]]).
    // 12. Let dateDifference be ? DifferenceISODateTime(startDateTime.[[ISOYear]], startDateTime.[[ISOMonth]], startDateTime.[[ISODay]], startDateTime.[[ISOHour]], startDateTime.[[ISOMinute]], startDateTime.[[ISOSecond]], startDateTime.[[ISOMillisecond]], startDateTime.[[ISOMicrosecond]], startDateTime.[[ISONanosecond]], endDateTime.[[ISOYear]], endDateTime.[[ISOMonth]], endDateTime.[[ISODay]], endDateTime.[[ISOHour]], endDateTime.[[ISOMinute]], endDateTime.[[ISOSecond]], endDateTime.[[ISOMillisecond]], endDateTime.[[ISOMicrosecond]], endDateTime.[[ISONanosecond]], relativeTo.[[Calendar]], "day", OrdinaryObjectCreate(null)).
    // 13. Let days be dateDifference.[[Days]].
    // 14. Let intermediateNs be ℝ(? AddZonedDateTime(ℤ(startNs), relativeTo.[[TimeZone]], relativeTo.[[Calendar]], 0, 0, 0, days, 0, 0, 0, 0, 0, 0)).
    // 15. If sign is 1, then
    // a. Repeat, while days > 0 and intermediateNs > endNs,
    // i. Set days to days - 1.
    // ii. Set intermediateNs to ℝ(? AddZonedDateTime(ℤ(startNs), relativeTo.[[TimeZone]], relativeTo.[[Calendar]], 0, 0, 0, days, 0, 0, 0, 0, 0, 0)).
    // 16. Set nanoseconds to endNs - intermediateNs.
    // 17. Let done be false.
    // 18. Repeat, while done is false,
    // a. Let oneDayFartherNs be ℝ(? AddZonedDateTime(ℤ(intermediateNs), relativeTo.[[TimeZone]], relativeTo.[[Calendar]], 0, 0, 0, sign, 0, 0, 0, 0, 0, 0)).
    // b. Set dayLengthNs to oneDayFartherNs - intermediateNs.
    // c. If (nanoseconds - dayLengthNs) × sign ≥ 0, then
    // i. Set nanoseconds to nanoseconds - dayLengthNs.
    // ii. Set intermediateNs to oneDayFartherNs.
    // iii. Set days to days + sign.
    // d. Else,
    // i. Set done to true.
    // 19. If days < 0 and sign = 1, throw a RangeError exception.
    // 20. If days > 0 and sign = -1, throw a RangeError exception.
    // 21. If nanoseconds < 0, then
    // a. Assert: sign is -1.
    // 22. If nanoseconds > 0 and sign = -1, throw a RangeError exception.
    // 23. Assert: The inequality abs(nanoseconds) < abs(dayLengthNs) holds.
    // 24. Return the Record { [[Days]]: days, [[Nanoseconds]]: nanoseconds, [[DayLength]]: abs(dayLengthNs) }.
    Err(JsNativeError::error()
        .with_message("%ZonedDateTime% not yet implemented.")
        .into())
}
