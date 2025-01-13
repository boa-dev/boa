//! A module that declares any functions for dealing with intervals or
//! timeouts.

use boa_engine::job::NativeJob;
use boa_engine::object::builtins::JsFunction;
use boa_engine::value::IntegerOrInfinity;
use boa_engine::{js_error, js_string, Context, Finalize, JsData, JsResult, JsValue, Trace};
use boa_gc::{Gc, GcRefCell};
use boa_interop::{IntoJsFunctionCopied, JsRest};
use std::collections::HashSet;
use std::time::SystemTime;

#[cfg(test)]
mod tests;

/// A clock implementation. This is useful if you want to mock the time in tests,
/// or also have a custom time implementation outside `[std::time::SystemTime]`.
pub trait Clock: Trace + Sized {
    /// Get the current time as an `Instant`.
    fn now(&self) -> SystemTime;
}

/// A default implementation of `Clock` for the standard library's `SystemTime`.
#[derive(Clone, Debug, Trace, Finalize)]
pub struct StdClock;

impl Clock for StdClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

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
    fn from_context(context: &mut Context) -> JsResult<Gc<GcRefCell<Self>>> {
        if !context.has_data::<Gc<GcRefCell<IntervalInnerState>>>() {
            context.insert_data(Gc::new(GcRefCell::new(Self::default())));
        }

        Ok(context
            .get_data::<Gc<GcRefCell<Self>>>()
            .expect("Should have inserted.")
            .clone())
    }

    /// Get whether an interval is still active.
    #[inline]
    fn is_interval_valid(&self, id: u32) -> bool {
        self.active_map.get(&id).is_some()
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
fn handle<C: Clock>(
    handler_map: Gc<GcRefCell<IntervalInnerState>>,
    id: u32,
    function_ref: JsFunction,
    args: Vec<JsValue>,
    start: SystemTime,
    delay_in_msec: u64,
    reschedule: bool,
    clock: Gc<C>,
    context: &mut Context,
) -> JsResult<JsValue> {
    if !handler_map.borrow().is_interval_valid(id) {
        return Ok(JsValue::undefined());
    }

    // `as_millis()` is u128, but we know it's less than `u64::MAX` (which would be waiting
    // for about 600 million years).
    let elapsed: u64 = clock
        .now()
        .duration_since(start)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX);

    if elapsed < delay_in_msec {
        // Reschedule the timeout.
        context.enqueue_job(NativeJob::new(move |context| {
            handle(
                handler_map,
                id,
                function_ref,
                args,
                start,
                delay_in_msec,
                reschedule,
                clock,
                context,
            )
        }));
        return Ok(JsValue::undefined());
    }

    // The spec says we should still reschedule an interval even if the function
    // throws an error.
    let result = function_ref.call(&JsValue::undefined(), &args, context);

    if reschedule {
        // Reschedule the timeout as an interval.
        let start = clock.now();
        context.enqueue_job(NativeJob::new(move |context| {
            handle(
                handler_map,
                id,
                function_ref,
                args,
                start,
                delay_in_msec,
                reschedule,
                clock,
                context,
            )
        }));
    } else {
        handler_map.borrow_mut().clear_interval(id);
    }

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
pub fn set_timeout<C: Clock + 'static>(
    function_ref: JsFunction,
    delay_in_msec: Option<JsValue>,
    rest: JsRest<'_>,
    context: &mut Context,
) -> JsResult<u32> {
    let handler_map = IntervalInnerState::from_context(context)?;
    let id = handler_map.borrow_mut().new_interval()?;

    let clock = context
        .get_data::<Gc<C>>()
        .ok_or(js_error!("Clock not found"))?
        .clone();

    // Spec says if delay is not a number, it should be equal to 0.
    let delay = delay_in_msec
        .unwrap_or_default()
        .to_integer_or_infinity(context)
        .unwrap_or(IntegerOrInfinity::Integer(0));
    // The spec converts the delay to a 32-bit signed integer.
    let delay = delay.clamp_finite(0, i64::from(i32::MAX)).unsigned_abs();
    let start = clock.now();

    // Get ownership of rest arguments.
    let rest = rest.to_vec();

    context.enqueue_job(NativeJob::new(move |context| {
        handle(
            handler_map,
            id,
            function_ref,
            rest,
            start,
            delay,
            false,
            clock.clone(),
            context,
        )
    }));

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
pub fn set_interval<C: Clock + 'static>(
    function_ref: JsFunction,
    delay_in_msec: Option<JsValue>,
    rest: JsRest<'_>,
    context: &mut Context,
) -> JsResult<u32> {
    let handler_map = IntervalInnerState::from_context(context)?;
    let id = handler_map.borrow_mut().new_interval()?;

    let clock = context
        .get_data::<Gc<C>>()
        .ok_or(js_error!("Clock not found"))?
        .clone();

    // Spec says if delay is not a number, it should be equal to 0.
    let delay = delay_in_msec
        .unwrap_or_default()
        .to_integer_or_infinity(context)
        .unwrap_or(IntegerOrInfinity::Integer(0));
    let delay = delay.clamp_finite(0, i64::MAX).unsigned_abs();
    let start = clock.now();

    // Get ownership of rest arguments.
    let rest = rest.to_vec();

    context.enqueue_job(NativeJob::new(move |context| {
        handle(
            handler_map,
            id,
            function_ref,
            rest,
            start,
            delay,
            true,
            clock.clone(),
            context,
        )
    }));

    Ok(id)
}

/// Clears a timeout or interval currently running.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/Window/clearTimeout).
///
/// Please note that this is the same exact method as `clearInterval`, as both can be
/// used interchangeably.
#[must_use]
pub fn clear_timeout(id: u32, context: &mut Context) -> JsResult<()> {
    let handler_map = IntervalInnerState::from_context(context)?;
    handler_map.borrow_mut().clear_interval(id);
    Ok(())
}

/// Register the interval module into the given context.
///
/// # Errors
/// Any error returned by the context when registering the global functions.
pub fn register(context: &mut Context) -> JsResult<()> {
    register_with_clock(context, StdClock)
}

/// Register the interval module into the given context with a custom clock.
///
/// # Errors
/// Any error returned by the context when registering the global functions.
pub fn register_with_clock<C: Clock + 'static>(context: &mut Context, clock: C) -> JsResult<()> {
    context.insert_data(Gc::new(clock));
    register_functions::<C>(context)
}

/// Register the interval module without any clock. This still needs the proper
/// typing for the clock, even if it is not registerd to the context.
///
/// # Errors
/// Any error returned by the context when registering the global functions.
pub fn register_functions<C: Clock + 'static>(context: &mut Context) -> JsResult<()> {
    let set_timeout_ = set_timeout::<C>.into_js_function_copied(context);
    context.register_global_callable(js_string!("setTimeout"), 1, set_timeout_)?;

    let set_interval_ = set_interval::<C>.into_js_function_copied(context);
    context.register_global_callable(js_string!("setInterval"), 1, set_interval_)?;

    // These two methods are identical, just under different names in JavaScript.
    let clear_timeout_ = clear_timeout.into_js_function_copied(context);
    context.register_global_callable(js_string!("clearTimeout"), 1, clear_timeout_.clone())?;
    context.register_global_callable(js_string!("clearInterval"), 1, clear_timeout_)?;

    Ok(())
}
