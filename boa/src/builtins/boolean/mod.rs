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

use crate::{
    builtins::{
        function::NativeFunctionData,
        object::{internal_methods_trait::ObjectInternalMethods, Object, ObjectKind, PROTOTYPE},
        value::{to_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use std::{borrow::Borrow, ops::Deref};

/// Create a new boolean object - [[Construct]]
pub fn construct_boolean(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    this.set_kind(ObjectKind::Boolean);

    // Get the argument, if any
    if let Some(ref value) = args.get(0) {
        this.set_internal_slot("BooleanData", to_boolean(value));
    } else {
        this.set_internal_slot("BooleanData", to_boolean(&to_value(false)));
    }

    // no need to return `this` as its passed by reference
    Ok(this.clone())
}

/// Return a boolean literal [[Call]]
pub fn call_boolean(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    // Get the argument, if any
    match args.get(0) {
        Some(ref value) => Ok(to_boolean(value)),
        None => Ok(to_boolean(&to_value(false))),
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
pub fn to_string(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let b = this_boolean_value(this);
    Ok(to_value(b.to_string()))
}

/// The valueOf() method returns the primitive value of a `Boolean` object.
///
/// More information:
///  - [ECMAScript reference][spec]
/// - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-boolean.prototype.valueof
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Boolean/valueOf
pub fn value_of(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(this_boolean_value(this))
}

/// Create a new `Boolean` object
pub fn create_constructor(global: &Value) -> Value {
    let mut boolean = Object::default();
    boolean.kind = ObjectKind::Function;
    boolean.set_internal_method("construct", construct_boolean);
    boolean.set_internal_method("call", call_boolean);
    // Create Prototype
    // https://tc39.es/ecma262/#sec-properties-of-the-boolean-prototype-object
    let boolean_prototype = ValueData::new_obj(Some(global));
    boolean_prototype.set_internal_slot("BooleanData", to_boolean(&to_value(false)));
    make_builtin_fn!(to_string, named "toString", of boolean_prototype);
    make_builtin_fn!(value_of, named "valueOf", of boolean_prototype);

    let boolean_value = to_value(boolean);
    boolean_prototype.set_field_slice("constructor", to_value(boolean_value.clone()));
    boolean_value.set_field_slice(PROTOTYPE, boolean_prototype);
    boolean_value
}

// === Utility Functions ===
/// [toBoolean](https://tc39.es/ecma262/#sec-toboolean)
/// Creates a new boolean value from the input
pub fn to_boolean(value: &Value) -> Value {
    match *value.deref().borrow() {
        ValueData::Object(_) => to_value(true),
        ValueData::String(ref s) if !s.is_empty() => to_value(true),
        ValueData::Rational(n) if n != 0.0 && !n.is_nan() => to_value(true),
        ValueData::Integer(n) if n != 0 => to_value(true),
        ValueData::Boolean(v) => to_value(v),
        _ => to_value(false),
    }
}

/// An Utility function used to get the internal BooleanData.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-thisbooleanvalue
pub fn this_boolean_value(value: &Value) -> Value {
    match *value.deref().borrow() {
        ValueData::Boolean(v) => to_value(v),
        ValueData::Object(ref v) => (v).deref().borrow().get_internal_slot("BooleanData"),
        _ => to_value(false),
    }
}
