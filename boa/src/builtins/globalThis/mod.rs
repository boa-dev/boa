#[cfg(test)]
mod tests;

use crate::{builtins::value::Value, BoaProfiler};

/// Initialize the `globalThis` property on the global object.
#[inline]
pub fn init(global: &Value) {
    let _timer = BoaProfiler::global().start_event("globalThis", "init");
    global.set_field("globalThis", Value::from(global.clone()));
}
