use gc::GcCell;
use js::value::{to_value, ResultValue, Value, ValueData};
use std::collections::HashMap;
pub static PROTOTYPE: &'static str = "prototype";
pub static INSTANCE_PROTOTYPE: &'static str = "__proto__";

pub type ObjectData = HashMap<String, Property>;

/// A Javascript Property
/// [Attributes of a Data Property](https://tc39.github.io/ecma262/#sec-property-attributes)
/// [Attributes of an Accessor Property](https://tc39.github.io/ecma262/#table-3)
/// A data property associates a key value with an ECMAScript language value and a set of Boolean attributes.
/// An accessor property associates a key value with one or two accessor functions, and a set of Boolean attributes.
pub struct Property {
    /// If the type of this can be changed and this can be deleted
    pub configurable: bool,
    /// If the property shows up in enumeration of the object
    pub enumerable: bool,
    /// If this property can be changed with an assignment
    pub writable: bool,
    /// The value associated with the property
    pub value: Value,
    /// The function serving as getter
    pub get: Value,
    /// The function serving as setter
    pub set: Value,
}

impl Property {
    /// Make a new property with the given value
    /// [Default Attributes](https://tc39.github.io/ecma262/#table-4)
    pub fn new(value: Value) -> Property {
        Property {
            configurable: false,
            enumerable: false,
            writable: false,
            value: value,
            get: GcCell::new(Value::Undefined),
            set: GcCell::new(Value::Undefined),
        }
    }
}

/// Create a new object
pub fn make_object() -> ResultValue {
    Ok(GcCell::new(ValueData::Undefined))
}

/// Create a new `Object` object
pub fn _create(global: Value) -> Value {
    let object = to_value(make_object);
    let object_ptr = object.borrow();
    let prototype = ValueData::new_obj(Some(global));
    // prototype.borrow().set_field_slice("hasOwnProperty", to_value(has_own_prop));
    // prototype.borrow().set_field_slice("toString", to_value(to_string));
    object_ptr.set_field_slice("length", to_value(1i32));
    // object_ptr.set_field_slice(PROTOTYPE, prototype);
    // object_ptr.set_field_slice("setPrototypeOf", to_value(set_proto_of));
    // object_ptr.set_field_slice("getPrototypeOf", to_value(get_proto_of));
    // object_ptr.set_field_slice("defineProperty", to_value(define_prop));
    object
}

/// Initialise the `Object` object on the global object
pub fn init(global: Value) {
    let global_ptr = global.borrow();
    global_ptr.set_field_slice("Object", _create(global));
}
