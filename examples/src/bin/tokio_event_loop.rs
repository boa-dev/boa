use boa_engine::context::time::JsInstant;
use boa_engine::job::{GenericJob, TimeoutJob};
use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, Script, Source,
    context::ContextBuilder,
    job::{Job, JobExecutor, NativeAsyncJob, PromiseJob},
    js_string,
    native_function::NativeFunction,
    property::Attribute,
};
use boa_runtime::Console;
use futures_concurrency::future::FutureGroup;
use futures_lite::{StreamExt, future};
use std::collections::BTreeMap;
use std::ops::DerefMut;
use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::{task, time};

// This example shows how to create an event loop using the tokio runtime.
// The example contains two "flavors" of event loops:
fn main() -> JsResult<()> {
    // An internally async event loop. This event loop blocks the execution of the thread
    // while executing tasks, but internally uses async to run its tasks.
    internally_async_event_loop()?;

    // An externally async event loop. This event loop can yield to the runtime to concurrently
    // run tasks with it.
    externally_async_event_loop()
}

/// An event queue using tokio to drive futures to completion.
struct Queue {
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    promise_jobs: RefCell<VecDeque<PromiseJob>>,
    timeout_jobs: RefCell<BTreeMap<JsInstant, TimeoutJob>>,
    generic_jobs: RefCell<VecDeque<GenericJob>>,
}

impl Queue {
    fn new() -> Self {
        Self {
            async_jobs: RefCell::default(),
            promise_jobs: RefCell::default(),
            timeout_jobs: RefCell::default(),
            generic_jobs: RefCell::default(),
        }
    }

    fn drain_timeout_jobs(&self, context: &mut Context) {
        let now = context.clock().now();

        let mut timeouts_borrow = self.timeout_jobs.borrow_mut();
        let mut jobs_to_keep = timeouts_borrow.split_off(&now);
        jobs_to_keep.retain(|_, job| !job.is_cancelled());
        let jobs_to_run = std::mem::replace(timeouts_borrow.deref_mut(), jobs_to_keep);
        drop(timeouts_borrow);

        for job in jobs_to_run.into_values() {
            if let Err(e) = job.call(context) {
                eprintln!("Uncaught {e}");
            }
        }
    }

    fn drain_jobs(&self, context: &mut Context) {
        context.enqueue_resolved_context_jobs();

        // Run the timeout jobs first.
        self.drain_timeout_jobs(context);

        let job = self.generic_jobs.borrow_mut().pop_front();
        if let Some(generic) = job
            && let Err(err) = generic.call(context)
        {
            eprintln!("Uncaught {err}");
        }

        let jobs = std::mem::take(&mut *self.promise_jobs.borrow_mut());
        for job in jobs {
            if let Err(e) = job.call(context) {
                eprintln!("Uncaught {e}");
            }
        }
        context.clear_kept_objects();
    }
}

impl JobExecutor for Queue {
    fn enqueue_job(self: Rc<Self>, job: Job, context: &mut Context) {
        match job {
            Job::PromiseJob(job) => self.promise_jobs.borrow_mut().push_back(job),
            Job::AsyncJob(job) => self.async_jobs.borrow_mut().push_back(job),
            Job::TimeoutJob(t) => {
                let now = context.clock().now();
                self.timeout_jobs.borrow_mut().insert(now + t.timeout(), t);
            }
            Job::GenericJob(g) => self.generic_jobs.borrow_mut().push_back(g),
            _ => panic!("unsupported job type"),
        }
    }

    // While the sync flavor of `run_jobs` will block the current thread until all the jobs have finished...
    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();

        task::LocalSet::default().block_on(&runtime, self.run_jobs_async(&RefCell::new(context)))
    }

    // ...the async flavor won't, which allows concurrent execution with external async tasks.
    async fn run_jobs_async(self: Rc<Self>, context: &RefCell<&mut Context>) -> JsResult<()> {
        let mut group = FutureGroup::new();
        loop {
            for job in std::mem::take(&mut *self.async_jobs.borrow_mut()) {
                group.insert(job.call(context));
            }

            if group.is_empty()
                && self.promise_jobs.borrow().is_empty()
                && self.timeout_jobs.borrow().is_empty()
                && self.generic_jobs.borrow().is_empty()
                && !context.borrow().has_pending_context_jobs()
            {
                // All queues are empty. We can exit.
                return Ok(());
            }

            // We have some jobs pending on the microtask queue. Try to poll the pending
            // tasks once to see if any of them finished, and run the pending microtasks
            // otherwise.
            if let Some(Err(err)) = future::poll_once(group.next()).await.flatten() {
                eprintln!("Uncaught {err}");
            };

            // Only one macrotask can be executed before the next drain of the microtask queue.
            self.drain_jobs(&mut context.borrow_mut());
            task::yield_now().await
        }
    }
}

// Example async function. Note that the returned future must be 'static.
fn delay(
    _this: &JsValue,
    args: &[JsValue],
    context: &RefCell<&mut Context>,
) -> impl Future<Output = JsResult<JsValue>> {
    let millis = args.get_or_undefined(0).to_u32(&mut context.borrow_mut());

    async move {
        let millis = millis?;
        println!("Delaying for {millis} milliseconds ...");
        let now = Instant::now();
        time::sleep(Duration::from_millis(u64::from(millis))).await;
        let elapsed = now.elapsed().as_secs_f64();
        Ok(elapsed.into())
    }
}

