//! This module implements the global `globalThis` property.
//!
//! The global globalThis property contains the global this value,
//! which is akin to the global object.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-globalthis
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/globalThis

use crate::{builtins::value::Value, BoaProfiler};

#[cfg(test)]
mod tests;

/// The JavaScript `globalThis`.
pub(crate) struct GlobalThis;

impl GlobalThis {
    /// The binding name of the property.
    pub(crate) const NAME: &'static str = "globalThis";

    /// Initialize the `globalThis` property on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) -> (&str, Value) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        (Self::NAME, global.clone())
    }
}
