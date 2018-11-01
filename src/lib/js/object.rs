use gc::Gc;
use js::value::{from_value, to_value, FromValue, ResultValue, ToValue, Value, ValueData};
use std::collections::HashMap;
pub static PROTOTYPE: &'static str = "prototype";
pub static INSTANCE_PROTOTYPE: &'static str = "__proto__";

pub type ObjectData = HashMap<String, Property>;

/// A Javascript Property AKA The Property Descriptor   
/// [[SPEC] - The Property Descriptor Specification Type](https://tc39.github.io/ecma262/#sec-property-descriptor-specification-type)   
/// [[SPEC] - Default Attribute Values](https://tc39.github.io/ecma262/#table-4)
#[derive(Trace, Finalize, Clone, Debug)]
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
            get: Gc::new(ValueData::Undefined),
            set: Gc::new(ValueData::Undefined),
        }
    }
}

impl ToValue for Property {
    fn to_value(&self) -> Value {
        let prop = ValueData::new_obj(None);
        prop.set_field_slice("configurable", to_value(self.configurable));
        prop.set_field_slice("enumerable", to_value(self.enumerable));
        prop.set_field_slice("writable", to_value(self.writable));
        prop.set_field_slice("value", self.value.clone());
        prop.set_field_slice("get", self.get.clone());
        prop.set_field_slice("set", self.set.clone());
        prop
    }
}

impl FromValue for Property {
    fn from_value(v: Value) -> Result<Property, &'static str> {
        Ok(Property {
            configurable: from_value(v.get_field_slice("configurable")).unwrap(),
            enumerable: from_value(v.get_field_slice("enumerable")).unwrap(),
            writable: from_value(v.get_field_slice("writable")).unwrap(),
            value: v.get_field_slice("value"),
            get: v.get_field_slice("get"),
            set: v.get_field_slice("set"),
        })
    }
}

/// Create a new object
pub fn make_object(_: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(Gc::new(ValueData::Undefined))
}

/// Get the prototype of an object
pub fn get_proto_of(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let obj = args.get(0).unwrap();
    Ok(obj.get_field_slice(INSTANCE_PROTOTYPE))
}

/// Set the prototype of an object
pub fn set_proto_of(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let obj = args.get(0).unwrap().clone();
    let proto = args.get(1).unwrap().clone();
    obj.set_field_slice(INSTANCE_PROTOTYPE, proto);
    Ok(obj)
}

/// Define a property in an object
pub fn define_prop(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let obj = args.get(0).unwrap();
    let prop = from_value::<String>(args.get(1).unwrap().clone()).unwrap();
    let desc = from_value::<Property>(args.get(2).unwrap().clone()).unwrap();
    obj.set_prop(prop, desc);
    Ok(Gc::new(ValueData::Undefined))
}

/// To string
pub fn to_string(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    Ok(to_value(this.to_string()))
}

/// Check if it has a property
pub fn has_own_prop(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let prop = if args.len() == 0 {
        None
    } else {
        from_value::<String>(args.get(0).unwrap().clone()).ok()
    };
    Ok(to_value(
        prop.is_some() && this.get_prop(prop.unwrap()).is_some(),
    ))
}

/// Create a new `Object` object
pub fn _create(global: Value) -> Value {
    let object = to_value(make_object);
    let object_ptr = object;
    let prototype = ValueData::new_obj(Some(global));
    prototype.set_field_slice("hasOwnProperty", to_value(has_own_prop));
    prototype.set_field_slice("toString", to_value(to_string));
    object_ptr.set_field_slice("length", to_value(1i32));
    object_ptr.set_field_slice(PROTOTYPE, prototype);
    object_ptr.set_field_slice("setPrototypeOf", to_value(set_proto_of));
    object_ptr.set_field_slice("getPrototypeOf", to_value(get_proto_of));
    object_ptr.set_field_slice("defineProperty", to_value(define_prop));
    object
}

/// Initialise the `Object` object on the global object
pub fn init(global: Value) {
    global.set_field_slice("Object", _create(global.clone()));
}
