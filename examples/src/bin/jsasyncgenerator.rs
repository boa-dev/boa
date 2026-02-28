//! Example demonstrating the `JsAsyncGenerator` API wrapper.
use boa_engine::{Context, JsValue, Source, object::builtins::JsAsyncGenerator};

fn main() {
    let mut context = Context::default();

    let result = context
        .eval(Source::from_bytes(
            "async function* count() { yield 1; yield 2; yield 3; } count()",
        ))
        .unwrap();

    let obj = result.as_object().unwrap().clone();
    let async_gen = JsAsyncGenerator::from_object(obj).unwrap();

    // next() returns a Promise
    let promise = async_gen.next(JsValue::undefined(), &mut context).unwrap();
    println!("next promise state: {:?}", promise.state());

    // return() resolves the generator early
    let promise = async_gen
        .r#return(JsValue::undefined(), &mut context)
        .unwrap();
    println!("return promise state: {:?}", promise.state());
}
