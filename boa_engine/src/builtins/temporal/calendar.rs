#![allow(dead_code, unused_variables)]
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::Attribute,
    realm::Realm,
    Context, JsBigInt, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

/// The `Temporal.Calendar` object.
#[derive(Debug, Clone)]
pub struct Calendar {
    identifier: JsString,
}

impl BuiltInObject for Calendar {
    const NAME: &'static str = "Temporal.Calendar";
}

impl IntrinsicObject for Calendar {
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

impl BuiltInConstructor for Calendar {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::calendar;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }
}

// -- `Calendar` Abstract Operations --

/// 12.2.1 `CreateTemporalCalendar ( identifier [ , newTarget ] )`
pub(crate) fn create_temporal_calendar(
    identifier: &JsString,
    new_target: Option<JsValue>,
) -> JsResult<JsValue> {
    // 1. Assert: IsBuiltinCalendar(identifier) is true.
    // 2. If newTarget is not provided, set newTarget to %Temporal.Calendar%.
    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Calendar.prototype%", « [[InitializedTemporalCalendar]], [[Identifier]] »).
    // 4. Set object.[[Identifier]] to the ASCII-lowercase of identifier.
    // 5. Return object.
    todo!()
}

/// 12.2.4 `CalendarDateAdd ( calendar, date, duration [ , options [ , dateAdd ] ] )`
pub(crate) fn calendar_date_add(
    calendar: &JsObject,
    date: &JsObject,
    duration: &JsObject,
    options: &JsValue,
    date_add: Option<&JsValue>,
) -> JsResult<JsObject> {
    todo!()
}

/// 12.2.5 `CalendarDateUntil ( calendar, one, two, options [ , dateUntil ] )`
pub(crate) fn calendar_date_until(
    calendar: &JsObject,
    one: &JsObject,
    two: &JsObject,
    options: &JsValue,
    date_until: Option<&JsValue>,
) -> JsResult<super::duration::DurationRecord> {
    todo!()
}

/// 12.2.31 `ISODaysInMonth ( year, month )`
pub(crate) fn iso_days_in_month(year: i32, month: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            28 + super::date_equations::mathematical_in_leap_year(
                super::date_equations::epoch_time_for_year(f64::from(year)),
            )
        }
        _ => unreachable!("an invalid month value is an implementation error."),
    }
}
