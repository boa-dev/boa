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


/// 12.2.4 CalendarDateAdd ( calendar, date, duration [ , options [ , dateAdd ] ] )
pub(crate) fn calendar_date_add(calendar: &JsObject, date: &JsObject, duration: JsObject, options: JsValue, date_add: Option<&JsValue>) -> JsResult<JsValue> {
    todo!()
}
