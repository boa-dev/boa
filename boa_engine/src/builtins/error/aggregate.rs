//! This module implements the global `AggregateError` object.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-aggregate-error
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AggregateError

use crate::{
    builtins::{
        iterable::iterable_to_list, Array, BuiltInBuilder, BuiltInConstructor, BuiltInObject,
        IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData},
    property::{Attribute, PropertyDescriptorBuilder},
    Context, JsArgs, JsResult, JsValue,
};
use boa_profiler::Profiler;

use super::{Error, ErrorKind};

#[derive(Debug, Clone, Copy)]
pub(crate) struct AggregateError;

impl IntrinsicObject for AggregateError {
    fn init(intrinsics: &Intrinsics) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let attribute = Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        BuiltInBuilder::from_standard_constructor::<Self>(intrinsics)
            .prototype(intrinsics.constructors().error().constructor())
            .inherits(Some(intrinsics.constructors().error().prototype()))
            .property("name", Self::NAME, attribute)
            .property("message", "", attribute)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for AggregateError {
    const NAME: &'static str = "AggregateError";
}

impl BuiltInConstructor for AggregateError {
    const LENGTH: usize = 2;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::aggregate_error;

    /// Create a new aggregate error object.
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
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
