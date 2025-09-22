//! A collection of [`MessageSender`]s basic implementations.

use crate::message::MessageSender;
use crate::store::JsValueStore;
use boa_engine::job::NativeAsyncJob;
use boa_engine::object::builtins::JsFunction;
use boa_engine::value::TryIntoJs;
use boa_engine::{
    Context, Finalize, JsData, JsError, JsResult, JsString, JsValue, Trace, js_string,
};
use futures::StreamExt;
use futures::channel::mpsc::UnboundedSender;

/// A [`MessageSender`] that reads the `onMessageQueue` property of the global
/// object and calls it if it is a function. Note that this does not support
/// event listeners, only checks the (made-up) `onMessageQueue` property. It
/// also does not check the `targetOrigin` and accepts all messages.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct OnMessageQueueSender {
    #[unsafe_ignore_trace]
    sender: UnboundedSender<JsValueStore>,
}

impl MessageSender for OnMessageQueueSender {
    fn send(&self, message: JsValueStore, _target_origin: Option<JsString>) -> JsResult<()> {
        self.sender
            .unbounded_send(message)
            .map_err(JsError::from_rust)?;
        Ok(())
    }

    fn stop(&self) -> JsResult<()> {
        self.sender.close_channel();
        Ok(())
    }
}

impl OnMessageQueueSender {
    /// Create a `MessageQueueSender` that sends messages to a message queue
    /// array in the destination context. This type is send/sync and can be
    /// registered in any and many separate `Context` for `postMessage`.
    pub fn create(destination: &mut Context) -> Self {
        let (sender, mut receiver) = futures::channel::mpsc::unbounded::<JsValueStore>();

        destination.enqueue_job(
            NativeAsyncJob::new(async move |ctx| {
                while let Some(store) = receiver.next().await {
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

                Ok(JsValue::undefined())
            })
            .into(),
        );

        Self { sender }
    }
}
