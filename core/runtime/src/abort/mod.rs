//! `AbortController` and `AbortSignal` Web API implementations.

use boa_engine::class::Class;
use boa_engine::job::GenericJob;
use boa_engine::object::builtins::JsFunction;
use boa_engine::realm::Realm;
use boa_engine::{
    Context, Finalize, JsData, JsError, JsNativeError, JsObject, JsResult, JsValue, Trace,
    boa_class, boa_module, js_error, js_string,
};
use boa_gc::GcRefCell;
use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(test)]
mod tests;

/// Cancellation token for cooperative abort.
#[derive(Debug, Clone)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    /// Cancel the token.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Returns `true` if cancelled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

fn make_abort_error(context: &mut Context) -> JsValue {
    let obj = JsNativeError::error()
        .with_message("signal is aborted without reason")
        .into_opaque(context);
    obj.set(js_string!("name"), js_string!("AbortError"), false, context)
        .ok();
    obj.into()
}

/// The JavaScript `AbortSignal` class.
#[derive(Debug, Clone, JsData, Trace, Finalize)]
pub struct JsAbortSignal {
    #[unsafe_ignore_trace]
    aborted: Cell<bool>,
    reason: GcRefCell<Option<JsValue>>,
    listeners: GcRefCell<Vec<JsFunction>>,
    #[unsafe_ignore_trace]
    cancel_token: CancellationToken,
}

impl Default for JsAbortSignal {
    fn default() -> Self {
        Self {
            aborted: Cell::new(false),
            reason: GcRefCell::default(),
            listeners: GcRefCell::default(),
            cancel_token: CancellationToken::new(),
        }
    }
}

impl JsAbortSignal {
    /// # Errors
    ///
    /// Returns an error if the signal has already been aborted.
    pub fn signal_abort(&self, reason: JsValue, context: &mut Context) -> JsResult<()> {
        if self.aborted.get() {
            return Ok(());
        }
        self.aborted.set(true);
        *self.reason.borrow_mut() = Some(reason);

        let listeners: Vec<JsFunction> = self.listeners.borrow_mut().drain(..).collect();

        let realm = context.realm().clone();
        for listener in listeners {
            context.enqueue_job(
                GenericJob::new(
                    move |context| {
                        listener.call(&JsValue::undefined(), &[], context)?;
                        Ok(JsValue::undefined())
                    },
                    realm.clone(),
                )
                .into(),
            );
        }

        self.cancel_token.cancel();

        Ok(())
    }

    /// Returns `true` if this signal has been aborted.
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        self.aborted.get()
    }

    /// Returns the abort reason.
    pub fn abort_reason(&self, context: &mut Context) -> JsValue {
        if !self.aborted.get() {
            return JsValue::undefined();
        }
        self.reason
            .borrow()
            .clone()
            .unwrap_or_else(|| make_abort_error(context))
    }

    /// Returns the cancellation token.
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }
}

#[boa_class(rename = "AbortSignal")]
#[boa(rename_all = "camelCase")]
impl JsAbortSignal {
    #[boa(constructor)]
    fn constructor() -> JsResult<Self> {
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }

    #[boa(getter)]
    fn aborted(&self) -> bool {
        self.aborted.get()
    }

    #[boa(getter)]
    fn reason(&self, context: &mut Context) -> JsValue {
        self.abort_reason(context)
    }

    fn throw_if_aborted(&self, context: &mut Context) -> JsResult<()> {
        if self.aborted.get() {
            Err(JsError::from_opaque(self.abort_reason(context)))
        } else {
            Ok(())
        }
    }

    fn add_event_listener(
        &self,
        event_type: boa_engine::JsString,
        callback: JsFunction,
        context: &mut Context,
    ) -> JsResult<()> {
        if event_type.to_std_string_escaped() != "abort" {
            return Err(js_error!(TypeError: "AbortSignal only supports the 'abort' event type"));
        }
        if self.aborted.get() {
            callback.call(&JsValue::undefined(), &[], context)?;
        } else {
            self.listeners.borrow_mut().push(callback);
        }
        Ok(())
    }

    fn remove_event_listener(&self, event_type: boa_engine::JsString, callback: JsFunction) {
        if event_type.to_std_string_escaped() != "abort" {
            return;
        }
        self.listeners
            .borrow_mut()
            .retain(|f| !JsObject::equals(f, &callback));
    }
}

/// The JavaScript `AbortController` class.
#[derive(Debug, Clone, JsData, Trace, Finalize)]
pub struct JsAbortController {
    signal: JsObject,
}

#[boa_class(rename = "AbortController")]
#[boa(rename_all = "camelCase")]
impl JsAbortController {
    #[boa(constructor)]
    fn constructor(context: &mut Context) -> JsResult<Self> {
        let signal_obj = Class::from_data(JsAbortSignal::default(), context)?;
        Ok(Self { signal: signal_obj })
    }

    #[boa(getter)]
    fn signal(&self) -> JsObject {
        self.signal.clone()
    }

    fn abort(&self, reason: Option<JsValue>, context: &mut Context) -> JsResult<()> {
        let abort_reason = reason.unwrap_or_else(|| make_abort_error(context));

        let Some(signal) = self.signal.downcast_ref::<JsAbortSignal>() else {
            return Err(js_error!(TypeError: "AbortController: invalid signal object"));
        };
        signal.signal_abort(abort_reason, context)
    }
}

/// `AbortController` and `AbortSignal` module.
#[boa_module]
pub mod js_module {
    type JsAbortController = super::JsAbortController;
    type JsAbortSignal = super::JsAbortSignal;
}

/// # Errors
/// Returns an error if registration fails.
pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
    js_module::boa_register(realm, context)
}
