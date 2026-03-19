//! Boa's implementation of the Web Worker API.
//!
//! The `Worker` class represents a Web Worker.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [WHATWG `Worker` specification][spec]
//!
//! [spec]: https://html.spec.whatwg.org/multipage/workers.html
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Worker

#[cfg(test)]
pub(crate) mod tests;

use boa_engine::class::Class;
use boa_engine::realm::Realm;
use boa_engine::value::Convert;
use boa_engine::{
    boa_class, boa_module, js_error, Context, Finalize, JsData, JsResult, JsValue, Source, Trace,
};
use std::sync::mpsc::{channel, Sender};
use std::thread;

/// The `Worker` class represents a Web Worker
#[derive(Debug, Clone, JsData, Trace, Finalize)]
#[boa_gc(unsafe_no_drop)]
pub struct Worker {
    #[unsafe_ignore_trace]
    sender: Sender<String>,
}

impl Worker {
    /// Register the `Worker` class into the realm. Pass `None` for the realm to
    /// register globally.
    pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
        js_module::boa_register(realm, context)
    }
}

#[boa_class(rename = "Worker")]
#[boa(rename_all = "camelCase")]
impl Worker {
    /// Create a new `Worker` object.
    #[boa(constructor)]
    pub fn new(url: Convert<String>, context: &mut Context) -> JsResult<Self> {
        let (tx, rx) = channel::<String>();

        let script = std::fs::read_to_string(&url.0)
            .map_err(|e| js_error!(TypeError: "Failed to read worker script '{}': {}", url.0, e))?;

        thread::spawn(move || {
            let mut worker_context = Context::default();
            // We should ideally register some runtime APIs here, like console.
            // But leaving it basic for now to avoid circular dependencies in `boa_runtime`.

            // Evaluate the initial script
            if let Err(e) = worker_context.eval(Source::from_bytes(&script)) {
                eprintln!("Worker error: {}", e);
                return;
            }

            // Simple message loop
            while let Ok(msg) = rx.recv() {
                let global = worker_context.global_object().clone();
                if let Ok(onmessage) =
                    global.get(boa_engine::js_string!("onmessage"), &mut worker_context)
                {
                    if let Some(func) = onmessage.as_callable() {
                        let event =
                            boa_engine::object::JsObject::default(&worker_context.intrinsics());
                        let _ = event.set(
                            boa_engine::js_string!("data"),
                            JsValue::from(boa_engine::js_string!(msg)),
                            false,
                            &mut worker_context,
                        );
                        let _ = func.call(&global.into(), &[event.into()], &mut worker_context);
                        let _ = worker_context.run_jobs();
                    }
                }
            }
        });

        Ok(Self { sender: tx })
    }

    /// `Worker.prototype.postMessage(message)`
    pub fn post_message(
        &self,
        message: Convert<String>,
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        self.sender
            .send(message.0)
            .map_err(|e| js_error!(Error: "Failed to send message to worker: {}", e))?;
        Ok(JsValue::undefined())
    }

    /// `Worker.prototype.terminate()`
    pub fn terminate(&self, _context: &mut Context) -> JsResult<JsValue> {
        // Since we are using an mpsc channel, dropping the sender isn't enough
        // unless we replace the sender with an Option.
        // For standard Web Workers, terminate stops it immediately. 
        // For now, this is a no-op that relies on the main process exiting.
        Ok(JsValue::undefined())
    }
}

/// JavaScript module containing the Worker class.
#[boa_module]
pub mod js_module {
    type Worker = super::Worker;
}
