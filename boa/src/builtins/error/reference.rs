//! This module implements the global `ReferenceError` object.
//!
//! Indicates an error that occurs when de-referencing an invalid reference
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-native-error-types-used-in-this-standard-rangeerror TODO
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ReferenceError

use crate::{
  builtins::{
      function::make_builtin_fn,
      function::make_constructor_fn,
      object::ObjectKind,
      value::{ResultValue, Value},
  },
  exec::Interpreter,
  profiler::BoaProfiler,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ReferenceError;

impl ReferenceError {
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
  pub(crate) fn to_string(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
      let name = this.get_field("name");
      let message = this.get_field("message");
      Ok(Value::from(format!("{}: {}", name, message)))
  }

  /// Create a new `ReferenceError` object.
  pub(crate) fn create(global: &Value) -> Value {
      let prototype = Value::new_object(Some(global));
      prototype.set_field("message", Value::from(""));

      make_builtin_fn(Self::to_string, "toString", &prototype, 0);

      make_constructor_fn("ReferenceError", 1, Self::make_error, global, prototype, true)
  }

  /// Initialise the global object with the `ReferenceError` object.
  pub(crate) fn init(global: &Value) {
      let _timer = BoaProfiler::global().start_event("referenceerror", "init");
      global.set_field("ReferenceError", Self::create(global));
  }
}
