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
    context::intrinsics::StandardConstructors,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, JsObject, ObjectData,
    },
    property::Attribute,
    Context, JsResult, JsString, JsValue,
};
use boa_profiler::Profiler;
use tap::{Conv, Pipe};

pub(crate) mod aggregate;
pub(crate) mod eval;
pub(crate) mod range;
pub(crate) mod reference;
pub(crate) mod syntax;
pub(crate) mod r#type;
pub(crate) mod uri;

#[cfg(test)]
mod tests;

pub(crate) use self::aggregate::AggregateError;
pub(crate) use self::eval::EvalError;
pub(crate) use self::r#type::TypeError;
pub(crate) use self::range::RangeError;
pub(crate) use self::reference::ReferenceError;
pub(crate) use self::syntax::SyntaxError;
pub(crate) use self::uri::UriError;

use super::JsArgs;

/// Built-in `Error` object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Error;

impl BuiltIn for Error {
    const NAME: &'static str = "Error";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().error().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .property("name", Self::NAME, attribute)
        .property("message", "", attribute)
        .method(Self::to_string, "toString", 0)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl Error {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    pub(crate) fn install_error_cause(
        o: &JsObject,
        options: &JsValue,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. If Type(options) is Object and ? HasProperty(options, "cause") is true, then
        if let Some(options) = options.as_object() {
            if options.has_property("cause", context)? {
                // a. Let cause be ? Get(options, "cause").
                let cause = options.get("cause", context)?;

                // b. Perform CreateNonEnumerableDataPropertyOrThrow(O, "cause", cause).
                o.create_non_enumerable_data_property_or_throw("cause", cause, context);
            }
        }

        // 2. Return unused.
        Ok(())
    }

    /// `Error( message [ , options ] )`
    ///
    /// Create a new error object.
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, let newTarget be the active function object; else let newTarget be NewTarget.

        // 2. Let O be ? OrdinaryCreateFromConstructor(newTarget, "%Error.prototype%", « [[ErrorData]] »).
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::error, context)?;
        let o = JsObject::from_proto_and_data(prototype, ObjectData::error());

        // 3. If message is not undefined, then
        let message = args.get_or_undefined(0);
        if !message.is_undefined() {
            // a. Let msg be ? ToString(message).
            let msg = message.to_string(context)?;

            // b. Perform CreateNonEnumerableDataPropertyOrThrow(O, "message", msg).
            o.create_non_enumerable_data_property_or_throw("message", msg, context);
        }

        // 4. Perform ? InstallErrorCause(O, options).
        Self::install_error_cause(&o, args.get_or_undefined(1), context)?;

        // 5. Return O.
        Ok(o.into())
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
