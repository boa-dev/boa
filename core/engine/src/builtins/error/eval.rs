//! This module implements the global `EvalError` object.
//!
//! Indicates an error regarding the global `eval()` function.
//! This exception is not thrown by JavaScript anymore, however
//! the `EvalError` object remains for compatibility.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-evalerror
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/EvalError

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    Context, JsArgs, JsResult, JsString, JsValue,
};
use boa_profiler::Profiler;

use super::{Error, ErrorObject};

/// JavaScript `EvalError` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EvalError;

impl IntrinsicObject for EvalError {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .prototype(realm.intrinsics().constructors().error().constructor())
            .inherits(Some(realm.intrinsics().constructors().error().prototype()))
            .property(js_string!("name"), Self::NAME, attribute)
            .property(js_string!("message"), js_string!(), attribute)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for EvalError {
    const NAME: JsString = StaticJsStrings::EVAL_ERROR;
}

impl BuiltInConstructor for EvalError {
    const LENGTH: usize = 1;
    const P: usize = 2;
    const SP: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::eval_error;

    /// Create a new error object.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, let newTarget be the active function object; else let newTarget be NewTarget.
        let new_target = &if new_target.is_undefined() {
            context
                .active_function_object()
                .unwrap_or_else(|| {
                    context
                        .intrinsics()
                        .constructors()
                        .eval_error()
                        .constructor()
                })
                .into()
        } else {
            new_target.clone()
        };
        // 2. Let O be ? OrdinaryCreateFromConstructor(newTarget, "%NativeError.prototype%", « [[ErrorData]] »).
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::eval_error, context)?;
        let o = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ErrorObject::Eval,
        );

        // 3. If message is not undefined, then
        let message = args.get_or_undefined(0);
        if !message.is_undefined() {
            // a. Let msg be ? ToString(message).
            let msg = message.to_string(context)?;

            // b. Perform CreateNonEnumerableDataPropertyOrThrow(O, "message", msg).
            o.create_non_enumerable_data_property_or_throw(js_string!("message"), msg, context);
        }

        // 4. Perform ? InstallErrorCause(O, options).
        Error::install_error_cause(&o, args.get_or_undefined(1), context)?;

        // 5. Return O.
        Ok(o.into())
    }
}
