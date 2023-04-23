// Allow lint so it, doesn't warn about `JsResult<>` unneeded return on functions.
#![allow(clippy::unnecessary_wraps)]

use boa_engine::{js_string, object::ObjectInitializer, property::Attribute, Context, JsObject};

mod function;
mod gc;
mod limits;
mod object;
mod optimizer;
mod realm;
mod shape;

fn create_boa_object(context: &mut Context<'_>) -> JsObject {
    let function_module = function::create_object(context);
    let object_module = object::create_object(context);
    let shape_module = shape::create_object(context);
    let optimizer_module = optimizer::create_object(context);
    let gc_module = gc::create_object(context);
    let realm_module = realm::create_object(context);
    let limits_module = limits::create_object(context);

    ObjectInitializer::new(context.realm().clone())
        .property(
            js_string!("function"),
            function_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_string!("object"),
            object_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_string!("shape"),
            shape_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_string!("optimizer"),
            optimizer_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_string!("gc"),
            gc_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_string!("realm"),
            realm_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_string!("limits"),
            limits_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .build()
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) fn init_boa_debug_object(context: &mut Context<'_>) {
    let boa_object = create_boa_object(context);
    context
        .register_global_property(
            js_string!("$boa"),
            boa_object,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .expect("cannot fail with the default object");
}
