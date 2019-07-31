use crate::{
    exec::Interpreter,
    js::{
        function::NativeFunctionData,
        value::{from_value, to_value, FromValue, ResultValue, ToValue, Value, ValueData},
    },
};
use gc::Gc;
use gc_derive::{Finalize, Trace};
use std::collections::HashMap;

/// Static `prototype`, usually set on constructors as a key to point to their respective prototype object.  
/// As this string will be used a lot throughout the program, its best being a static global string which will be referenced
pub static PROTOTYPE: &'static str = "prototype";

/// Static `__proto__`, usually set on Object instances as a key to point to their respective prototype object.  
/// As this string will be used a lot throughout the program, its best being a static global string which will be referenced
pub static INSTANCE_PROTOTYPE: &'static str = "__proto__";

/// `ObjectData` is the representation of an object in JavaScript
#[derive(Trace, Finalize, Debug, Clone)]
pub struct ObjectData {
    /// Kind
    pub kind: ObjectKind,
    /// Internal Slots
    pub internal_slots: Box<HashMap<String, Value>>,
    /// Properties
    pub properties: Box<HashMap<String, Property>>,
    /// Symbol Properties
    pub sym_properties: Box<HashMap<usize, Property>>,
}

impl ObjectData {
    /// Return a new ObjectData struct, with `kind` set to Ordinary
    pub fn default() -> Self {
        Self {
            kind: ObjectKind::Ordinary,
            internal_slots: Box::new(HashMap::new()),
            properties: Box::new(HashMap::new()),
            sym_properties: Box::new(HashMap::new()),
        }
    }
}
#[derive(Trace, Finalize, Clone, Debug)]
pub enum ObjectKind {
    Function,
    Array,
    String,
    Symbol,
    Error,
    Ordinary,
}

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
    /// Checks if the provided Value can be used as a property key.
    pub fn is_property_key(value: &Value) -> bool {
        value.is_string() // || value.is_symbol() // Uncomment this when we are handeling symbols.
    }

    /// Make a new property with the given value
    pub fn new(value: Value) -> Self {
        Self {
            configurable: false,
            enumerable: false,
            writable: false,
            value,
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
    fn from_value(v: Value) -> Result<Self, &'static str> {
        Ok(Self {
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
pub fn make_object(_: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Gc::new(ValueData::Undefined))
}

/// Get the prototype of an object
pub fn get_proto_of(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).unwrap();
    Ok(obj.get_field_slice(INSTANCE_PROTOTYPE))
}

/// Set the prototype of an object
pub fn set_proto_of(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).unwrap().clone();
    let proto = args.get(1).unwrap().clone();
    obj.set_field_slice(INSTANCE_PROTOTYPE, proto);
    Ok(obj)
}

/// Define a property in an object
pub fn define_prop(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).unwrap();
    let prop = from_value::<String>(args.get(1).unwrap().clone()).unwrap();
    let desc = from_value::<Property>(args.get(2).unwrap().clone()).unwrap();
    obj.set_prop(prop, desc);
    Ok(Gc::new(ValueData::Undefined))
}

/// To string
pub fn to_string(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(this.to_string()))
}

/// Check if it has a property
pub fn has_own_prop(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let prop = if args.is_empty() {
        None
    } else {
        from_value::<String>(args.get(0).unwrap().clone()).ok()
    };
    Ok(to_value(
        prop.is_some() && this.get_prop(&prop.unwrap()).is_some(),
    ))
}

/// Create a new `Object` object
pub fn _create(global: &Value) -> Value {
    let object = to_value(make_object as NativeFunctionData);
    let prototype = ValueData::new_obj(Some(global));
    prototype.set_field_slice(
        "hasOwnProperty",
        to_value(has_own_prop as NativeFunctionData),
    );
    prototype.set_field_slice("toString", to_value(to_string as NativeFunctionData));
    object.set_field_slice("length", to_value(1_i32));
    object.set_field_slice(PROTOTYPE, prototype);
    object.set_field_slice(
        "setPrototypeOf",
        to_value(set_proto_of as NativeFunctionData),
    );
    object.set_field_slice(
        "getPrototypeOf",
        to_value(get_proto_of as NativeFunctionData),
    );
    object.set_field_slice(
        "defineProperty",
        to_value(define_prop as NativeFunctionData),
    );
    object
}

/// Initialise the `Object` object on the global object
pub fn init(global: &Value) {
    global.set_field_slice("Object", _create(global));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_property_key_test() {
        let v = Value::new(ValueData::String(String::from("Boop")));
        assert!(Property::is_property_key(&v));

        let v = Value::new(ValueData::Boolean(true));
        assert!(!Property::is_property_key(&v));
    }
}
