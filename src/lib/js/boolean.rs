use gc::Gc;
use js::value::{to_value, ResultValue, Value, ValueData};

/// Create a new boolean
pub fn make_boolean(_: Vec<Value>, _: Value, _: Value, this: Value) -> ResultValue {
    // This value is used by console.log and other routines to match Object type
    // to its Javascript Identifier (global constructor method name)
    //this.set_private_field_slice("type", to_value("Boolean"));
    Ok(Gc::new(ValueData::Undefined))
}
/// Create a new `Boolean` object
pub fn _create(global: Value) -> Value {
    let boolean = to_value(make_boolean);
    boolean
}
/// Initialise the global object with the `Error` object
pub fn init(global: &Value) {
    let global_ptr = global.borrow();
    global_ptr.set_field_slice("Boolean", _create(global));
}
