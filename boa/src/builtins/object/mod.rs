//! This module implements the global `Object` object.
//!
//! The `Object` class represents one of JavaScript's data types.
//!
//! It is used to store various keyed collections and more complex entities.
//! Objects can be created using the `Object()` constructor or the
//! object initializer / literal syntax.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object

use crate::{
    builtins::{
        function::Function,
        property::Property,
        value::{same_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::{unsafe_empty_trace, Finalize, Trace};
use rustc_hash::FxHashMap;
use std::{
    borrow::Borrow,
    fmt::{self, Debug, Display, Error, Formatter},
    ops::Deref,
};

use super::function::{make_builtin_fn, make_constructor_fn};
pub use internal_methods_trait::ObjectInternalMethods;
pub use internal_state::{InternalState, InternalStateCell};

pub mod internal_methods_trait;
mod internal_state;

#[cfg(test)]
mod tests;

/// Static `prototype`, usually set on constructors as a key to point to their respective prototype object.
pub static PROTOTYPE: &str = "prototype";

/// Static `__proto__`, usually set on Object instances as a key to point to their respective prototype object.
pub static INSTANCE_PROTOTYPE: &str = "__proto__";

/// The internal representation of an JavaScript object.
#[derive(Trace, Finalize, Clone)]
pub struct Object {
    /// The type of the object.
    pub kind: ObjectKind,
    /// Internal Slots
    pub internal_slots: FxHashMap<String, Value>,
    /// Properties
    pub properties: FxHashMap<String, Property>,
    /// Symbol Properties
    pub sym_properties: FxHashMap<i32, Property>,
    /// Some rust object that stores internal state
    pub state: Option<InternalStateCell>,
    /// Function
    pub func: Option<Function>,
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        writeln!(f, "\tkind: {}", self.kind)?;
        writeln!(f, "\tstate: {:?}", self.state)?;
        writeln!(f, "\tfunc: {:?}", self.func)?;
        writeln!(f, "\tproperties: {{")?;
        for (key, _) in self.properties.iter() {
            writeln!(f, "\t\t{}", key)?;
        }
        writeln!(f, "\t }}")?;
        write!(f, "}}")
    }
}

impl ObjectInternalMethods for Object {
    /// `Object.setPropertyOf(obj, prototype)`
    ///
    /// This method sets the prototype (i.e., the internal `[[Prototype]]` property)
    /// of a specified object to another object or `null`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/setPrototypeOf
    fn set_prototype_of(&mut self, val: Value) -> bool {
        debug_assert!(val.is_object() || val.is_null());
        let current = self.get_internal_slot(PROTOTYPE);
        if same_value(&current, &val, false) {
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
            } else if same_value(&Value::from(self.clone()), &p, false) {
                return false;
            } else {
                p = p.get_internal_slot(PROTOTYPE);
            }
        }
        self.set_internal_slot(PROTOTYPE, val);
        true
    }

    /// Helper function for property insertion.
    fn insert_property(&mut self, name: String, p: Property) {
        self.properties.insert(name, p);
    }

    /// Helper function for property removal.
    fn remove_property(&mut self, name: &str) {
        self.properties.remove(name);
    }

    /// Helper function to set an internal slot
    fn set_internal_slot(&mut self, name: &str, val: Value) {
        self.internal_slots.insert(name.to_string(), val);
    }

    /// Helper function to get an immutable internal slot or Null
    fn get_internal_slot(&self, name: &str) -> Value {
        match self.internal_slots.get(name) {
            Some(v) => v.clone(),
            None => Value::null(),
        }
    }

    /// The specification returns a Property Descriptor or Undefined.
    ///
    /// These are 2 separate types and we can't do that here.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
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
                let sym_id = (**sym)
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

    /// Define an own property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc
    #[allow(clippy::option_unwrap_used)]
    fn define_own_property(&mut self, property_key: String, desc: Property) -> bool {
        let mut current = self.get_own_property(&Value::from(property_key.to_string()));
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
        let mut object = Self {
            kind: ObjectKind::Ordinary,
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            sym_properties: FxHashMap::default(),
            state: None,
            func: None,
        };

        object.set_internal_slot("extensible", Value::from(true));
        object
    }

    /// Return a new ObjectData struct, with `kind` set to Ordinary
    pub fn function() -> Self {
        let mut object = Self {
            kind: ObjectKind::Function,
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            sym_properties: FxHashMap::default(),
            state: None,
            func: None,
        };

        object.set_internal_slot("extensible", Value::from(true));
        object
    }

    /// ObjectCreate is used to specify the runtime creation of new ordinary objects.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-objectcreate
    // TODO: proto should be a &Value here
    pub fn create(proto: Value) -> Self {
        let mut obj = Self::default();
        obj.internal_slots
            .insert(INSTANCE_PROTOTYPE.to_string(), proto);
        obj.internal_slots
            .insert("extensible".to_string(), Value::from(true));
        obj
    }

    /// Set the function this object wraps
    pub fn set_func(&mut self, val: Function) {
        self.func = Some(val);
    }

    /// Return a new Boolean object whose `[[BooleanData]]` internal slot is set to argument.
    fn from_boolean(argument: &Value) -> Self {
        let mut obj = Self {
            kind: ObjectKind::Boolean,
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            sym_properties: FxHashMap::default(),
            state: None,
            func: None,
        };

        obj.internal_slots
            .insert("BooleanData".to_string(), argument.clone());
        obj
    }

    /// Return a new `Number` object whose `[[NumberData]]` internal slot is set to argument.
    fn from_number(argument: &Value) -> Self {
        let mut obj = Self {
            kind: ObjectKind::Number,
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            sym_properties: FxHashMap::default(),
            state: None,
            func: None,
        };

        obj.internal_slots
            .insert("NumberData".to_string(), argument.clone());
        obj
    }

    /// Return a new `String` object whose `[[StringData]]` internal slot is set to argument.
    fn from_string(argument: &Value) -> Self {
        let mut obj = Self {
            kind: ObjectKind::String,
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            sym_properties: FxHashMap::default(),
            state: None,
            func: None,
        };

        obj.internal_slots
            .insert("StringData".to_string(), argument.clone());
        obj
    }

    /// Return a new `BigInt` object whose `[[BigIntData]]` internal slot is set to argument.
    fn from_bigint(argument: &Value) -> Self {
        let mut obj = Self {
            kind: ObjectKind::BigInt,
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            sym_properties: FxHashMap::default(),
            state: None,
            func: None,
        };

        obj.internal_slots
            .insert("BigIntData".to_string(), argument.clone());
        obj
    }

    /// Converts the `Value` to an `Object` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-toobject
    pub fn from(value: &Value) -> Result<Self, ()> {
        match *value.deref().borrow() {
            ValueData::Boolean(_) => Ok(Self::from_boolean(value)),
            ValueData::Rational(_) => Ok(Self::from_number(value)),
            ValueData::String(_) => Ok(Self::from_string(value)),
            ValueData::BigInt(_) => Ok(Self::from_bigint(value)),
            ValueData::Object(ref obj) => Ok((*obj).deref().borrow().clone()),
            _ => Err(()),
        }
    }

    /// It determines if Object is a callable function with a [[Call]] internal method.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iscallable
    pub fn is_callable(&self) -> bool {
        match self.func {
            Some(ref function) => function.is_callable(),
            None => false,
        }
    }

    /// It determines if Object is a function object with a [[Construct]] internal method.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isconstructor
    pub fn is_constructable(&self) -> bool {
        match self.func {
            Some(ref function) => function.is_constructable(),
            None => false,
        }
    }
}

/// Defines the different types of objects.
#[derive(Finalize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ObjectKind {
    Function,
    Array,
    String,
    Symbol,
    Error,
    Ordinary,
    Boolean,
    Number,
    BigInt,
}

