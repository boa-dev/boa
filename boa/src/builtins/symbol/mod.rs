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
    builtins::value::{ResultValue, Value, ValueData},
    exec::Interpreter,
    BoaProfiler,
};
use gc::{Finalize, Trace};

#[derive(Debug, Finalize, Trace, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(Option<String>, u32);

impl Symbol {
    /// Returns the `Symbol`s description.
    pub fn description(&self) -> Option<&str> {
        self.0.as_deref()
    }

    /// Returns the `Symbol`s hash.
    pub fn hash(&self) -> u32 {
        self.1
    }

    fn this_symbol_value(value: &Value, ctx: &mut Interpreter) -> Result<Self, Value> {
        match value.data() {
            ValueData::Symbol(ref symbol) => return Ok(symbol.clone()),
            ValueData::Object(ref object) => {
                let object = object.borrow();
                if let Some(symbol) = object.as_symbol() {
                    return Ok(symbol.clone());
                }
            }
            _ => {}
        }

        ctx.throw_type_error("'this' is not a Symbol")?;
        unreachable!();
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
    pub(crate) fn call(_: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let description = match args.get(0) {
            Some(ref value) if !value.is_undefined() => Some(ctx.to_string(value)?),
            _ => None,
        };

        Ok(Value::symbol(Symbol(description, ctx.generate_hash())))
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
    pub(crate) fn to_string(this: &mut Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let symbol = Self::this_symbol_value(this, ctx)?;
        let description = symbol.description().unwrap_or("");
        Ok(Value::from(format!("Symbol({})", description)))
    }

    /// Create a new `Symbol` object.
    pub(crate) fn create(global: &Value) -> Value {
        // Create prototype object
        let prototype = Value::new_object(Some(global));

        make_builtin_fn(Self::to_string, "toString", &prototype, 0);
        make_constructor_fn("Symbol", 1, Self::call, global, prototype, false)
    }

    /// Initialise the `Symbol` object on the global object.
    #[inline]
    pub fn init(global: &Value) {
        let _timer = BoaProfiler::global().start_event("symbol", "init");
        let symbol = Self::create(global);
        global
            .as_object_mut()
            .unwrap()
            .insert_field("Symbol", symbol);
    }
}
