//! Boa's implementation of the Message API (mainly `postMessage` and
//! supporting functions).
//!
//! More information:
//! - [MDN documentation][<https://developer.mozilla.org/en-US/docs/Web/API/Window/postMessage>]

use crate::store::JsValueStore;
use boa_engine::realm::Realm;
use boa_engine::value::TryFromJs;
use boa_engine::{
    Context, Finalize, JsData, JsResult, JsString, JsValue, NativeObject, Trace, boa_module,
    js_error,
};
use std::rc::Rc;

#[cfg(test)]
mod tests;

pub mod senders;

/// A sender of a message. When registering the [`postMessage`][post_message]
/// API, one must also register an implementation of this trait in the context.
///
/// This trait receives a [`JsValueStore`] value and ensures it gets
/// delivered to the correct location, be it a Rust API, or another context
/// altogether.
pub trait MessageSender: NativeObject {
    /// Send a message to the necessary context.
    ///
    /// The second argument is the `targetOrigin` argument passed to
    /// `postMessage`. No default value exists, and so it must be
    /// resolved by the application. It can be used to filter messages.
    ///
    /// # Errors
    /// Any errors in transit should be returned here. An error should be
    /// returned before the full transfer is done and tell the application
    /// that it should retry if possible.
    ///
    /// If an error happens after the transfer is done but before the
    /// message is handled, this must be handled by the host application,
    /// e.g., using a `messageerror` event.
    fn send(&self, message: JsValueStore, target_origin: Option<JsString>) -> JsResult<()>;

    /// Stop the sender from receiving messages. The default implementation
    /// does nothing, but a successful implementation should try to stop
    /// any queued jobs or anything that might block the context from
    /// terminating.
    ///
    /// # Errors
    /// Any errors in transit should be returned here.
    fn stop(&self) -> JsResult<()> {
        Ok(())
    }
}

/// A reference counted pointer to a `MessageSender` implementation. This is so we
/// can add this to the context, but we need to be able to an `Rc<>` structure to
/// make API calls.
#[derive(Debug, Trace, Finalize, JsData)]
struct MessageSenderRc<T: MessageSender>(#[unsafe_ignore_trace] Rc<T>);

impl<T: MessageSender> Clone for MessageSenderRc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Options that can be passed as the second argument to `postMessage`.
#[derive(Debug, Default, TryFromJs, Trace, Finalize)]
pub struct PostMessageOptions {
    transfer: Option<Vec<JsValue>>,
    target_origin: Option<JsString>,
}

/// Get a `MessageSender` instance from the context.
fn get_sender<T: MessageSender>(context: &mut Context) -> JsResult<Rc<T>> {
    // Try fetching from the context first, then the current realm. Else fail.
    if let Some(sender) = context
        .get_data::<MessageSenderRc<T>>()
        .cloned()
        .or_else(|| {
            context
                .realm()
                .host_defined()
                .get::<MessageSenderRc<T>>()
                .cloned()
        })
    {
        Ok(sender.0.clone())
    } else {
        Err(
            js_error!(Error: "Implementation of postMessage requires a sender registered in the context"),
        )
    }
}

/// JavaScript module containing the `postMessage` function and supporting types.
#[boa_module]
pub mod js_module {
    use crate::message::{MessageSender, PostMessageOptions, get_sender};
    use crate::store::JsValueStore;
    use boa_engine::value::TryFromJs;
    use boa_engine::{Context, JsValue};
    use boa_engine::{JsResult, js_error};

    /// The `postMessage` function. See [the mdn documentation][mdn].
    ///
    /// # Errors
    /// Either an error from serializing the [`JsValue`] into a store, or the
    /// sender returned an error.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Window/postMessage
    #[allow(clippy::needless_pass_by_value)]
    pub fn post_message<T: MessageSender>(
        message: JsValue,
        target_origin_or_options: Option<JsValue>,
        transfer: Option<Vec<JsValue>>,
        context: &mut Context,
    ) -> JsResult<()> {
        // Build the options based on arguments.
        let (target_origin, transfer) = if let Some(target_origin_or_options) =
            target_origin_or_options
        {
            if let Ok(options) = PostMessageOptions::try_from_js(&target_origin_or_options, context)
            {
                (options.target_origin.clone(), options.transfer.clone())
            } else if let Some(target_origin) = target_origin_or_options.as_string() {
                (Some(target_origin), transfer)
            } else {
                return Err(js_error!(TypeError: "targetOrigin must be a string or option object"));
            }
        } else {
            (None, None)
        };

        let message = JsValueStore::try_from_js(&message, context, transfer.unwrap_or_default())?;
        let sender = get_sender::<T>(context)?;
        sender.send(message, target_origin)
    }
}

#[doc(inline)]
pub use js_module::post_message;

/// Register the `postMessage` function in the realm or context.
///
/// # Errors
/// If any of the classes fail to register, an error is returned.
pub fn register<S: MessageSender>(
    sender: S,
    realm: Option<Realm>,
    context: &mut Context,
) -> JsResult<()> {
    if let Some(ref realm) = realm {
        realm
            .host_defined_mut()
            .insert(MessageSenderRc(Rc::new(sender)));
    } else {
        context.insert_data(MessageSenderRc(Rc::new(sender)));
    }
    js_module::boa_register::<S>(realm, context)?;

    Ok(())
}
