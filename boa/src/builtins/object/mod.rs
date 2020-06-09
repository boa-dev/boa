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
        value::{ResultValue, Value, ValueData},
        BigInt, Symbol,
    },
    exec::Interpreter,
    BoaProfiler,
};
use gc::{Finalize, Trace};
use rustc_hash::FxHashMap;
use std::{
    fmt::{Debug, Display, Error, Formatter},
    ops::Deref,
};

use super::function::{make_builtin_fn, make_constructor_fn};
pub use internal_state::{InternalState, InternalStateCell};

pub mod internal_methods;
mod internal_state;

#[cfg(test)]
mod tests;

/// Static `prototype`, usually set on constructors as a key to point to their respective prototype object.
pub static PROTOTYPE: &str = "prototype";

/// Static `__proto__`, usually set on Object instances as a key to point to their respective prototype object.
pub static INSTANCE_PROTOTYPE: &str = "__proto__";

/// The internal representation of an JavaScript object.
#[derive(Debug, Trace, Finalize, Clone)]
pub struct Object {
    /// The type of the object.
    pub data: ObjectData,
    /// Internal Slots
    internal_slots: FxHashMap<String, Value>,
    /// Properties
    properties: FxHashMap<String, Property>,
    /// Symbol Properties
    symbol_properties: FxHashMap<u32, Property>,
    /// Some rust object that stores internal state
    state: Option<InternalStateCell>,
}

/// Defines the different types of objects.
#[derive(Debug, Trace, Finalize, Clone)]
pub enum ObjectData {
    Array,
    BigInt(BigInt),
    Boolean(bool),
    Function(Function),
    String(String),
    Number(f64),
    Symbol(Symbol),
    Error,
    Ordinary,
}

impl Display for ObjectData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Function(_) => "Function",
                Self::Array => "Array",
                Self::String(_) => "String",
                Self::Symbol(_) => "Symbol",
                Self::Error => "Error",
                Self::Ordinary => "Ordinary",
                Self::Boolean(_) => "Boolean",
                Self::Number(_) => "Number",
                Self::BigInt(_) => "BigInt",
            }
        )
    }
}

impl Default for Object {
    /// Return a new ObjectData struct, with `kind` set to Ordinary
    fn default() -> Self {
        let mut object = Self {
            data: ObjectData::Ordinary,
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            state: None,
        };

        object.set_internal_slot("extensible", Value::from(true));
        object
    }
}

impl Object {
    pub fn new() -> Self {
        Default::default()
    }

    /// Return a new ObjectData struct, with `kind` set to Ordinary
    pub fn function(function: Function) -> Self {
        let _timer = BoaProfiler::global().start_event("Object::Function", "object");

        let mut object = Self {
            data: ObjectData::Function(function),
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            state: None,
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

    /// Return a new Boolean object whose `[[BooleanData]]` internal slot is set to argument.
    pub fn boolean(value: bool) -> Self {
        Self {
            data: ObjectData::Boolean(value),
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            state: None,
        }
    }

    /// Return a new `Number` object whose `[[NumberData]]` internal slot is set to argument.
    pub fn number(value: f64) -> Self {
        Self {
            data: ObjectData::Number(value),
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            state: None,
        }
    }

    /// Return a new `String` object whose `[[StringData]]` internal slot is set to argument.
    pub fn string(value: String) -> Self {
        Self {
            data: ObjectData::String(value),
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            state: None,
        }
    }

    /// Return a new `BigInt` object whose `[[BigIntData]]` internal slot is set to argument.
    pub fn bigint(value: BigInt) -> Self {
        Self {
            data: ObjectData::BigInt(value),
            internal_slots: FxHashMap::default(),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            state: None,
        }
    }

    /// Converts the `Value` to an `Object` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-toobject
    pub fn from(value: &Value) -> Result<Self, ()> {
        match *value.data() {
            ValueData::Boolean(a) => Ok(Self::boolean(a)),
            ValueData::Rational(a) => Ok(Self::number(a)),
            ValueData::Integer(a) => Ok(Self::number(f64::from(a))),
            ValueData::String(ref a) => Ok(Self::string(a.clone())),
            ValueData::BigInt(ref bigint) => Ok(Self::bigint(bigint.clone())),
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
    #[inline]
    pub fn is_callable(&self) -> bool {
        matches!(self.data, ObjectData::Function(ref f) if f.is_callable())
    }

    /// It determines if Object is a function object with a [[Construct]] internal method.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isconstructor
    #[inline]
    pub fn is_constructable(&self) -> bool {
        matches!(self.data, ObjectData::Function(ref f) if f.is_constructable())
    }

    /// Checks if it an `Array` object.
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self.data, ObjectData::Array)
    }

    #[inline]
    pub fn as_array(&self) -> Option<()> {
        match self.data {
            ObjectData::Array => Some(()),
            _ => None,
        }
    }

    /// Checks if it a `String` object.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self.data, ObjectData::String(_))
    }

    #[inline]
    pub fn as_string(&self) -> Option<&String> {
        match self.data {
            ObjectData::String(ref string) => Some(string),
            _ => None,
        }
    }

    /// Checks if it a `Function` object.
    #[inline]
    pub fn is_function(&self) -> bool {
        matches!(self.data, ObjectData::Function(_))
    }

    #[inline]
    pub fn as_function(&self) -> Option<&Function> {
        match self.data {
            ObjectData::Function(ref function) => Some(function),
            _ => None,
        }
    }

    /// Checks if it a Symbol object.
    #[inline]
    pub fn is_symbol(&self) -> bool {
        matches!(self.data, ObjectData::Symbol(_))
    }

    #[inline]
    pub fn as_symbol(&self) -> Option<&Symbol> {
        match self.data {
            ObjectData::Symbol(ref symbol) => Some(symbol),
            _ => None,
        }
    }

    /// Checks if it an Error object.
    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(self.data, ObjectData::Error)
    }

