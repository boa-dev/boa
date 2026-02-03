//! Boa's implementation of the `performance` Web API object.
//!
//! The `performance` object provides access to performance-related information.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [W3C High Resolution Time specification][spec]
//!
//! [spec]: https://w3c.github.io/hr-time/
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Performance

use boa_engine::{
    Context, JsData, JsNativeError, JsResult, JsValue, NativeFunction,
    context::time::JsInstant,
    js_string,
    object::{FunctionObjectBuilder, ObjectInitializer},
    property::Attribute,
};
use boa_gc::{Finalize, Trace};

#[cfg(test)]
mod tests;

/// The `Performance` object.
#[derive(Debug, Trace, Finalize, JsData)]
pub struct Performance {
    #[unsafe_ignore_trace]
    time_origin: JsInstant,
}

impl Performance {
    /// Create a new `Performance` object.
    #[must_use]
    pub fn new(context: &Context) -> Self {
        Self {
            time_origin: context.clock().now(),
        }
    }

    /// Register the `Performance` object in the context.
    ///
    /// # Errors
    /// Returns an error if the `performance` property already exists in the global object.
    pub fn register(context: &mut Context) -> JsResult<()> {
        let performance = Self::new(context);

        let get_time_origin = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_fn_ptr(Self::get_time_origin),
        )
        .name(js_string!("get timeOrigin"))
        .length(0)
        .build();

        let performance_obj = ObjectInitializer::with_native_data(performance, context)
            .function(NativeFunction::from_fn_ptr(Self::now), js_string!("now"), 0)
            .accessor(
                js_string!("timeOrigin"),
                Some(get_time_origin),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .build();

        context.register_global_property(
            js_string!("performance"),
            performance_obj,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )?;

        Ok(())
    }

    /// `Performance.timeOrigin` getter
    ///
    /// The `timeOrigin` read-only property returns the high resolution timestamp
    /// that is used as the baseline for performance-related timestamps.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [W3C specification][spec]
    ///
    /// [spec]: https://w3c.github.io/hr-time/#dom-performance-timeorigin
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Performance/timeOrigin
    fn get_time_origin(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // The timeOrigin attribute MUST return the number of milliseconds in the duration returned by get
        // time origin timestamp for the relevant global object of this.
        //
        // The time values returned when getting Performance.timeOrigin MUST use the same monotonic clock
        // that is shared by time origins, and whose reference point is the [ECMA-262] time definition -
        // see 9. Security Considerations.

        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Performance object")
        })?;

        let performance = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Performance object")
        })?;

        #[allow(clippy::cast_precision_loss)]
        let time_origin_millis = performance.time_origin.nanos_since_epoch() as f64 / 1_000_000.0;
        Ok(JsValue::from(time_origin_millis))
    }

    /// `performance.now()`
    ///
    /// The `now()` method returns a high resolution timestamp in milliseconds.
    /// It represents the time elapsed since `time_origin`.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [W3C specification][spec]
    ///
    /// [spec]: https://w3c.github.io/hr-time/#dom-performance-now
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Performance/now
    fn now(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // The now() method MUST return the number of milliseconds in the current high resolution time
        // given this's relevant global object (a duration).
        //
        // The time values returned when calling the now() method on Performance objects with the
        // same time origin MUST use the same monotonic clock. The difference between any two
        // chronologically recorded time values returned from the now() method MUST never be
        // negative if the two time values have the same time origin.

        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Performance object")
        })?;

        let performance = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Performance object")
        })?;

        // Step 1: Get time origin from the Performance object
        let time_origin = performance.time_origin;

        // Step 2: Get current high resolution time
        let now = context.clock().now();

        // Step 3: Calculate duration from time_origin to now in milliseconds
        let elapsed = now - time_origin;

        #[allow(clippy::cast_precision_loss)]
        let milliseconds = elapsed.as_nanos() as f64 / 1_000_000.0;

        Ok(JsValue::from(milliseconds))
    }
}
