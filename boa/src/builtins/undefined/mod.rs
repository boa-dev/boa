//! This module implements the global `undefined` property.
//!
//! The global undefined property represents the primitive value undefined.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-undefined
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/undefined

use crate::{builtins::BuiltIn, property::Attribute, BoaProfiler, Context, Value};

#[cfg(test)]
mod tests;

/// JavaScript global `undefined` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Undefined;

impl BuiltIn for Undefined {
    const NAME: &'static str = "undefined";

    fn attribute() -> Attribute {
        Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT
    }

    fn init(_context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        (Self::NAME, Value::undefined(), Self::attribute())
    }
}
