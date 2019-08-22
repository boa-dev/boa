use crate::{
    exec::Interpreter,
    js::{
        function::NativeFunctionData,
        property::Property,
        value::{from_value, to_value, ResultValue, Value, ValueData},
    },
};
use gc::Gc;
use gc_derive::{Finalize, Trace};
use std::{borrow::Borrow, collections::HashMap, ops::Deref};

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
}

impl Object {
    /// Return a new ObjectData struct, with `kind` set to Ordinary
    pub fn default() -> Self {
        Object {
            kind: ObjectKind::Ordinary,
            internal_slots: Box::new(HashMap::new()),
            properties: Box::new(HashMap::new()),
            sym_properties: Box::new(HashMap::new()),
        }
    }

    /// Return a new Boolean object whose [[BooleanData]] internal slot is set to argument.
    fn from_boolean(argument: &Value) -> Self {
        let mut obj = Object {
            kind: ObjectKind::Boolean,
            internal_slots: Box::new(HashMap::new()),
            properties: Box::new(HashMap::new()),
            sym_properties: Box::new(HashMap::new()),
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
            _ => Err(()),
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
    Boolean,
    Number,
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
    obj.set_internal_slot(INSTANCE_PROTOTYPE, proto);
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
