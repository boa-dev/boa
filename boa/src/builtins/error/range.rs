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
        object::ObjectData,
        value::{ResultValue, Value},
    },
    exec::Interpreter,
    profiler::BoaProfiler,
};

/// JavaScript `RangeError` impleentation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RangeError;

impl RangeError {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "RangeError";

    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

    /// Create a new error object.
    pub(crate) fn make_error(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        if let Some(message) = args.get(0) {
            this.set_field("message", ctx.to_string(message)?);
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
    pub(crate) fn to_string(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let name = ctx.to_string(&this.get_field("name"))?;
        let message = ctx.to_string(&this.get_field("message"))?;
        Ok(Value::from(format!("{}: {}", name, message)))
    }

    /// Initialise the global object with the `RangeError` object.
    #[inline]
    pub(crate) fn init(global: &Value) -> (&str, Value) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let prototype = Value::new_object(Some(global));
        prototype.set_field("name", Self::NAME);
        prototype.set_field("message", "");

        make_builtin_fn(Self::to_string, "toString", &prototype, 0);

        let range_error_object = make_constructor_fn(
            Self::NAME,
            Self::LENGTH,
            Self::make_error,
            global,
            prototype,
            true,
            true,
        );

        (Self::NAME, range_error_object)
    }
}
