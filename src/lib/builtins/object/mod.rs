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
pub mod internal_state;

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
    pub sym_properties: Box<HashMap<usize, Property>>,
    /// Some rust object that stores internal state
    pub state: Option<Box<InternalStateCell>>,
}

impl Object {
    /// Return a new ObjectData struct, with `kind` set to Ordinary
    pub fn default() -> Self {
        Object {
            kind: ObjectKind::Ordinary,
            internal_slots: Box::new(HashMap::new()),
            properties: Box::new(HashMap::new()),
            sym_properties: Box::new(HashMap::new()),
            state: None,
        }
    }

    /// ObjectCreate is used to specify the runtime creation of new ordinary objects
    ///
    /// https://tc39.es/ecma262/#sec-objectcreate
    pub fn create(proto: Value) -> Object {
        let mut obj = Object::default();
        obj.internal_slots
            .insert(INSTANCE_PROTOTYPE.to_string(), proto.clone());
        obj.internal_slots
            .insert("extensible".to_string(), to_value(true));
        obj
    }

    /// Utility function to set an internal slot which is a function
    pub fn set_internal_method(&mut self, name: &str, val: NativeFunctionData) {
        self.internal_slots.insert(name.to_string(), to_value(val));
    }

    /// Return a new Boolean object whose [[BooleanData]] internal slot is set to argument.
    pub fn from_boolean(argument: &Value) -> Self {
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
    pub fn from_number(argument: &Value) -> Self {
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
    pub fn from_string(argument: &Value) -> Self {
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
            } else if same_value(&to_value(std::clone::Clone::clone(self)), &p, false) {
                return false;
            } else {
                p = p.get_internal_slot(PROTOTYPE);
            }
        }
        self.set_internal_slot(PROTOTYPE, val);
        true
    }

    /// Insert property into properties hashmap
    fn insert_property(&mut self, name: String, p: Property) {
        self.properties.insert(name, p);
    }

    /// Remove property from properties hashmap
    fn remove_property(&mut self, name: String) {
        self.properties.remove(&name.to_string());
    }

    /// Utility function to get an immutable internal slot or Null
    fn get_internal_slot(&self, name: &str) -> Value {
        match self.internal_slots.get(name) {
            Some(v) => v.clone(),
            None => Gc::new(ValueData::Null),
        }
    }

    /// Utility function to set an internal slot
    fn set_internal_slot(&mut self, name: &str, val: Value) {
        self.internal_slots.insert(name.to_string(), val);
    }

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
    /// The specification returns a Property Descriptor or Undefined. These are 2 separate types and we can't do that here.
    fn get_own_property(&self, prop: &Value) -> Property {
        debug_assert!(Property::is_property_key(prop));
        match self.properties.get(&prop.to_string()) {
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
fn make_object(_: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Gc::new(ValueData::Undefined))
}

/// Get the prototype of an object
fn get_proto_of(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object");
    Ok(obj.get_field_slice(INSTANCE_PROTOTYPE))
}

/// Set the prototype of an object
fn set_proto_of(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object").clone();
    let proto = args.get(1).expect("Cannot get object").clone();
    obj.set_internal_slot(INSTANCE_PROTOTYPE, proto);
    Ok(obj)
}

/// Define a property in an object
fn define_prop(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object");
    let prop = from_value::<String>(args.get(1).expect("Cannot get object").clone())
        .expect("Cannot get object");
    let desc = from_value::<Property>(args.get(2).expect("Cannot get object").clone())
        .expect("Cannot get object");
    obj.set_prop(prop, desc);
    Ok(Gc::new(ValueData::Undefined))
}

/// To string
fn to_string(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(this.to_string()))
}

/// Check if it has a property
fn has_own_prop(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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
pub fn create_constructor(global: &Value) -> Value {
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
