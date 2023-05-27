//! Boa's implementation of ECMAScript's global `AsyncFunction` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-async-function-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AsyncFunction

use crate::{
    builtins::{function::BuiltInFunctionObject, BuiltInObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    realm::Realm,
    Context, JsResult, JsValue,
};
use boa_profiler::Profiler;

use super::{BuiltInBuilder, BuiltInConstructor, IntrinsicObject};

/// The internal representation of an `AsyncFunction` object.
#[derive(Debug, Clone, Copy)]
pub struct AsyncFunction;

impl IntrinsicObject for AsyncFunction {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::from_standard_constructor_static_shape::<Self>(
            realm,
            &boa_builtins::ASYNC_FUNCTION_CONSTRUCTOR_STATIC_SHAPE,
            &boa_builtins::ASYNC_FUNCTION_PROTOTYPE_STATIC_SHAPE,
        )
        .prototype(realm.intrinsics().constructors().function().constructor())
        .inherits(Some(
            realm.intrinsics().constructors().function().prototype(),
        ))
        .property(Self::NAME)
        .build();
    }

    fn get(intrinsics: &Intrinsics) -> crate::object::JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for AsyncFunction {
    const NAME: &'static str = "AsyncFunction";
}

impl BuiltInConstructor for AsyncFunction {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::async_function;

    /// `AsyncFunction ( p1, p2, … , pn, body )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-async-function-constructor-arguments
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let active_function = context.vm.active_function.clone().unwrap_or_else(|| {
            context
                .intrinsics()
                .constructors()
                .async_function()
                .constructor()
        });
        BuiltInFunctionObject::create_dynamic_function(
            active_function,
            new_target,
            args,
            true,
            false,
            context,
        )
        .map(Into::into)
    }
}
