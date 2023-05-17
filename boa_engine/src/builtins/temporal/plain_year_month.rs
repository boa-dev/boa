#![allow(dead_code, unused_variables)]

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::Attribute,
    realm::Realm,
    Context, JsObject, JsResult, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

/// The `Temporal.PlainYearMonth` object.
#[derive(Debug, Clone)]
pub struct PlainYearMonth {
    iso_year: i32,
    iso_month: i32,
    iso_day: i32,
    calendar: JsObject,
}

impl BuiltInObject for PlainYearMonth {
    const NAME: &'static str = "Temporal.PlainYearMonth";
}

impl IntrinsicObject for PlainYearMonth {
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

impl BuiltInConstructor for PlainYearMonth {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_year_month;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        todo!()
    }
}
