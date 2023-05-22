//! Boa's API to create and customize `ECMAScript` jobs and job queues.
//!
//! [`NativeJob`] is an ECMAScript [Job], or a closure that runs an `ECMAScript` computation when
//! there's no other computation running.
//!
//! [`JobCallback`] is an ECMAScript [`JobCallback`] record, containing an `ECMAScript` function
//! that is executed when a promise is either fulfilled or rejected.
//!
//! [`JobQueue`] is a trait encompassing the required functionality for a job queue; this allows
//! implementing custom event loops, custom handling of Jobs or other fun things.
//! This trait is also accompanied by two implementors of the trait:
//! - [`IdleJobQueue`], which is a queue that does nothing, and the default queue if no queue is
//! provided. Useful for hosts that want to disable promises.
//! - [`SimpleJobQueue`], which is a simple FIFO queue that runs all jobs to completion, bailing
//! on the first error encountered.
//!
//! [Job]: https://tc39.es/ecma262/#sec-jobs
//! [JobCallback]: https://tc39.es/ecma262/#sec-jobcallback-records

use std::{any::Any, cell::RefCell, collections::VecDeque, fmt::Debug, future::Future, pin::Pin};

use crate::{
    object::{JsFunction, NativeObject},
    realm::Realm,
    vm::ActiveRunnable,
    Context, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};

/// The [`Future`] job passed to the [`JobQueue::enqueue_future_job`] operation.
pub type FutureJob = Pin<Box<dyn Future<Output = NativeJob> + 'static>>;

/// An ECMAScript [Job] closure.
///
/// The specification allows scheduling any [`NativeJob`] closure by the host into the job queue.
/// However, host-defined jobs must abide to a set of requirements.
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
/// `NativeJob`s API differs slightly on the last requirement, since it allows closures returning
/// [`JsResult`], but it's okay because `NativeJob`s are handled by the host anyways; a host could
/// pass a closure returning `Err` and handle the error on [`JobQueue::run_jobs`], making the closure
/// effectively run as if it never returned `Err`.
///
/// ## [`Trace`]?
///
/// `NativeJob` doesn't implement `Trace` because it doesn't need to; all jobs can only be run once
/// and putting a [`JobQueue`] on a garbage collected object should definitely be discouraged.
///
/// On the other hand, it seems like this type breaks all the safety checks of the
/// [`NativeFunction`] API, since you can capture any `Trace` variable into the closure... but it
/// doesn't!
/// The garbage collector doesn't need to trace the captured variables because the closures
/// are always stored on the [`JobQueue`], which is always rooted, which means the captured variables
/// are also rooted, allowing us to capture any variable in the closure for free!
///
/// [Job]: https://tc39.es/ecma262/#sec-jobs
/// [`NativeFunction`]: crate::native_function::NativeFunction
pub struct NativeJob {
    #[allow(clippy::type_complexity)]
    f: Box<dyn FnOnce(&mut Context<'_>) -> JsResult<JsValue>>,
    realm: Option<Realm>,
    active_runnable: Option<ActiveRunnable>,
}

impl Debug for NativeJob {
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
        Self {
            f: Box::new(f),
            realm: None,
            active_runnable: None,
        }
    }

    /// Creates a new `NativeJob` from a closure and an execution realm.
    pub fn with_realm<F>(f: F, realm: Realm, context: &mut Context<'_>) -> Self
    where
        F: FnOnce(&mut Context<'_>) -> JsResult<JsValue> + 'static,
    {
        Self {
            f: Box::new(f),
            realm: Some(realm),
            active_runnable: context.vm.active_runnable.clone(),
        }
    }

    /// Gets a reference to the execution realm of the job.
    pub const fn realm(&self) -> Option<&Realm> {
        self.realm.as_ref()
    }

    /// Calls the native job with the specified [`Context`].
    ///
    /// # Note
    ///
    /// If the native job has an execution realm defined, this sets the running execution
    /// context to the realm's before calling the inner closure, and resets it after execution.
    pub fn call(mut self, context: &mut Context<'_>) -> JsResult<JsValue> {
        // If realm is not null, each time job is invoked the implementation must perform
        // implementation-defined steps such that execution is prepared to evaluate ECMAScript
        // code at the time of job's invocation.
        if let Some(realm) = self.realm {
            let old_realm = context.enter_realm(realm);

            // Let scriptOrModule be GetActiveScriptOrModule() at the time HostEnqueuePromiseJob is
            // invoked. If realm is not null, each time job is invoked the implementation must
            // perform implementation-defined steps such that scriptOrModule is the active script or
            // module at the time of job's invocation.
            std::mem::swap(&mut context.vm.active_runnable, &mut self.active_runnable);

            let result = (self.f)(context);

            context.enter_realm(old_realm);
            std::mem::swap(&mut context.vm.active_runnable, &mut self.active_runnable);

            result
        } else {
            (self.f)(context)
        }
    }
}

/// [`JobCallback`][spec] records.
///
/// [spec]: https://tc39.es/ecma262/#sec-jobcallback-records
#[derive(Trace, Finalize)]
pub struct JobCallback {
    callback: JsFunction,
    host_defined: Box<dyn NativeObject>,
}

impl Debug for JobCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobCallback")
            .field("callback", &self.callback)
            .field("host_defined", &"dyn NativeObject")
            .finish()
    }
}

