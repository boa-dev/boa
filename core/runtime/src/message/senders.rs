//! A collection of [`MessageSender`]s basic implementations.

use crate::message::MessageSender;
use crate::store::JsValueStore;
use boa_engine::job::NativeAsyncJob;
use boa_engine::object::builtins::JsFunction;
use boa_engine::value::TryIntoJs;
use boa_engine::{
    Context, Finalize, JsData, JsError, JsResult, JsString, JsValue, Trace, js_string,
};
use futures::channel::mpsc::UnboundedSender;
use futures::channel::oneshot;
use futures::{FutureExt, StreamExt};
use std::sync::{Arc, Mutex};

/// A [`MessageSender`] that reads the `onMessageQueue` property of the global
/// object and calls it if it is a function. Note that this does not support
/// event listeners, only checks the (made-up) `onMessageQueue` property. It
/// also does not check the `targetOrigin` and accepts all messages.
///
/// Call [`MessageSender::stop`] (or drop the last clone of this sender) before
/// driving the destination context's event loop to completion with
/// `context.run_jobs()`, so the internal async job can terminate and the
/// executor is not blocked indefinitely.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct OnMessageQueueSender {
    #[unsafe_ignore_trace]
    sender: UnboundedSender<JsValueStore>,
    /// Shared one-shot used to signal the async receiver job to exit.
    /// Wrapped in `Arc<Mutex<Option<...>>>` so that any clone can trigger it
    /// and `Drop` on the last clone fires it automatically.
    #[unsafe_ignore_trace]
    shutdown: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl MessageSender for OnMessageQueueSender {
    fn send(&self, message: JsValueStore, _target_origin: Option<JsString>) -> JsResult<()> {
        self.sender
            .unbounded_send(message)
            .map_err(JsError::from_rust)?;
        Ok(())
    }

    /// Close the message channel and signal the async receiver job to exit,
    /// allowing the destination context's event loop to terminate cleanly.
    fn stop(&self) -> JsResult<()> {
        self.sender.close_channel();
        self.signal_shutdown();
        Ok(())
    }
}

impl OnMessageQueueSender {
    fn signal_shutdown(&self) {
        if let Some(tx) = self
            .shutdown
            .lock()
            .ok()
            .and_then(|mut guard| guard.take())
        {
            // Ignore send errors: the receiver has already gone away, which
            // means the async job already finished — nothing left to do.
            let _ = tx.send(());
        }
    }
}

impl Drop for OnMessageQueueSender {
    fn drop(&mut self) {
        // Only signal when *this* is the last clone.  `Arc::strong_count` is
        // still 1 here because the refcount decrement happens after `drop`
        // returns, so the check is sound.
        if Arc::strong_count(&self.shutdown) == 1 {
            self.signal_shutdown();
        }
    }
}

impl OnMessageQueueSender {
    /// Create a `MessageQueueSender` that sends messages to a message queue
    /// array in the destination context. This type is send/sync and can be
    /// registered in any and many separate `Context` for `postMessage`.
    pub fn create(destination: &mut Context) -> Self {
        let (sender, mut receiver) = futures::channel::mpsc::unbounded::<JsValueStore>();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        destination.enqueue_job(
            NativeAsyncJob::new(async move |ctx| {
                let mut shutdown_rx = shutdown_rx.fuse();

                loop {
                    futures::select! {
                        msg = receiver.next() => match msg {
                            Some(store) => {
                                let context = &mut ctx.borrow_mut();
                                let v = store.try_into_js(context)?;
                                let global = context.global_object();
                                if let Some(x) = global
                                    .get(js_string!("onMessageQueue"), context)?
                                    .as_callable()
                                    .and_then(JsFunction::from_object)
                                {
                                    x.call(&JsValue::undefined(), &[v], context)?;
                                }
                            }
                            // Channel closed — all senders have been dropped or
                            // stop() was called; exit the receiver loop.
                            None => break,
                        },
                        // stop() or Drop on the last sender clone fired the
                        // shutdown signal; exit immediately.
                        _ = shutdown_rx => break,
                    }
                }

                Ok(JsValue::undefined())
            })
            .into(),
        );

        Self {
            sender,
            shutdown: Arc::new(Mutex::new(Some(shutdown_tx))),
        }
    }
}
