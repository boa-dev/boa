//! This module implements the global `ReferenceError` object.
//!
//! Indicates an error that occurs when de-referencing an invalid reference
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-referenceerror
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ReferenceError

use crate::{
    builtins::{function::make_builtin_fn, function::make_constructor_fn},
    object::ObjectData,
    profiler::BoaProfiler,
    Context, Result, Value,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ReferenceError;

impl ReferenceError {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "ReferenceError";

    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// Create a new error object.
    pub(crate) fn make_error(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
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
    pub(crate) fn to_string(this: &Value, _: &[Value], ctx: &mut Context) -> Result<Value> {
        let name = this.get_field("name").to_string(ctx)?;
        let message = this.get_field("message").to_string(ctx)?;

        Ok(Value::from(format!("{}: {}", name, message)))
    }

    /// Initialise the global object with the `ReferenceError` object.
    pub(crate) fn init(interpreter: &mut Context) -> (&'static str, Value) {
        let global = interpreter.global_object();
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let prototype = Value::new_object(Some(global));
        prototype.set_field("name", Self::NAME);
        prototype.set_field("message", "");

        make_builtin_fn(Self::to_string, "toString", &prototype, 0, interpreter);

        let reference_error_object = make_constructor_fn(
            Self::NAME,
            Self::LENGTH,
            Self::make_error,
            global,
            prototype,
            true,
            true,
        );

        (Self::NAME, reference_error_object)
    }
}
