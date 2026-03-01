//! Example demonstrating the `JsAsyncGenerator` API wrapper.
use boa_engine::{
    Context, JsString, JsValue, Source, builtins::promise::PromiseState,
    object::builtins::JsAsyncGenerator,
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

    // next() returns a Promise; run_jobs() is required to resolve it
    let promise = async_gen.next(JsValue::undefined(), &mut context).unwrap();
    drop(context.run_jobs());
    if let PromiseState::Fulfilled(val) = promise.state() {
        let result_obj = val.as_object().unwrap();
        let value = result_obj.get(JsString::from("value"), &context).unwrap();
        assert_eq!(value, JsValue::from(1));
    }

    // return() resolves the generator early with the given value
    let promise = async_gen.r#return(JsValue::from(42), &mut context).unwrap();
    drop(context.run_jobs());
    if let PromiseState::Fulfilled(val) = promise.state() {
        let result_obj = val.as_object().unwrap();
        let value = result_obj.get(JsString::from("value"), &context).unwrap();
        assert_eq!(value, JsValue::from(42));
    }
}
