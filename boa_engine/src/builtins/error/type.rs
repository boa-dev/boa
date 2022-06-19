//! This module implements the global `TypeError` object.
//!
//! The `TypeError` object represents an error when an operation could not be performed,
//! typically (but not exclusively) when a value is not of the expected type.
//!
//! A `TypeError` may be thrown when:
//!  - an operand or argument passed to a function is incompatible with the type expected by that operator or function.
//!  - when attempting to modify a value that cannot be changed.
//!  - when attempting to use a value in an inappropriate way.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-typeerror
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypeError

use crate::{
    builtins::{function::Function, BuiltIn, JsArgs},
    context::intrinsics::StandardConstructors,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, JsObject, ObjectData,
    },
    property::{Attribute, PropertyDescriptor},
    Context, JsResult, JsValue,
};
use boa_profiler::Profiler;
use tap::{Conv, Pipe};

use super::Error;

/// JavaScript `TypeError` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TypeError;

impl BuiltIn for TypeError {
    const NAME: &'static str = "TypeError";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let error_constructor = context.intrinsics().constructors().error().constructor();
        let error_prototype = context.intrinsics().constructors().error().prototype();

        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().type_error().clone(),
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

impl TypeError {
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
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::type_error, context)?;
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
        Error::install_error_cause(&o, args.get_or_undefined(1), context)?;

        // 5. Return O.
        Ok(o.into())
    }
}

pub(crate) fn create_throw_type_error(context: &mut Context) -> JsObject {
    fn throw_type_error(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        context.throw_type_error("'caller', 'callee', and 'arguments' properties may not be accessed on strict mode functions or the arguments objects for calls to them")
    }

    let function = JsObject::from_proto_and_data(
        context.intrinsics().constructors().function().prototype(),
        ObjectData::function(Function::Native {
            function: throw_type_error,
            constructor: None,
        }),
    );

    let property = PropertyDescriptor::builder()
        .writable(false)
        .enumerable(false)
        .configurable(false);
    function.insert_property("name", property.clone().value("ThrowTypeError"));
    function.insert_property("length", property.value(0));

    function
}
