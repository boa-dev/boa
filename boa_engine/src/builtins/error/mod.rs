//! Boa's implementation of ECMAScript's global `Error` object.
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
    builtins::BuiltInObject,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData},
    property::Attribute,
    realm::Realm,
    string::{common::StaticJsStrings, utf16},
    Context, JsArgs, JsResult, JsString, JsValue,
};
use boa_profiler::Profiler;

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

use super::{BuiltInBuilder, BuiltInConstructor, IntrinsicObject};

/// The kind of a `NativeError` object, per the [ECMAScript spec][spec].
///
/// This is used internally to convert between [`JsObject`] and
/// [`JsNativeError`] correctly, but it can also be used to manually create `Error`
/// objects. However, the recommended way to create them is to construct a
/// `JsNativeError` first, then call [`JsNativeError::to_opaque`],
/// which will assign its prototype, properties and kind automatically.
///
/// For a description of every error kind and its usage, see
/// [`JsNativeErrorKind`][crate::error::JsNativeErrorKind].
///
/// [spec]: https://tc39.es/ecma262/#sec-error-objects
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ErrorKind {
    /// The `AggregateError` object type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-aggregate-error-objects
    Aggregate,

    /// The `Error` object type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-error-objects
    Error,

    /// The `EvalError` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-evalerror
    Eval,

    /// The `TypeError` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror
    Type,

    /// The `RangeError` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-rangeerror
    Range,

    /// The `ReferenceError` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-referenceerror
    Reference,

    /// The `SyntaxError` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-syntaxerror
    Syntax,

    /// The `URIError` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-urierror
    Uri,
}

/// Built-in `Error` object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Error;

impl IntrinsicObject for Error {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(utf16!("name"), Self::NAME, attribute)
            .property(utf16!("message"), js_string!(), attribute)
            .method(Self::to_string, js_string!("toString"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Error {
    const NAME: JsString = StaticJsStrings::ERROR;
}

impl BuiltInConstructor for Error {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::error;

    /// `Error( message [ , options ] )`
    ///
    /// Create a new error object.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, let newTarget be the active function object; else let newTarget be NewTarget.
        let new_target = &if new_target.is_undefined() {
            context
                .active_function_object()
                .unwrap_or_else(|| context.intrinsics().constructors().error().constructor())
                .into()
        } else {
            new_target.clone()
        };

        // 2. Let O be ? OrdinaryCreateFromConstructor(newTarget, "%Error.prototype%", « [[ErrorData]] »).
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::error, context)?;
        let o = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ObjectData::error(ErrorKind::Error),
        );

        // 3. If message is not undefined, then
        let message = args.get_or_undefined(0);
        if !message.is_undefined() {
            // a. Let msg be ? ToString(message).
            let msg = message.to_string(context)?;

            // b. Perform CreateNonEnumerableDataPropertyOrThrow(O, "message", msg).
            o.create_non_enumerable_data_property_or_throw(utf16!("message"), msg, context);
        }

        // 4. Perform ? InstallErrorCause(O, options).
        Self::install_error_cause(&o, args.get_or_undefined(1), context)?;

        // 5. Return O.
        Ok(o.into())
    }
}

impl Error {
    pub(crate) fn install_error_cause(
        o: &JsObject,
        options: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        // 1. If Type(options) is Object and ? HasProperty(options, "cause") is true, then
        if let Some(options) = options.as_object() {
            if options.has_property(utf16!("cause"), context)? {
                // a. Let cause be ? Get(options, "cause").
                let cause = options.get(utf16!("cause"), context)?;

                // b. Perform CreateNonEnumerableDataPropertyOrThrow(O, "cause", cause).
                o.create_non_enumerable_data_property_or_throw(utf16!("cause"), cause, context);
            }
        }

        // 2. Return unused.
        Ok(())
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
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If Type(O) is not Object, throw a TypeError exception.
        let o = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an Object"))?;

        // 3. Let name be ? Get(O, "name").
        let name = o.get(js_string!("name"), context)?;

        // 4. If name is undefined, set name to "Error"; otherwise set name to ? ToString(name).
        let name = if name.is_undefined() {
            js_string!("Error")
        } else {
            name.to_string(context)?
        };

        // 5. Let msg be ? Get(O, "message").
        let msg = o.get(js_string!("message"), context)?;

        // 6. If msg is undefined, set msg to the empty String; otherwise set msg to ? ToString(msg).
        let msg = if msg.is_undefined() {
            js_string!()
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
        Ok(js_string!(&name, utf16!(": "), &msg).into())
    }
}
