use js::function::Function;
use js::value::{to_value, ResultValue, Value};

/// Create a new array
pub fn make_array(_: Vec<Value>, _: Value, _: Value, this: Value) -> ResultValue {
    this.set_field_slice("length", to_value(0i32));
    Ok(Value::undefined())
}
/// Create a new `Array` object
pub fn _create() -> Value {
    let array = Function::make(make_array, &[]);
    array
}
/// Initialise the global object with the `Array` object
pub fn init(global: Value) {
    global.set_field_slice("Array", _create());
}
