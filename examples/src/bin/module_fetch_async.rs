use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use boa_engine::{
    Context, JsNativeError, JsResult, JsString, JsValue, Module,
    builtins::promise::PromiseState,
    job::{Job, JobExecutor, NativeAsyncJob, PromiseJob},
    js_string,
    module::ModuleLoader,
};
use boa_parser::Source;
use futures_concurrency::future::FutureGroup;
use isahc::{
    AsyncReadResponseExt, Request, RequestExt,
    config::{Configurable, RedirectPolicy},
};
use smol::{future, stream::StreamExt};

#[derive(Debug, Default)]
struct HttpModuleLoader;

impl ModuleLoader for HttpModuleLoader {
    async fn load_imported_module(
        self: Rc<Self>,
        _referrer: boa_engine::module::Referrer,
        specifier: JsString,
        context: &RefCell<&mut Context>,
    ) -> JsResult<Module> {
        let url = specifier.to_std_string_escaped();

        // Adding some prints to show the non-deterministic nature of the async fetches.
        // Try to run the example several times to see how sometimes the fetches start in order
        // but finish in disorder.
        println!("Fetching `{url}`...");

        // This could also retry fetching in case there's an error while requesting the module.
        let response = async {
            let request = Request::get(&url)
                .redirect_policy(RedirectPolicy::Limit(5))
                .body(())?;
            let response = request.send_async().await?.text().await?;
            Ok(response)
        }
        .await
        .map_err(|err: isahc::Error| JsNativeError::typ().with_message(err.to_string()))?;

        println!("Finished fetching `{url}`");

        // Could also add a path if needed.
        let source = Source::from_bytes(&response);

        Module::parse(source, None, &mut context.borrow_mut())
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

    let context = &mut Context::builder()
        .job_executor(Rc::new(Queue::new()))
        // NEW: sets the context module loader to our custom loader
        .module_loader(Rc::new(HttpModuleLoader))
        .build()?;

    let module = Module::parse(Source::from_bytes(SRC.as_bytes()), None, context)?;

    // Calling `Module::load_link_evaluate` takes care of having to define promise handlers for
    // `Module::load` and `Module::evaluate`.
    let promise = module.load_link_evaluate(context);

    // Important to call `Context::run_jobs`, or else all the futures and promises won't be
    // pushed forward by the job queue.
    context.run_jobs()?;

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
        js_string!("aGVsbG8=")
    );
    assert_eq!(
        default
            .get(1, context)?
            .as_string()
            .ok_or_else(|| JsNativeError::typ().with_message("array element was not a string"))?,
        js_string!("d29ybGQ=")
    );

    Ok(())
}

// Taken from the `smol_event_loop.rs` example.
/// An event queue using smol to drive futures to completion.
struct Queue {
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    promise_jobs: RefCell<VecDeque<PromiseJob>>,
}

impl Queue {
    fn new() -> Self {
        Self {
            async_jobs: RefCell::default(),
            promise_jobs: RefCell::default(),
        }
    }

    fn drain_jobs(&self, context: &mut Context) {
        let jobs = std::mem::take(&mut *self.promise_jobs.borrow_mut());
        for job in jobs {
            if let Err(e) = job.call(context) {
                eprintln!("Uncaught {e}");
            }
        }
    }
}

impl JobExecutor for Queue {
    fn enqueue_job(self: Rc<Self>, job: Job, _context: &mut Context) {
        match job {
            Job::PromiseJob(job) => self.promise_jobs.borrow_mut().push_back(job),
            Job::AsyncJob(job) => self.async_jobs.borrow_mut().push_back(job),
            _ => panic!("unsupported job type"),
        }
    }

    // While the sync flavor of `run_jobs` will block the current thread until all the jobs have finished...
    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()> {
        smol::block_on(smol::LocalExecutor::new().run(self.run_jobs_async(&RefCell::new(context))))
    }

    // ...the async flavor won't, which allows concurrent execution with external async tasks.
    async fn run_jobs_async(self: Rc<Self>, context: &RefCell<&mut Context>) -> JsResult<()> {
        // Early return in case there were no jobs scheduled.
        if self.promise_jobs.borrow().is_empty() && self.async_jobs.borrow().is_empty() {
            return Ok(());
        }
        let mut group = FutureGroup::new();
        loop {
            for job in std::mem::take(&mut *self.async_jobs.borrow_mut()) {
                group.insert(job.call(context));
            }

            if group.is_empty() && self.promise_jobs.borrow().is_empty() {
                // Both queues are empty. We can exit.
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
            future::yield_now().await
        }
    }
}
