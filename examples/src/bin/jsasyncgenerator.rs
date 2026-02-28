//! Example demonstrating the `JsAsyncGenerator` API wrapper.
use boa_engine::{
    Context, JsValue, Source, builtins::promise::PromiseState, object::builtins::JsAsyncGenerator,
};

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
    drop(context.run_jobs());
    if let PromiseState::Fulfilled(val) = promise.state() {
        println!("next resolved with: {}", val.display());
    }

    // return() resolves the generator early
    let promise = async_gen.r#return(JsValue::from(42), &mut context).unwrap();
    drop(context.run_jobs());
    if let PromiseState::Fulfilled(val) = promise.state() {
        println!("return resolved with: {}", val.display());
    }
}
