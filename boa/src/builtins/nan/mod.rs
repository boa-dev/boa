#[cfg(test)] mod tests;

use crate::{builtins::value::Value, BoaProfiler};

/// Initialize the `NaN` property on the global object.
#[inline]
pub fn init(global: &Value) {
    let _timer = BoaProfiler::global().start_event("NaN", "init");
    global.set_field("NaN", Value::from(f64::NAN));
}