// Example interval function, but using a `NativeAsyncJob` instead of an async
// function to schedule the async job.
fn interval(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let Some(function) = args.get_or_undefined(0).as_callable() else {
        return Err(JsNativeError::typ()
            .with_message("arg must be a callable")
            .into());
    };

    let this = this.clone();
    let delay = args.get_or_undefined(1).to_u32(context)?;
    let args = args.get(2..).unwrap_or_default().to_vec();

    context.enqueue_job(
        NativeAsyncJob::with_realm(
            async move |context: &RefCell<&mut Context>| {
                let mut timer = time::interval(Duration::from_millis(u64::from(delay)));
                for _ in 0..10 {
                    timer.tick().await;
                    if let Err(err) = function.call(&this, &args, &mut context.borrow_mut()) {
                        eprintln!("Uncaught {err}");
                    }
                }
                Ok(JsValue::undefined())
            },
            context.realm().clone(),
        )
        .into(),
    );

    Ok(JsValue::undefined())
}

/// Adds the custom runtime to the context.
fn add_runtime(context: &mut Context) {
    // First add the `console` object, to be able to call `console.log()`.
    let console = Console::init(context);
    context
        .register_global_property(Console::NAME, console, Attribute::all())
        .expect("the console builtin shouldn't exist");

    // Then, bind the defined async function to the ECMAScript function "delay".
    context
        .register_global_builtin_callable(
            js_string!("delay"),
            1,
            NativeFunction::from_async_fn(delay),
        )
        .expect("the delay builtin shouldn't exist");

    // Finally, bind the defined async job to the ECMAScript function "interval".
    context
        .register_global_builtin_callable(
            js_string!("interval"),
            1,
            NativeFunction::from_fn_ptr(interval),
        )
        .expect("the delay builtin shouldn't exist");
}

// Script that does multiple calls to multiple async timers.
const SCRIPT: &str = r"
    function print(elapsed) {
        console.log(`Finished delay. Elapsed time: ${elapsed * 1000} ms`);
    }

    delay(1000).then(print);
    delay(500).then(print);
    delay(200).then(print);
    delay(600).then(print);
    delay(30).then(print);

    let i = 0;
    function counter() {
        console.log(`Iteration number ${i} for JS interval`);
        i += 1;
    }

    interval(counter, 100);

    for(let i = 0; i <= 100000; i++) {
        // Emulate a long-running evaluation of a script.
    }
";

// This flavor is most recommended when you have an application that:
//  - Needs to wait until the engine finishes executing; depends on the execution result to continue.
//  - Delegates the execution of the application to the engine's event loop.
fn internally_async_event_loop() -> JsResult<()> {
    println!("====== Internally async event loop. ======");

    // Initialize the queue and the context
    let queue = Queue::new();
    let context = &mut ContextBuilder::new()
        .job_executor(Rc::new(queue))
        .build()
        .unwrap();

    // Then, add the custom runtime.
    add_runtime(context);

    let now = Instant::now();
    println!("Evaluating script...");
    context.eval(Source::from_bytes(SCRIPT)).unwrap();

    // Important to run this after evaluating, since this is what triggers to run the enqueued jobs.
    println!("Running jobs...");
    context.run_jobs()?;

    println!("Total elapsed time: {:?}\n", now.elapsed());

    Ok(())
}

// This flavor is most recommended when you have an application that:
//  - Cannot afford to block until the engine finishes executing.
//  - Needs to process IO requests between executions that will be consumed by the engine.
#[tokio::main]
async fn externally_async_event_loop() -> JsResult<()> {
    println!("====== Externally async event loop. ======");
    // Initialize the queue and the context
    let queue = Rc::new(Queue::new());
    let context = &mut ContextBuilder::new()
        .job_executor(queue.clone())
        .build()
        .unwrap();

    // Then, add the custom runtime.
    add_runtime(context);

    let now = Instant::now();

    // Example of an asynchronous workload that must be run alongside the engine.
    let counter = async {
        tokio::spawn(async {
            let mut interval = time::interval(Duration::from_millis(100));
            println!("Starting tokio interval job...");
            for i in 0..10 {
                interval.tick().await;
                println!("Executed interval tick {i}");
            }
            println!("Finished tokio interval job...")
        })
        .await
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()).into())
    };

    let local_set = &mut task::LocalSet::default();
    let engine = local_set.run_until(async {
        let script = Script::parse(Source::from_bytes(SCRIPT), None, context).unwrap();

        // `Script::evaluate_async` will yield to the executor from time to time, Unlike `Context::run`
        // or `Script::evaluate` which block the current thread until the execution finishes.
        println!("Evaluating script...");
        script.evaluate_async(context).await.unwrap();

        // Run the jobs asynchronously, which avoids blocking the main thread.
        println!("Running jobs...");
        queue.run_jobs_async(&RefCell::new(context)).await
    });

    tokio::try_join!(counter, engine)?;

    println!("Total elapsed time: {:?}\n", now.elapsed());

    Ok(())
}
