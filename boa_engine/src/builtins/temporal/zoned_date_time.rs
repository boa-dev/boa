#![allow(dead_code, unused_variables)]
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::Attribute,
    realm::Realm,
    Context, JsBigInt, JsObject, JsResult, JsSymbol, JsValue,
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
    const NAME: &'static str = "Temporal.ZonedDateTime";
}

impl IntrinsicObject for ZonedDateTime {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

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
        todo!()
    }
}

// -- ZonedDateTime Abstract Operations --

/// 6.5.7 NanosecondsToDays ( nanoseconds, relativeTo )
pub(crate) fn nanoseconds_to_days(
    nanoseconds: i32,
    relativeTo: JsValue,
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
    todo!()
}
