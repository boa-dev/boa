#![allow(dead_code, unused_variables)]
use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::Attribute,
    realm::Realm,
    Context, JsObject, JsResult, JsSymbol, JsValue, JsNativeError,
};
use boa_profiler::Profiler;

/// The `Temporal.PlainTime` object.
#[derive(Debug, Clone, Copy)]
pub struct PlainTime {
    iso_hour: i32,        // integer between 0-23
    iso_minute: i32,      // integer between 0-59
    iso_second: i32,      // integer between 0-59
    iso_millisecond: i32, // integer between 0-999
    iso_microsecond: i32, // integer between 0-999
    iso_nanosecond: i32,  // integer between 0-999
}

impl BuiltInObject for PlainTime {
    const NAME: &'static str = "Temporal.PlainTime";
}

impl IntrinsicObject for PlainTime {
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

impl BuiltInConstructor for PlainTime {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_time;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::range().with_message("Not yet implemented.").into())
    }
}
