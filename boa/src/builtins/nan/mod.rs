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

use crate::{builtins::BuiltIn, property::Attribute, BoaProfiler, Context, JsValue};

/// JavaScript global `NaN` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NaN;

impl BuiltIn for NaN {
    const NAME: &'static str = "NaN";

    fn attribute() -> Attribute {
        Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT
    }

    fn init(_: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        (Self::NAME, f64::NAN.into(), Self::attribute())
    }
}
