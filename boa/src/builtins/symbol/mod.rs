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

use crate::{
    builtins::BuiltIn,
    object::{ConstructorBuilder, FunctionBuilder},
    property::Attribute,
    symbol::RcSymbol,
    value::Value,
    BoaProfiler, Context, Result,
};

#[derive(Debug, Clone, Copy)]
pub struct Symbol;

impl BuiltIn for Symbol {
    const NAME: &'static str = "Symbol";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // https://tc39.es/ecma262/#sec-well-known-symbols
        let well_known_symbols = context.well_known_symbols();

        let symbol_async_iterator = well_known_symbols.async_iterator_symbol();
        let symbol_has_instance = well_known_symbols.has_instance_symbol();
        let symbol_is_concat_spreadable = well_known_symbols.is_concat_spreadable_symbol();
        let symbol_iterator = well_known_symbols.iterator_symbol();
        let symbol_match = well_known_symbols.match_symbol();
        let symbol_match_all = well_known_symbols.match_all_symbol();
        let symbol_replace = well_known_symbols.replace_symbol();
        let symbol_search = well_known_symbols.search_symbol();
        let symbol_species = well_known_symbols.species_symbol();
        let symbol_split = well_known_symbols.split_symbol();
        let symbol_to_primitive = well_known_symbols.to_primitive_symbol();
        let symbol_to_string_tag = well_known_symbols.to_string_tag_symbol();
        let symbol_unscopables = well_known_symbols.unscopables_symbol();

        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT;

        let get_description = FunctionBuilder::new(context, Self::get_description)
            .name("get description")
            .constructable(false)
            .callable(true)
            .build();

        let symbol_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().symbol_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .static_property("asyncIterator", symbol_async_iterator, attribute)
        .static_property("hasInstance", symbol_has_instance, attribute)
        .static_property("isConcatSpreadable", symbol_is_concat_spreadable, attribute)
        .static_property("iterator", symbol_iterator, attribute)
        .static_property("match", symbol_match, attribute)
        .static_property("matchAll", symbol_match_all, attribute)
        .static_property("replace", symbol_replace, attribute)
        .static_property("search", symbol_search, attribute)
        .static_property("species", symbol_species, attribute)
        .static_property("split", symbol_split, attribute)
        .static_property("toPrimitive", symbol_to_primitive, attribute)
        .static_property("toStringTag", symbol_to_string_tag, attribute)
        .static_property("unscopables", symbol_unscopables, attribute)
        .method(Self::to_string, "toString", 0)
        .accessor(
            "description",
            Some(get_description),
            None,
            Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .callable(true)
        .constructable(false)
        .build();

        (Self::NAME, symbol_object.into(), Self::attribute())
    }
}

impl Symbol {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 0;

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
    pub(crate) fn constructor(
        new_target: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        if new_target.is_undefined() {
            return context.throw_type_error("Symbol is not a constructor");
        }
        let description = match args.get(0) {
            Some(ref value) if !value.is_undefined() => Some(value.to_string(context)?),
            _ => None,
        };

        Ok(context.construct_symbol(description).into())
    }

    fn this_symbol_value(value: &Value, context: &mut Context) -> Result<RcSymbol> {
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

        Err(context.construct_type_error("'this' is not a Symbol"))
    }

    /// `Symbol.prototype.toString()`
    ///
    /// This method returns a string representing the specified `Symbol` object.
    ///
    /// More information:
    /// - [MDN documentation][mdn]
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-symbol.prototype.tostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/toString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        let symbol = Self::this_symbol_value(this, context)?;
        let description = symbol.description().unwrap_or("");
        Ok(Value::from(format!("Symbol({})", description)))
    }

    /// `get Symbol.prototype.description`
    ///
    /// This accessor returns the description of the `Symbol` object.
    ///
    /// More information:
    /// - [MDN documentation][mdn]
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-symbol.prototype.description
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/description
    pub(crate) fn get_description(
        this: &Value,
        _: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        if let Some(ref description) = Self::this_symbol_value(this, context)?.description {
            Ok(description.clone().into())
        } else {
            Ok(Value::undefined())
        }
    }
}
