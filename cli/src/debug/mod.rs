// Allow lint so it, doesn't warn about `JsResult<>` unneeded return on functions.
#![allow(clippy::unnecessary_wraps)]

use boa_engine::{js_str, object::ObjectInitializer, property::Attribute, Context, JsObject};

mod function;
mod gc;
mod limits;
mod object;
mod optimizer;
mod realm;
mod shape;
mod string;

fn create_boa_object(context: &mut Context) -> JsObject {
    let function_module = function::create_object(context);
    let object_module = object::create_object(context);
    let shape_module = shape::create_object(context);
    let optimizer_module = optimizer::create_object(context);
    let gc_module = gc::create_object(context);
    let realm_module = realm::create_object(context);
    let limits_module = limits::create_object(context);
    let string_module = string::create_string(context);

    ObjectInitializer::new(context)
        .property(
            js_str!("function"),
            function_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_str!("object"),
            object_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_str!("shape"),
            shape_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_str!("optimizer"),
            optimizer_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_str!("gc"),
            gc_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_str!("realm"),
            realm_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_str!("limits"),
            limits_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_str!("string"),
            string_module,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .build()
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) fn init_boa_debug_object(context: &mut Context) {
    let boa_object = create_boa_object(context);
    context
        .register_global_property(
            js_str!("$boa"),
            boa_object,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .expect("cannot fail with the default object");
}
