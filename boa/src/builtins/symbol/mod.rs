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
    builtins::value::{RcString, RcSymbol, ResultValue, Value},
    exec::Interpreter,
    BoaProfiler,
};
use gc::{Finalize, Trace};

#[derive(Debug, Finalize, Trace, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(Option<RcString>, u32);

impl Symbol {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "Symbol";

    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 0;

    /// Returns the `Symbol`s description.
    pub fn description(&self) -> Option<&str> {
        self.0.as_deref()
    }

    /// Returns the `Symbol`s hash.
    pub fn hash(&self) -> u32 {
        self.1
    }

    fn this_symbol_value(value: &Value, ctx: &mut Interpreter) -> Result<RcSymbol, Value> {
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
    pub(crate) fn call(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
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
    pub(crate) fn to_string(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let symbol = Self::this_symbol_value(this, ctx)?;
        let description = symbol.description().unwrap_or("");
        Ok(Value::from(format!("Symbol({})", description)))
    }

    /// Initialise the `Symbol` object on the global object.
    #[inline]
    pub fn init(global: &Value) -> (&str, Value) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        // Create prototype object
        let prototype = Value::new_object(Some(global));

        make_builtin_fn(Self::to_string, "toString", &prototype, 0);

        let symbol_object = make_constructor_fn(
            Self::NAME,
            Self::LENGTH,
            Self::call,
            global,
            prototype,
            false,
            true,
        );

        (Self::NAME, symbol_object)
    }
}
