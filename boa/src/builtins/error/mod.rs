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
        function::{make_builtin_fn, make_constructor_fn},
        object::ObjectKind,
        value::{ResultValue, Value},
    },
    exec::Interpreter,
};

// mod eval;
pub(crate) mod range;
// mod reference;
// mod syntax;
// mod type_err;
// mod uri;

pub(crate) use self::range::RangeError;

/// Built-in `Error` object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Error;

impl Error {
    /// Create a new error object.
    pub(crate) fn make_error(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
        if !args.is_empty() {
            this.set_field(
                "message",
                Value::from(
                    args.get(0)
                        .expect("failed getting error message")
                        .to_string(),
                ),
            );
        }
        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        this.set_kind(ObjectKind::Error);
        Ok(Value::undefined())
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
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
        let name = this.get_field("name");
        let message = this.get_field("message");
        Ok(Value::from(format!("{}: {}", name, message)))
    }

    /// Create a new `Error` object.
    pub(crate) fn create(global: &Value) -> Value {
        let prototype = Value::new_object(Some(global));
        prototype.set_field("message", Value::from(""));

        make_builtin_fn(Self::to_string, "toString", &prototype, 0);

        make_constructor_fn("Error", 1, Self::make_error, global, prototype, true)
    }

    /// Initialise the global object with the `Error` object.
    pub(crate) fn init(global: &Value) {
        global.set_field("Error", Self::create(global));
    }
}
