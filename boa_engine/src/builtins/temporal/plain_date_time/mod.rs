//! Boa's implementation of the ECMAScript `Temporal.PlainDateTime` builtin object.
#![allow(dead_code, unused_variables)]

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::Attribute,
    realm::Realm,
    string::common::StaticJsStrings,
    Context, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};
use boa_profiler::Profiler;

use boa_temporal::datetime::DateTime as InnerDateTime;

/// The `Temporal.PlainDateTime` object.
#[derive(Debug, Clone)]
pub struct PlainDateTime {
    pub(crate) inner: InnerDateTime,
}

impl PlainDateTime {
    fn new(inner: InnerDateTime) -> Self {
        Self { inner }
    }

    pub(crate) fn inner(&self) -> &InnerDateTime {
        &self.inner
    }
}

impl BuiltInObject for PlainDateTime {
    const NAME: JsString = StaticJsStrings::PLAIN_DATETIME;
}

impl IntrinsicObject for PlainDateTime {
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

impl BuiltInConstructor for PlainDateTime {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::plain_date_time;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::range()
            .with_message("Not yet implemented.")
            .into())
    }
}

// ==== `PlainDateTime` Accessor Properties ====

impl PlainDateTime {
    fn calendar_id(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn year(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn month(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn month_code(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn day(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn hour(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn minute(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn second(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn millisecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn microsecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn nanosecond(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn era(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }

    fn era_year(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Err(JsNativeError::error()
            .with_message("calendars not yet implemented.")
            .into())
    }
}

// ==== `PlainDateTime` Abstract Operations` ====

// See `IsoDateTimeRecord`