    #[inline]
    pub fn as_error(&self) -> Option<()> {
        match self.data {
            ObjectData::Error => Some(()),
            _ => None,
        }
    }

    /// Checks if it a Boolean object.
    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(self.data, ObjectData::Boolean(_))
    }

    #[inline]
    pub fn as_boolean(&self) -> Option<bool> {
        match self.data {
            ObjectData::Boolean(boolean) => Some(boolean),
            _ => None,
        }
    }

    /// Checks if it a `Number` object.
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self.data, ObjectData::Number(_))
    }

    #[inline]
    pub fn as_number(&self) -> Option<f64> {
        match self.data {
            ObjectData::Number(number) => Some(number),
            _ => None,
        }
    }

    /// Checks if it a `BigInt` object.
    #[inline]
    pub fn is_bigint(&self) -> bool {
        matches!(self.data, ObjectData::BigInt(_))
    }

    #[inline]
    pub fn as_bigint(&self) -> Option<&BigInt> {
        match self.data {
            ObjectData::BigInt(ref bigint) => Some(bigint),
            _ => None,
        }
    }

    /// Checks if it an ordinary object.
    #[inline]
    pub fn is_ordinary(&self) -> bool {
        matches!(self.data, ObjectData::Ordinary)
    }

    #[inline]
    pub fn internal_slots(&self) -> &FxHashMap<String, Value> {
        &self.internal_slots
    }

    #[inline]
    pub fn internal_slots_mut(&mut self) -> &mut FxHashMap<String, Value> {
        &mut self.internal_slots
    }

    #[inline]
    pub fn properties(&self) -> &FxHashMap<String, Property> {
        &self.properties
    }

    #[inline]
    pub fn properties_mut(&mut self) -> &mut FxHashMap<String, Property> {
        &mut self.properties
    }

    #[inline]
    pub fn symbol_properties(&self) -> &FxHashMap<u32, Property> {
        &self.symbol_properties
    }

    #[inline]
    pub fn symbol_properties_mut(&mut self) -> &mut FxHashMap<u32, Property> {
        &mut self.symbol_properties
    }

    #[inline]
    pub fn state(&self) -> &Option<InternalStateCell> {
        &self.state
    }

    #[inline]
    pub fn state_mut(&mut self) -> &mut Option<InternalStateCell> {
        &mut self.state
    }
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

    make_builtin_fn(set_prototype_of, "setPrototypeOf", &object, 2);
    make_builtin_fn(get_prototype_of, "getPrototypeOf", &object, 1);
    make_builtin_fn(define_property, "defineProperty", &object, 3);

    object
}

/// Initialise the `Object` object on the global object.
#[inline]
pub fn init(global: &Value) {
    let _timer = BoaProfiler::global().start_event("object", "init");

    let object = create(global);
    global
        .as_object_mut()
        .unwrap()
        .insert_field("Object", object);
}
