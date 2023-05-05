//! Boa's implementation of ECMAScript's `Temporal.Instant` object.

use crate::{
    builtins::{IntrinsicObject, BuiltInObject, BuiltInBuilder, BuiltInConstructor},
    context::intrinsics::{Intrinsics, StandardConstructors, StandardConstructor},
    realm::Realm,
    property::Attribute,
    Context, JsValue, JsResult, JsBigInt, JsObject, JsSymbol, JsNativeError, JsArgs,
};
use boa_profiler::Profiler;

#[derive(Debug, Clone)]
pub struct Instant {
    pub(crate) initialized_temporal_instant: Option<u16>,
    pub(crate) nanoseconds: JsBigInt,
}

impl BuiltInObject for Instant {
    const NAME: &'static str = "Temporal.Instant";
}

impl IntrinsicObject for Instant {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
        .static_property(JsSymbol::to_string_tag(), Self::NAME, Attribute::CONFIGURABLE)
        .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for Instant {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor = StandardConstructors::instant;

    fn constructor(
            new_target: &JsValue,
            args: &[JsValue],
            context: &mut Context<'_>,
        ) -> JsResult<JsValue> {
            // 1. If NewTarget is undefined, then
            if new_target.is_undefined() {
                // a. Throw a TypeError exception.
                return Err(JsNativeError::typ().with_message("Temporal.Instant new target cannot be undefined.").into());
            };
            let epoch_nanos = args.get_or_undefined(0).to_bigint(context)?;
            // 2. Let epochNanoseconds be ? ToBigInt(epochNanoseconds).
            // 3. If ! IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
            if !is_valid_epoch_nanos(&epoch_nanos) {

            };
            // 4. Return ? CreateTemporalInstant(epochNanoseconds, NewTarget).
            create_temporal_instant(epoch_nanos, new_target, context)
    }
}

// -- Instant Abstract Operations --

fn is_valid_epoch_nanos(_epoch_nanos: &JsBigInt) -> bool {
    todo!()
}

fn create_temporal_instant(_epoch_nanos: JsBigInt, _new_target: &JsValue, _context: &mut Context<'_>) -> JsResult<JsValue> {
    todo!()
}
