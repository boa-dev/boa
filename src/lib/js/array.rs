use gc::Gc;
use crate::js::function::NativeFunctionData;
use crate::js::value::{to_value, ResultValue, Value, ValueData};

/// Create a new array
pub fn make_array(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let this_ptr = this.clone();
    this_ptr.set_field_slice("length", to_value(0i32));
    Ok(Gc::new(ValueData::Undefined))
}
/// Create a new `Array` object
pub fn _create() -> Value {
    let array = to_value(make_array as NativeFunctionData);
    array
}
/// Initialise the global object with the `Array` object
pub fn init(global: Value) {
    global.set_field_slice("Array", _create());
}
