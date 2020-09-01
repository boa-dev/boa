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
        object::ObjectData,
    },
    exec::Interpreter,
    profiler::BoaProfiler,
    Result, Value,
};

pub(crate) mod range;
pub(crate) mod reference;
pub(crate) mod syntax;
pub(crate) mod r#type;
// pub(crate) mod eval;
// pub(crate) mod uri;

pub(crate) use self::r#type::TypeError;
pub(crate) use self::range::RangeError;
pub(crate) use self::reference::ReferenceError;
pub(crate) use self::syntax::SyntaxError;
// pub(crate) use self::eval::EvalError;
// pub(crate) use self::uri::UriError;

/// Built-in `Error` object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Error;

impl Error {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "Error";

    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// Create a new error object.
    pub(crate) fn make_error(this: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Value> {
        if let Some(message) = args.get(0) {
            this.set_field("message", message.to_string(ctx)?);
        }

        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        this.set_data(ObjectData::Error);
        Err(this.clone())
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
    pub(crate) fn to_string(this: &Value, _: &[Value], _: &mut Interpreter) -> Result<Value> {
        let name = this.get_field("name");
        let message = this.get_field("message");
        Ok(Value::from(format!(
            "{}: {}",
            name.display(),
            message.display()
        )))
    }

    /// Initialise the global object with the `Error` object.
    #[inline]
    pub(crate) fn init(interpreter: &mut Interpreter) -> (&'static str, Value) {
        let global = interpreter.global();
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let prototype = Value::new_object(Some(global));
        prototype.set_field("name", Self::NAME);
        prototype.set_field("message", "");

        make_builtin_fn(Self::to_string, "toString", &prototype, 0, interpreter);

        let error_object = make_constructor_fn(
            Self::NAME,
            Self::LENGTH,
            Self::make_error,
            global,
            prototype,
            true,
            true,
        );

        (Self::NAME, error_object)
    }
}
