//! `AbortController` and `AbortSignal` Web API implementations.

use boa_engine::class::Class;
use boa_engine::object::builtins::JsFunction;
use boa_engine::realm::Realm;
use boa_engine::{
    Context, Finalize, JsData, JsError, JsNativeError, JsObject, JsResult, JsValue, Trace,
    boa_class, boa_module, js_error, js_string,
};
use boa_gc::GcRefCell;
use std::cell::Cell;

#[cfg(test)]
mod tests;

fn make_abort_error(context: &mut Context) -> JsValue {
    let obj = JsNativeError::error()
        .with_message("signal is aborted without reason")
        .into_opaque(context);
    obj.set(js_string!("name"), js_string!("AbortError"), false, context)
        .ok();
    obj.into()
}

/// The JavaScript `AbortSignal` class.
#[derive(Debug, Clone, Default, JsData, Trace, Finalize)]
pub struct JsAbortSignal {
    #[unsafe_ignore_trace]
    aborted: Cell<bool>,
    reason: GcRefCell<Option<JsValue>>,
    listeners: GcRefCell<Vec<JsFunction>>,
}

impl JsAbortSignal {
    fn signal_abort(&self, reason: JsValue, context: &mut Context) -> JsResult<()> {
        if self.aborted.get() {
            return Ok(());
        }
        self.aborted.set(true);
        *self.reason.borrow_mut() = Some(reason);

        let listeners: Vec<JsFunction> = self.listeners.borrow().clone();
        for listener in &listeners {
            listener.call(&JsValue::undefined(), &[], context)?;
        }

        Ok(())
    }

    /// Returns `true` if this signal has been aborted.
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        self.aborted.get()
    }

    /// Returns the abort reason.
    pub fn abort_reason(&self) -> JsValue {
        self.reason
            .borrow()
            .clone()
            .unwrap_or_else(|| js_string!("AbortError").into())
    }
}

#[boa_class(rename = "AbortSignal")]
#[boa(rename_all = "camelCase")]
impl JsAbortSignal {
    #[boa(constructor)]
    fn constructor() -> Self {
        Self::default()
    }

    #[boa(getter)]
    fn aborted(&self) -> bool {
        self.aborted.get()
    }

    #[boa(getter)]
    fn reason(&self) -> JsValue {
        self.abort_reason()
    }

    fn throw_if_aborted(&self) -> JsResult<()> {
        if self.aborted.get() {
            Err(JsError::from_opaque(self.abort_reason()))
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
