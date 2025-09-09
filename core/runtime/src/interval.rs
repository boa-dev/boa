//! A module that declares any functions for dealing with intervals or
//! timeouts.

use boa_engine::interop::JsRest;
use boa_engine::job::{NativeJob, TimeoutJob};
use boa_engine::object::builtins::JsFunction;
use boa_engine::value::{IntegerOrInfinity, Nullable};
use boa_engine::{
    Context, Finalize, IntoJsFunctionCopied, JsData, JsResult, JsValue, Trace, js_error, js_string,
};
use boa_gc::{Gc, GcRefCell};
use std::collections::HashSet;

#[cfg(test)]
mod tests;

/// The internal state of the interval module. The value is whether the interval
/// function is still active.
#[derive(Default, Trace, Finalize, JsData)]
struct IntervalInnerState {
    active_map: HashSet<u32>,
    next_id: u32,
}

impl IntervalInnerState {
    /// Get the interval handler map from the context, or add it to the context if not
    /// present.
    fn from_context(context: &mut Context) -> Gc<GcRefCell<Self>> {
        if !context.has_data::<Gc<GcRefCell<IntervalInnerState>>>() {
            context.insert_data(Gc::new(GcRefCell::new(Self::default())));
        }

        context
            .get_data::<Gc<GcRefCell<Self>>>()
            .expect("Should have inserted.")
            .clone()
    }

    /// Get whether an interval is still active.
    #[inline]
    fn is_interval_valid(&self, id: u32) -> bool {
        self.active_map.contains(&id)
    }

    /// Create an interval ID, insert it in the active map and return it.
    fn new_interval(&mut self) -> JsResult<u32> {
        if self.next_id == u32::MAX {
            return Err(js_error!(Error: "Interval ID overflow"));
        }
        self.next_id += 1;
        self.active_map.insert(self.next_id);
        Ok(self.next_id)
    }

    /// Delete an interval ID from the active map.
    fn clear_interval(&mut self, id: u32) {
        self.active_map.remove(&id);
    }
}

/// Inner handler function for handling intervals and timeout.
#[allow(clippy::too_many_arguments)]
fn handle(
    handler_map: Gc<GcRefCell<IntervalInnerState>>,
    id: u32,
    function_ref: JsFunction,
    args: Vec<JsValue>,
    reschedule: Option<u64>,
    context: &mut Context,
) -> JsResult<JsValue> {
    // Check if it's still valid.
    if !handler_map.borrow().is_interval_valid(id) {
        return Ok(JsValue::undefined());
    }

    // Call the handler function.
    // The spec says we should still reschedule an interval even if the function
    // throws an error.
    let result = function_ref.call(&JsValue::undefined(), &args, context);
    if let Some(delay) = reschedule {
        if handler_map.borrow().is_interval_valid(id) {
            let job = TimeoutJob::new(
                NativeJob::new(move |context| {
                    handle(handler_map, id, function_ref, args, reschedule, context)
                }),
                delay,
            );
            context.enqueue_job(job.into());
        }
        return result;
    }

    handler_map.borrow_mut().clear_interval(id);
    result
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

    let handler_map = IntervalInnerState::from_context(context);
    let id = handler_map.borrow_mut().new_interval()?;

    // Spec says if delay is not a number, it should be equal to 0.
    let delay = delay_in_msec
        .unwrap_or_default()
        .to_integer_or_infinity(context)
        .unwrap_or(IntegerOrInfinity::Integer(0));
    // The spec converts the delay to a 32-bit signed integer.
    let delay = u64::from(delay.clamp_finite(0, u32::MAX));

    // Get ownership of rest arguments.
    let rest = rest.to_vec();

    let job = TimeoutJob::new(
        NativeJob::new(move |context| handle(handler_map, id, function_ref, rest, None, context)),
        delay,
    );
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

    let handler_map = IntervalInnerState::from_context(context);
    let id = handler_map.borrow_mut().new_interval()?;

    // Spec says if delay is not a number, it should be equal to 0.
    let delay = delay_in_msec
        .unwrap_or_default()
        .to_integer_or_infinity(context)
        .unwrap_or(IntegerOrInfinity::Integer(0));
    let delay = u64::from(delay.clamp_finite(0, u32::MAX));

    // Get ownership of rest arguments.
    let rest = rest.to_vec();

    let job = TimeoutJob::new(
        NativeJob::new(move |context| {
            handle(handler_map, id, function_ref, rest, Some(delay), context)
        }),
        delay,
    );
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
    handler_map.borrow_mut().clear_interval(id);
}

/// Register the interval module into the given context.
///
/// # Errors
/// Any error returned by the context when registering the global functions.
pub fn register(context: &mut Context) -> JsResult<()> {
    register_functions(context)
}

/// Register the interval module without any clock. This still needs the proper
/// typing for the clock, even if it is not registerd to the context.
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
