//! This module implements the global `Symbol` object.
//!
//! The data type symbol is a primitive data type.
//! The `Symbol()` function returns a value of type symbol, has static properties that expose
//! several members of built-in objects, has static methods that expose the global symbol registry,
//! and resembles a built-in object class, but is incomplete as a constructor because it does not
//! support the syntax "`new Symbol()`".
//!
//! Every symbol value returned from `Symbol()` is unique.
//!
//! More information:
//! - [MDN documentation][mdn]
//! - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-symbol-value
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol

#[cfg(test)]
mod tests;

use super::function::{make_builtin_fn, make_constructor_fn};
use crate::{
    builtins::{
        property::{Attribute, Property},
        value::{RcString, RcSymbol, Value},
    },
    exec::Interpreter,
    BoaProfiler, Result,
};
use gc::{Finalize, Trace};

#[derive(Debug, Finalize, Trace, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    hash: u32,
    description: Option<RcString>,
}

impl Symbol {
    pub(crate) fn new(hash: u32, description: Option<RcString>) -> Self {
        Self { hash, description }
    }
}

impl Symbol {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "Symbol";

    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 0;

    /// Returns the `Symbol`s description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the `Symbol`s hash.
    pub fn hash(&self) -> u32 {
        self.hash
    }

    fn this_symbol_value(value: &Value, ctx: &mut Interpreter) -> Result<RcSymbol> {
        match value {
            Value::Symbol(ref symbol) => return Ok(symbol.clone()),
            Value::Object(ref object) => {
                let object = object.borrow();
                if let Some(symbol) = object.as_symbol() {
                    return Ok(symbol);
                }
            }
            _ => {}
        }

        Err(ctx.construct_type_error("'this' is not a Symbol"))
    }

    /// The `Symbol()` constructor returns a value of type symbol.
    ///
    /// It is incomplete as a constructor because it does not support
    /// the syntax `new Symbol()` and it is not intended to be subclassed.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-symbol-description
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/Symbol
    pub(crate) fn call(_: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Value> {
        let description = match args.get(0) {
            Some(ref value) if !value.is_undefined() => Some(value.to_string(ctx)?),
            _ => None,
        };

        Ok(ctx.construct_symbol(description).into())
    }

    /// `Symbol.prototype.toString()`
    ///
    /// This method returns a string representing the specified `Symbol` object.
    ///
    /// /// More information:
    /// - [MDN documentation][mdn]
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-symbol.prototype.tostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/toString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(this: &Value, _: &[Value], ctx: &mut Interpreter) -> Result<Value> {
        let symbol = Self::this_symbol_value(this, ctx)?;
        let description = symbol.description().unwrap_or("");
        Ok(Value::from(format!("Symbol({})", description)))
    }

    /// Initialise the `Symbol` object on the global object.
    #[inline]
    pub fn init(interpreter: &mut Interpreter) -> (&'static str, Value) {
        // Define the Well-Known Symbols
        // https://tc39.es/ecma262/#sec-well-known-symbols
        let symbol_async_iterator =
            interpreter.construct_symbol(Some("Symbol.asyncIterator".into()));
        let symbol_has_instance = interpreter.construct_symbol(Some("Symbol.hasInstance".into()));
        let symbol_is_concat_spreadable =
            interpreter.construct_symbol(Some("Symbol.isConcatSpreadable".into()));
        let symbol_iterator = interpreter.construct_symbol(Some("Symbol.iterator".into()));
        let symbol_match = interpreter.construct_symbol(Some("Symbol.match".into()));
        let symbol_match_all = interpreter.construct_symbol(Some("Symbol.matchAll".into()));
        let symbol_replace = interpreter.construct_symbol(Some("Symbol.replace".into()));
        let symbol_search = interpreter.construct_symbol(Some("Symbol.search".into()));
        let symbol_species = interpreter.construct_symbol(Some("Symbol.species".into()));
        let symbol_split = interpreter.construct_symbol(Some("Symbol.split".into()));
        let symbol_to_primitive = interpreter.construct_symbol(Some("Symbol.toPrimitive".into()));
        let symbol_to_string_tag = interpreter.construct_symbol(Some("Symbol.toStringTag".into()));
        let symbol_unscopables = interpreter.construct_symbol(Some("Symbol.unscopables".into()));

        let global = interpreter.global();
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // Create prototype object
        let prototype = Value::new_object(Some(global));

        make_builtin_fn(Self::to_string, "toString", &prototype, 0, interpreter);

        let symbol_object = make_constructor_fn(
            Self::NAME,
            Self::LENGTH,
            Self::call,
            global,
            prototype,
            false,
            true,
        );

        {
            let mut symbol_object = symbol_object.as_object_mut().unwrap();
            let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT;
            symbol_object.insert_property(
                "asyncIterator",
                Property::data_descriptor(symbol_async_iterator.into(), attribute),
            );
            symbol_object.insert_property(
                "hasInstance",
                Property::data_descriptor(symbol_has_instance.into(), attribute),
            );
            symbol_object.insert_property(
                "isConcatSpreadable",
                Property::data_descriptor(symbol_is_concat_spreadable.into(), attribute),
            );
            symbol_object.insert_property(
                "iterator",
                Property::data_descriptor(symbol_iterator.into(), attribute),
            );
            symbol_object.insert_property(
                "match",
                Property::data_descriptor(symbol_match.into(), attribute),
            );
            symbol_object.insert_property(
                "matchAll",
                Property::data_descriptor(symbol_match_all.into(), attribute),
            );
            symbol_object.insert_property(
                "replace",
                Property::data_descriptor(symbol_replace.into(), attribute),
            );
            symbol_object.insert_property(
                "search",
                Property::data_descriptor(symbol_search.into(), attribute),
            );
            symbol_object.insert_property(
                "species",
                Property::data_descriptor(symbol_species.into(), attribute),
            );
            symbol_object.insert_property(
                "split",
                Property::data_descriptor(symbol_split.into(), attribute),
            );
            symbol_object.insert_property(
                "toPrimitive",
                Property::data_descriptor(symbol_to_primitive.into(), attribute),
            );
            symbol_object.insert_property(
                "toStringTag",
                Property::data_descriptor(symbol_to_string_tag.into(), attribute),
            );
            symbol_object.insert_property(
                "unscopables",
                Property::data_descriptor(symbol_unscopables.into(), attribute),
            );
        }

        (Self::NAME, symbol_object)
    }
}
