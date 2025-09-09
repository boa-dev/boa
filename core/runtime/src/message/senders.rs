//! A collection of [`MessageSender`]s basic implementations.

use crate::message::MessageSender;
use crate::store::JsValueStore;
use boa_engine::job::{NativeJob, PromiseJob, TimeoutJob};
use boa_engine::object::builtins::JsFunction;
use boa_engine::value::TryIntoJs;
use boa_engine::{
    Context, Finalize, JsData, JsError, JsResult, JsString, JsValue, Trace, js_string,
};
use std::sync::mpsc::{Receiver, Sender};

/// A [`MessageSender`] that reads the `onMessageQueue` property of the global
/// object and calls it if it is a function. Note that this does not support
/// event listeners, only checks the (made-up) `onMessageQueue` property. It
/// also does not check the `targetOrigin` and accept all messages.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct OnMessageQueueSender {
    #[unsafe_ignore_trace]
    sender: Sender<JsValueStore>,

    #[unsafe_ignore_trace]
    stop_sender: Sender<()>,
}

impl MessageSender for OnMessageQueueSender {
    fn send(&self, message: JsValueStore, _target_origin: Option<JsString>) -> JsResult<()> {
        self.sender.send(message).map_err(JsError::from_rust)?;
        Ok(())
    }

    fn stop(&self) -> JsResult<()> {
        self.stop_sender.send(()).map_err(JsError::from_rust)
    }
}

impl OnMessageQueueSender {
    /// Create a `MessageQueueSender` that sends messages to a message queue
    /// array in the destination context. This type is send/sync and can be
    /// registered in any and many separate `Context` for `postMessage`.
    pub fn create(destination: &mut Context, interval: u64) -> Self {
        fn job_handler(
            receiver: Receiver<JsValueStore>,
            stop_receiver: Receiver<()>,
            interval: u64,
            context: &mut Context,
        ) -> JsResult<JsValue> {
            if let Ok(recv) = receiver.try_recv() {
                let v = recv.try_into_js(context)?;
                let global = context.global_object();
                if let Some(x) = global
                    .get(js_string!("onMessageQueue"), context)?
                    .as_callable()
                    .and_then(JsFunction::from_object)
                {
                    x.call(&JsValue::undefined(), &[v], context)?;
                }
            }

            // Queue the handle again if we haven't stopped.
            if stop_receiver.try_recv().is_err() {
                context.enqueue_job(
                    TimeoutJob::recurring(
                        NativeJob::new(move |context| {
                            job_handler(receiver, stop_receiver, interval, context)
                        }),
                        interval,
                    )
                    .into(),
                );
            }

            Ok(JsValue::undefined())
        }

        let (sender, receiver) = std::sync::mpsc::channel::<JsValueStore>();
        let (stop_sender, stop_receiver) = std::sync::mpsc::channel::<()>();

        // The first job is hooked up as a promise job to be run immediately.
        destination.enqueue_job(
            PromiseJob::new(move |context| job_handler(receiver, stop_receiver, interval, context))
                .into(),
        );

        Self {
            sender,
            stop_sender,
        }
    }
}
