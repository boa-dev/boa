//! This module implements the global `AsyncFunction` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-async-function-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AsyncFunction

use crate::{
    builtins::{
        function::{ConstructorKind, Function},
        BuiltIn,
    },
    object::ObjectData,
    property::PropertyDescriptor,
    symbol::WellKnownSymbols,
    Context, JsResult, JsValue,
};
use boa_profiler::Profiler;

#[derive(Debug, Clone, Copy)]
pub struct AsyncFunction;

impl BuiltIn for AsyncFunction {
    const NAME: &'static str = "AsyncFunction";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let prototype = &context
            .intrinsics()
            .constructors()
            .async_function()
            .prototype;
        let constructor = &context
            .intrinsics()
            .constructors()
            .async_function()
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
            .value(prototype.clone())
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
            .value(constructor.clone())
            .writable(false)
            .enumerable(false)
            .configurable(true);
        prototype.borrow_mut().insert("constructor", property);
        let property = PropertyDescriptor::builder()
            .value(Self::NAME)
            .writable(false)
            .enumerable(false)
            .configurable(true);
        prototype
            .borrow_mut()
            .insert(WellKnownSymbols::to_string_tag(), property);

        None
    }
}

impl AsyncFunction {
    /// `AsyncFunction ( p1, p2, … , pn, body )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-async-function-constructor-arguments
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        crate::builtins::function::BuiltInFunctionObject::create_dynamic_function(
            new_target, args, true, false, context,
        )
        .map(Into::into)
    }
}
