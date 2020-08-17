//! This module implements the global `Infinity` property.
//!
//! The global property `Infinity` is a numeric value representing infinity.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-value-properties-of-the-global-object-infinity
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Infinity

#[cfg(test)]
mod tests;

use super::property::Attribute;
use crate::{builtins::value::Value, BoaProfiler, Interpreter};
use std::borrow::BorrowMut;

/// JavaScript global `Infinity` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Infinity;

impl Infinity {
    /// The binding name of the property.
    pub(crate) const NAME: &'static str = "Infinity";

    /// Initialize the `Infinity` property on the global object.
    #[inline]
    pub(crate) fn init(interpreter: &mut Interpreter) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");
        let mut global = interpreter.global().as_object_mut().expect("Expect object");
        global.borrow_mut().insert_property(
            Self::NAME,
            Value::from(f64::INFINITY),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );
    }
}
