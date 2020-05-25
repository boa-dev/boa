//! This module implements the global `RangeError` object.
//!
//! Indicates a value that is not in the set or range of allowable values.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-rangeerror
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RangeError

use crate::{
    builtins::{
        function::make_builtin_fn,
        function::make_constructor_fn,
        object::ObjectKind,
        value::{ResultValue, Value},
    },
    exec::Interpreter,
};

/// JavaScript `RangeError` impleentation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RangeError;

impl RangeError {
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

    /// Create a new `RangeError` object.
    pub(crate) fn create(global: &Value) -> Value {
        let prototype = Value::new_object(Some(global));
        prototype.set_field("message", Value::from(""));
        prototype.set_field("name", Value::from("RangeError"));
        make_builtin_fn(Self::to_string, "toString", &prototype, 0);
        make_constructor_fn(Self::make_error, global, prototype)
    }

    /// Runs a `new RangeError(message)`.
    pub(crate) fn run_new<M>(message: M, interpreter: &mut Interpreter) -> ResultValue
    where
        M: Into<String>,
    {
        use crate::{
            exec::Executable,
            syntax::ast::{
                node::{Call, Identifier, New},
                Const,
            },
        };

        New::from(Call::new(
            Identifier::from("RangeError"),
            vec![Const::from(message.into()).into()],
        ))
        .run(interpreter)
    }

    /// Initialise the global object with the `RangeError` object.
    pub(crate) fn init(global: &Value) {
        global.set_field("RangeError", Self::create(global));
    }
}
