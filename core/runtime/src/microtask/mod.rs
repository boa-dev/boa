//! Microtask-related functions and types.
use boa_engine::realm::Realm;
use boa_engine::{Context, JsResult, boa_module};

#[cfg(test)]
mod tests;

/// JavaScript module containing the `queueMicrotask` function.
#[boa_module]
pub mod js_module {
    use boa_engine::job::{Job, PromiseJob};
    use boa_engine::object::builtins::JsFunction;
    use boa_engine::{Context, JsValue};

    /// The [`queueMicrotask()`][mdn] method of the `Window` interface queues a
    /// microtask to be executed at a safe time prior to control returning to
    /// the browser's event loop.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Window/queueMicrotask
    pub fn queue_microtask(callback: JsFunction, context: &Context) {
        context.enqueue_job(Job::from(PromiseJob::new(move |context| {
            callback.call(&JsValue::undefined(), &[], context)
        })));
    }
}

/// Register the `queueMicrotask` function to the realm or context.
///
/// # Errors
/// Returns an error if the microtask extension cannot be registered.
pub fn register(realm: Option<Realm>, context: &Context) -> JsResult<()> {
    js_module::boa_register(realm, context)
}
