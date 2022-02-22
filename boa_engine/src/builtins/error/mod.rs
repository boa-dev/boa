//! This module implements the global `Error` object.
//!
//! Error objects are thrown when runtime errors occur.
//! The Error object can also be used as a base object for user-defined exceptions.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-error-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error

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

pub(crate) mod eval;
pub(crate) mod range;
pub(crate) mod reference;
pub(crate) mod syntax;
pub(crate) mod r#type;
pub(crate) mod uri;

#[cfg(test)]
mod tests;

pub(crate) use self::eval::EvalError;
pub(crate) use self::r#type::TypeError;
pub(crate) use self::range::RangeError;
pub(crate) use self::reference::ReferenceError;
pub(crate) use self::syntax::SyntaxError;
pub(crate) use self::uri::UriError;

/// Built-in `Error` object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Error;

impl BuiltIn for Error {
    const NAME: &'static str = "Error";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);

    fn init(context: &mut Context) -> JsValue {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        let error_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().error_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .property("name", Self::NAME, attribute)
        .property("message", "", attribute)
        .method(Self::to_string, "toString", 0)
        .build();

        error_object.into()
    }
}

impl Error {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// `Error( message )`
    ///
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

    /// `Error.prototype.toString()`
    ///
    /// The toString() method returns a string representing the specified Error object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-error.prototype.tostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/toString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if !this.is_object() {
            return context.throw_type_error("'this' is not an Object");
        }
        let name = this.get_field("name", context)?;
        let name_to_string;
        let name = if name.is_undefined() {
            "Error"
        } else {
            name_to_string = name.to_string(context)?;
            name_to_string.as_str()
        };

        let message = this.get_field("message", context)?;
        let message_to_string;
        let message = if message.is_undefined() {
            ""
        } else {
            message_to_string = message.to_string(context)?;
            message_to_string.as_str()
        };

        if name.is_empty() {
            Ok(message.into())
        } else if message.is_empty() {
            Ok(name.into())
        } else {
            Ok(format!("{name}: {message}").into())
        }
    }
}
