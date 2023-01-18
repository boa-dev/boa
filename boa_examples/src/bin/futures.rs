use std::{
    cell::RefCell,
    collections::VecDeque,
    time::{Duration, Instant},
};

use boa_engine::{
    builtins::JsArgs,
    context::ContextBuilder,
    job::{FutureJob, JobQueue, NativeJob},
    native_function::NativeFunction,
    Context, JsResult, JsValue,
};
use futures::{stream::FuturesUnordered, Future};
use smol::{future, stream::StreamExt, LocalExecutor};

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
        self.futures.borrow_mut().push(future)
    }

    fn run_jobs(&self, context: &mut boa_engine::Context<'_>) {
        // Example implementation of a job queue that also drives futures to completion.
        loop {
            // Need to check if both `futures` and `jobs` are empty, since any of the inner
            // futures/jobs could schedule more futures/jobs.
            if self.jobs.borrow().is_empty() && self.futures.borrow().is_empty() {
                return;
            }

            // Blocks on all the enqueued futures, driving them all to completion.
            // This implementation is not optimal because it blocks the main thread until
            // the completion of the futures, which will delay running the generated jobs.
            // A more optimal implementation could spawn tasks for each native job, interleave resolving
            // futures with resolving promises, or other fun things!
            future::block_on(self.executor.run(async {
                while let Some(job) = self.futures.borrow_mut().next().await {
                    // Important to either run or schedule the returned `job` into the job queue,
                    // since that's what allows updating the `Promise` seen by ECMAScript for when the
                    // future completes.
                    self.jobs.borrow_mut().push_back(job);
                }
            }));

            let jobs = std::mem::take(&mut *self.jobs.borrow_mut());
            for job in jobs {
                if let Err(e) = job.call(context) {
                    eprintln!("Uncaught {e}");
                }
            }
        }
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
    let queue = Queue::new(executor);
    let context = &mut ContextBuilder::new().job_queue(&queue).build();

    // Bind the defined async function to the ECMAScript function "delay".
    context.register_global_builtin_callable("delay", 1, NativeFunction::from_async_fn(delay));

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

    context.eval(script).unwrap();

    // Important to run this after evaluating, since this is what triggers to run the enqueued jobs.
    context.run_jobs();

    // Example output:

    // Delaying for 1000 milliseconds ...
    // Delaying for 500 milliseconds ...
    // Delaying for 200 milliseconds ...
    // Delaying for 600 milliseconds ...
    // Delaying for 30 milliseconds ...
    // Finished. elapsed time: 30.095278 ms
    // Finished. elapsed time: 200.111445 ms
    // Finished. elapsed time: 500.20203200000003 ms
    // Finished. elapsed time: 600.1167800000001 ms
    // Finished. elapsed time: 1000.13678 ms

    // The system concurrently drove several timers to completion!
}
