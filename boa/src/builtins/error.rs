use crate::{
    builtins::{
        function::NativeFunctionData,
        object::{ObjectKind, PROTOTYPE},
        value::{to_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::Gc;

/// Create a new error
pub fn make_error(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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
/// Get the string representation of the error
pub fn to_string(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let name = this.get_field_slice("name");
    let message = this.get_field_slice("message");
    Ok(to_value(format!("{}: {}", name, message)))
}
/// Create a new `Error` object
pub fn _create(global: &Value) -> Value {
    let prototype = ValueData::new_obj(Some(global));
    prototype.set_field_slice("message", to_value(""));
    prototype.set_field_slice("name", to_value("Error"));
    make_builtin_fn!(to_string, named "toString", of prototype);
    let error = to_value(make_error as NativeFunctionData);
    error.set_field_slice(PROTOTYPE, prototype);
    error
}
/// Initialise the global object with the `Error` object
pub fn init(global: &Value) {
    global.set_field_slice("Error", _create(global));
}
