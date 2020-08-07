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
        map::ordered_map::OrderedMap,
        property::Property,
        value::{RcBigInt, RcString, RcSymbol, ResultValue, Value},
        BigInt, Date, RegExp,
    },
    exec::Interpreter,
    BoaProfiler,
};
use gc::{Finalize, Trace};
use rustc_hash::FxHashMap;
use std::fmt::{Debug, Display, Error, Formatter};

use super::function::{make_builtin_fn, make_constructor_fn};
use crate::builtins::value::same_value;
pub use internal_state::{InternalState, InternalStateCell};

pub mod gcobject;
pub mod internal_methods;
mod internal_state;

pub use gcobject::GcObject;

#[cfg(test)]
mod tests;

/// Static `prototype`, usually set on constructors as a key to point to their respective prototype object.
pub static PROTOTYPE: &str = "prototype";

// /// Static `__proto__`, usually set on Object instances as a key to point to their respective prototype object.
// pub static INSTANCE_PROTOTYPE: &str = "__proto__";

/// The internal representation of an JavaScript object.
#[derive(Trace, Finalize, Clone)]
pub struct Object {
    /// The type of the object.
    pub data: ObjectData,
    /// Properties
    properties: FxHashMap<RcString, Property>,
    /// Symbol Properties
    symbol_properties: FxHashMap<u32, Property>,
    /// Instance prototype `__proto__`.
    prototype: Value,
    /// Some rust object that stores internal state
    state: Option<InternalStateCell>,
    /// Whether it can have new properties added to it.
    extensible: bool,
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("data", &self.data)
            .field("properties", &self.properties)
            .field("symbol_properties", &self.symbol_properties)
            .field("state", &self.state)
            .field("extensible", &self.extensible)
            .finish()
    }
}

/// Defines the different types of objects.
#[derive(Debug, Trace, Finalize, Clone)]
pub enum ObjectData {
    Array,
    Map(OrderedMap<Value, Value>),
    RegExp(RegExp),
    BigInt(RcBigInt),
    Boolean(bool),
    Function(Function),
    String(RcString),
    Number(f64),
    Symbol(RcSymbol),
    Error,
    Ordinary,
    Date(Date),
    Global,
}

impl Display for ObjectData {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Array => "Array",
                Self::Function(_) => "Function",
                Self::RegExp(_) => "RegExp",
                Self::Map(_) => "Map",
                Self::String(_) => "String",
                Self::Symbol(_) => "Symbol",
                Self::Error => "Error",
                Self::Ordinary => "Ordinary",
                Self::Boolean(_) => "Boolean",
                Self::Number(_) => "Number",
                Self::BigInt(_) => "BigInt",
                Self::Date(_) => "Date",
                Self::Global => "Global",
            }
        )
    }
}

impl Default for Object {
    /// Return a new ObjectData struct, with `kind` set to Ordinary
    #[inline]
    fn default() -> Self {
        Self {
            data: ObjectData::Ordinary,
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
            state: None,
            extensible: true,
        }
    }
}

impl Object {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Return a new ObjectData struct, with `kind` set to Ordinary
    pub fn function(function: Function, prototype: Value) -> Self {
        let _timer = BoaProfiler::global().start_event("Object::Function", "object");

        Self {
            data: ObjectData::Function(function),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype,
            state: None,
            extensible: true,
        }
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
        obj.prototype = proto;
        obj
    }

    /// Return a new Boolean object whose `[[BooleanData]]` internal slot is set to argument.
    pub fn boolean(value: bool) -> Self {
        Self {
            data: ObjectData::Boolean(value),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
            state: None,
            extensible: true,
        }
    }

    /// Return a new `Number` object whose `[[NumberData]]` internal slot is set to argument.
    pub fn number(value: f64) -> Self {
        Self {
            data: ObjectData::Number(value),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
            state: None,
            extensible: true,
        }
    }

    /// Return a new `String` object whose `[[StringData]]` internal slot is set to argument.
    pub fn string<S>(value: S) -> Self
    where
        S: Into<RcString>,
    {
        Self {
            data: ObjectData::String(value.into()),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
            state: None,
            extensible: true,
        }
    }

    /// Return a new `BigInt` object whose `[[BigIntData]]` internal slot is set to argument.
    pub fn bigint(value: RcBigInt) -> Self {
        Self {
            data: ObjectData::BigInt(value),
            properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
            state: None,
            extensible: true,
        }
    }

