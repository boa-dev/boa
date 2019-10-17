use crate::{
    exec::Interpreter,
    js::{
        function::NativeFunctionData,
        property::Property,
        value::{from_value, same_value, to_value, ResultValue, Value, ValueData},
    },
};
use gc::Gc;
use gc_derive::{Finalize, Trace};
use std::{borrow::Borrow, collections::HashMap, ops::Deref};

pub use internal_state::{InternalState, InternalStateCell};

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

    /// Utility function to get an immutable internal slot or Null
    pub fn get_internal_slot(&self, name: &str) -> Value {
        match self.internal_slots.get(name) {
            Some(v) => v.clone(),
            None => Gc::new(ValueData::Null),
        }
    }

    /// Utility function to set an internal slot
    pub fn set_internal_slot(&mut self, name: &str, val: Value) {
        self.internal_slots.insert(name.to_string(), val);
    }

    /// Utility function to set an internal slot which is a function
    pub fn set_internal_method(&mut self, name: &str, val: NativeFunctionData) {
        self.internal_slots.insert(name.to_string(), to_value(val));
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

    /// Returns either the prototype or null
    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
    pub fn get_prototype_of(&self) -> Value {
        match self.internal_slots.get(PROTOTYPE) {
            Some(v) => v.clone(),
            None => Gc::new(ValueData::Null),
        }
    }

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
    pub fn set_prototype_of(&mut self, val: Value) -> bool {
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

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible
    pub fn is_extensible(&self) -> bool {
        match self.internal_slots.get("extensible") {
            Some(ref v) => {
                // try dereferencing it: `&(*v).clone()`
                from_value((*v).clone()).expect("boolean expected")
            }
            None => false,
        }
    }

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
    pub fn prevent_extensions(&mut self) -> bool {
        self.set_internal_slot("extensible", to_value(false));
        true
    }
    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
    /// The specification returns a Property Descriptor or Undefined. These are 2 separate types and we can't do that here.
    pub fn get_own_property(&self, prop: &Value) -> Property {
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

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
    pub fn has_property(&self, val: &Value) -> bool {
        debug_assert!(Property::is_property_key(val));
        let prop = self.get_own_property(val);
        if prop.value.is_none() {
            let parent: Value = self.get_prototype_of();
            if !parent.is_null() {
                // the parent value variant should be an object
                // In the unlikely event it isn't return false
                return match *parent {
                    ValueData::Object(ref obj) => obj.borrow().has_property(val),
                    _ => false,
                };
            }
            return false;
        }

        true
    }

    #[allow(clippy::option_unwrap_used)]
    pub fn define_own_property(&mut self, property_key: String, desc: Property) -> bool {
        let mut current = self.get_own_property(&to_value(property_key.to_string()));
        let extensible = self.is_extensible();

        // https://tc39.es/ecma262/#sec-validateandapplypropertydescriptor
        // There currently isn't a property, lets create a new one
        if current.value.is_none() || current.value.as_ref().expect("failed").is_undefined() {
            if !extensible {
                return false;
            }

            let mut p = Property::new();
            if desc.is_generic_descriptor() || desc.is_data_descriptor() {
                p.value = Some(desc.value.clone().unwrap_or_default());
                p.writable = Some(desc.writable.unwrap_or_default());
                p.configurable = Some(desc.configurable.unwrap_or_default());
                p.enumerable = Some(desc.enumerable.unwrap_or_default());
            } else {
                p.get = Some(desc.get.clone().unwrap_or_default());
                p.set = Some(desc.set.clone().unwrap_or_default());
                p.configurable = Some(desc.configurable.unwrap_or_default());
                p.enumerable = Some(desc.enumerable.unwrap_or_default());
            };
            self.properties.insert(property_key, p);
            return true;
        }
        // If every field is absent we don't need to set anything
        if desc.is_none() {
            return true;
        }

        // 4
        if current.configurable.unwrap_or(false) {
            if desc.configurable.is_some() && desc.configurable.unwrap() {
                return false;
            }

            if desc.enumerable.is_some()
                && (desc.enumerable.as_ref().unwrap() == current.enumerable.as_ref().unwrap())
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
            self.properties.insert(property_key, current.clone());
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
        Property::assign(&mut current, &desc);
        true
    }

    // [[Delete]]
    pub fn delete(&mut self, prop_key: &Value) -> bool {
        debug_assert!(Property::is_property_key(prop_key));
        let desc = self.get_own_property(prop_key);
        if desc
            .value
            .clone()
            .expect("unable to get value")
            .is_undefined()
        {
            return true;
        }
        if desc.configurable.expect("unable to get value") {
            self.properties.remove(&prop_key.to_string());
            return true;
        }

        false
    }

    // [[Get]]
    pub fn get(&self, val: &Value) -> Value {
        debug_assert!(Property::is_property_key(val));
        let desc = self.get_own_property(val);
        if desc.value.clone().is_none()
            || desc
                .value
                .clone()
                .expect("Failed to get object")
                .is_undefined()
        {
            // parent will either be null or an Object
            let parent = self.get_prototype_of();
            if parent.is_null() {
                return Gc::new(ValueData::Undefined);
            }

            let parent_obj = Object::from(&parent).expect("Failed to get object");

            return parent_obj.get(val);
        }

        if desc.is_data_descriptor() {
            return desc.value.clone().expect("failed to extract value");
        }

        let getter = desc.get.clone();
        if getter.is_none() || getter.expect("Failed to get object").is_undefined() {
            return Gc::new(ValueData::Undefined);
        }

        // TODO!!!!! Call getter from here
        Gc::new(ValueData::Undefined)
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
