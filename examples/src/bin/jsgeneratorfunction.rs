//! Example demonstrating the `JsGeneratorFunction` API wrapper.
use boa_engine::{
    Context, JsValue, Source,
    object::builtins::{JsGenerator, JsGeneratorFunction},
};

fn main() {
    let mut context = Context::default();

    let result = context
        .eval(Source::from_bytes(
            "function* count() { yield 1; yield 2; yield 3; } count",
        ))
        .unwrap();
    let obj = result.as_object().unwrap().clone();

    let gen_fn = JsGeneratorFunction::from_object(obj).unwrap();

    // Call the generator function to obtain a generator instance
    let gen_result = gen_fn
        .call(&JsValue::undefined(), &[], &mut context)
        .unwrap();
    let gen_obj = gen_result.as_object().unwrap().clone();

    let generator = JsGenerator::from_object(gen_obj).unwrap();

    // Iterate the generator
    let result = generator.next(JsValue::undefined(), &mut context).unwrap();
    println!("next: {}", result.display());

    let result = generator.next(JsValue::undefined(), &mut context).unwrap();
    println!("next: {}", result.display());
}
