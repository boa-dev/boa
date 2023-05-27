//! Boa's implementation of ECMAScript's `AsyncGeneratorFunction` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorfunction-objects

use crate::{
    builtins::{function::BuiltInFunctionObject, BuiltInObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::JsObject,
    realm::Realm,
    value::JsValue,
    Context, JsResult,
};
use boa_profiler::Profiler;

use super::{BuiltInBuilder, BuiltInConstructor, IntrinsicObject};

/// The internal representation of an `AsyncGeneratorFunction` object.
#[derive(Debug, Clone, Copy)]
pub struct AsyncGeneratorFunction;

impl IntrinsicObject for AsyncGeneratorFunction {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::from_standard_constructor_static_shape::<Self>(
            realm,
            &boa_builtins::ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_STATIC_SHAPE,
            &boa_builtins::ASYNC_GENERATOR_FUNCTION_PROTOTYPE_STATIC_SHAPE,
        )
        .inherits(Some(
            realm.intrinsics().constructors().function().prototype(),
        ))
        .property(realm.intrinsics().objects().async_generator())
        .property(Self::NAME)
        .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for AsyncGeneratorFunction {
    const NAME: &'static str = "AsyncGeneratorFunction";
}

impl BuiltInConstructor for AsyncGeneratorFunction {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::async_generator_function;

    /// `AsyncGeneratorFunction ( p1, p2, … , pn, body )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorfunction
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let active_function = context.vm.active_function.clone().unwrap_or_else(|| {
            context
                .intrinsics()
                .constructors()
                .generator_function()
                .constructor()
        });
        BuiltInFunctionObject::create_dynamic_function(
            active_function,
            new_target,
            args,
            true,
            true,
            context,
        )
        .map(Into::into)
    }
}
