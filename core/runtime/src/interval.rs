//! A module that declares any functions for dealing with intervals or
//! timeouts.

use boa_engine::interop::JsRest;
use boa_engine::job::{CancellationToken, IntervalJob, NativeJobFn};
use boa_engine::job::{NativeJob, TimeoutJob};
use boa_engine::object::builtins::JsFunction;
use boa_engine::value::{IntegerOrInfinity, Nullable};
use boa_engine::{Context, IntoJsFunctionCopied, JsResult, JsValue, js_error, js_string};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

/// The internal state of the interval module. The value is whether the interval
/// function is still active.
#[derive(Default)]
struct IntervalInnerState {
    active_map: HashMap<u32, CancellationToken>,
    id: u32,
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
    fn next_id(&mut self) -> JsResult<u32> {
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
        self.active_map.remove(&id)
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

    // Spec says if delay is not a number, it should be equal to 0.
    let delay = delay_in_msec
        .unwrap_or_default()
        .to_integer_or_infinity(context)
        .unwrap_or(IntegerOrInfinity::Integer(0));
    // The spec converts the delay to a 32-bit signed integer.
    let delay = u64::from(delay.clamp_finite(0, u32::MAX));

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

    Ok(id)
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

    // Spec says if delay is not a number, it should be equal to 0.
    let delay = delay_in_msec
        .unwrap_or_default()
        .to_integer_or_infinity(context)
        .unwrap_or(IntegerOrInfinity::Integer(0));
    let delay = u64::from(delay.clamp_finite(0, u32::MAX));

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

    Ok(id)
}

/// Clears a timeout or interval currently running.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/Window/clearTimeout).
///
/// Please note that this is the same exact method as `clearInterval`, as both can be
/// used interchangeably.
pub fn clear_timeout(id: Nullable<Option<u32>>, context: &mut Context) {
    let Some(id) = id.flatten() else {
        return;
    };
    let handler_map = IntervalInnerState::from_context(context);
    if let Some(token) = handler_map.clear_interval(id) {
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
