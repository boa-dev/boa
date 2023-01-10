//! `ECMAScript` jobs and job queues.
//!
//! This module contains Boa's API to create, customize and interact with `ECMAScript` jobs
//! and job queues.
//!
//! [`NativeJob`] is an ECMAScript [Job], or a closure that runs an `ECMAScript` computation when there's no
//! other computation running.
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

use std::{any::Any, cell::RefCell, collections::VecDeque, fmt::Debug};

use crate::{
    object::{JsFunction, NativeObject},
    Context, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};

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
        Self { f: Box::new(f) }
    }

    /// Calls the native job with the specified [`Context`].
    pub fn call(self, context: &mut Context<'_>) -> JsResult<JsValue> {
        (self.f)(context)
    }
}

/// [`JobCallback`][spec] records
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
        JobCallback {
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
    /// Enqueues a [`NativeJob`] on the job queue. Note that host-defined [Jobs] need to satisfy
    /// a set of requirements for them to be spec-compliant.
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
}

/// A job queue that does nothing.
///
/// This is the default job queue for the [`Context`], and is useful if you want to disable
/// the promise capabilities of the engine.
///
/// If you want to enable running promise jobs, see [`SimpleQueue`].
#[derive(Debug, Clone, Copy)]
pub struct IdleJobQueue;

impl JobQueue for IdleJobQueue {
    fn enqueue_promise_job(&self, _: NativeJob, _: &mut Context<'_>) {}

    fn run_jobs(&self, _: &mut Context<'_>) {}
}

/// A simple FIFO job queue that bails on the first error.
///
/// To enable running promise jobs on the engine, you need to pass it to the [`ContextBuilder`]:
///
/// ```
/// use boa_engine::{context::ContextBuilder, job::SimpleJobQueue};
///
/// let queue = SimpleJobQueue::new();
/// let context = ContextBuilder::new().job_queue(&queue).build();
/// ```
///
/// [`ContextBuilder`]: crate::context::ContextBuilder
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
}
