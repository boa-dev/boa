use crate::{builtins::value::Value, BoaProfiler};

/// Initialise the `Math` object on the global object.
#[inline]
pub fn init(global: &Value) {
    let _timer = BoaProfiler::global().start_event("NaN", "init");
    global.set_field("NaN", Value::from(f64::NAN));
}
