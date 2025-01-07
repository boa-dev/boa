use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use boa_engine::{
    builtins::promise::PromiseState,
    job::{FutureJob, JobQueue, NativeJob},
    js_string,
    module::ModuleLoader,
    Context, JsNativeError, JsResult, JsString, JsValue, Module,
};
use boa_parser::Source;
use futures_concurrency::future::FutureGroup;
use isahc::{
    config::{Configurable, RedirectPolicy},
    AsyncReadResponseExt, Request, RequestExt,
};
use smol::{future, stream::StreamExt, LocalExecutor};

// Most of the boilerplate is taken from the `futures.rs` example.
// This file only explains what is exclusive of async module loading.

#[derive(Debug, Default)]
struct HttpModuleLoader;

impl ModuleLoader for HttpModuleLoader {
    fn load_imported_module(
        &self,
        _referrer: boa_engine::module::Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut Context)>,
        context: &mut Context,
    ) {
        let url = specifier.to_std_string_escaped();

        let fetch = async move {
            // Adding some prints to show the non-deterministic nature of the async fetches.
            // Try to run the example several times to see how sometimes the fetches start in order
            // but finish in disorder.
            println!("Fetching `{url}`...");
            // This could also retry fetching in case there's an error while requesting the module.
            let body: Result<_, isahc::Error> = async {
                let mut response = Request::get(&url)
                    .redirect_policy(RedirectPolicy::Limit(5))
                    .body(())?
                    .send_async()
                    .await?;

                Ok(response.text().await?)
            }
            .await;
            println!("Finished fetching `{url}`");

            // Since the async context cannot take the `context` by ref, we have to continue
            // parsing inside a new `NativeJob` that will be enqueued into the promise job queue.
            NativeJob::new(move |context| -> JsResult<JsValue> {
                let body = match body {
                    Ok(body) => body,
                    Err(err) => {
                        // On error we always call `finish_load` to notify the load promise about the
                        // error.
                        finish_load(
                            Err(JsNativeError::typ().with_message(err.to_string()).into()),
                            context,
                        );

                        // Just returns anything to comply with `NativeJob::new`'s signature.
                        return Ok(JsValue::undefined());
                    }
                };

                // Could also add a path if needed.
                let source = Source::from_bytes(body.as_bytes());

                let module = Module::parse(source, None, context);

                // We don't do any error handling, `finish_load` takes care of that for us.
                finish_load(module, context);

                // Also needed to match `NativeJob::new`.
                Ok(JsValue::undefined())
            })
        };

        // Just enqueue the future for now. We'll advance all the enqueued futures inside our custom
        // `JobQueue`.
        context
            .job_queue()
            .enqueue_future_job(Box::pin(fetch), context)
    }
}

fn main() -> JsResult<()> {
    // A simple snippet that imports modules from the web instead of the file system.
    const SRC: &str = r#"
        import YAML from 'https://esm.run/yaml@2.3.4';
        import fromAsync from 'https://esm.run/array-from-async@3.0.0';
        import { Base64 } from 'https://esm.run/js-base64@3.7.6';

        const data = `
            object:
                array: ["hello", "world"]
                key: "value"
        `;

        const object = YAML.parse(data).object;

        let result = await fromAsync([
            Promise.resolve(Base64.encode(object.array[0])),
            Promise.resolve(Base64.encode(object.array[1])),
        ]);

        export default result;
    "#;

    let queue = Rc::new(Queue::new(LocalExecutor::new()));
    let context = &mut Context::builder()
        .job_queue(queue)
        // NEW: sets the context module loader to our custom loader
        .module_loader(Rc::new(HttpModuleLoader))
        .build()?;

    let module = Module::parse(Source::from_bytes(SRC.as_bytes()), None, context)?;

    // Calling `Module::load_link_evaluate` takes care of having to define promise handlers for
    // `Module::load` and `Module::evaluate`.
    let promise = module.load_link_evaluate(context);

    // Important to call `Context::run_jobs`, or else all the futures and promises won't be
    // pushed forward by the job queue.
    context.run_jobs();

    match promise.state() {
        // Our job queue guarantees that all promises and futures are finished after returning
        // from `Context::run_jobs`.
        // Some other job queue designs only execute a "microtick" or a single pass through the
        // pending promises and futures. In that case, you can pass this logic as a promise handler
        // for `promise` instead.
        PromiseState::Pending => panic!("module didn't execute!"),
        // All modules after successfully evaluating return `JsValue::undefined()`.
        PromiseState::Fulfilled(v) => {
            assert_eq!(v, JsValue::undefined())
        }
        PromiseState::Rejected(err) => {
            panic!("{}", err.display());
        }
    }

    let default = module
        .namespace(context)
        .get(js_string!("default"), context)?;

    // `default` should contain the result of our calculations.
    let default = default
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("default export was not an object"))?;

    assert_eq!(
        default
            .get(0, context)?
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("array element was not a string"))?,
        &js_string!("aGVsbG8=")
    );
    assert_eq!(
        default
            .get(1, context)?
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("array element was not a string"))?,
        &js_string!("d29ybGQ=")
    );

    Ok(())
}

// Taken from the `futures.rs` example.
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
