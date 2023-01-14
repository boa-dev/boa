//! Data structures for the microtask job queue.

use crate::{object::JsFunction, Context, JsResult, JsValue};
use boa_gc::{Finalize, Trace};

/// An ECMAScript [Job] closure.
///
/// The specification allows scheduling any [`NativeJob`] closure by the host into the job queue.
/// However, custom jobs must abide to a list of requirements.
///
/// ### Requirements
///
/// - At some future point in time, when there is no running execution context and the execution
/// context stack is empty, the implementation must:
///     - Perform any host-defined preparation steps.
///     - Invoke the Job Abstract Closure.
///     - Perform any host-defined cleanup steps, after which the execution context stack must be empty.
/// - Only one Job may be actively undergoing evaluation at any point in time.
/// - Once evaluation of a Job starts, it must run to completion before evaluation of any other Job starts.
/// - The Abstract Closure must return a normal completion, implementing its own handling of errors.
///
/// `NativeJob`'s API differs slightly on the last requirement, since it allows closures returning
/// [`JsResult`], but it's okay because `NativeJob`s are handled by the host anyways; a host could
/// pass a closure returning `Err` and handle the error on `JobQueue::run_jobs`, making the closure
/// effectively run as if it never returned `Err`.
///
/// ## [`Trace`]?
///
/// `NativeJob` doesn't implement `Trace` because it doesn't need to; all jobs can only be run once
/// and putting a `JobQueue` on a garbage collected object should definitely be discouraged.
///
/// On the other hand, it seems like this type breaks all the safety checks of the
/// [`NativeFunction`] API, since you can capture any `Trace` variable into the closure... but it
/// doesn't!
/// The garbage collector doesn't need to trace the captured variables because the closures
/// are always stored on the `JobQueue`, which is always rooted, which means the captured variables
/// are also rooted, allowing us to capture any variable in the closure for free!
///
/// [Job]: https://tc39.es/ecma262/#sec-jobs
/// [`NativeFunction`]: crate::native_function::NativeFunction
pub struct NativeJob {
    #[allow(clippy::type_complexity)]
    f: Box<dyn FnOnce(&mut Context<'_>) -> JsResult<JsValue>>,
}

impl std::fmt::Debug for NativeJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeJob").field("f", &"Closure").finish()
    }
}

impl NativeJob {
    /// Creates a new `NativeJob` from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut Context<'_>) -> JsResult<JsValue> + 'static,
    {
        Self { f: Box::new(f) }
    }

    /// Calls the native job with the specified [`Context`].
    pub fn call(self, context: &mut Context<'_>) -> JsResult<JsValue> {
        (self.f)(context)
    }
}

/// `JobCallback` records
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-jobcallback-records
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JobCallback {
    callback: JsFunction,
}

impl JobCallback {
    /// `HostMakeJobCallback ( callback )`
    ///
    /// The host-defined abstract operation `HostMakeJobCallback` takes argument `callback` (a
    /// function object) and returns a `JobCallback` Record.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hostmakejobcallback
    pub fn make_job_callback(callback: JsFunction) -> Self {
        // 1. Return the JobCallback Record { [[Callback]]: callback, [[HostDefined]]: empty }.
        Self { callback }
    }

    /// `HostCallJobCallback ( jobCallback, V, argumentsList )`
    ///
    /// The host-defined abstract operation `HostCallJobCallback` takes arguments `jobCallback` (a
    /// `JobCallback` Record), `V` (an ECMAScript language value), and `argumentsList` (a `List` of
    /// ECMAScript language values) and returns either a normal completion containing an ECMAScript
    /// language value or a throw completion.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hostcalljobcallback
    ///
    /// # Panics
    ///
    /// Panics if the `JobCallback` is not callable.
    pub fn call_job_callback(
        &self,
        v: &JsValue,
        arguments_list: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // It must perform and return the result of Call(jobCallback.[[Callback]], V, argumentsList).
        // 1. Assert: IsCallable(jobCallback.[[Callback]]) is true.
        // 2. Return ? Call(jobCallback.[[Callback]], V, argumentsList).
        self.callback.call(v, arguments_list, context)
    }
}
