//! This module implements the global `SyntaxError` object.
//!
//! The `SyntaxError` object represents an error when trying to interpret syntactically invalid code.
//! It is thrown when the JavaScript context encounters tokens or token order that does not conform
//! to the syntax of the language when parsing code.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-syntaxerror
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SyntaxError

use crate::{
    builtins::{BuiltIn, JsArgs},
    context::intrinsics::StandardConstructors,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, JsObject, ObjectData,
    },
    property::Attribute,
    Context, JsResult, JsValue,
};
use boa_profiler::Profiler;
use tap::{Conv, Pipe};

use super::{Error, ErrorKind};

/// JavaScript `SyntaxError` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SyntaxError;

impl BuiltIn for SyntaxError {
    const NAME: &'static str = "SyntaxError";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let error_constructor = context.intrinsics().constructors().error().constructor();
        let error_prototype = context.intrinsics().constructors().error().prototype();

        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().syntax_error().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .inherit(error_prototype)
        .custom_prototype(error_constructor)
        .property("name", Self::NAME, attribute)
        .property("message", "", attribute)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl SyntaxError {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// Create a new error object.
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, let newTarget be the active function object; else let newTarget be NewTarget.
        // 2. Let O be ? OrdinaryCreateFromConstructor(newTarget, "%NativeError.prototype%", « [[ErrorData]] »).
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::syntax_error,
            context,
        )?;
        let o = JsObject::from_proto_and_data(prototype, ObjectData::error(ErrorKind::Syntax));

        // 3. If message is not undefined, then
        let message = args.get_or_undefined(0);
        if !message.is_undefined() {
            // a. Let msg be ? ToString(message).
            let msg = message.to_string(context)?;

            // b. Perform CreateNonEnumerableDataPropertyOrThrow(O, "message", msg).
            o.create_non_enumerable_data_property_or_throw("message", msg, context);
        }

        // 4. Perform ? InstallErrorCause(O, options).
        Error::install_error_cause(&o, args.get_or_undefined(1), context)?;

        // 5. Return O.
        Ok(o.into())
    }
}
