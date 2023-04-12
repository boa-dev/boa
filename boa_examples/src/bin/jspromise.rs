/// Creates a new promise object from an executor function.
///
/// It is equivalent to calling the [`Promise()`] constructor, which makes it share the same
/// execution semantics as the constructor:
/// - The executor function `executor` is called synchronously just after the promise is created.
/// - The executor return value is ignored.
/// - Any error thrown within the execution of `executor` will call the `reject` function
/// of the newly created promise, unless either `resolve` or `reject` were already called
/// beforehand.
///
/// `executor` receives as an argument the [`ResolvingFunctions`] needed to settle the promise,
/// which can be done by either calling the `resolve` function or the `reject` function.
///
use std::error::Error;
use boa_engine::{
job::SimpleJobQueue,
object::builtins::JsPromise,
builtins::promise::PromiseState,
Context, JsValue, js_string};

fn main() -> Result<(), Box<dyn Error>> {

    let queue = &SimpleJobQueue::new();
    let context = &mut Context::builder().job_queue(queue).build()?;

    let promise = JsPromise::new(|resolvers, context| {
        let result = js_string!("hello world").into();
        resolvers.resolve.call(&JsValue::undefined(), &[result], context)?;
        Ok(JsValue::undefined())
    }, context)?;

    context.run_jobs();

    assert_eq!(promise.state()?, PromiseState::Fulfilled(js_string!("hello world").into()));
    Ok(())
}
