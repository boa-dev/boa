use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    mem,
    pin::{Pin, pin},
    rc::Rc,
};

use boa_engine::{
    Context, JsResult, JsValue,
    job::{GenericJob, Job, JobExecutor, NativeAsyncJob, PromiseJob},
};
use futures_concurrency::future::FutureGroup;
use smol::{future::FutureExt, stream::StreamExt};
use unsend::{Event, EventListener, EventListenerRc};

use crate::{logger::SharedExternalPrinterLogger, uncaught_job_error};

pub(crate) struct Executor {
    promise_jobs: RefCell<VecDeque<PromiseJob>>,
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    generic_jobs: RefCell<VecDeque<GenericJob>>,
    finalization_registry_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    wake_event: Event<()>,
    idle_event: Event<()>,
    idle_counter: Cell<u8>,

    stop_event: Event<()>,
    printer: SharedExternalPrinterLogger,
}

impl Executor {
    pub(crate) fn new(printer: SharedExternalPrinterLogger) -> Self {
        Self {
            promise_jobs: RefCell::default(),
            async_jobs: RefCell::default(),
            generic_jobs: RefCell::default(),
            finalization_registry_jobs: RefCell::default(),
            wake_event: Event::new(),
            idle_event: Event::new(),
            idle_counter: Cell::new(0),
            stop_event: Event::new(),
            printer,
        }
    }

    pub(crate) fn stop(&self) {
        self.promise_jobs.borrow_mut().clear();
        self.async_jobs.borrow_mut().clear();
        self.generic_jobs.borrow_mut().clear();
        self.finalization_registry_jobs.borrow_mut().clear();
        self.stop_event.notify(u8::MAX);
    }

    /// Waits until there are any new jobs to be handled.
    ///
    /// This will also restore the provided `listener` such that it can keep
    /// listening for more events.
    async fn wait_for_events<'a>(&'a self, mut listener: Pin<&mut EventListener<'a, ()>>) {
        self.idle_event.notify(u8::MAX);

        self.idle_counter.update(|n| n + 1);
        (&mut listener).await;
        // Restore the listener after usage.
        listener.as_mut().listen();
        self.idle_counter.update(|n| n - 1);
    }

    /// Continually run all pending promise jobs, yielding to the async
    /// executor after every successful run.
    async fn run_promise_jobs(&self, context: &RefCell<&mut Context>) {
        let mut listener = pin!(EventListener::new(&self.wake_event));
        loop {
            let jobs = mem::take(&mut *self.promise_jobs.borrow_mut());
            if jobs.is_empty() {
                self.wait_for_events(listener.as_mut()).await;
                continue;
            }

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
        let mut listener = pin!(EventListener::new(&self.wake_event));
        loop {
            let job = self.generic_jobs.borrow_mut().pop_front();
            let Some(job) = job else {
                self.wait_for_events(listener.as_mut()).await;
                continue;
            };

            {
                let context = &mut context.borrow_mut();
                if let Err(err) = job.call(context) {
                    self.printer.print(uncaught_job_error(&err));
                }
                context.clear_kept_objects();
            }

            smol::future::yield_now().await;
        }
    }

    /// Continually run all pending async jobs.
    //
    /// This does not need to yield to the async executor after every run because
    /// it assumes that every async job will not would never
    /// exit.
    async fn run_async_jobs(&self, context: &RefCell<&mut Context>) {
        let mut group = FutureGroup::new();
        let mut listener = pin!(EventListener::new(&self.wake_event));
        loop {
            if self.async_jobs.borrow().is_empty() && group.is_empty() {
                self.wait_for_events(listener.as_mut()).await;
            }

            for job in mem::take(&mut *self.async_jobs.borrow_mut()) {
                group.insert(job.call(context));
            }

            let wake = async {
                (&mut listener).await;

                // Restore the listener since it should have been consumed by
                // the await.
                listener.as_mut().listen();
            };

            let next_job = async {
                if let Some(Err(err)) = group.next().await {
                    self.printer.print(uncaught_job_error(&err));
                }
            };

            wake.or(next_job).await;

            context.borrow_mut().clear_kept_objects();
        }
    }

    /// Continually run all finalization registry async jobs.
    ///
    /// This does not need to yield to the async executor after every run because
    /// it assumes that every async job will not block the execution thread.
    async fn run_finalization_registry_jobs(&self, context: &RefCell<&mut Context>) {
        let mut group = FutureGroup::new();
        let mut listener = pin!(EventListener::new(&self.wake_event));
        loop {
            if self.finalization_registry_jobs.borrow().is_empty() && group.is_empty() {
                (&mut listener).await;

                // Restore the listener since it should have been consumed by
                // the await.
                listener.as_mut().listen();
            }

            for job in mem::take(&mut *self.finalization_registry_jobs.borrow_mut()) {
                group.insert(job.call(context));
            }

            let wake = async {
                (&mut listener).await;

                // Restore the listener since it should have been consumed by
                // the await.
                listener.as_mut().listen();
            };

            let next_job = async {
                if let Some(Err(err)) = group.next().await {
                    self.printer.print(uncaught_job_error(&err));
                }
            };

            wake.or(next_job).await;
        }
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
                        // Clamp timeout to prevent setTimeout(fn, 0) loops
                        // from starving the main event loop. 1ms to match Node:
                        // https://nodejs.org/api/timers.html#settimeoutcallback-delay-args
                        const MIN_TIMEOUT: std::time::Duration =
                            std::time::Duration::from_millis(1);
                        let timeout = std::cmp::max(job.timeout().into(), MIN_TIMEOUT);
                        let timer = async {
                            smol::Timer::after(timeout).await;
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
            Job::FinalizationRegistryCleanupJob(job) => {
                self.finalization_registry_jobs.borrow_mut().push_back(job);
            }
            job => self.printer.print(format!("unsupported job type {job:?}")),
        }
        self.wake_event.notify(u8::MAX);
    }

    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()> {
        smol::block_on(self.run_jobs_async(&RefCell::new(context)))
    }

    async fn run_jobs_async(self: Rc<Self>, context: &RefCell<&mut Context>) -> JsResult<()> {
        let executor = smol::LocalExecutor::new();
        let async_task = executor.spawn(self.run_async_jobs(context));
        let generic_task = executor.spawn(self.run_generic_jobs(context));
        let promise_task = executor.spawn(self.run_promise_jobs(context));

        let foreground = async {
            async_task.await;
            generic_task.await;
            promise_task.await;
        };

        let background = async {
            let mut listener = pin!(EventListener::new(&self.idle_event));
            let mut run_fr_jobs = pin!(self.run_finalization_registry_jobs(context));
            loop {
                let idle_tasks = self.idle_counter.get();
                // We need to have all 3 tasks idle to exit from the event loop.
                if idle_tasks >= 3 {
                    return;
                }

                // Since there are still pending tasks awaiting for IO
                // (probably the async jobs), run any pending finalization registry
                // jobs now that the thread is free to do things.
                //
                // We still need to handle idle event notifications though, because
                // only awaiting the finalization registry jobs would never
                // exit.
                async {
                    (&mut listener).await;

                    // Restore the listener since it should have been consumed by
                    // the await.
                    listener.as_mut().listen();
                }
                .or(&mut run_fr_jobs)
                .await;
            }
        };

        // Stop signal has priority over everything else.
        EventListener::new(&self.stop_event)
            .or(executor.run(foreground))
            .or(background)
            .await;

        Ok(())
    }
}
