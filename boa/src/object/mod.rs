//! This module implements the Rust representation of a JavaScript object.

use crate::{
    builtins::{
        array::array_iterator::ArrayIterator,
        function::{Function, NativeFunction},
        map::map_iterator::MapIterator,
        map::ordered_map::OrderedMap,
        regexp::regexp_string_iterator::RegExpStringIterator,
        set::ordered_set::OrderedSet,
        set::set_iterator::SetIterator,
        string::string_iterator::StringIterator,
        Date, RegExp,
    },
    context::StandardConstructor,
    gc::{Finalize, Trace},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    BoaProfiler, Context, JsBigInt, JsResult, JsString, JsSymbol, JsValue,
};
use std::{
    any::Any,
    fmt::{self, Debug, Display},
    ops::{Deref, DerefMut},
};

#[cfg(test)]
mod tests;

mod gcobject;
pub(crate) mod internal_methods;
mod operations;
mod property_map;

use crate::builtins::object::for_in_iterator::ForInIterator;
pub use gcobject::{JsObject, RecursionLimiter, Ref, RefMut};
use internal_methods::InternalObjectMethods;
pub use operations::IntegrityLevel;
pub use property_map::*;

use self::internal_methods::{
    array::ARRAY_EXOTIC_INTERNAL_METHODS, string::STRING_EXOTIC_INTERNAL_METHODS,
    ORDINARY_INTERNAL_METHODS,
};

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
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    #[inline]
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

/// The internal representation of an JavaScript object.
#[derive(Debug, Trace, Finalize)]
pub struct Object {
    /// The type of the object.
    pub data: ObjectData,
    properties: PropertyMap,
    /// Instance prototype `__proto__`.
    prototype: JsValue,
    /// Whether it can have new properties added to it.
    extensible: bool,
}

/// Defines the kind of an object and its internal methods
#[derive(Trace, Finalize)]
pub struct ObjectData {
    kind: ObjectKind,
    internal_methods: &'static InternalObjectMethods,
}

/// Defines the different types of objects.
#[derive(Debug, Trace, Finalize)]
pub enum ObjectKind {
    Array,
    ArrayIterator(ArrayIterator),
    Map(OrderedMap<JsValue>),
    MapIterator(MapIterator),
    RegExp(Box<RegExp>),
    RegExpStringIterator(RegExpStringIterator),
    BigInt(JsBigInt),
    Boolean(bool),
    ForInIterator(ForInIterator),
    Function(Function),
    Set(OrderedSet<JsValue>),
    SetIterator(SetIterator),
    String(JsString),
    StringIterator(StringIterator),
    Number(f64),
    Symbol(JsSymbol),
    Error,
    Ordinary,
    Date(Date),
    Global,
    NativeObject(Box<dyn NativeObject>),
}

impl ObjectData {
    /// Create the `Array` object data and reference its exclusive internal methods
    pub fn array() -> Self {
        Self {
            kind: ObjectKind::Array,
            internal_methods: &ARRAY_EXOTIC_INTERNAL_METHODS,
        }
    }

