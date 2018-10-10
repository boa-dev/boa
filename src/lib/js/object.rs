use gc::Gc;
use js::value::Value;
use std::collections::HashMap;
pub static PROTOTYPE: &'static str = "prototype";
pub static INSTANCE_PROTOTYPE: &'static str = "__proto__";

pub type ObjectData = HashMap<String, Property>;

/// A Javascript Property
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
    pub fn new(value: Value) -> Property {
        Property {
            configurable: false,
            enumerable: false,
            writable: false,
            value: value,
            // get: Gc::new(VUndefined),
            // set: Gc::new(VUndefined),
        }
    }
}
