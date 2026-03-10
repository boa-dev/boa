use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    mem,
    ops::ControlFlow,
    rc::Rc,
};

use boa_engine::{
    Context, JsResult,
    job::{GenericJob, Job, JobExecutor, NativeAsyncJob, PromiseJob},
};
use event_listener::{Event, IntoNotification};
use futures_concurrency::future::FutureGroup;
use smol::{future::FutureExt, stream::StreamExt};

use crate::{logger::SharedExternalPrinterLogger, uncaught_job_error};

pub(crate) struct Executor {
    promise_jobs: RefCell<VecDeque<PromiseJob>>,
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    generic_jobs: RefCell<VecDeque<GenericJob>>,
    new_event: Event<ControlFlow<()>>,
    idle_tasks_counter: Cell<u8>,

    printer: SharedExternalPrinterLogger,
}

impl Executor {
    pub(crate) fn new(printer: SharedExternalPrinterLogger) -> Self {
        Self {
            promise_jobs: RefCell::default(),
            async_jobs: RefCell::default(),
            generic_jobs: RefCell::default(),
            new_event: Event::with_tag(),
            idle_tasks_counter: Cell::new(0),
            printer,
        }
    }

    pub(crate) fn stop(&self) {
        self.promise_jobs.borrow_mut().clear();
        self.async_jobs.borrow_mut().clear();
        self.generic_jobs.borrow_mut().clear();
        self.new_event.notify(
            self.idle_tasks_counter
                .get()
                .additional()
                .relaxed()
                .tag_with(|| ControlFlow::Break(())),
        );
    }

    /// Continually run all pending promise jobs, yielding to the async
    /// executor after every successful run.
    async fn run_promise_jobs(&self, context: &RefCell<&mut Context>) {
        loop {
            if self.promise_jobs.borrow().is_empty() && self.wait_for_events().await.is_break() {
                return;
            }

            let jobs = mem::take(&mut *self.promise_jobs.borrow_mut());
            {
                let context = &mut context.borrow_mut();
                for job in jobs {
                    if let Err(e) = job.call(context) {
                        self.printer.print(uncaught_job_error(&e));
                    }
                }
                context.clear_kept_objects();
            }
            smol::future::yield_now().await;
        }
    }

    /// Continually run a single pending generic job, yielding to the async
    /// executor after every successful run.
    async fn run_generic_jobs(&self, context: &RefCell<&mut Context>) {
        loop {
            if self.generic_jobs.borrow().is_empty() && self.wait_for_events().await.is_break() {
                return;
            }

            let job = self.generic_jobs.borrow_mut().pop_front();
            if let Some(generic) = job
                && let Err(err) = generic.call(&mut context.borrow_mut())
            {
                self.printer.print(uncaught_job_error(&err));
            }

            context.borrow_mut().clear_kept_objects();
            smol::future::yield_now().await;
        }
    }

    /// Continually run all pending async jobs.
    ///
    /// This does not need to yield to the async executor after every run because
    /// it assumes that every async job will not block the execution thread.
    async fn run_async_tasks(&self, context: &RefCell<&mut Context>) {
        let mut group = FutureGroup::new();
        loop {
            if self.async_jobs.borrow().is_empty()
                && group.is_empty()
                && self.wait_for_events().await.is_break()
            {
                return;
            }

            for job in mem::take(&mut *self.async_jobs.borrow_mut()) {
                group.insert(job.call(context));
            }

            let next_job = async {
                if let Some(Err(err)) = group.next().await {
                    self.printer.print(uncaught_job_error(&err));
                }
                ControlFlow::Continue(())
            };

            // This can only exit if the main program is exiting, so
            // it doesn't matter if we drop all pending futures.
            if next_job.or(self.new_event.listen()).await.is_break() {
                return;
            }

            context.borrow_mut().clear_kept_objects();
        }
    }

    /// Checks for any events that need to be handled.
    ///
    /// Returns `ControlFlow::Break` if all tasks are paused for lack
    /// of new jobs, or if the event loop was manually stopped using the
    /// `Executor::stop()` method.
    async fn wait_for_events(&self) -> ControlFlow<()> {
        let idle_tasks = self.idle_tasks_counter.get();

        // we need to have all 3 tasks idle (counting the task executing
        // this check) to exit from the event loop.
        if idle_tasks >= 2 {
            self.new_event.notify(
                idle_tasks
                    .additional()
                    .relaxed()
                    .tag_with(|| ControlFlow::Break(())),
            );
            return ControlFlow::Break(());
        }
        self.idle_tasks_counter.set(idle_tasks + 1);
        let result = self.new_event.listen().await;
        self.idle_tasks_counter
            .set(self.idle_tasks_counter.get() - 1);
        result
    }
}

impl JobExecutor for Executor {
    fn enqueue_job(self: Rc<Self>, job: Job, _context: &mut Context) {
        match job {
            Job::PromiseJob(job) => self.promise_jobs.borrow_mut().push_back(job),
            Job::AsyncJob(job) => self.async_jobs.borrow_mut().push_back(job),
            Job::TimeoutJob(job) => {
                self.async_jobs
                    .borrow_mut()
                    .push_back(NativeAsyncJob::new(async move |context| {
                        smol::Timer::after(job.timeout().into()).await;
                        job.call(&mut context.borrow_mut())
                    }));
            }
            Job::GenericJob(job) => self.generic_jobs.borrow_mut().push_back(job),
            job => self.printer.print(format!("unsupported job type {job:?}")),
        }
        self.new_event.notify(
            self.idle_tasks_counter
                .get()
                .additional()
                .relaxed()
                .tag_with(|| ControlFlow::Continue(())),
        );
    }

    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()> {
        smol::block_on(self.run_jobs_async(&RefCell::new(context)))
    }

    async fn run_jobs_async(self: Rc<Self>, context: &RefCell<&mut Context>) -> JsResult<()> {
        let executor = smol::LocalExecutor::new();
        let async_task = executor.spawn(self.run_async_tasks(context));
        let generic_task = executor.spawn(self.run_generic_jobs(context));
        let promise_task = executor.spawn(self.run_promise_jobs(context));

        executor
            .run(async {
                async_task.await;
                generic_task.await;
                promise_task.await;
            })
            .await;

        Ok(())
    }
}
