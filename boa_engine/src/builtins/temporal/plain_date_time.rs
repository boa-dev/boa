#![allow(dead_code, unused_variables)]
use crate::{
    builtins::{date::utils, BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::Attribute,
    realm::Realm,
    Context, JsBigInt, JsObject, JsResult, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

/// The `Temporal.PlainDateTime` object.
#[derive(Debug, Clone)]
pub struct PlainDateTime {
    iso_year: i32,
    iso_month: i32,
    iso_day: i32,
    iso_hour: i32,        // integer between 0-23
    iso_minute: i32,      // integer between 0-59
    iso_second: i32,      // integer between 0-59
    iso_millisecond: i32, // integer between 0-999
    iso_microsecond: i32, // integer between 0-999
    iso_nanosecond: i32,  // integer between 0-999
    calendar: JsObject,
}

impl BuiltInObject for PlainDateTime {
    const NAME: &'static str = "Temporal.PlainDateTime";
}

impl IntrinsicObject for PlainDateTime {
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

impl BuiltInConstructor for PlainDateTime {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_date_time;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }
}

fn get_utc_epoch_nanos(
    y: i64,
    mo: i64,
    d: i64,
    h: i64,
    m: i64,
    sec: i64,
    ms: i64,
    mis: i64,
    ns: i64,
) -> JsBigInt {
    // NOTE(nekevss): specification calls to assert that the number is integral,
    // the unwraps primarily function as that assertion, although admittedly clunkily.
    let day = utils::make_day(y, mo, d).expect("must be valid number.");
    let time = utils::make_time(h, m, sec, ms).expect("must be valid number.");

    let ms = utils::make_date(day, time).expect("must be valid number.");

    JsBigInt::from((ms * 1_000_000) + (mis * 1_000) + ns)
}

// -- `PlainDateTime` Abstract Operations --

/// 5.5.1 `ISODateTimeWithinLimits ( year, month, day, hour, minute, second, millisecond, microsecond, nanosecond )`
pub(crate) fn iso_datetime_within_limits(
    y: i32,
    mo: i32,
    d: i32,
    h: i32,
    m: i32,
    sec: i32,
    ms: i32,
    mis: i32,
    ns: i32,
) -> bool {
    assert!(super::plain_date::IsoDateRecord::new(y, mo, d).is_valid_iso_date());

    let ns = get_utc_epoch_nanos(
        i64::from(y),
        i64::from(mo),
        i64::from(d),
        i64::from(h),
        i64::from(m),
        i64::from(sec),
        i64::from(ms),
        i64::from(mis),
        i64::from(ns),
    )
    .to_f64();

    let iso_min = super::NS_MIN_INSTANT as f64 - super::NS_PER_DAY as f64;
    let iso_max = super::NS_MAX_INSTANT as f64 + super::NS_PER_DAY as f64;

    (iso_min..=iso_max).contains(&ns)
}