    /// Converts the `Value` to an `Object` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-toobject
    pub fn from(value: &Value) -> Result<Self, ()> {
        match *value {
            Value::Boolean(a) => Ok(Self::boolean(a)),
            Value::Rational(a) => Ok(Self::number(a)),
            Value::Integer(a) => Ok(Self::number(f64::from(a))),
            Value::String(ref a) => Ok(Self::string(a.clone())),
            Value::BigInt(ref bigint) => Ok(Self::bigint(bigint.clone())),
            Value::Object(ref obj) => Ok(obj.borrow().clone()),
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

    /// Checks if it is a `Map` object.pub
    #[inline]
    pub fn is_map(&self) -> bool {
        matches!(self.data, ObjectData::Map(_))
    }

    #[inline]
    pub fn as_map_ref(&self) -> Option<&OrderedMap<Value, Value>> {
        match self.data {
            ObjectData::Map(ref map) => Some(map),
            _ => None,
        }
    }

    #[inline]
    pub fn as_map_mut(&mut self) -> Option<&mut OrderedMap<Value, Value>> {
        match &mut self.data {
            ObjectData::Map(map) => Some(map),
            _ => None,
        }
    }

    /// Checks if it a `String` object.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self.data, ObjectData::String(_))
    }

    #[inline]
    pub fn as_string(&self) -> Option<RcString> {
        match self.data {
            ObjectData::String(ref string) => Some(string.clone()),
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
    pub fn as_symbol(&self) -> Option<RcSymbol> {
        match self.data {
            ObjectData::Symbol(ref symbol) => Some(symbol.clone()),
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

    /// Checks if it a `RegExp` object.
    #[inline]
    pub fn is_regexp(&self) -> bool {
        matches!(self.data, ObjectData::RegExp(_))
    }

    #[inline]
    pub fn as_regexp(&self) -> Option<&RegExp> {
        match self.data {
            ObjectData::RegExp(ref regexp) => Some(regexp),
            _ => None,
        }
    }

    /// Checks if it an ordinary object.
    #[inline]
    pub fn is_ordinary(&self) -> bool {
        matches!(self.data, ObjectData::Ordinary)
    }

    #[inline]
    pub fn properties(&self) -> &FxHashMap<RcString, Property> {
        &self.properties
    }

    #[inline]
    pub fn properties_mut(&mut self) -> &mut FxHashMap<RcString, Property> {
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

    pub fn prototype(&self) -> &Value {
        &self.prototype
    }

    pub fn set_prototype(&mut self, prototype: Value) {
        assert!(prototype.is_null() || prototype.is_object());
        self.prototype = prototype
    }
}

/// Create a new object.
pub fn make_object(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    if let Some(arg) = args.get(0) {
        if !arg.is_null_or_undefined() {
            return Ok(Value::object(Object::from(arg).unwrap()));
        }
    }
    let global = &ctx.realm.global_obj;

    let object = Value::new_object(Some(global));

    Ok(object)
}

/// `Object.create( proto, [propertiesObject] )`
///
/// Creates a new object from the provided prototype.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-object.create
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/create
pub fn create(_: &Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    let prototype = args.get(0).cloned().unwrap_or_else(Value::undefined);
    let properties = args.get(1).cloned().unwrap_or_else(Value::undefined);

    if properties != Value::Undefined {
        unimplemented!("propertiesObject argument of Object.create")
    }

    match prototype {
        Value::Object(_) | Value::Null => Ok(Value::new_object_from_prototype(
            prototype,
            ObjectData::Ordinary,
        )),
        _ => interpreter.throw_type_error(format!(
            "Object prototype may only be an Object or null: {}",
            prototype.display()
        )),
    }
}

/// Uses the SameValue algorithm to check equality of objects
pub fn is(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let x = args.get(0).cloned().unwrap_or_else(Value::undefined);
    let y = args.get(1).cloned().unwrap_or_else(Value::undefined);

    Ok(same_value(&x, &y).into())
}

/// Get the `prototype` of an object.
pub fn get_prototype_of(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object");
    Ok(obj
        .as_object()
        .map_or_else(Value::undefined, |object| object.prototype.clone()))
}

/// Set the `prototype` of an object.
pub fn set_prototype_of(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object").clone();
    let proto = args.get(1).expect("Cannot get object").clone();
    obj.as_object_mut().unwrap().prototype = proto;
    Ok(obj)
}

/// Define a property in an object
pub fn define_property(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("Cannot get object");
    let prop = args.get(1).expect("Cannot get object").to_string(ctx)?;
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
pub fn to_string(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    // FIXME: it should not display the object.
    Ok(Value::from(this.display().to_string()))
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
pub fn has_own_property(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let prop = if args.is_empty() {
        None
    } else {
        Some(args.get(0).expect("Cannot get object").to_string(ctx)?)
    };
    let own_property = this
        .as_object()
        .as_deref()
        .expect("Cannot get THIS object")
        .get_own_property(&prop.expect("cannot get prop").into());
    if own_property.is_none() {
        Ok(Value::from(false))
    } else {
        Ok(Value::from(true))
    }
}

pub fn property_is_enumerable(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    let key = match args.get(0) {
        None => return Ok(Value::from(false)),
        Some(key) => key,
    };

    let key = key.to_property_key(ctx)?;
    let own_property = this.to_object(ctx).map(|obj| {
        obj.as_object()
            .expect("Unable to deref object")
            .get_own_property(&key)
    });

    Ok(own_property.map_or(Value::from(false), |own_prop| {
        Value::from(own_prop.enumerable_or(false))
    }))
}

/// Initialise the `Object` object on the global object.
#[inline]
pub fn init(interpreter: &mut Interpreter) -> (&'static str, Value) {
    let global = interpreter.global();
    let _timer = BoaProfiler::global().start_event("object", "init");

    let prototype = Value::new_object(None);

    make_builtin_fn(
        has_own_property,
        "hasOwnProperty",
        &prototype,
        0,
        interpreter,
    );
    make_builtin_fn(
        property_is_enumerable,
        "propertyIsEnumerable",
        &prototype,
        0,
        interpreter,
    );
    make_builtin_fn(to_string, "toString", &prototype, 0, interpreter);

    let object = make_constructor_fn("Object", 1, make_object, global, prototype, true, true);

    // static methods of the builtin Object
    make_builtin_fn(create, "create", &object, 2, interpreter);
    make_builtin_fn(set_prototype_of, "setPrototypeOf", &object, 2, interpreter);
    make_builtin_fn(get_prototype_of, "getPrototypeOf", &object, 1, interpreter);
    make_builtin_fn(define_property, "defineProperty", &object, 3, interpreter);
    make_builtin_fn(is, "is", &object, 2, interpreter);

    ("Object", object)
}
