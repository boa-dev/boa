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
    builtins::BuiltIn,
    context::StandardObjects,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, JsObject, ObjectData,
    },
    property::Attribute,
    Context, JsResult, JsValue,
};
use boa_profiler::Profiler;

/// JavaScript `EvalError` impleentation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EvalError;

impl BuiltIn for EvalError {
    const NAME: &'static str = "EvalError";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);

    fn init(context: &mut Context) -> JsValue {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let error_prototype = context.standard_objects().error_object().prototype();
        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        let eval_error_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().eval_error_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .inherit(error_prototype)
        .property("name", Self::NAME, attribute)
        .property("message", "", attribute)
        .build();

        eval_error_object.into()
    }
}

impl EvalError {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// Create a new error object.
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype =
            get_prototype_from_constructor(new_target, StandardObjects::error_object, context)?;
        let obj = JsObject::from_proto_and_data(prototype, ObjectData::error());
        if let Some(message) = args.get(0) {
            if !message.is_undefined() {
                obj.set("message", message.to_string(context)?, false, context)?;
            }
        }
        Ok(obj.into())
    }
}
