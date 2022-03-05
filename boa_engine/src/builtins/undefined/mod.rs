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

use crate::{builtins::BuiltIn, property::Attribute, Context, JsValue};
use boa_profiler::Profiler;

#[cfg(test)]
mod tests;

/// JavaScript global `undefined` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Undefined;

impl BuiltIn for Undefined {
    const NAME: &'static str = "undefined";

    const ATTRIBUTE: Attribute = Attribute::READONLY
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::PERMANENT);

    fn init(_: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        Some(JsValue::undefined())
    }
}
