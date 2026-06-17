//! A module that declares any functions for dealing with intervals or
//! timeouts.

use boa_engine::interop::JsRest;
use boa_engine::job::{CancellationToken, IntervalJob, NativeJobFn};
use boa_engine::job::{NativeJob, TimeoutJob};
use boa_engine::object::builtins::JsFunction;

use boa_engine::{Context, IntoJsFunctionCopied, JsResult, JsValue, js_error, js_string};
use std::collections::HashMap;
use std::num::NonZeroU32;

#[cfg(test)]
mod tests;

/// The internal state of the interval module. The value is whether the interval
/// function is still active.
struct IntervalInnerState {
    active_map: HashMap<NonZeroU32, CancellationToken>,
    id: NonZeroU32,
}

impl Default for IntervalInnerState {
    fn default() -> Self {
        Self {
            active_map: HashMap::new(),
            id: NonZeroU32::MIN,
        }
    }
}

impl IntervalInnerState {
    /// Get the interval handler map from the context, or add it to the context if not
    /// present.
    fn from_context(context: &mut Context) -> &mut Self {
        if !context.has_data::<Self>() {
            context.insert_data(Self::default());
        }

        context
            .host_defined_mut()
            .get_mut::<Self>()
            .expect("Should have inserted.")
    }

    /// Create an interval ID.
    fn next_id(&mut self) -> JsResult<NonZeroU32> {
        self.active_map.retain(|_, v| !v.revoked());
        let id = self.id;
        self.id = id
            .checked_add(1)
            .ok_or_else(|| js_error!(Error: "Interval ID overflow"))?;
        Ok(id)
    }

    /// Delete an interval ID from the active map.
    fn clear_interval(&mut self, id: u32) -> Option<CancellationToken> {
        self.active_map.retain(|_, v| !v.revoked());
        let id = NonZeroU32::new(id)?;
        self.active_map.remove(&id)
    }

    /// Drains and returns every active timer/interval token.
    fn drain_tokens(&mut self) -> Vec<CancellationToken> {
        std::mem::take(&mut self.active_map).into_values().collect()
    }
}

/// Set a timeout to call the given function after the given delay.
/// The `code` version of this function is not supported at the moment.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope/setTimeout).
///
/// # Errors
/// Any errors when trying to read the context, converting the arguments or
/// enqueuing the job.
pub fn set_timeout(
    function_ref: Option<JsFunction>,
    delay_in_msec: Option<JsValue>,
    rest: JsRest<'_>,
    context: &mut Context,
) -> JsResult<u32> {
    let Some(function_ref) = function_ref else {
        return Ok(0);
    };

    // The spec converts the delay to a WebIDL `long`, which maps to `i32`.
    // Negative values are clamped to 0.
    let delay_i32 = delay_in_msec.unwrap_or_default().to_i32(context)?;
    let delay = u64::from(u32::try_from(delay_i32).unwrap_or(0));

    let state = IntervalInnerState::from_context(context);
    let id = state.next_id()?;

    // Get ownership of rest arguments.
    let rest = rest.to_vec();

    let job = TimeoutJob::new(
        NativeJob::new(move |context| {
            let result = function_ref.call(&JsValue::undefined(), &rest, context);
            let state = IntervalInnerState::from_context(context);
            state.active_map.remove(&id);
            result
        }),
        delay,
    );
    let token = job.cancellation_token().clone();

    token.push_callback(move |context| {
        let state = IntervalInnerState::from_context(context);
        state.active_map.remove(&id);
    });

    state.active_map.insert(id, token);

    context.enqueue_job(job.into());

    Ok(id.get())
}

/// Call a given function on an interval with the given delay.
/// The `code` version of this function is not supported at the moment.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope/setInterval).
///
/// # Errors
/// Any errors when trying to read the context, converting the arguments or
/// enqueuing the job.
pub fn set_interval(
    function_ref: Option<JsFunction>,
    delay_in_msec: Option<JsValue>,
    rest: JsRest<'_>,
    context: &mut Context,
) -> JsResult<u32> {
    let Some(function_ref) = function_ref else {
        return Ok(0);
    };

    // The spec converts the delay to a WebIDL `long`, which maps to `i32`.
    // Negative values are clamped to 0.
    let delay_i32 = delay_in_msec.unwrap_or_default().to_i32(context)?;
    let delay = u64::from(u32::try_from(delay_i32).unwrap_or(0));

    let state = IntervalInnerState::from_context(context);
    let id = state.next_id()?;

    // Get ownership of rest arguments.
    let rest = rest.to_vec();

    let job = IntervalJob::new(
        NativeJobFn::new(move |context| function_ref.call(&JsValue::undefined(), &rest, context)),
        delay,
    );
    let token = job.cancellation_token().clone();

    token.push_callback(move |context| {
        let state = IntervalInnerState::from_context(context);
        state.active_map.remove(&id);
    });

    state.active_map.insert(id, token);

    context.enqueue_job(job.into());

    Ok(id.get())
}

/// Clears a timeout or interval currently running.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/Window/clearTimeout).
///
/// Please note that this is the same exact method as `clearInterval`, as both can be
/// used interchangeably. Invalid, zero, or negative IDs are silently ignored,
/// matching browser behavior.
///
/// # Errors
/// Returns an error if the `id` argument cannot be converted to an `i32`.
pub fn clear_timeout(id: Option<JsValue>, context: &mut Context) -> JsResult<JsValue> {
    let id = id.unwrap_or_default().to_i32(context)?;
    if id > 0 {
        let handler_map = IntervalInnerState::from_context(context);
        if let Some(token) = handler_map.clear_interval(id.cast_unsigned()) {
            token.cancel(context);
        }
    }
    Ok(JsValue::undefined())
}

/// Cancels every currently active timer and interval registered through
/// this module's `setTimeout` / `setInterval`.
///
/// Intended for test teardown and graceful shutdown: after this call, any
/// pending [`boa_engine::job::TimeoutJob`] / [`boa_engine::job::IntervalJob`]
/// in the executor's queue will be skipped on its next tick, allowing
/// `run_jobs_async` to exit naturally without disturbing unrelated
/// `PromiseJob`, `NativeAsyncJob`, or `GenericJob` work.
///
/// Note: timers scheduled *after* this call returns are not affected.
pub fn clear_all(context: &mut Context) {
    let state = IntervalInnerState::from_context(context);
    // Drain first to avoid re-entrant mutation: each token's cancel callback
    // tries to remove its own id from `active_map`.
    let tokens = state.drain_tokens();
    for token in tokens {
        token.cancel(context);
    }
}

/// Register the interval module into the given context.
///
/// # Errors
/// Any error returned by the context when registering the global functions.
pub fn register(context: &mut Context) -> JsResult<()> {
    register_functions(context)
}

/// Register the interval module without any clock. This still needs the proper
/// typing for the clock, even if it is not registered to the context.
///
/// # Errors
/// Any error returned by the context when registering the global functions.
pub fn register_functions(context: &mut Context) -> JsResult<()> {
    let set_timeout_ = set_timeout.into_js_function_copied(context);
    context.register_global_callable(js_string!("setTimeout"), 1, set_timeout_)?;

    let set_interval_ = set_interval.into_js_function_copied(context);
    context.register_global_callable(js_string!("setInterval"), 1, set_interval_)?;

    // These two methods are identical, just under different names in JavaScript.
    let clear_timeout_ = clear_timeout.into_js_function_copied(context);
    context.register_global_callable(js_string!("clearTimeout"), 1, clear_timeout_.clone())?;
    context.register_global_callable(js_string!("clearInterval"), 1, clear_timeout_)?;

    Ok(())
}
