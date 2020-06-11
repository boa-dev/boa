//! This module implements the global `NaN` property.
//!
//! The global `NaN` is a property of the global object. In other words,
//! it is a variable in global scope.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-value-properties-of-the-global-object-nan
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/NaN

#[cfg(test)]
mod tests;

use crate::{builtins::value::Value, BoaProfiler};

/// JavaScript global `NaN` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NaN;

impl NaN {
    /// Initialize the `NaN` property on the global object.
    #[inline]
    pub(crate) fn init(_: &Value) -> (&str, Value) {
        let _timer = BoaProfiler::global().start_event("NaN", "init");

        ("NaN", Value::from(f64::NAN))
    }
}
