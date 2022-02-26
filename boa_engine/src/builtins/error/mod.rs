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
    Context, JsResult, JsString, JsValue,
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
        // 1. Let O be the this value.
        let o = if let Some(o) = this.as_object() {
            o
        // 2. If Type(O) is not Object, throw a TypeError exception.
        } else {
            return context.throw_type_error("'this' is not an Object");
        };

        // 3. Let name be ? Get(O, "name").
        let name = o.get("name", context)?;

        // 4. If name is undefined, set name to "Error"; otherwise set name to ? ToString(name).
        let name = if name.is_undefined() {
            JsString::new("Error")
        } else {
            name.to_string(context)?
        };

        // 5. Let msg be ? Get(O, "message").
        let msg = o.get("message", context)?;

        // 6. If msg is undefined, set msg to the empty String; otherwise set msg to ? ToString(msg).
        let msg = if msg.is_undefined() {
            JsString::empty()
        } else {
            msg.to_string(context)?
        };

        // 7. If name is the empty String, return msg.
        if name.is_empty() {
            return Ok(msg.into());
        }

        // 8. If msg is the empty String, return name.
        if msg.is_empty() {
            return Ok(name.into());
        }

        // 9. Return the string-concatenation of name, the code unit 0x003A (COLON),
        // the code unit 0x0020 (SPACE), and msg.
        Ok(format!("{name}: {msg}").into())
    }
}
