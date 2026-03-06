use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    mem,
    rc::Rc,
};

use boa_engine::{
    Context, JsResult,
    context::time::JsInstant,
    job::{GenericJob, Job, JobExecutor, NativeAsyncJob, PromiseJob, TimeoutJob},
};
use futures_concurrency::future::FutureGroup;
use futures_lite::{StreamExt, future};

use crate::{logger::SharedExternalPrinterLogger, uncaught_job_error};

pub(crate) struct Executor {
    promise_jobs: RefCell<VecDeque<PromiseJob>>,
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    timeout_jobs: RefCell<BTreeMap<JsInstant, Vec<TimeoutJob>>>,
    generic_jobs: RefCell<VecDeque<GenericJob>>,

    printer: SharedExternalPrinterLogger,
}

impl Executor {
    pub(crate) fn new(printer: SharedExternalPrinterLogger) -> Self {
        Self {
            promise_jobs: RefCell::default(),
            async_jobs: RefCell::default(),
            timeout_jobs: RefCell::default(),
            generic_jobs: RefCell::default(),
            printer,
        }
    }

    pub(crate) fn clear(&self) {
        self.promise_jobs.borrow_mut().clear();
        self.async_jobs.borrow_mut().clear();
        self.timeout_jobs.borrow_mut().clear();
        self.generic_jobs.borrow_mut().clear();
    }

    fn is_empty(&self) -> bool {
        self.promise_jobs.borrow().is_empty()
            && self.async_jobs.borrow().is_empty()
            && self.timeout_jobs.borrow().is_empty()
            && self.generic_jobs.borrow().is_empty()
    }

    fn drain_timeout_jobs(&self, context: &mut Context) {
        let now = context.clock().now();

        let mut timeouts_borrow = self.timeout_jobs.borrow_mut();
        let mut jobs_to_keep = timeouts_borrow.split_off(&now);
        jobs_to_keep.retain(|_, jobs| {
            jobs.retain(|job| !job.is_cancelled());
            !jobs.is_empty()
        });
        let jobs_to_run = mem::replace(&mut *timeouts_borrow, jobs_to_keep);
        drop(timeouts_borrow);

        for jobs in jobs_to_run.into_values() {
            for job in jobs {
                if !job.is_cancelled()
                    && let Err(e) = job.call(context)
                {
                    self.printer.print(uncaught_job_error(&e));
                }
            }
        }
    }

    fn drain_generic_jobs(&self, context: &mut Context) {
        let job = self.generic_jobs.borrow_mut().pop_front();
        if let Some(generic) = job
            && let Err(err) = generic.call(context)
        {
            self.printer.print(uncaught_job_error(&err));
        }
    }
}

impl JobExecutor for Executor {
    fn enqueue_job(self: Rc<Self>, job: Job, context: &mut Context) {
        match job {
            Job::PromiseJob(job) => self.promise_jobs.borrow_mut().push_back(job),
            Job::AsyncJob(job) => self.async_jobs.borrow_mut().push_back(job),
            Job::TimeoutJob(job) => {
                let now = context.clock().now();
                self.timeout_jobs
                    .borrow_mut()
                    .entry(now + job.timeout())
                    .or_default()
                    .push(job);
            }
            Job::GenericJob(job) => self.generic_jobs.borrow_mut().push_back(job),
            job => self.printer.print(format!("unsupported job type {job:?}")),
        }
    }

    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()> {
        future::block_on(self.run_jobs_async(&RefCell::new(context)))
    }

    async fn run_jobs_async(self: Rc<Self>, context: &RefCell<&mut Context>) -> JsResult<()> {
        let mut group = FutureGroup::new();

        loop {
            for job in mem::take(&mut *self.async_jobs.borrow_mut()) {
                group.insert(job.call(context));
            }

            if let Some(Err(e)) = future::poll_once(group.next()).await.flatten() {
                self.printer.print(uncaught_job_error(&e));
            }

            // We particularly want to make this check here such that the
            // event loop is cancelled almost immediately after the channel with
            // the reader gets closed.
            if self.is_empty() && group.is_empty() {
                return Ok(());
            }

            {
                let context = &mut context.borrow_mut();
                self.drain_timeout_jobs(context);
                self.drain_generic_jobs(context);

                let jobs = mem::take(&mut *self.promise_jobs.borrow_mut());
                for job in jobs {
                    if let Err(e) = job.call(context) {
                        self.printer.print(uncaught_job_error(&e));
                    }
                }
            }
            context.borrow_mut().clear_kept_objects();

            future::yield_now().await;
        }
    }
}
