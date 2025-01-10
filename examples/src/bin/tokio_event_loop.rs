use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    time::{Duration, Instant},
};

use boa_engine::{
    context::ContextBuilder,
    job::{JobQueue, NativeAsyncJob, NativeJob},
    js_string,
    native_function::NativeFunction,
    property::Attribute,
    Context, JsArgs, JsNativeError, JsResult, JsValue, Script, Source,
};
use boa_runtime::Console;
use futures_concurrency::future::FutureGroup;
use futures_lite::{future, StreamExt};
use tokio::{task, time};

// This example shows how to create an event loop using the tokio runtime.
// The example contains two "flavors" of event loops:
fn main() {
    // An internally async event loop. This event loop blocks the execution of the thread
    // while executing tasks, but internally uses async to run its tasks.
    internally_async_event_loop();

    // An externally async event loop. This event loop can yield to the runtime to concurrently
    // run tasks with it.
    externally_async_event_loop();
}

/// An event queue using tokio to drive futures to completion.
struct Queue {
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    jobs: RefCell<VecDeque<NativeJob>>,
}

impl Queue {
    fn new() -> Self {
        Self {
            async_jobs: RefCell::default(),
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

impl JobQueue for Queue {
    fn enqueue_job(&self, job: NativeJob, _context: &mut Context) {
        self.jobs.borrow_mut().push_back(job);
    }

    fn enqueue_async_job(&self, async_job: NativeAsyncJob, _context: &mut Context) {
        self.async_jobs.borrow_mut().push_back(async_job);
    }

    // While the sync flavor of `run_jobs` will block the current thread until all the jobs have finished...
    fn run_jobs(&self, context: &mut Context) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();

        task::LocalSet::default().block_on(&runtime, self.run_jobs_async(&RefCell::new(context)));
    }

    // ...the async flavor won't, which allows concurrent execution with external async tasks.
    fn run_jobs_async<'a, 'b, 'fut>(
        &'a self,
        context: &'b RefCell<&mut Context>,
    ) -> Pin<Box<dyn Future<Output = ()> + 'fut>>
    where
        'a: 'fut,
        'b: 'fut,
    {
        Box::pin(async move {
            // Early return in case there were no jobs scheduled.
            if self.jobs.borrow().is_empty() && self.async_jobs.borrow().is_empty() {
                return;
            }
            let mut group = FutureGroup::new();
            loop {
                for job in std::mem::take(&mut *self.async_jobs.borrow_mut()) {
                    group.insert(job.call(context));
                }

                if self.jobs.borrow().is_empty() {
                    let Some(result) = group.next().await else {
                        // Both queues are empty. We can exit.
                        return;
                    };

                    if let Err(err) = result {
                        eprintln!("Uncaught {err}");
                    }

                    continue;
                }

                // We have some jobs pending on the microtask queue. Try to poll the pending
                // tasks once to see if any of them finished, and run the pending microtasks
                // otherwise.
                let Some(result) = future::poll_once(group.next()).await.flatten() else {
                    // No completed jobs. Run the microtask queue once.
                    self.drain_jobs(&mut context.borrow_mut());

                    task::yield_now().await;
                    continue;
                };

                if let Err(err) = result {
                    eprintln!("Uncaught {err}");
                }

                // Only one macrotask can be executed before the next drain of the microtask queue.
                self.drain_jobs(&mut context.borrow_mut());
            }
        })
    }
}

// Example async function. Note that the returned future must be 'static.
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
        time::sleep(Duration::from_millis(u64::from(millis))).await;
        let elapsed = now.elapsed().as_secs_f64();
        Ok(elapsed.into())
    }
}

// Example interval function. We cannot use a function returning async in this case since it would
// borrow the context for too long, but using a `NativeAsyncJob` we can!
fn interval(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let Some(function) = args.get_or_undefined(0).as_callable().cloned() else {
        return Err(JsNativeError::typ()
            .with_message("arg must be a callable")
            .into());
    };

    let this = this.clone();
    let delay = args.get_or_undefined(1).to_u32(context)?;
    let args = args.get(2..).unwrap_or_default().to_vec();

    context.enqueue_async_job(NativeAsyncJob::with_realm(
        move |context| {
            Box::pin(async move {
                let mut timer = time::interval(Duration::from_millis(u64::from(delay)));
                for _ in 0..10 {
                    timer.tick().await;
                    if let Err(err) = function.call(&this, &args, &mut context.borrow_mut()) {
                        eprintln!("Uncaught {err}");
                    }
                }
                Ok(JsValue::undefined())
            })
        },
        context.realm().clone(),
    ));

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
            js_string!("timeout"),
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

    timeout(counter, 100);

    for(let i = 0; i <= 100000; i++) {
        // Emulate a long-running evaluation of a script.
    }
";

// This flavor is most recommended when you have an application that:
//  - Needs to wait until the engine finishes executing; depends on the execution result to continue.
//  - Delegates the execution of the application to the engine's event loop.
fn internally_async_event_loop() {
    println!("====== Internally async event loop. ======");

    // Initialize the queue and the context
    let queue = Queue::new();
    let context = &mut ContextBuilder::new()
        .job_queue(Rc::new(queue))
        .build()
        .unwrap();

    // Then, add the custom runtime.
    add_runtime(context);

    let now = Instant::now();
    println!("Evaluating script...");
    context.eval(Source::from_bytes(SCRIPT)).unwrap();

    // Important to run this after evaluating, since this is what triggers to run the enqueued jobs.
    println!("Running jobs...");
    context.run_jobs();

    println!("Total elapsed time: {:?}\n", now.elapsed());
}

// This flavor is most recommended when you have an application that:
//  - Cannot afford to block until the engine finishes executing.
//  - Needs to process IO requests between executions that will be consumed by the engine.
#[tokio::main]
async fn externally_async_event_loop() {
    println!("====== Externally async event loop. ======");
    // Initialize the queue and the context
    let queue = Queue::new();
    let context = &mut ContextBuilder::new()
        .job_queue(Rc::new(queue))
        .build()
        .unwrap();

    // Then, add the custom runtime.
    add_runtime(context);

    let now = Instant::now();

    // Example of an asynchronous workload that must be run alongside the engine.
    let counter = tokio::spawn(async {
        let mut interval = time::interval(Duration::from_millis(100));
        println!("Starting tokio interval job...");
        for i in 0..10 {
            interval.tick().await;
            println!("Executed interval tick {i}");
        }
        println!("Finished tokio interval job...")
    });

    let local_set = &mut task::LocalSet::default();
    let engine = local_set.run_until(async {
        let script = Script::parse(Source::from_bytes(SCRIPT), None, context).unwrap();

        // `Script::evaluate_async` will yield to the executor from time to time, Unlike `Context::run`
        // or `Script::evaluate` which block the current thread until the execution finishes.
        println!("Evaluating script...");
        script.evaluate_async(context).await.unwrap();

        // Run the jobs asynchronously, which avoids blocking the main thread.
        println!("Running jobs...");
        context.run_jobs_async().await;
        Ok(())
    });

    tokio::try_join!(counter, engine).unwrap();

    println!("Total elapsed time: {:?}\n", now.elapsed());
}
