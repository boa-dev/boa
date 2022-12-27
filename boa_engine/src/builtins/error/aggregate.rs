//! This module implements the global `AggregateError` object.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-aggregate-error
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AggregateError

use crate::{
    builtins::{iterable::iterable_to_list, Array, BuiltIn, JsArgs},
    context::intrinsics::StandardConstructors,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, JsObject, ObjectData,
    },
    property::{Attribute, PropertyDescriptorBuilder},
    Context, JsResult, JsValue,
};
use boa_profiler::Profiler;
use tap::{Conv, Pipe};

use super::{Error, ErrorKind};

#[derive(Debug, Clone, Copy)]
pub(crate) struct AggregateError;

impl BuiltIn for AggregateError {
    const NAME: &'static str = "AggregateError";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let error_constructor = context.intrinsics().constructors().error().constructor();
        let error_prototype = context.intrinsics().constructors().error().prototype();

        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context
                .intrinsics()
                .constructors()
                .aggregate_error()
                .clone(),
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

impl AggregateError {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 2;

    /// Create a new aggregate error object.
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, let newTarget be the active function object; else let newTarget be NewTarget.
        // 2. Let O be ? OrdinaryCreateFromConstructor(newTarget, "%AggregateError.prototype%", « [[ErrorData]] »).
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::aggregate_error,
            context,
        )?;
        let o = JsObject::from_proto_and_data(prototype, ObjectData::error(ErrorKind::Aggregate));

        // 3. If message is not undefined, then
        let message = args.get_or_undefined(1);
        if !message.is_undefined() {
            // a. Let msg be ? ToString(message).
            let msg = message.to_string(context)?;

            // b. Perform CreateNonEnumerableDataPropertyOrThrow(O, "message", msg).
            o.create_non_enumerable_data_property_or_throw("message", msg, context);
        }

        // 4. Perform ? InstallErrorCause(O, options).
        Error::install_error_cause(&o, args.get_or_undefined(2), context)?;

        // 5. Let errorsList be ? IterableToList(errors).
        let errors = args.get_or_undefined(0);
        let errors_list = iterable_to_list(context, errors, None)?;
        // 6. Perform ! DefinePropertyOrThrow(O, "errors",
        //    PropertyDescriptor {
        //      [[Configurable]]: true,
        //      [[Enumerable]]: false,
        //      [[Writable]]: true,
        //      [[Value]]: CreateArrayFromList(errorsList)
        //    }).
        o.define_property_or_throw(
            "errors",
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(false)
                .writable(true)
                .value(Array::create_array_from_list(errors_list, context))
                .build(),
            context,
        )
        .expect("should not fail according to spec");

        // 5. Return O.
        Ok(o.into())
    }
}