    /// Create the `ArrayIterator` object data
    pub fn array_iterator(array_iterator: ArrayIterator) -> Self {
        Self {
            kind: ObjectKind::ArrayIterator(array_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Map` object data
    pub fn map(map: OrderedMap<JsValue>) -> Self {
        Self {
            kind: ObjectKind::Map(map),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `MapIterator` object data
    pub fn map_iterator(map_iterator: MapIterator) -> Self {
        Self {
            kind: ObjectKind::MapIterator(map_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `RegExp` object data
    pub fn reg_exp(reg_exp: Box<RegExp>) -> Self {
        Self {
            kind: ObjectKind::RegExp(reg_exp),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `RegExpStringIterator` object data
    pub fn reg_exp_string_iterator(reg_exp_string_iterator: RegExpStringIterator) -> Self {
        Self {
            kind: ObjectKind::RegExpStringIterator(reg_exp_string_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `BigInt` object data
    pub fn big_int(big_int: JsBigInt) -> Self {
        Self {
            kind: ObjectKind::BigInt(big_int),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Boolean` object data
    pub fn boolean(boolean: bool) -> Self {
        Self {
            kind: ObjectKind::Boolean(boolean),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `ForInIterator` object data
    pub fn for_in_iterator(for_in_iterator: ForInIterator) -> Self {
        Self {
            kind: ObjectKind::ForInIterator(for_in_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Function` object data
    pub fn function(function: Function) -> Self {
        Self {
            kind: ObjectKind::Function(function),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Set` object data
    pub fn set(set: OrderedSet<JsValue>) -> Self {
        Self {
            kind: ObjectKind::Set(set),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `SetIterator` object data
    pub fn set_iterator(set_iterator: SetIterator) -> Self {
        Self {
            kind: ObjectKind::SetIterator(set_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `String` object data and reference its exclusive internal methods
    pub fn string(string: JsString) -> Self {
        Self {
            kind: ObjectKind::String(string),
            internal_methods: &STRING_EXOTIC_INTERNAL_METHODS,
        }
    }

    /// Create the `StringIterator` object data
    pub fn string_iterator(string_iterator: StringIterator) -> Self {
        Self {
            kind: ObjectKind::StringIterator(string_iterator),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Number` object data
    pub fn number(number: f64) -> Self {
        Self {
            kind: ObjectKind::Number(number),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Symbol` object data
    pub fn symbol(symbol: JsSymbol) -> Self {
        Self {
            kind: ObjectKind::Symbol(symbol),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Error` object data
    pub fn error() -> Self {
        Self {
            kind: ObjectKind::Error,
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Ordinary` object data
    pub fn ordinary() -> Self {
        Self {
            kind: ObjectKind::Ordinary,
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Date` object data
    pub fn date(date: Date) -> Self {
        Self {
            kind: ObjectKind::Date(date),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `Global` object data
    pub fn global() -> Self {
        Self {
            kind: ObjectKind::Global,
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }

    /// Create the `NativeObject` object data
    pub fn native_object(native_object: Box<dyn NativeObject>) -> Self {
        Self {
            kind: ObjectKind::NativeObject(native_object),
            internal_methods: &ORDINARY_INTERNAL_METHODS,
        }
    }
}

impl Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Array => "Array",
                Self::ArrayIterator(_) => "ArrayIterator",
                Self::ForInIterator(_) => "ForInIterator",
                Self::Function(_) => "Function",
                Self::RegExp(_) => "RegExp",
                Self::RegExpStringIterator(_) => "RegExpStringIterator",
                Self::Map(_) => "Map",
                Self::MapIterator(_) => "MapIterator",
                Self::Set(_) => "Set",
                Self::SetIterator(_) => "SetIterator",
                Self::String(_) => "String",
                Self::StringIterator(_) => "StringIterator",
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

impl Debug for ObjectData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectData")
            .field("kind", &self.kind)
            .field("internal_methods", &"internal_methods")
            .finish()
    }
}

impl Default for Object {
    /// Return a new ObjectData struct, with `kind` set to Ordinary
    #[inline]
    fn default() -> Self {
        Self {
            data: ObjectData::ordinary(),
            properties: PropertyMap::default(),
            prototype: JsValue::null(),
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
    #[inline]
    pub fn function(function: Function, prototype: JsValue) -> Self {
        let _timer = BoaProfiler::global().start_event("Object::Function", "object");

        Self {
            data: ObjectData::function(function),
            properties: PropertyMap::default(),
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
    #[inline]
    pub fn create(proto: JsValue) -> Self {
        let mut obj = Self::new();
        obj.prototype = proto;
        obj
    }

    /// Return a new Boolean object whose `[[BooleanData]]` internal slot is set to argument.
    #[inline]
    pub fn boolean(value: bool) -> Self {
        Self {
            data: ObjectData::boolean(value),
            properties: PropertyMap::default(),
            prototype: JsValue::null(),
            extensible: true,
        }
    }

    /// Return a new `Number` object whose `[[NumberData]]` internal slot is set to argument.
    #[inline]
    pub fn number(value: f64) -> Self {
        Self {
            data: ObjectData::number(value),
            properties: PropertyMap::default(),
            prototype: JsValue::null(),
            extensible: true,
        }
    }

    /// Return a new `String` object whose `[[StringData]]` internal slot is set to argument.
    #[inline]
    pub fn string<S>(value: S) -> Self
    where
        S: Into<JsString>,
    {
        Self {
            data: ObjectData::string(value.into()),
            properties: PropertyMap::default(),
            prototype: JsValue::null(),
            extensible: true,
        }
    }

    /// Return a new `BigInt` object whose `[[BigIntData]]` internal slot is set to argument.
    #[inline]
    pub fn bigint(value: JsBigInt) -> Self {
        Self {
            data: ObjectData::big_int(value),
            properties: PropertyMap::default(),
            prototype: JsValue::null(),
            extensible: true,
        }
    }

    /// Create a new native object of type `T`.
    #[inline]
    pub fn native_object<T>(value: T) -> Self
    where
        T: NativeObject,
    {
        Self {
            data: ObjectData::native_object(Box::new(value)),
            properties: PropertyMap::default(),
            prototype: JsValue::null(),
            extensible: true,
        }
    }

    #[inline]
    pub fn kind(&self) -> &ObjectKind {
        &self.data.kind
    }

    /// It determines if Object is a callable function with a `[[Call]]` internal method.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iscallable
    #[inline]
    // todo: functions are not the only objects that are callable.
    // todo: e.g. https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist
    pub fn is_callable(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Function(_),
                ..
            }
        )
    }

    /// It determines if Object is a function object with a `[[Construct]]` internal method.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isconstructor
    #[inline]
    // todo: functions are not the only objects that are constructable.
    // todo: e.g. https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
    pub fn is_constructable(&self) -> bool {
        matches!(self.data, ObjectData{kind: ObjectKind::Function(ref f), ..} if f.is_constructable())
    }

    /// Checks if it an `Array` object.
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Array,
                ..
            }
        )
    }

    #[inline]
    pub fn as_array(&self) -> Option<()> {
        match self.data {
            ObjectData {
                kind: ObjectKind::Array,
                ..
            } => Some(()),
            _ => None,
        }
    }

    /// Checks if it is an `ArrayIterator` object.
    #[inline]
    pub fn is_array_iterator(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::ArrayIterator(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_array_iterator(&self) -> Option<&ArrayIterator> {
        match self.data {
            ObjectData {
                kind: ObjectKind::ArrayIterator(ref iter),
                ..
            } => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn as_array_iterator_mut(&mut self) -> Option<&mut ArrayIterator> {
        match &mut self.data {
            ObjectData {
                kind: ObjectKind::ArrayIterator(iter),
                ..
            } => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string_iterator_mut(&mut self) -> Option<&mut StringIterator> {
        match &mut self.data {
            ObjectData {
                kind: ObjectKind::StringIterator(iter),
                ..
            } => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn as_regexp_string_iterator_mut(&mut self) -> Option<&mut RegExpStringIterator> {
        match &mut self.data {
            ObjectData {
                kind: ObjectKind::RegExpStringIterator(iter),
                ..
            } => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn as_for_in_iterator(&self) -> Option<&ForInIterator> {
        match &self.data {
            ObjectData {
                kind: ObjectKind::ForInIterator(iter),
                ..
            } => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn as_for_in_iterator_mut(&mut self) -> Option<&mut ForInIterator> {
        match &mut self.data {
            ObjectData {
                kind: ObjectKind::ForInIterator(iter),
                ..
            } => Some(iter),
            _ => None,
        }
    }

    /// Checks if it is a `Map` object.pub
    #[inline]
    pub fn is_map(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Map(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_map_ref(&self) -> Option<&OrderedMap<JsValue>> {
        match self.data {
            ObjectData {
                kind: ObjectKind::Map(ref map),
                ..
            } => Some(map),
            _ => None,
        }
    }

    #[inline]
    pub fn as_map_mut(&mut self) -> Option<&mut OrderedMap<JsValue>> {
        match &mut self.data {
            ObjectData {
                kind: ObjectKind::Map(map),
                ..
            } => Some(map),
            _ => None,
        }
    }

    #[inline]
    pub fn is_map_iterator(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::MapIterator(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_map_iterator_ref(&self) -> Option<&MapIterator> {
        match &self.data {
            ObjectData {
                kind: ObjectKind::MapIterator(iter),
                ..
            } => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn as_map_iterator_mut(&mut self) -> Option<&mut MapIterator> {
        match &mut self.data {
            ObjectData {
                kind: ObjectKind::MapIterator(iter),
                ..
            } => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn is_set(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Set(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_set_ref(&self) -> Option<&OrderedSet<JsValue>> {
        match self.data {
            ObjectData {
                kind: ObjectKind::Set(ref set),
                ..
            } => Some(set),
            _ => None,
        }
    }

    #[inline]
    pub fn as_set_mut(&mut self) -> Option<&mut OrderedSet<JsValue>> {
        match &mut self.data {
            ObjectData {
                kind: ObjectKind::Set(set),
                ..
            } => Some(set),
            _ => None,
        }
    }

    #[inline]
    pub fn as_set_iterator_mut(&mut self) -> Option<&mut SetIterator> {
        match &mut self.data {
            ObjectData {
                kind: ObjectKind::SetIterator(iter),
                ..
            } => Some(iter),
            _ => None,
        }
    }

    /// Checks if it a `String` object.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::String(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_string(&self) -> Option<JsString> {
        match self.data {
            ObjectData {
                kind: ObjectKind::String(ref string),
                ..
            } => Some(string.clone()),
            _ => None,
        }
    }

    /// Checks if it a `Function` object.
    #[inline]
    pub fn is_function(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Function(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_function(&self) -> Option<&Function> {
        match self.data {
            ObjectData {
                kind: ObjectKind::Function(ref function),
                ..
            } => Some(function),
            _ => None,
        }
    }

    /// Checks if it a Symbol object.
    #[inline]
    pub fn is_symbol(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Symbol(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_symbol(&self) -> Option<JsSymbol> {
        match self.data {
            ObjectData {
                kind: ObjectKind::Symbol(ref symbol),
                ..
            } => Some(symbol.clone()),
            _ => None,
        }
    }

    /// Checks if it an Error object.
    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Error,
                ..
            }
        )
    }

    #[inline]
    pub fn as_error(&self) -> Option<()> {
        match self.data {
            ObjectData {
                kind: ObjectKind::Error,
                ..
            } => Some(()),
            _ => None,
        }
    }

    /// Checks if it a Boolean object.
    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Boolean(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_boolean(&self) -> Option<bool> {
        match self.data {
            ObjectData {
                kind: ObjectKind::Boolean(boolean),
                ..
            } => Some(boolean),
            _ => None,
        }
    }

    /// Checks if it a `Number` object.
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Number(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_number(&self) -> Option<f64> {
        match self.data {
            ObjectData {
                kind: ObjectKind::Number(number),
                ..
            } => Some(number),
            _ => None,
        }
    }

    /// Checks if it a `BigInt` object.
    #[inline]
    pub fn is_bigint(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::BigInt(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_bigint(&self) -> Option<&JsBigInt> {
        match self.data {
            ObjectData {
                kind: ObjectKind::BigInt(ref bigint),
                ..
            } => Some(bigint),
            _ => None,
        }
    }

    #[inline]
    pub fn is_date(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Date(_),
                ..
            }
        )
    }

    pub fn as_date(&self) -> Option<&Date> {
        match self.data {
            ObjectData {
                kind: ObjectKind::Date(ref date),
                ..
            } => Some(date),
            _ => None,
        }
    }

    /// Checks if it a `RegExp` object.
    #[inline]
    pub fn is_regexp(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::RegExp(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_regexp(&self) -> Option<&RegExp> {
        match self.data {
            ObjectData {
                kind: ObjectKind::RegExp(ref regexp),
                ..
            } => Some(regexp),
            _ => None,
        }
    }

    /// Checks if it an ordinary object.
    #[inline]
    pub fn is_ordinary(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::Ordinary,
                ..
            }
        )
    }

    #[inline]
    pub fn prototype_instance(&self) -> &JsValue {
        &self.prototype
    }

    /// Sets the prototype instance of the object.
    ///
    /// [More information][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-invariants-of-the-essential-internal-methods
    #[inline]
    #[track_caller]
    pub fn set_prototype_instance(&mut self, prototype: JsValue) -> bool {
        assert!(prototype.is_null() || prototype.is_object());
        if self.extensible {
            self.prototype = prototype;
            true
        } else {
            // If target is non-extensible, [[SetPrototypeOf]] must return false
            // unless V is the SameValue as the target's observed [[GetPrototypeOf]] value.
            JsValue::same_value(&prototype, &self.prototype)
        }
    }

    /// Similar to `Value::new_object`, but you can pass a prototype to create from, plus a kind
    #[inline]
    pub fn with_prototype(proto: JsValue, data: ObjectData) -> Object {
        let mut object = Object::new();
        object.data = data;
        object.set_prototype_instance(proto);
        object
    }

    /// Returns `true` if it holds an Rust type that implements `NativeObject`.
    #[inline]
    pub fn is_native_object(&self) -> bool {
        matches!(
            self.data,
            ObjectData {
                kind: ObjectKind::NativeObject(_),
                ..
            }
        )
    }

    #[inline]
    pub fn as_native_object(&self) -> Option<&dyn NativeObject> {
        match self.data {
            ObjectData {
                kind: ObjectKind::NativeObject(ref object),
                ..
            } => Some(object.as_ref()),
            _ => None,
        }
    }

    /// Reeturn `true` if it is a native object and the native type is `T`.
    #[inline]
    pub fn is<T>(&self) -> bool
    where
        T: NativeObject,
    {
        match self.data {
            ObjectData {
                kind: ObjectKind::NativeObject(ref object),
                ..
            } => object.deref().as_any().is::<T>(),
            _ => false,
        }
    }

    /// Downcast a reference to the object,
    /// if the object is type native object type `T`.
    #[inline]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: NativeObject,
    {
        match self.data {
            ObjectData {
                kind: ObjectKind::NativeObject(ref object),
                ..
            } => object.deref().as_any().downcast_ref::<T>(),
            _ => None,
        }
    }

    /// Downcast a mutable reference to the object,
    /// if the object is type native object type `T`.
    #[inline]
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: NativeObject,
    {
        match self.data {
            ObjectData {
                kind: ObjectKind::NativeObject(ref mut object),
                ..
            } => object.deref_mut().as_mut_any().downcast_mut::<T>(),
            _ => None,
        }
    }

    #[inline]
    pub fn properties(&self) -> &PropertyMap {
        &self.properties
    }

    /// Helper function for property insertion.
    #[inline]
    pub(crate) fn insert<K, P>(&mut self, key: K, property: P) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.properties.insert(key.into(), property.into())
    }

    /// Helper function for property removal.
    #[inline]
    pub(crate) fn remove(&mut self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        self.properties.remove(key)
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name that a `Some` is returned
    /// with that field, otherwise None is retuned.
    #[inline]
    pub fn insert_property<K, P>(&mut self, key: K, property: P) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.insert(key, property)
    }
}

/// The functions binding.
///
/// Specifies what is the name of the function object (`name` property),
/// and the binding name of the function object which can be different
/// from the function name.
///
/// The only way to construct this is with the `From` trait.
///
/// There are two implementations:
///  - From a single type `T` which implements `Into<FunctionBinding>` which sets the binding
/// name and the function name to the same value
///  - From a tuple `(B: Into<PropertyKey>, N: AsRef<str>)` the `B` is the binding name
/// and the `N` is the function name.
#[derive(Debug, Clone)]
pub struct FunctionBinding {
    binding: PropertyKey,
    name: JsString,
}

impl From<&str> for FunctionBinding {
    #[inline]
    fn from(name: &str) -> Self {
        let name: JsString = name.into();

        Self {
            binding: name.clone().into(),
            name,
        }
    }
}

impl From<String> for FunctionBinding {
    #[inline]
    fn from(name: String) -> Self {
        let name: JsString = name.into();

        Self {
            binding: name.clone().into(),
            name,
        }
    }
}

impl From<JsString> for FunctionBinding {
    #[inline]
    fn from(name: JsString) -> Self {
        Self {
            binding: name.clone().into(),
            name,
        }
    }
}

impl<B, N> From<(B, N)> for FunctionBinding
where
    B: Into<PropertyKey>,
    N: AsRef<str>,
{
    #[inline]
    fn from((binding, name): (B, N)) -> Self {
        Self {
            binding: binding.into(),
            name: name.as_ref().into(),
        }
    }
}

/// Builder for creating native function objects
#[derive(Debug)]
pub struct FunctionBuilder<'context> {
    context: &'context mut Context,
    function: Option<Function>,
    name: JsString,
    length: usize,
}

impl<'context> FunctionBuilder<'context> {
    /// Create a new `FunctionBuilder` for creating a native function.
    #[inline]
    pub fn native(context: &'context mut Context, function: NativeFunction) -> Self {
        Self {
            context,
            function: Some(Function::Native {
                function: function.into(),
                constructable: false,
            }),
            name: JsString::default(),
            length: 0,
        }
    }

    /// Create a new `FunctionBuilder` for creating a closure function.
    #[inline]
    pub fn closure<F>(context: &'context mut Context, function: F) -> Self
    where
        F: Fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue> + Copy + 'static,
    {
        Self {
            context,
            function: Some(Function::Closure {
                function: Box::new(function),
                constructable: false,
            }),
            name: JsString::default(),
            length: 0,
        }
    }

    /// Specify the name property of object function object.
    ///
    /// The default is `""` (empty string).
    #[inline]
    pub fn name<N>(&mut self, name: N) -> &mut Self
    where
        N: AsRef<str>,
    {
        self.name = name.as_ref().into();
        self
    }

    /// Specify the length property of object function object.
    ///
    /// How many arguments this function takes.
    ///
    /// The default is `0`.
    #[inline]
    pub fn length(&mut self, length: usize) -> &mut Self {
        self.length = length;
        self
    }

    /// Specify the whether the object function object can be called with `new` keyword.
    ///
    /// The default is `false`.
    #[inline]
    pub fn constructable(&mut self, yes: bool) -> &mut Self {
        match self.function.as_mut() {
            Some(Function::Native { constructable, .. }) => *constructable = yes,
            Some(Function::Closure { constructable, .. }) => *constructable = yes,
            _ => unreachable!(),
        }
        self
    }

    /// Build the function object.
    #[inline]
    pub fn build(&mut self) -> JsObject {
        let mut function = Object::function(
            self.function.take().unwrap(),
            self.context
                .standard_objects()
                .function_object()
                .prototype()
                .into(),
        );
        let property = PropertyDescriptor::builder()
            .writable(false)
            .enumerable(false)
            .configurable(true);
        function.insert_property("name", property.clone().value(self.name.clone()));
        function.insert_property("length", property.value(self.length));

        JsObject::new(function)
    }

    /// Initializes the `Function.prototype` function object.
    pub(crate) fn build_function_prototype(&mut self, object: &JsObject) {
        let mut object = object.borrow_mut();
        object.data = ObjectData::function(self.function.take().unwrap());
        object.set_prototype_instance(
            self.context
                .standard_objects()
                .object_object()
                .prototype()
                .into(),
        );

        let property = PropertyDescriptor::builder()
            .writable(false)
            .enumerable(false)
            .configurable(true);
        object.insert_property("name", property.clone().value(self.name.clone()));
        object.insert_property("length", property.value(self.length));
    }
}

/// Builder for creating objects with properties.
///
/// # Examples
///
/// ```
/// # use boa::{Context, JsValue, object::ObjectInitializer, property::Attribute};
/// let mut context = Context::new();
/// let object = ObjectInitializer::new(&mut context)
///     .property(
///         "hello",
///         "world",
///         Attribute::all()
///     )
///     .property(
///         1,
///         1,
///         Attribute::all()
///     )
///     .function(|_, _, _| Ok(JsValue::undefined()), "func", 0)
///     .build();
/// ```
///
/// The equivalent in JavaScript would be:
/// ```text
/// let object = {
///     hello: "world",
///     "1": 1,
///     func: function() {}
/// }
/// ```
#[derive(Debug)]
pub struct ObjectInitializer<'context> {
    context: &'context mut Context,
    object: JsObject,
}

impl<'context> ObjectInitializer<'context> {
    /// Create a new `ObjectBuilder`.
    #[inline]
    pub fn new(context: &'context mut Context) -> Self {
        let object = context.construct_object();
        Self { context, object }
    }

    /// Add a function to the object.
    #[inline]
    pub fn function<B>(&mut self, function: NativeFunction, binding: B, length: usize) -> &mut Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = FunctionBuilder::native(self.context, function)
            .name(binding.name)
            .length(length)
            .constructable(false)
            .build();

        self.object.borrow_mut().insert_property(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Add a property to the object.
    #[inline]
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.object.borrow_mut().insert(key, property);
        self
    }

    /// Build the object.
    #[inline]
    pub fn build(&mut self) -> JsObject {
        self.object.clone()
    }
}

/// Builder for creating constructors objects, like `Array`.
pub struct ConstructorBuilder<'context> {
    context: &'context mut Context,
    constructor_function: NativeFunction,
    constructor_object: JsObject,
    prototype: JsObject,
    name: JsString,
    length: usize,
    callable: bool,
    constructable: bool,
    inherit: Option<JsValue>,
}

impl Debug for ConstructorBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConstructorBuilder")
            .field("name", &self.name)
            .field("length", &self.length)
            .field("constructor", &self.constructor_object)
            .field("prototype", &self.prototype)
            .field("inherit", &self.inherit)
            .field("callable", &self.callable)
            .field("constructable", &self.constructable)
            .finish()
    }
}

impl<'context> ConstructorBuilder<'context> {
    /// Create a new `ConstructorBuilder`.
    #[inline]
    pub fn new(context: &'context mut Context, constructor: NativeFunction) -> Self {
        Self {
            context,
            constructor_function: constructor,
            constructor_object: JsObject::new(Object::default()),
            prototype: JsObject::new(Object::default()),
            length: 0,
            name: JsString::default(),
            callable: true,
            constructable: true,
            inherit: None,
        }
    }

    #[inline]
    pub(crate) fn with_standard_object(
        context: &'context mut Context,
        constructor: NativeFunction,
        object: StandardConstructor,
    ) -> Self {
        Self {
            context,
            constructor_function: constructor,
            constructor_object: object.constructor,
            prototype: object.prototype,
            length: 0,
            name: JsString::default(),
            callable: true,
            constructable: true,
            inherit: None,
        }
    }

    /// Add new method to the constructors prototype.
    #[inline]
    pub fn method<B>(&mut self, function: NativeFunction, binding: B, length: usize) -> &mut Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = FunctionBuilder::native(self.context, function)
            .name(binding.name)
            .length(length)
            .constructable(false)
            .build();

        self.prototype.borrow_mut().insert_property(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Add new static method to the constructors object itself.
    #[inline]
    pub fn static_method<B>(
        &mut self,
        function: NativeFunction,
        binding: B,
        length: usize,
    ) -> &mut Self
    where
        B: Into<FunctionBinding>,
    {
        let binding = binding.into();
        let function = FunctionBuilder::native(self.context, function)
            .name(binding.name)
            .length(length)
            .constructable(false)
            .build();

        self.constructor_object.borrow_mut().insert_property(
            binding.binding,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true),
        );
        self
    }

    /// Add new data property to the constructor's prototype.
    #[inline]
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.prototype.borrow_mut().insert(key, property);
        self
    }

    /// Add new static data property to the constructor object itself.
    #[inline]
    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let property = PropertyDescriptor::builder()
            .value(value)
            .writable(attribute.writable())
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.constructor_object.borrow_mut().insert(key, property);
        self
    }

    /// Add new accessor property to the constructor's prototype.
    #[inline]
    pub fn accessor<K>(
        &mut self,
        key: K,
        get: Option<JsObject>,
        set: Option<JsObject>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        let property = PropertyDescriptor::builder()
            .maybe_get(get)
            .maybe_set(set)
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.prototype.borrow_mut().insert(key, property);
        self
    }

    /// Add new static accessor property to the constructor object itself.
    #[inline]
    pub fn static_accessor<K>(
        &mut self,
        key: K,
        get: Option<JsObject>,
        set: Option<JsObject>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        let property = PropertyDescriptor::builder()
            .maybe_get(get)
            .maybe_set(set)
            .enumerable(attribute.enumerable())
            .configurable(attribute.configurable());
        self.constructor_object.borrow_mut().insert(key, property);
        self
    }

    /// Add new property to the constructor's prototype.
    #[inline]
    pub fn property_descriptor<K, P>(&mut self, key: K, property: P) -> &mut Self
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let property = property.into();
        self.prototype.borrow_mut().insert(key, property);
        self
    }

    /// Add new static property to the constructor object itself.
    #[inline]
    pub fn static_property_descriptor<K, P>(&mut self, key: K, property: P) -> &mut Self
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let property = property.into();
        self.constructor_object.borrow_mut().insert(key, property);
        self
    }

    /// Specify how many arguments the constructor function takes.
    ///
    /// Default is `0`.
    #[inline]
    pub fn length(&mut self, length: usize) -> &mut Self {
        self.length = length;
        self
    }

    /// Specify the name of the constructor function.
    ///
    /// Default is `"[object]"`
    #[inline]
    pub fn name<N>(&mut self, name: N) -> &mut Self
    where
        N: AsRef<str>,
    {
        self.name = name.as_ref().into();
        self
    }

    /// Specify whether the constructor function can be called.
    ///
    /// Default is `true`
    #[inline]
    pub fn callable(&mut self, callable: bool) -> &mut Self {
        self.callable = callable;
        self
    }

    /// Specify whether the constructor function can be called with `new` keyword.
    ///
    /// Default is `true`
    #[inline]
    pub fn constructable(&mut self, constructable: bool) -> &mut Self {
        self.constructable = constructable;
        self
    }

    /// Specify the prototype this constructor object inherits from.
    ///
    /// Default is `Object.prototype`
    #[inline]
    pub fn inherit(&mut self, prototype: JsValue) -> &mut Self {
        assert!(prototype.is_object() || prototype.is_null());
        self.inherit = Some(prototype);
        self
    }

    /// Return the current context.
    #[inline]
    pub fn context(&mut self) -> &'_ mut Context {
        self.context
    }

    /// Build the constructor function object.
    pub fn build(&mut self) -> JsObject {
        // Create the native function
        let function = Function::Native {
            function: self.constructor_function.into(),
            constructable: self.constructable,
        };

        let length = PropertyDescriptor::builder()
            .value(self.length)
            .writable(false)
            .enumerable(false)
            .configurable(true);
        let name = PropertyDescriptor::builder()
            .value(self.name.clone())
            .writable(false)
            .enumerable(false)
            .configurable(true);

        {
            let mut constructor = self.constructor_object.borrow_mut();
            constructor.data = ObjectData::function(function);
            constructor.insert("length", length);
            constructor.insert("name", name);

            constructor.set_prototype_instance(
                self.context
                    .standard_objects()
                    .function_object()
                    .prototype()
                    .into(),
            );

            constructor.insert_property(
                PROTOTYPE,
                PropertyDescriptor::builder()
                    .value(self.prototype.clone())
                    .writable(false)
                    .enumerable(false)
                    .configurable(false),
            );
        }

        {
            let mut prototype = self.prototype.borrow_mut();
            prototype.insert_property(
                "constructor",
                PropertyDescriptor::builder()
                    .value(self.constructor_object.clone())
                    .writable(true)
                    .enumerable(false)
                    .configurable(true),
            );

            if let Some(proto) = self.inherit.take() {
                prototype.set_prototype_instance(proto);
            } else {
                prototype.set_prototype_instance(
                    self.context
                        .standard_objects()
                        .object_object()
                        .prototype()
                        .into(),
                );
            }
        }

        self.constructor_object.clone()
    }
}
