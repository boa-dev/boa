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
    symbol::{JsSymbol, WellKnownSymbols},
    value::Value,
    BoaProfiler, Context, JsString, Result,
};

use std::cell::RefCell;

use rustc_hash::FxHashMap;

thread_local! {
    static GLOBAL_SYMBOL_REGISTRY: RefCell<GlobalSymbolRegistry> = RefCell::new(GlobalSymbolRegistry::new());
}

struct GlobalSymbolRegistry {
    keys: FxHashMap<JsString, JsSymbol>,
    symbols: FxHashMap<JsSymbol, JsString>,
}

impl GlobalSymbolRegistry {
    fn new() -> Self {
        Self {
            keys: FxHashMap::default(),
            symbols: FxHashMap::default(),
        }
    }

    fn get_or_insert_key(&mut self, key: JsString) -> JsSymbol {
        if let Some(symbol) = self.keys.get(&key) {
            return symbol.clone();
        }

        let symbol = JsSymbol::new(Some(key.clone()));
        self.keys.insert(key.clone(), symbol.clone());
        self.symbols.insert(symbol.clone(), key);
        symbol
    }

    fn get_symbol(&self, sym: JsSymbol) -> Option<JsString> {
        if let Some(key) = self.symbols.get(&sym) {
            return Some(key.clone());
        }

        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Symbol;

impl BuiltIn for Symbol {
    const NAME: &'static str = "Symbol";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let symbol_async_iterator = WellKnownSymbols::async_iterator();
        let symbol_has_instance = WellKnownSymbols::has_instance();
        let symbol_is_concat_spreadable = WellKnownSymbols::is_concat_spreadable();
        let symbol_iterator = WellKnownSymbols::iterator();
        let symbol_match = WellKnownSymbols::match_();
        let symbol_match_all = WellKnownSymbols::match_all();
        let symbol_replace = WellKnownSymbols::replace();
        let symbol_search = WellKnownSymbols::search();
        let symbol_species = WellKnownSymbols::species();
        let symbol_split = WellKnownSymbols::split();
        let symbol_to_primitive = WellKnownSymbols::to_primitive();
        let symbol_to_string_tag = WellKnownSymbols::to_string_tag();
        let symbol_unscopables = WellKnownSymbols::unscopables();

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
        .static_method(Self::for_, "for", 1)
        .static_method(Self::key_for, "keyFor", 1)
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
        .static_property("toStringTag", symbol_to_string_tag.clone(), attribute)
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
        .property(
            symbol_to_string_tag,
            Self::NAME,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
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

        Ok(JsSymbol::new(description).into())
    }

    fn this_symbol_value(value: &Value, context: &mut Context) -> Result<JsSymbol> {
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
        Ok(symbol.to_string().into())
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
        let symbol = Self::this_symbol_value(this, context)?;
        if let Some(ref description) = symbol.description() {
            Ok(description.clone().into())
        } else {
            Ok(Value::undefined())
        }
    }

    /// `Symbol.for( key )`
    ///
    /// More information:
    /// - [MDN documentation][mdn]
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-symbol.prototype.for
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/for
    pub(crate) fn for_(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let stringKey be ? ToString(key).
        let string_key = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?;
        // 2. For each element e of the GlobalSymbolRegistry List, do
        //     a. If SameValue(e.[[Key]], stringKey) is true, return e.[[Symbol]].
        // 3. Assert: GlobalSymbolRegistry does not currently contain an entry for stringKey.
        // 4. Let newSymbol be a new unique Symbol value whose [[Description]] value is stringKey.
        // 5. Append the Record { [[Key]]: stringKey, [[Symbol]]: newSymbol } to the GlobalSymbolRegistry List.
        // 6. Return newSymbol.
        Ok(GLOBAL_SYMBOL_REGISTRY
            .with(move |registry| {
                let mut registry = registry.borrow_mut();
                registry.get_or_insert_key(string_key)
            })
            .into())
    }

    /// `Symbol.keyFor( sym )`
    ///
    ///
    /// More information:
    /// - [MDN documentation][mdn]
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-symbol.prototype.keyfor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/keyFor
    pub(crate) fn key_for(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let sym = args.get(0).cloned().unwrap_or_default();
        // 1. If Type(sym) is not Symbol, throw a TypeError exception.
        if let Some(sym) = sym.as_symbol() {
            // 2. For each element e of the GlobalSymbolRegistry List (see 20.4.2.2), do
            //     a. If SameValue(e.[[Symbol]], sym) is true, return e.[[Key]].
            // 3. Assert: GlobalSymbolRegistry does not currently contain an entry for sym.
            // 4. Return undefined.
            let symbol = GLOBAL_SYMBOL_REGISTRY.with(move |registry| {
                let registry = registry.borrow();
                registry.get_symbol(sym)
            });

            Ok(symbol.map(Value::from).unwrap_or_default())
        } else {
            context.throw_type_error("Symbol.keyFor: sym is not a symbol")
        }
    }
}
