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