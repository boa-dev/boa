//! Boa's implementation of ECMAScript's global `GeneratorFunction` object.
//!
//! The `GeneratorFunction` constructor creates a new generator function object.
//! In ECMAScript, every generator function is actually a `GeneratorFunction` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-generatorfunction-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/GeneratorFunction

use crate::{
    builtins::{function::BuiltInFunctionObject, BuiltInObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    realm::Realm,
    value::JsValue,
    Context, JsResult,
};
use boa_profiler::Profiler;

use super::{BuiltInBuilder, BuiltInConstructor, IntrinsicObject};

/// The internal representation of a `Generator` object.
#[derive(Debug, Clone, Copy)]
pub struct GeneratorFunction;

impl IntrinsicObject for GeneratorFunction {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::from_standard_constructor_static_shape::<Self>(
            realm,
            &boa_builtins::GENERATOR_FUNCTION_CONSTRUCTOR_STATIC_SHAPE,
            &boa_builtins::GENERATOR_FUNCTION_PROTOTYPE_STATIC_SHAPE,
        )
        .inherits(Some(
            realm.intrinsics().constructors().function().prototype(),
        ))
        .property(realm.intrinsics().objects().generator())
        .property(Self::NAME)
        .build();
    }

    fn get(intrinsics: &Intrinsics) -> crate::object::JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for GeneratorFunction {
    const NAME: &'static str = "GeneratorFunction";
}

impl BuiltInConstructor for GeneratorFunction {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::generator_function;

    /// `GeneratorFunction ( p1, p2, … , pn, body )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-generatorfunction
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
            false,
            true,
            context,
        )
        .map(Into::into)
    }
}
