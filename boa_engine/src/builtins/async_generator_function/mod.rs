//! This module implements the `AsyncGeneratorFunction` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorfunction-objects

use crate::{
    builtins::{
        function::{BuiltInFunctionObject, ConstructorKind, Function},
        BuiltIn,
    },
    object::ObjectData,
    property::PropertyDescriptor,
    symbol::WellKnownSymbols,
    value::JsValue,
    Context, JsResult,
};
use boa_profiler::Profiler;

/// The internal representation on a `AsyncGeneratorFunction` object.
#[derive(Debug, Clone, Copy)]
pub struct AsyncGeneratorFunction;

impl BuiltIn for AsyncGeneratorFunction {
    const NAME: &'static str = "AsyncGeneratorFunction";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let prototype = &context
            .intrinsics()
            .constructors()
            .async_generator_function()
            .prototype;
        let constructor = &context
            .intrinsics()
            .constructors()
            .async_generator_function()
            .constructor;

        constructor.set_prototype(Some(
            context.intrinsics().constructors().function().constructor(),
        ));
        let property = PropertyDescriptor::builder()
            .value(1)
            .writable(false)
            .enumerable(false)
            .configurable(true);
        constructor.borrow_mut().insert("length", property);
        let property = PropertyDescriptor::builder()
            .value(Self::NAME)
            .writable(false)
            .enumerable(false)
            .configurable(true);
        constructor.borrow_mut().insert("name", property);
        let property = PropertyDescriptor::builder()
            .value(
                context
                    .intrinsics()
                    .constructors()
                    .async_generator_function()
                    .prototype(),
            )
            .writable(false)
            .enumerable(false)
            .configurable(false);
        constructor.borrow_mut().insert("prototype", property);
        constructor.borrow_mut().data = ObjectData::function(Function::Native {
            function: Self::constructor,
            constructor: Some(ConstructorKind::Base),
        });

        prototype.set_prototype(Some(
            context.intrinsics().constructors().function().prototype(),
        ));
        let property = PropertyDescriptor::builder()
            .value(
                context
                    .intrinsics()
                    .constructors()
                    .async_generator_function()
                    .constructor(),
            )
            .writable(false)
            .enumerable(false)
            .configurable(true);
        prototype.borrow_mut().insert("constructor", property);
        let property = PropertyDescriptor::builder()
            .value(
                context
                    .intrinsics()
                    .constructors()
                    .async_generator()
                    .prototype(),
            )
            .writable(false)
            .enumerable(false)
            .configurable(true);
        prototype.borrow_mut().insert("prototype", property);
        let property = PropertyDescriptor::builder()
            .value("AsyncGeneratorFunction")
            .writable(false)
            .enumerable(false)
            .configurable(true);
        prototype
            .borrow_mut()
            .insert(WellKnownSymbols::to_string_tag(), property);

        None
    }
}

impl AsyncGeneratorFunction {
    /// `AsyncGeneratorFunction ( p1, p2, … , pn, body )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-asyncgeneratorfunction
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        BuiltInFunctionObject::create_dynamic_function(new_target, args, true, true, context)
            .map(Into::into)
    }
}
