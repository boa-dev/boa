#![allow(dead_code, unused_variables)]
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::Attribute,
    realm::Realm,
    Context, JsObject, JsResult, JsSymbol, JsValue,
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