impl Display for ObjectKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Function => "Function",
                Self::Array => "Array",
                Self::String => "String",
                Self::Symbol => "Symbol",
                Self::Error => "Error",
                Self::Ordinary => "Ordinary",
                Self::Boolean => "Boolean",
                Self::Number => "Number",
                Self::BigInt => "BigInt",
            }
        )
    }
}

/// `Trace` implementation for `ObjectKind`.
///
/// This is indeed safe, but we need to mark this as an empty trace because neither
// `NativeFunctionData` nor Node hold any GC'd objects, but Gc doesn't know that. So we need to
/// signal it manually. `rust-gc` does not have a `Trace` implementation for `fn(_, _, _)`.
///
/// <https://github.com/Manishearth/rust-gc/blob/master/gc/src/trace.rs>
/// Waiting on <https://github.com/Manishearth/rust-gc/issues/87> until we can derive Copy
unsafe impl Trace for ObjectKind {
    unsafe_empty_trace!();
}

/// Create a new object.
pub fn make_object(_: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    if let Some(arg) = args.get(0) {
        if !arg.is_null_or_undefined() {
            return Ok(Value::object(Object::from(arg).unwrap()));
        }
    }
    let global = &ctx.realm.global_obj;

    let object = Value::new_object(Some(global));

    Ok(object)
}

/// Get the `prototype` of an object.
pub fn get_prototype_of(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object");
    Ok(obj.get_field(INSTANCE_PROTOTYPE))
}

/// Set the `prototype` of an object.
pub fn set_prototype_of(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object").clone();
    let proto = args.get(1).expect("Cannot get object").clone();
    obj.set_internal_slot(INSTANCE_PROTOTYPE, proto);
    Ok(obj)
}

/// Define a property in an object
pub fn define_property(_: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object");
    let prop = ctx.to_string(args.get(1).expect("Cannot get object"))?;
    let desc = Property::from(args.get(2).expect("Cannot get object"));
    obj.set_property(prop, desc);
    Ok(Value::undefined())
}

/// `Object.prototype.toString()`
///
/// This method returns a string representing the object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-object.prototype.tostring
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/toString
pub fn to_string(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(this.to_string()))
}

/// `Object.prototype.hasOwnPrototype( property )`
///
/// The method returns a boolean indicating whether the object has the specified property
/// as its own property (as opposed to inheriting it).
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-object.prototype.hasownproperty
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/hasOwnProperty
pub fn has_own_property(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let prop = if args.is_empty() {
        None
    } else {
        Some(ctx.to_string(args.get(0).expect("Cannot get object"))?)
    };
    let own_property = this
        .as_object()
        .as_deref()
        .expect("Cannot get THIS object")
        .get_own_property(&Value::string(&prop.expect("cannot get prop")));
    if own_property.is_none() {
        Ok(Value::from(false))
    } else {
        Ok(Value::from(true))
    }
}

/// Create a new `Object` object.
pub fn create(global: &Value) -> Value {
    let prototype = Value::new_object(None);

    make_builtin_fn(has_own_property, "hasOwnProperty", &prototype, 0);
    make_builtin_fn(to_string, "toString", &prototype, 0);

    let object = make_constructor_fn("Object", 1, make_object, global, prototype, true);

    object.set_field("length", Value::from(1));
    make_builtin_fn(set_prototype_of, "setPrototypeOf", &object, 2);
    make_builtin_fn(get_prototype_of, "getPrototypeOf", &object, 1);
    make_builtin_fn(define_property, "defineProperty", &object, 3);

    object
}

/// Initialise the `Object` object on the global object.
#[inline]
pub fn init(global: &Value) {
    global.set_field("Object", create(global));
}
