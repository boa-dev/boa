use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    mem,
    ops::ControlFlow,
    pin::{Pin, pin},
    rc::Rc,
};

use boa_engine::{
    Context, JsResult, JsValue,
    job::{GenericJob, Job, JobExecutor, NativeAsyncJob, PromiseJob},
};
use futures_concurrency::future::FutureGroup;
use smol::{future::FutureExt, stream::StreamExt};
use unsend::{Event, EventListener, EventListenerRc, IntoNotification};

use crate::{logger::SharedExternalPrinterLogger, uncaught_job_error};

pub(crate) struct Executor {
    promise_jobs: RefCell<VecDeque<PromiseJob>>,
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    generic_jobs: RefCell<VecDeque<GenericJob>>,
    event: Event<ControlFlow<()>>,
    idle_tasks_counter: Cell<u8>,

    printer: SharedExternalPrinterLogger,
}

impl Executor {
    pub(crate) fn new(printer: SharedExternalPrinterLogger) -> Self {
        Self {
            promise_jobs: RefCell::default(),
            async_jobs: RefCell::default(),
            generic_jobs: RefCell::default(),
            event: Event::new(),
            idle_tasks_counter: Cell::new(0),
            printer,
        }
    }

    pub(crate) fn stop(&self) {
        self.promise_jobs.borrow_mut().clear();
        self.async_jobs.borrow_mut().clear();
        self.generic_jobs.borrow_mut().clear();
        self.event
            .notify(u8::MAX.tag_with(|| ControlFlow::Break(())));
    }

    /// Continually run all pending promise jobs, yielding to the async
    /// executor after every successful run.
    async fn run_promise_jobs(&self, context: &RefCell<&mut Context>) {
        let mut listener = EventListener::new(&self.event);
        loop {
            if self.promise_jobs.borrow().is_empty() {
                if self.wait_for_events(pin!(listener)).await.is_break() {
                    return;
                }

                // Restore the listener since it should have been consumed by
                // `wait_for_events`.
                listener = EventListener::new(&self.event);
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
        let mut listener = EventListener::new(&self.event);
        loop {
            if self.generic_jobs.borrow().is_empty() {
                if self.wait_for_events(pin!(listener)).await.is_break() {
                    return;
                }

                // Restore the listener since it should have been consumed by
                // `wait_for_events`.
                listener = EventListener::new(&self.event);
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
        let mut listener = self.event.listen();
        loop {
            if self.async_jobs.borrow().is_empty() && group.is_empty() {
                if self.wait_for_events(listener.as_mut()).await.is_break() {
                    return;
                }

                // Restore the listener since it should have been consumed by
                // `wait_for_events`.
                listener = self.event.listen();
            }

            for job in mem::take(&mut *self.async_jobs.borrow_mut()) {
                group.insert(job.call(context));
            }

            let event_listener = async {
                let result = (&mut listener).await;

                // Restore the listener since it should have been consumed by
                // the await.
                listener = self.event.listen();
                result
            };

            let next_job = async {
                if let Some(Err(err)) = group.next().await {
                    self.printer.print(uncaught_job_error(&err));
                }
                ControlFlow::Continue(())
            };

            // This can only exit if the main program is exiting, so
            // it doesn't matter if we drop all pending futures.
            if event_listener.or(next_job).await.is_break() {
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
    async fn wait_for_events<'a>(
        &'a self,
        listener: Pin<&mut EventListener<'a, ControlFlow<()>>>,
    ) -> ControlFlow<()> {
        let idle_tasks = self.idle_tasks_counter.get();

        // We need to have all 3 tasks idle (counting the task executing
        // this check) to exit from the event loop.
        if idle_tasks >= 2 {
            self.event
                .notify(u8::MAX.tag_with(|| ControlFlow::Break(())));
            return ControlFlow::Break(());
        }

        self.idle_tasks_counter.set(idle_tasks + 1);
        let result = listener.await;

        // Cannot reuse `idle_tasks` since the counter could have updated.
        self.idle_tasks_counter.update(|n| n - 1);

        result
    }
}

impl JobExecutor for Executor {
    fn enqueue_job(self: Rc<Self>, job: Job, _context: &mut Context) {
        match job {
            Job::PromiseJob(job) => self.promise_jobs.borrow_mut().push_back(job),
            Job::AsyncJob(job) => self.async_jobs.borrow_mut().push_back(job),
            Job::TimeoutJob(job) => {
                let event = Rc::new(Event::new());
                let listener = EventListenerRc::new(Rc::clone(&event));
                job.set_cancellation_callback(move || {
                    event.notify(u8::MAX);
                });
                self.async_jobs
                    .borrow_mut()
                    .push_back(NativeAsyncJob::new(async move |context| {
                        let timer = async {
                            smol::Timer::after(job.timeout().into()).await;
                            job.call(&mut context.borrow_mut())
                        };
                        let cancel = async {
                            listener.await;
                            Ok(JsValue::undefined())
                        };
                        timer.or(cancel).await
                    }));
            }
            Job::GenericJob(job) => self.generic_jobs.borrow_mut().push_back(job),
            job => self.printer.print(format!("unsupported job type {job:?}")),
        }
        self.event
            .notify(u8::MAX.tag_with(|| ControlFlow::Continue(())));
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
