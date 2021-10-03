//! This module implements the global `Intl` object.
//!
//! `Intl` is a built-in object that has properties and methods for i18n. It's not a function object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma402/#intl-object

use crate::{
    builtins::BuiltIn, object::ObjectInitializer, property::Attribute, symbol::WellKnownSymbols,
    BoaProfiler, Context, JsValue,
};

#[cfg(test)]
mod tests;

/// JavaScript `Intl` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Intl;

impl BuiltIn for Intl {
    const NAME: &'static str = "Intl";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);

    fn init(context: &mut Context) -> JsValue {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let string_tag = WellKnownSymbols::to_string_tag();
        let object = ObjectInitializer::new(context)
            .property(
                string_tag,
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();

        object.into()
    }
}