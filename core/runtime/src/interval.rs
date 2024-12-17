//! A module that declares any functions for dealing with intervals or
//! timeouts.

use boa_engine::job::NativeJob;
use boa_engine::object::builtins::JsFunction;
use boa_engine::{js_error, js_string, Context, Finalize, JsData, JsResult, JsValue, Trace};
use boa_gc::{Gc, GcRefCell};
use boa_interop::{IntoJsFunctionCopied, JsRest};
use std::collections::HashMap;
use std::time::SystemTime;

/// The latest timeout ID that was created. This is internal only and shouldn't be
/// accessed outside.
#[derive(Trace, Finalize, JsData)]
struct LatestIntervalId(i32);

impl LatestIntervalId {
    pub fn new_id(context: &mut Context) -> JsResult<i32> {
        let id = match context.get_data::<GcRefCell<LatestIntervalId>>() {
            None => {
                context.insert_data(GcRefCell::new(LatestIntervalId(0)));
                0
            }
            Some(id) => {
                let mut id = id.borrow_mut();
                if id.0 == i32::MAX {
                    return Err(js_error!(RangeError: "maximum interval ID reached"));
                }

                id.0 += 1;
                id.0
            }
        };

        Ok(id)
    }
}

/// The internal state of the interval module. The value is whether the interval
/// function is still active.
#[derive(Trace, Finalize, JsData)]
struct IntervalHandlerMap(HashMap<i32, GcRefCell<bool>>);

impl IntervalHandlerMap {
    pub fn from_context(context: &mut Context) -> Gc<GcRefCell<Self>> {
        if !context.has_data::<Gc<GcRefCell<IntervalHandlerMap>>>() {
            context.insert_data(Gc::new(GcRefCell::new(IntervalHandlerMap(HashMap::new()))));
        }

        context
            .get_data::<Gc<GcRefCell<IntervalHandlerMap>>>()
            .expect("Should have inserted.")
            .clone()
    }
}

/// Set a timeout to call the given function after the given delay.
/// The `code` version of this function is not supported at the moment.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope/setTimeout)
///
/// # Errors
/// Any errors when trying to read the context, converting the arguments or
/// enqueuing the job.
pub fn set_timeout(
    function_ref: JsFunction,
    delay_in_msec: Option<JsValue>,
    rest: JsRest<'_>,
    context: &mut Context,
) -> JsResult<i32> {
    let id = LatestIntervalId::new_id(context)?;
    let handler_map = IntervalHandlerMap::from_context(context);

    let delay = delay_in_msec
        .unwrap_or_default()
        .to_number(context)
        .unwrap_or_default();
    let delay = if delay.is_nan() { 0.0 } else { delay };
    let start = SystemTime::now();

    // Get ownership of rest arguments.
    let rest = rest.0.to_vec();

    context.enqueue_job(NativeJob::new(move |context| {
        fn handle(
            handler_map: Gc<GcRefCell<IntervalHandlerMap>>,
            id: i32,
            function_ref: JsFunction,
            rest: Vec<JsValue>,
            start: SystemTime,
            delay_in_msec: f64,
            context: &mut Context,
        ) -> JsResult<JsValue> {
            let still_good = handler_map
                .borrow()
                .0
                .get(&id)
                .map_or(false, |x| *x.borrow());

            if !still_good {
                return Ok(JsValue::undefined());
            }

            let elapsed = start
                .elapsed()
                .expect("Time should go forward")
                .as_secs_f64()
                * 1000.0;

            if elapsed > delay_in_msec {
                function_ref.call(&JsValue::undefined(), &rest, context)
            } else {
                // Reschedule the timeout.
                context.enqueue_job(NativeJob::new(move |context| {
                    handle(
                        handler_map,
                        id,
                        function_ref,
                        rest,
                        start,
                        delay_in_msec,
                        context,
                    )
                }));

                Ok(JsValue::undefined())
            }
        }

        handle(handler_map, id, function_ref, rest, start, delay, context)
    }));

    Ok(id)
}

/// Register the interval module into the given context.
///
/// # Errors
/// Any error returned by the context when registering the global functions.
pub fn register(context: &mut Context) -> JsResult<()> {
    let set_timeout_ = set_timeout.into_js_function_copied(context);
    context.register_global_callable(js_string!("setTimeout"), 1, set_timeout_)?;
    Ok(())
}
