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

use crate::{builtins::BuiltIn, property::Attribute, Context, JsValue};
use boa_profiler::Profiler;

/// JavaScript global `Infinity` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Infinity;

impl BuiltIn for Infinity {
    const NAME: &'static str = "Infinity";

    const ATTRIBUTE: Attribute = Attribute::READONLY
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::PERMANENT);

    fn init(_: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        Some(f64::INFINITY.into())
    }
}
