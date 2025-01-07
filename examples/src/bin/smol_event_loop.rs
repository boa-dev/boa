use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    rc::Rc,
    time::{Duration, Instant},
};

use boa_engine::{
    context::ContextBuilder,
    job::{FutureJob, JobQueue, NativeJob},
    js_string,
    native_function::NativeFunction,
    property::Attribute,
    Context, JsArgs, JsResult, JsValue, Source,
};
use boa_runtime::Console;
use futures_concurrency::future::FutureGroup;
use smol::{future, stream::StreamExt, LocalExecutor};

/// An event queue that also drives futures to completion.
struct Queue<'a> {
    executor: LocalExecutor<'a>,
    futures: RefCell<Vec<FutureJob>>,
    jobs: RefCell<VecDeque<NativeJob>>,
}

impl<'a> Queue<'a> {
    fn new(executor: LocalExecutor<'a>) -> Self {
        Self {
            executor,
            futures: RefCell::default(),
            jobs: RefCell::default(),
        }
    }

    fn drain_jobs(&self, context: &mut Context) {
        let jobs = std::mem::take(&mut *self.jobs.borrow_mut());
        for job in jobs {
            if let Err(e) = job.call(context) {
                eprintln!("Uncaught {e}");
            }
        }
    }
}

impl JobQueue for Queue<'_> {
    fn enqueue_promise_job(&self, job: NativeJob, _context: &mut Context) {
        self.jobs.borrow_mut().push_back(job);
    }

    fn enqueue_future_job(&self, future: FutureJob, _context: &mut Context) {
        self.futures.borrow_mut().push(future);
    }

    fn run_jobs(&self, context: &mut Context) {
        // Early return in case there were no jobs scheduled.
        if self.jobs.borrow().is_empty() && self.futures.borrow().is_empty() {
            return;
        }

        future::block_on(self.executor.run(async move {
            let mut group = FutureGroup::new();
            loop {
                group.extend(std::mem::take(&mut *self.futures.borrow_mut()));

                if self.jobs.borrow().is_empty() {
                    let Some(job) = group.next().await else {
                        // Both queues are empty. We can exit.
                        return;
                    };

                    // Important to schedule the returned `job` into the job queue, since that's
                    // what allows updating the `Promise` seen by ECMAScript for when the future
                    // completes.
                    self.enqueue_promise_job(job, context);
                    continue;
                }

                // We have some jobs pending on the microtask queue. Try to poll the pending
                // tasks once to see if any of them finished, and run the pending microtasks
                // otherwise.
                let Some(job) = future::poll_once(group.next()).await.flatten() else {
                    // No completed jobs. Run the microtask queue once.
                    self.drain_jobs(context);
                    continue;
                };

                // Important to schedule the returned `job` into the job queue, since that's
                // what allows updating the `Promise` seen by ECMAScript for when the future
                // completes.
                self.enqueue_promise_job(job, context);

                // Only one macrotask can be executed before the next drain of the microtask queue.
                self.drain_jobs(context);
            }
        }));
    }
}

// Example async code. Note that the returned future must be 'static.
fn delay(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> impl Future<Output = JsResult<JsValue>> {
    let millis = args.get_or_undefined(0).to_u32(context);

    async move {
        let millis = millis?;
        println!("Delaying for {millis} milliseconds ...");
        let now = Instant::now();
        smol::Timer::after(Duration::from_millis(u64::from(millis))).await;
        let elapsed = now.elapsed().as_secs_f64();
        Ok(elapsed.into())
    }
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
}

fn main() {
    // Initialize the required executors and the context
    let executor = LocalExecutor::new();
    let queue = Queue::new(executor);
    let context = &mut ContextBuilder::new()
        .job_queue(Rc::new(queue))
        .build()
        .unwrap();

    // Then, add a custom runtime.
    add_runtime(context);

    // Multiple calls to multiple async timers.
    let script = r"
        function print(elapsed) {
            console.log(`Finished. elapsed time: ${elapsed * 1000} ms`)
        }
        delay(1000).then(print);
        delay(500).then(print);
        delay(200).then(print);
        delay(600).then(print);
        delay(30).then(print);
    ";

    let now = Instant::now();
    context.eval(Source::from_bytes(script)).unwrap();

    // Important to run this after evaluating, since this is what triggers to run the enqueued jobs.
    context.run_jobs();

    println!("Total elapsed time: {:?}", now.elapsed());

    // Example output:

    // Delaying for 1000 milliseconds ...
    // Delaying for 500 milliseconds ...
    // Delaying for 200 milliseconds ...
    // Delaying for 600 milliseconds ...
    // Delaying for 30 milliseconds ...
    // Finished. elapsed time: 30.073821000000002 ms
    // Finished. elapsed time: 200.079116 ms
    // Finished. elapsed time: 500.10745099999997 ms
    // Finished. elapsed time: 600.098433 ms
    // Finished. elapsed time: 1000.118099 ms
    // Total elapsed time: 1.002628715s

    // The queue concurrently drove several timers to completion!
}
