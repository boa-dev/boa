//! This module implements the global `Boolean` object.
//!
//! The `Boolean` object is an object wrapper for a boolean value.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-boolean-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean

#[cfg(test)]
mod tests;

use super::function::{make_builtin_fn, make_constructor_fn};
use crate::{
    builtins::{
        object::ObjectData,
        value::{ResultValue, Value},
    },
    exec::Interpreter,
    BoaProfiler,
};

/// Boolean implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Boolean;

impl Boolean {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "Boolean";

    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// An Utility function used to get the internal [[BooleanData]].
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-thisbooleanvalue
    fn this_boolean_value(value: &Value, ctx: &mut Interpreter) -> Result<bool, Value> {
        match value {
            Value::Boolean(boolean) => return Ok(*boolean),
            Value::Object(ref object) => {
                let object = object.borrow();
                if let Some(boolean) = object.as_boolean() {
                    return Ok(boolean);
                }
            }
            _ => {}
        }

        Err(ctx.construct_type_error("'this' is not a boolean"))
    }

    /// `[[Construct]]` Create a new boolean object
    ///
    /// `[[Call]]` Creates a new boolean primitive
    pub(crate) fn construct_boolean(
        this: &Value,
        args: &[Value],
        _: &mut Interpreter,
    ) -> ResultValue {
        // Get the argument, if any
        let data = args.get(0).map(|x| x.to_boolean()).unwrap_or(false);
        this.set_data(ObjectData::Boolean(data));

        Ok(Value::from(data))
    }

    /// The `toString()` method returns a string representing the specified `Boolean` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-boolean-object
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean/toString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let boolean = Self::this_boolean_value(this, ctx)?;
        Ok(Value::from(boolean.to_string()))
    }

    /// The valueOf() method returns the primitive value of a `Boolean` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-boolean.prototype.valueof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean/valueOf
    #[inline]
    pub(crate) fn value_of(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(Value::from(Self::this_boolean_value(this, ctx)?))
    }

    /// Create a new `Boolean` object.
    pub(crate) fn create(global: &Value) -> Value {
        // Create Prototype
        // https://tc39.es/ecma262/#sec-properties-of-the-boolean-prototype-object
        let prototype = Value::new_object(Some(global));

        make_builtin_fn(Self::to_string, "toString", &prototype, 0);
        make_builtin_fn(Self::value_of, "valueOf", &prototype, 0);

        make_constructor_fn(
            Self::NAME,
            Self::LENGTH,
            Self::construct_boolean,
            global,
            prototype,
            true,
        )
    }

    /// Initialise the `Boolean` object on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) -> (&str, Value) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        (Self::NAME, Self::create(global))
    }
}
