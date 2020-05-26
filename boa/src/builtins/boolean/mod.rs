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
        object::{internal_methods_trait::ObjectInternalMethods, ObjectKind},
        value::{ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use std::{borrow::Borrow, ops::Deref};

/// Boolean implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Boolean;

impl Boolean {
    /// `[[Construct]]` Create a new boolean object
    ///
    /// `[[Call]]` Creates a new boolean primitive
    pub(crate) fn construct_boolean(
        this: &mut Value,
        args: &[Value],
        _: &mut Interpreter,
    ) -> ResultValue {
        this.set_kind(ObjectKind::Boolean);

        // Get the argument, if any
        if let Some(ref value) = args.get(0) {
            this.set_internal_slot("BooleanData", Self::to_boolean(value));
        } else {
            this.set_internal_slot("BooleanData", Self::to_boolean(&Value::from(false)));
        }

        match args.get(0) {
            Some(ref value) => Ok(Self::to_boolean(value)),
            None => Ok(Self::to_boolean(&Value::from(false))),
        }
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
    pub(crate) fn to_string(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
        let b = Self::this_boolean_value(this);
        Ok(Value::from(b.to_string()))
    }

    /// The valueOf() method returns the primitive value of a `Boolean` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    /// - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-boolean.prototype.valueof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean/valueOf
    pub(crate) fn value_of(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
        Ok(Self::this_boolean_value(this))
    }

    // === Utility Functions ===
    /// [toBoolean](https://tc39.es/ecma262/#sec-toboolean)
    /// Creates a new boolean value from the input
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_boolean(value: &Value) -> Value {
        match *value.deref().borrow() {
            ValueData::Object(_) => Value::from(true),
            ValueData::String(ref s) if !s.is_empty() => Value::from(true),
            ValueData::Rational(n) if n != 0.0 && !n.is_nan() => Value::from(true),
            ValueData::Integer(n) if n != 0 => Value::from(true),
            ValueData::Boolean(v) => Value::from(v),
            _ => Value::from(false),
        }
    }

    /// An Utility function used to get the internal BooleanData.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-thisbooleanvalue
    pub(crate) fn this_boolean_value(value: &Value) -> Value {
        match *value.deref().borrow() {
            ValueData::Boolean(v) => Value::from(v),
            ValueData::Object(ref v) => (v).deref().borrow().get_internal_slot("BooleanData"),
            _ => Value::from(false),
        }
    }

    /// Create a new `Boolean` object.
    pub(crate) fn create(global: &Value) -> Value {
        // Create Prototype
        // https://tc39.es/ecma262/#sec-properties-of-the-boolean-prototype-object
        let prototype = Value::new_object(Some(global));
        prototype.set_internal_slot("BooleanData", Self::to_boolean(&Value::from(false)));

        make_builtin_fn(Self::to_string, "toString", &prototype, 0);
        make_builtin_fn(Self::value_of, "valueOf", &prototype, 0);

        make_constructor_fn(
            "Boolean",
            1,
            Self::construct_boolean,
            global,
            prototype,
            true,
        )
    }

    /// Initialise the `Boolean` object on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) {
        global.set_field("Boolean", Self::create(global));
    }
}
