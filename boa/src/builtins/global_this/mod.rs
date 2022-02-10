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

use crate::{builtins::BuiltIn, property::Attribute, BoaProfiler, Context, JsValue};

#[cfg(test)]
mod tests;

/// The JavaScript `globalThis`.
pub(crate) struct GlobalThis;

impl BuiltIn for GlobalThis {
    const NAME: &'static str = "globalThis";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);

    fn init(context: &mut Context) -> JsValue {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        context.global_object().clone().into()
    }
}
