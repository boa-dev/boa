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

use super::property::Attribute;
use crate::{builtins::value::Value, BoaProfiler, Interpreter};
use std::borrow::BorrowMut;

/// JavaScript global `NaN` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NaN;

impl NaN {
    /// The binding name of the property.
    pub(crate) const NAME: &'static str = "NaN";

    /// Initialize the `NaN` property on the global object.
    #[inline]
    pub(crate) fn init(interpreter: &mut Interpreter) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let mut global = interpreter.global().as_object_mut().expect("Expect object");
        global.borrow_mut().insert_property(
            Self::NAME,
            Value::from(f64::NAN),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );
    }
}
