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

#[cfg(test)]
mod tests;

use crate::{builtins::value::Value, BoaProfiler, Interpreter};

/// JavaScript global `undefined` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Undefined;
use super::property::Attribute;
use std::borrow::BorrowMut;

impl Undefined {
    /// The binding name of the property.
    pub(crate) const NAME: &'static str = "undefined";

    /// Initialize the `undefined` property on the global object.
    #[inline]
    pub(crate) fn init(interpreter: &mut Interpreter) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");
        let mut global = interpreter.global().as_object_mut().unwrap();
        global.borrow_mut().insert_property(
            Self::NAME,
            Value::undefined(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
    }
}
