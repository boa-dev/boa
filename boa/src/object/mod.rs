//! This module implements the Rust representation of a JavaScript object.

use crate::{
    builtins::{function::Function, map::ordered_map::OrderedMap, BigInt, Date, RegExp},
    property::{Property, PropertyKey},
    value::{RcBigInt, RcString, RcSymbol, Value},
    BoaProfiler,
};
use gc::{Finalize, Trace};
use rustc_hash::FxHashMap;
use std::fmt::{Debug, Display, Error, Formatter};
use std::{any::Any, result::Result as StdResult};

mod gcobject;
mod internal_methods;
mod iter;

pub use gcobject::{GcObject, Ref, RefMut};
pub use iter::*;

/// Static `prototype`, usually set on constructors as a key to point to their respective prototype object.
pub static PROTOTYPE: &str = "prototype";

/// This trait allows Rust types to be passed around as objects.
///
/// This is automatically implemented, when a type implements `Debug`, `Any` and `Trace`.
pub trait NativeObject: Debug + Any + Trace {
    /// Convert the Rust type which implements `NativeObject` to a `&dyn Any`.
    fn as_any(&self) -> &dyn Any;

    /// Convert the Rust type which implements `NativeObject` to a `&mut dyn Any`.
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<T: Any + Debug + Trace> NativeObject for T {
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

/// The internal representation of an JavaScript object.
#[derive(Debug, Trace, Finalize)]
pub struct Object {
    /// The type of the object.
    pub data: ObjectData,
    indexed_properties: FxHashMap<u32, Property>,
    /// Properties
    string_properties: FxHashMap<RcString, Property>,
    /// Symbol Properties
    symbol_properties: FxHashMap<RcSymbol, Property>,
    /// Instance prototype `__proto__`.
    prototype: Value,
    /// Whether it can have new properties added to it.
    extensible: bool,
}

/// Defines the different types of objects.
#[derive(Debug, Trace, Finalize)]
pub enum ObjectData {
    Array,
    Map(OrderedMap<Value, Value>),
    RegExp(Box<RegExp>),
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
    NativeObject(Box<dyn NativeObject>),
}

impl Display for ObjectData {
    fn fmt(&self, f: &mut Formatter<'_>) -> StdResult<(), Error> {
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
                Self::NativeObject(_) => "NativeObject",
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
            indexed_properties: FxHashMap::default(),
            string_properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
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
            indexed_properties: FxHashMap::default(),
            string_properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype,
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
            indexed_properties: FxHashMap::default(),
            string_properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
            extensible: true,
        }
    }

    /// Return a new `Number` object whose `[[NumberData]]` internal slot is set to argument.
    pub fn number(value: f64) -> Self {
        Self {
            data: ObjectData::Number(value),
            indexed_properties: FxHashMap::default(),
            string_properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
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
            indexed_properties: FxHashMap::default(),
            string_properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
            extensible: true,
        }
    }

    /// Return a new `BigInt` object whose `[[BigIntData]]` internal slot is set to argument.
    pub fn bigint(value: RcBigInt) -> Self {
        Self {
            data: ObjectData::BigInt(value),
            indexed_properties: FxHashMap::default(),
            string_properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
            extensible: true,
        }
    }

    /// Create a new native object of type `T`.
    pub fn native_object<T>(value: T) -> Self
    where
        T: NativeObject,
    {
        Self {
            data: ObjectData::NativeObject(Box::new(value)),
            indexed_properties: FxHashMap::default(),
            string_properties: FxHashMap::default(),
            symbol_properties: FxHashMap::default(),
            prototype: Value::null(),
            extensible: true,
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

    pub fn prototype_instance(&self) -> &Value {
        &self.prototype
    }

    pub fn set_prototype_instance(&mut self, prototype: Value) {
        assert!(prototype.is_null() || prototype.is_object());
        self.prototype = prototype
    }

    /// Returns `true` if it holds an Rust type that implements `NativeObject`.
    pub fn is_native_object(&self) -> bool {
        matches!(self.data, ObjectData::NativeObject(_))
    }

    /// Reeturn `true` if it is a native object and the native type is `T`.
    pub fn is<T>(&self) -> bool
    where
        T: NativeObject,
    {
        use std::ops::Deref;
        match self.data {
            ObjectData::NativeObject(ref object) => object.deref().as_any().is::<T>(),
            _ => false,
        }
    }

    /// Downcast a reference to the object,
    /// if the object is type native object type `T`.
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: NativeObject,
    {
        use std::ops::Deref;
        match self.data {
            ObjectData::NativeObject(ref object) => object.deref().as_any().downcast_ref::<T>(),
            _ => None,
        }
    }

    /// Downcast a mutable reference to the object,
    /// if the object is type native object type `T`.
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: NativeObject,
    {
        use std::ops::DerefMut;
        match self.data {
            ObjectData::NativeObject(ref mut object) => {
                object.deref_mut().as_mut_any().downcast_mut::<T>()
            }
            _ => None,
        }
    }
}
