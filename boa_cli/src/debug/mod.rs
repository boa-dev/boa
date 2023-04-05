// Allow lint so it, doesn't warn about `JsResult<>` unneeded return on functions.
#![allow(clippy::unnecessary_wraps)]

use boa_engine::{object::ObjectInitializer, property::Attribute, Context, JsObject};

mod function;
mod gc;
mod object;
mod optimizer;

fn create_boa_object(context: &mut Context<'_>) -> JsObject {
    let function_module = function::create_object(context);
    let object_module = object::create_object(context);
    let optimizer_module = optimizer::create_object(context);
    let gc_module = gc::create_object(context);

    ObjectInitializer::new(context)
        .property(
            "function",
            function_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            "object",
            object_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            "optimizer",
            optimizer_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            "gc",
            gc_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .build()
}

pub(crate) fn init_boa_debug_object(context: &mut Context<'_>) {
    let boa_object = create_boa_object(context);
    context
        .register_global_property(
            "$boa",
            boa_object,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .expect("cannot fail with the default object");
}
