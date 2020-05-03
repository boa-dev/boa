//! This module implements the global `Error` object.
//!
//! Error objects are thrown when runtime errors occur.
//! The Error object can also be used as a base object for user-defined exceptions.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-error-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error

use crate::{
    builtins::{
        object::{internal_methods_trait::ObjectInternalMethods, Object, ObjectKind, PROTOTYPE},
        value::{to_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::Gc;

/// Create a new error object.
pub fn make_error(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    if !args.is_empty() {
        this.set_field_slice(
            "message",
            to_value(
                args.get(0)
                    .expect("failed getting error message")
                    .to_string(),
            ),
        );
    }
    // This value is used by console.log and other routines to match Object type
    // to its Javascript Identifier (global constructor method name)
    this.set_kind(ObjectKind::Error);
    Ok(Gc::new(ValueData::Undefined))
}

/// `Error.prototype.toString()`
///
/// The toString() method returns a string representing the specified Error object.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-error.prototype.tostring
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/toString
pub fn to_string(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let name = this.get_field_slice("name");
    let message = this.get_field_slice("message");
    Ok(to_value(format!("{}: {}", name, message)))
}

/// Create a new `Error` object.
pub fn _create(global: &Value) -> Value {
    let prototype = ValueData::new_obj(Some(global));
    prototype.set_field_slice("message", to_value(""));
    prototype.set_field_slice("name", to_value("Error"));
    make_builtin_fn!(to_string, named "toString", of prototype);
    make_constructor_fn!(make_error, global, prototype)
}

/// Initialise the global object with the `Error` object.
pub fn init(global: &Value) {
    global.set_field_slice("Error", _create(global));
}
