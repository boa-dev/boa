use crate::{
    builtins::{
        function::NativeFunctionData,
        property::Property,
        value::{from_value, same_value, to_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::Gc;
use gc_derive::{Finalize, Trace};
use std::{borrow::Borrow, collections::HashMap, ops::Deref};

pub use internal_methods_trait::ObjectInternalMethods;
pub use internal_state::{InternalState, InternalStateCell};

pub mod internal_methods_trait;
mod internal_state;

/// Static `prototype`, usually set on constructors as a key to point to their respective prototype object.
pub static PROTOTYPE: &str = "prototype";

/// Static `__proto__`, usually set on Object instances as a key to point to their respective prototype object.
pub static INSTANCE_PROTOTYPE: &str = "__proto__";

/// `ObjectData` is the representation of an object in JavaScript
#[derive(Trace, Finalize, Debug, Clone)]
pub struct Object {
    /// Kind
    pub kind: ObjectKind,
    /// Internal Slots
    pub internal_slots: Box<HashMap<String, Value>>,
    /// Properties
    pub properties: Box<HashMap<String, Property>>,
    /// Symbol Properties
    pub sym_properties: Box<HashMap<i32, Property>>,
    /// Some rust object that stores internal state
    pub state: Option<Box<InternalStateCell>>,
}

impl ObjectInternalMethods for Object {
    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
    fn set_prototype_of(&mut self, val: Value) -> bool {
        debug_assert!(val.is_object() || val.is_null());
        let current = self.get_internal_slot(PROTOTYPE);
        if current == val {
            return true;
        }
        let extensible = self.get_internal_slot("extensible");
        if extensible.is_null() {
            return false;
        }
        let mut p = val.clone();
        let mut done = false;
        while !done {
            if p.is_null() {
                done = true
            } else if same_value(&to_value(self.clone()), &p, false) {
                return false;
            } else {
                p = p.get_internal_slot(PROTOTYPE);
            }
        }
        self.set_internal_slot(PROTOTYPE, val);
        true
    }

    fn insert_property(&mut self, name: String, p: Property) {
        self.properties.insert(name, p);
    }

    fn remove_property(&mut self, name: &str) {
        self.properties.remove(name);
    }

    /// Utility function to set an internal slot
    fn set_internal_slot(&mut self, name: &str, val: Value) {
        self.internal_slots.insert(name.to_string(), val);
    }

    /// Utility function to get an immutable internal slot or Null
    fn get_internal_slot(&self, name: &str) -> Value {
        match self.internal_slots.get(name) {
            Some(v) => v.clone(),
            None => Gc::new(ValueData::Null),
        }
    }

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
    /// The specification returns a Property Descriptor or Undefined. These are 2 separate types and we can't do that here.
    fn get_own_property(&self, prop: &Value) -> Property {
        debug_assert!(Property::is_property_key(prop));
        // Prop could either be a String or Symbol
        match *(*prop) {
            ValueData::String(ref st) => {
                match self.properties.get(st) {
                    // If O does not have an own property with key P, return undefined.
                    // In this case we return a new empty Property
                    None => Property::default(),
                    Some(ref v) => {
                        let mut d = Property::default();
                        if v.is_data_descriptor() {
                            d.value = v.value.clone();
                            d.writable = v.writable;
                        } else {
                            debug_assert!(v.is_accessor_descriptor());
                            d.get = v.get.clone();
                            d.set = v.set.clone();
                        }
                        d.enumerable = v.enumerable;
                        d.configurable = v.configurable;
                        d
                    }
                }
            }
            ValueData::Symbol(ref sym) => {
                let sym_id = sym
                    .borrow()
                    .get_internal_slot("SymbolData")
                    .to_string()
                    .parse::<i32>()
                    .expect("Could not get Symbol ID");
                match self.sym_properties.get(&sym_id) {
                    // If O does not have an own property with key P, return undefined.
                    // In this case we return a new empty Property
                    None => Property::default(),
                    Some(ref v) => {
                        let mut d = Property::default();
                        if v.is_data_descriptor() {
                            d.value = v.value.clone();
                            d.writable = v.writable;
                        } else {
                            debug_assert!(v.is_accessor_descriptor());
                            d.get = v.get.clone();
                            d.set = v.set.clone();
                        }
                        d.enumerable = v.enumerable;
                        d.configurable = v.configurable;
                        d
                    }
                }
            }
            _ => Property::default(),
        }
    }

    #[allow(clippy::option_unwrap_used)]
    fn define_own_property(&mut self, property_key: String, desc: Property) -> bool {
        let mut current = self.get_own_property(&to_value(property_key.to_string()));
        let extensible = self.is_extensible();

        // https://tc39.es/ecma262/#sec-validateandapplypropertydescriptor
        // There currently isn't a property, lets create a new one
        if current.value.is_none() || current.value.as_ref().expect("failed").is_undefined() {
            if !extensible {
                return false;
            }
            if desc.value.is_some() && desc.value.clone().unwrap().is_symbol() {
                let sym_id = desc
                    .value
                    .clone()
                    .unwrap()
                    .to_string()
                    .parse::<i32>()
                    .expect("parsing failed");
                self.sym_properties.insert(sym_id, desc);
            } else {
                self.properties.insert(property_key, desc);
            }
            return true;
        }
        // If every field is absent we don't need to set anything
        if desc.is_none() {
            return true;
        }

        // 4
        if !current.configurable.unwrap_or(false) {
            if desc.configurable.is_some() && desc.configurable.unwrap() {
                return false;
            }

            if desc.enumerable.is_some()
                && (desc.enumerable.as_ref().unwrap() != current.enumerable.as_ref().unwrap())
            {
                return false;
            }
        }

        // 5
        if desc.is_generic_descriptor() {
            // 6
        } else if current.is_data_descriptor() != desc.is_data_descriptor() {
            // a
            if !current.configurable.unwrap() {
                return false;
            }
            // b
            if current.is_data_descriptor() {
                // Convert to accessor
                current.value = None;
                current.writable = None;
            } else {
                // c
                // convert to data
                current.get = None;
                current.set = None;
            }

            if current.value.is_some() && current.value.clone().unwrap().is_symbol() {
                let sym_id = current
                    .value
                    .clone()
                    .unwrap()
                    .to_string()
                    .parse::<i32>()
                    .expect("parsing failed");
                self.sym_properties.insert(sym_id, current);
            } else {
                self.properties.insert(property_key.clone(), current);
            }
        // 7
        } else if current.is_data_descriptor() && desc.is_data_descriptor() {
            // a
            if !current.configurable.unwrap() && !current.writable.unwrap() {
                if desc.writable.is_some() && desc.writable.unwrap() {
                    return false;
                }

                if desc.value.is_some()
                    && !same_value(
                        &desc.value.clone().unwrap(),
                        &current.value.clone().unwrap(),
                        false,
                    )
                {
                    return false;
                }

                return true;
            }
        // 8
        } else {
            if !current.configurable.unwrap() {
                if desc.set.is_some()
                    && !same_value(
                        &desc.set.clone().unwrap(),
                        &current.set.clone().unwrap(),
                        false,
                    )
                {
                    return false;
                }

                if desc.get.is_some()
                    && !same_value(
                        &desc.get.clone().unwrap(),
                        &current.get.clone().unwrap(),
                        false,
                    )
                {
                    return false;
                }
            }

            return true;
        }
        // 9
        self.properties.insert(property_key, desc);
        true
    }
}

impl Object {
    /// Return a new ObjectData struct, with `kind` set to Ordinary
    pub fn default() -> Self {
        let mut object = Object {
            kind: ObjectKind::Ordinary,
            internal_slots: Box::new(HashMap::new()),
            properties: Box::new(HashMap::new()),
            sym_properties: Box::new(HashMap::new()),
            state: None,
        };

        object.set_internal_slot("extensible", to_value(true));
        object
    }

    /// ObjectCreate is used to specify the runtime creation of new ordinary objects
    ///
    /// https://tc39.es/ecma262/#sec-objectcreate
    // TODO: proto should be a &Value here
    pub fn create(proto: Value) -> Object {
        let mut obj = Object::default();
        obj.internal_slots
            .insert(INSTANCE_PROTOTYPE.to_string(), proto);
        obj.internal_slots
            .insert("extensible".to_string(), to_value(true));
        obj
    }

    /// Utility function to set an internal slot which is a function
    pub fn set_internal_method(&mut self, name: &str, val: NativeFunctionData) {
        self.internal_slots.insert(name.to_string(), to_value(val));
    }

    /// Utility function to set a method on this object
    /// The native function will live in the `properties` field of the Object
    pub fn set_method(&mut self, name: &str, val: NativeFunctionData) {
        self.properties
            .insert(name.to_string(), Property::default().value(to_value(val)));
    }

    /// Return a new Boolean object whose [[BooleanData]] internal slot is set to argument.
    fn from_boolean(argument: &Value) -> Self {
        let mut obj = Object {
            kind: ObjectKind::Boolean,
            internal_slots: Box::new(HashMap::new()),
            properties: Box::new(HashMap::new()),
            sym_properties: Box::new(HashMap::new()),
            state: None,
        };

        obj.internal_slots
            .insert("BooleanData".to_string(), argument.clone());
        obj
    }

    /// Return a new Number object whose [[NumberData]] internal slot is set to argument.
    fn from_number(argument: &Value) -> Self {
        let mut obj = Object {
            kind: ObjectKind::Number,
            internal_slots: Box::new(HashMap::new()),
            properties: Box::new(HashMap::new()),
            sym_properties: Box::new(HashMap::new()),
            state: None,
        };

        obj.internal_slots
            .insert("NumberData".to_string(), argument.clone());
        obj
    }

    /// Return a new String object whose [[StringData]] internal slot is set to argument.
    fn from_string(argument: &Value) -> Self {
        let mut obj = Object {
            kind: ObjectKind::String,
            internal_slots: Box::new(HashMap::new()),
            properties: Box::new(HashMap::new()),
            sym_properties: Box::new(HashMap::new()),
            state: None,
        };

        obj.internal_slots
            .insert("StringData".to_string(), argument.clone());
        obj
    }

    // https://tc39.es/ecma262/#sec-toobject
    pub fn from(value: &Value) -> Result<Self, ()> {
        match *value.deref().borrow() {
            ValueData::Boolean(_) => Ok(Self::from_boolean(value)),
            ValueData::Number(_) => Ok(Self::from_number(value)),
            ValueData::String(_) => Ok(Self::from_string(value)),
            ValueData::Object(ref obj) => Ok(obj.borrow().clone()),
            _ => Err(()),
        }
    }
}

#[derive(Trace, Finalize, Clone, Debug, Eq, PartialEq)]
pub enum ObjectKind {
    Function,
    Array,
    String,
    Symbol,
    Error,
    Ordinary,
    Boolean,
    Number,
}

/// Create a new object
pub fn make_object(_: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Gc::new(ValueData::Undefined))
}

/// Get the prototype of an object
pub fn get_proto_of(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object");
    Ok(obj.get_field_slice(INSTANCE_PROTOTYPE))
}

/// Set the prototype of an object
pub fn set_proto_of(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object").clone();
    let proto = args.get(1).expect("Cannot get object").clone();
    obj.set_internal_slot(INSTANCE_PROTOTYPE, proto);
    Ok(obj)
}

/// Define a property in an object
pub fn define_prop(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object");
    let prop = from_value::<String>(args.get(1).expect("Cannot get object").clone())
        .expect("Cannot get object");
    let desc = from_value::<Property>(args.get(2).expect("Cannot get object").clone())
        .expect("Cannot get object");
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
        from_value::<String>(args.get(0).expect("Cannot get object").clone()).ok()
    };
    Ok(to_value(
        prop.is_some() && this.get_prop(&prop.expect("Cannot get object")).is_some(),
    ))
}

/// Create a new `Object` object
pub fn create_constructor(_: &Value) -> Value {
    let object = to_value(make_object as NativeFunctionData);
    // Prototype chain ends here VV
    let mut prototype = Object::default();
    prototype.set_method("hasOwnProperty", has_own_prop);
    prototype.set_method("toString", to_string);

    object.set_field_slice("length", to_value(1_i32));
    object.set_field_slice(PROTOTYPE, to_value(prototype));
    make_builtin_fn!(set_proto_of, named "setPrototypeOf", with length 2, of object);
    make_builtin_fn!(get_proto_of, named "getPrototypeOf", with length 1, of object);
    make_builtin_fn!(define_prop, named "defineProperty", with length 3, of object);
    object
}
