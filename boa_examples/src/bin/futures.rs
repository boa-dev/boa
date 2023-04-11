use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    time::{Duration, Instant},
};

use boa_engine::{
    context::ContextBuilder,
    job::{FutureJob, JobQueue, NativeJob},
    native_function::NativeFunction,
    Context, JsArgs, JsResult, JsValue, Source,
};
use futures_util::{stream::FuturesUnordered, Future};
use smol::{future, stream::StreamExt, LocalExecutor};

/// An event queue that also drives futures to completion.
struct Queue<'a> {
    executor: LocalExecutor<'a>,
    futures: RefCell<FuturesUnordered<FutureJob>>,
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
}

impl<'a> JobQueue for Queue<'a> {
    fn enqueue_promise_job(&self, job: NativeJob, _context: &mut boa_engine::Context<'_>) {
        self.jobs.borrow_mut().push_back(job);
    }

    fn enqueue_future_job(&self, future: FutureJob, _context: &mut boa_engine::Context<'_>) {
        self.futures.borrow().push(future)
    }

    fn run_jobs(&self, context: &mut boa_engine::Context<'_>) {
        // Early return in case there were no jobs scheduled.
        if self.jobs.borrow().is_empty() && self.futures.borrow().is_empty() {
            return;
        }

        let context = RefCell::new(context);

        future::block_on(self.executor.run(async move {
            // Used to sync the finalization of both tasks
            let finished = Cell::new(0b00u8);

            let fqueue = async {
                loop {
                    if self.futures.borrow().is_empty() {
                        finished.set(finished.get() | 0b01);
                        if finished.get() >= 0b11 {
                            // All possible futures and jobs were completed. Exit.
                            return;
                        }
                        // All possible jobs were completed, but `jqueue` could have
                        // pending jobs. Yield to the executor to try to progress on
                        // `jqueue` until we have more pending futures.
                        future::yield_now().await;
                        continue;
                    }
                    finished.set(finished.get() & 0b10);

                    // Blocks on all the enqueued futures, driving them all to completion.
                    let futures = &mut std::mem::take(&mut *self.futures.borrow_mut());
                    while let Some(job) = futures.next().await {
                        // Important to schedule the returned `job` into the job queue, since that's
                        // what allows updating the `Promise` seen by ECMAScript for when the future
                        // completes.
                        self.enqueue_promise_job(job, &mut context.borrow_mut());
                    }
                }
            };

            let jqueue = async {
                loop {
                    if self.jobs.borrow().is_empty() {
                        finished.set(finished.get() | 0b10);
                        if finished.get() >= 0b11 {
                            // All possible futures and jobs were completed. Exit.
                            return;
                        }
                        // All possible jobs were completed, but `fqueue` could have
                        // pending futures. Yield to the executor to try to progress on
                        // `fqueue` until we have more pending jobs.
                        future::yield_now().await;
                        continue;
                    };
                    finished.set(finished.get() & 0b01);

                    let jobs = std::mem::take(&mut *self.jobs.borrow_mut());
                    for job in jobs {
                        if let Err(e) = job.call(&mut context.borrow_mut()) {
                            eprintln!("Uncaught {e}");
                        }
                        future::yield_now().await;
                    }
                }
            };

            // Wait for both queues to complete
            future::zip(fqueue, jqueue).await;
        }))
    }
}

// Example async code. Note that the returned future must be 'static.
fn delay(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context<'_>,
) -> impl Future<Output = JsResult<JsValue>> {
    let millis = args.get_or_undefined(0).to_u32(context);

    async move {
        let millis = millis?;
        println!("Delaying for {millis} milliseconds ...");
        let now = Instant::now();
        smol::Timer::after(Duration::from_millis(millis as u64)).await;
        let elapsed = now.elapsed().as_secs_f64();
        Ok(elapsed.into())
    }
}

fn main() {
    // Initialize the required executors and the context
    let executor = LocalExecutor::new();
    let queue: &dyn JobQueue = &Queue::new(executor);
    let context = &mut ContextBuilder::new().job_queue(queue).build().unwrap();

    // Bind the defined async function to the ECMAScript function "delay".
    context
        .register_global_builtin_callable("delay", 1, NativeFunction::from_async_fn(delay))
        .unwrap();

    // Multiple calls to multiple async timers.
    let script = r#"
        function print(elapsed) {
            console.log(`Finished. elapsed time: ${elapsed * 1000} ms`)
        }
        delay(1000).then(print);
        delay(500).then(print);
        delay(200).then(print);
        delay(600).then(print);
        delay(30).then(print);
    "#;

    let now = Instant::now();
    context.eval_script(Source::from_bytes(script)).unwrap();

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