impl JobCallback {
    /// Creates a new `JobCallback`.
    pub fn new<T: Any + Trace>(callback: JsFunction, host_defined: T) -> Self {
        Self {
            callback,
            host_defined: Box::new(host_defined),
        }
    }

    /// Gets the inner callback of the job.
    pub const fn callback(&self) -> &JsFunction {
        &self.callback
    }

    /// Gets a reference to the host defined additional field as an `Any` trait object.
    pub fn host_defined(&self) -> &dyn Any {
        self.host_defined.as_any()
    }

    /// Gets a mutable reference to the host defined additional field as an `Any` trait object.
    pub fn host_defined_mut(&mut self) -> &mut dyn Any {
        self.host_defined.as_mut_any()
    }
}

/// A queue of `ECMAscript` [Jobs].
///
/// This is the main API that allows creating custom event loops with custom job queues.
///
/// [Jobs]: https://tc39.es/ecma262/#sec-jobs
pub trait JobQueue {
    /// [`HostEnqueuePromiseJob ( job, realm )`][spec].
    ///
    /// Enqueues a [`NativeJob`] on the job queue.
    ///
    /// # Requirements
    ///
    /// Per the [spec]:
    /// > An implementation of `HostEnqueuePromiseJob` must conform to the requirements in [9.5][Jobs] as well as the
    ///   following:
    /// > - If `realm` is not null, each time `job` is invoked the implementation must perform implementation-defined steps
    ///     such that execution is prepared to evaluate ECMAScript code at the time of job's invocation.
    /// > - Let `scriptOrModule` be `GetActiveScriptOrModule()` at the time `HostEnqueuePromiseJob` is invoked. If realm
    ///     is not null, each time job is invoked the implementation must perform implementation-defined steps such that
    ///     `scriptOrModule` is the active script or module at the time of job's invocation.
    /// > - Jobs must run in the same order as the `HostEnqueuePromiseJob` invocations that scheduled them.
    ///
    /// Of all the requirements, Boa guarantees the first two by its internal implementation of `NativeJob`, meaning
    /// the implementer must only guarantee that jobs are run in the same order as they're enqueued.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hostenqueuepromisejob
    /// [Jobs]: https://tc39.es/ecma262/#sec-jobs
    fn enqueue_promise_job(&self, job: NativeJob, context: &mut Context<'_>);

    /// Runs all jobs in the queue.
    ///
    /// Running a job could enqueue more jobs in the queue. The implementor of the trait
    /// determines if the method should loop until there are no more queued jobs or if
    /// it should only run one iteration of the queue.
    fn run_jobs(&self, context: &mut Context<'_>);

    /// Enqueues a new [`Future`] job on the job queue.
    ///
    /// On completion, `future` returns a new [`NativeJob`] that needs to be enqueued into the
    /// job queue to update the state of the inner `Promise`, which is what ECMAScript sees. Failing
    /// to do this will leave the inner `Promise` in the `pending` state, which won't call any `then`
    /// or `catch` handlers, even if `future` was already completed.
    fn enqueue_future_job(&self, future: FutureJob, context: &mut Context<'_>);
}

/// A job queue that does nothing.
///
/// This queue is mostly useful if you want to disable the promise capabilities of the engine. This
/// can be done by passing a reference to it to the [`ContextBuilder`]:
///
/// ```
/// use boa_engine::{context::ContextBuilder, job::{JobQueue, IdleJobQueue}};
///
/// let queue: &dyn JobQueue = &IdleJobQueue;
/// let context = ContextBuilder::new().job_queue(queue).build();
/// ```
///
/// [`ContextBuilder`]: crate::context::ContextBuilder
#[derive(Debug, Clone, Copy)]
pub struct IdleJobQueue;

impl JobQueue for IdleJobQueue {
    fn enqueue_promise_job(&self, _: NativeJob, _: &mut Context<'_>) {}

    fn run_jobs(&self, _: &mut Context<'_>) {}

    fn enqueue_future_job(&self, _: FutureJob, _: &mut Context<'_>) {}
}

/// A simple FIFO job queue that bails on the first error.
///
/// This is the default job queue for the [`Context`], but it is mostly pretty limited for
/// custom event queues.
///
/// To disable running promise jobs on the engine, see [`IdleJobQueue`].
#[derive(Default)]
pub struct SimpleJobQueue(RefCell<VecDeque<NativeJob>>);

impl Debug for SimpleJobQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SimpleQueue").field(&"..").finish()
    }
}

impl SimpleJobQueue {
    /// Creates an empty `SimpleJobQueue`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl JobQueue for SimpleJobQueue {
    fn enqueue_promise_job(&self, job: NativeJob, _: &mut Context<'_>) {
        // If realm is not null ...
        // TODO
        // Let scriptOrModule be ...
        // TODO
        self.0.borrow_mut().push_back(job);
    }

    fn run_jobs(&self, context: &mut Context<'_>) {
        // Yeah, I have no idea why Rust extends the lifetime of a `RefCell` that should be immediately
        // dropped after calling `pop_front`.
        let mut next_job = self.0.borrow_mut().pop_front();
        while let Some(job) = next_job {
            if job.call(context).is_err() {
                self.0.borrow_mut().clear();
                return;
            };
            next_job = self.0.borrow_mut().pop_front();
        }
    }

    fn enqueue_future_job(&self, future: FutureJob, context: &mut Context<'_>) {
        let job = pollster::block_on(future);
        self.enqueue_promise_job(job, context);
    }
}
