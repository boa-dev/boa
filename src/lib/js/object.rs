use js::value::Value;
use std::collections::HashMap;
pub static PROTOTYPE: &'static str = "prototype";
pub static INSTANCE_PROTOTYPE: &'static str = "__proto__";

pub type ObjectData = HashMap<String, Property>;

/// A Javascript Property
/// [Attributes of a Data Property](https://tc39.github.io/ecma262/#sec-property-attributes)
/// [Attributes of an Accessor Property](https://tc39.github.io/ecma262/#table-3)
/// A data property associates a key value with an ECMAScript language value and a set of Boolean attributes.
/// An accessor property associates a key value with one or two accessor functions, and a set of Boolean attributes.
#[derive(Trace, Finalize)]
pub struct Property {
    /// If the type of this can be changed and this can be deleted
    pub configurable: bool,
    /// If the property shows up in enumeration of the object
    pub enumerable: bool,
    /// If this property can be changed with an assignment
    pub writable: bool,
    /// The value associated with the property
    pub value: Value,
    // pub get: Value,
    // pub set: Value,
}

impl Property {
    /// Make a new property with the given value
    pub fn new(value: Value) -> Property {
        Property {
            configurable: false,
            enumerable: false,
            writable: false,
            value: value,
            // get: Value::undefined(),
            // set: Value::undefined(),
        }
    }
}
