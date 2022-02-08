//! This module implements the global `GeneratorFunction` object.
//!
//! The `GeneratorFunction` constructor creates a new generator function object.
//! In JavaScript, every generator function is actually a `GeneratorFunction` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-generatorfunction-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/GeneratorFunction

use crate::{
    builtins::{function::Function, BuiltIn},
    context::StandardObjects,
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData},
    property::{Attribute, PropertyDescriptor},
    symbol::WellKnownSymbols,
    value::JsValue,
    Context, JsResult,
};
use boa_profiler::Profiler;

/// The internal representation on a `Generator` object.
#[derive(Debug, Clone, Copy)]
pub struct GeneratorFunction;

impl BuiltIn for GeneratorFunction {
    const NAME: &'static str = "GeneratorFunction";

    const ATTRIBUTE: Attribute = Attribute::NON_ENUMERABLE.union(Attribute::WRITABLE);

    fn init(context: &mut Context) -> JsValue {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let prototype = &context
            .standard_objects()
            .generator_function_object()
            .prototype;
        let constructor = &context
            .standard_objects()
            .generator_function_object()
            .constructor;

        constructor.set_prototype(Some(
            context.standard_objects().function_object().constructor(),
        ));
        let property = PropertyDescriptor::builder()
            .value(1)
            .writable(false)
            .enumerable(false)
            .configurable(true);
        constructor.borrow_mut().insert("length", property);
        let property = PropertyDescriptor::builder()
            .value("GeneratorFunction")
            .writable(false)
            .enumerable(false)
            .configurable(true);
        constructor.borrow_mut().insert("name", property);
        let property = PropertyDescriptor::builder()
            .value(
                context
                    .standard_objects()
                    .generator_function_object()
                    .prototype(),
            )
            .writable(false)
            .enumerable(false)
            .configurable(false);
        constructor.borrow_mut().insert("prototype", property);
        constructor.borrow_mut().data = ObjectData::function(Function::Native {
            function: Self::constructor,
            constructor: true,
        });

        prototype.set_prototype(Some(
            context.standard_objects().function_object().prototype(),
        ));
        let property = PropertyDescriptor::builder()
            .value(
                context
                    .standard_objects()
                    .generator_function_object()
                    .constructor(),
            )
            .writable(false)
            .enumerable(false)
            .configurable(true);
        prototype.borrow_mut().insert("constructor", property);
        let property = PropertyDescriptor::builder()
            .value(context.standard_objects().generator_object().prototype())
            .writable(false)
            .enumerable(false)
            .configurable(true);
        prototype.borrow_mut().insert("prototype", property);
        let property = PropertyDescriptor::builder()
            .value("GeneratorFunction")
            .writable(false)
            .enumerable(false)
            .configurable(true);
        prototype
            .borrow_mut()
            .insert(WellKnownSymbols::to_string_tag(), property);

        JsValue::null()
    }
}

impl GeneratorFunction {
    pub(crate) fn constructor(
        new_target: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardObjects::generator_function_object,
            context,
        )?;

        let this = JsObject::from_proto_and_data(
            prototype,
            ObjectData::function(Function::Native {
                function: |_, _, _| Ok(JsValue::undefined()),
                constructor: true,
            }),
        );

        Ok(this.into())
    }
}
