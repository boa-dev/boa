//! This module implements the Rust representation of a JavaScript object.

use crate::{
    builtins::{
        array::array_iterator::ArrayIterator,
        function::{BuiltInFunction, Function, FunctionFlags, NativeFunction},
        map::ordered_map::OrderedMap,
        string::string_iterator::StringIterator,
        BigInt, Date, RegExp,
    },
    context::StandardConstructor,
    gc::{Finalize, Trace},
    property::{Attribute, Property, PropertyKey},
    value::{RcBigInt, RcString, RcSymbol, Value},
    BoaProfiler, Context,
};
use rustc_hash::FxHashMap;
use std::{
    any::Any,
    fmt::{self, Debug, Display},
    ops::{Deref, DerefMut},
};

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
    ArrayIterator(ArrayIterator),
    Map(OrderedMap<Value, Value>),
    RegExp(Box<RegExp>),
    BigInt(RcBigInt),
    Boolean(bool),
    Function(Function),
    String(RcString),
    StringIterator(StringIterator),
    Number(f64),
    Symbol(RcSymbol),
    Error,
    Ordinary,
    Date(Date),
    Global,
    NativeObject(Box<dyn NativeObject>),
}

impl Display for ObjectData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Array => "Array",
                Self::ArrayIterator(_) => "ArrayIterator",
                Self::Function(_) => "Function",
                Self::RegExp(_) => "RegExp",
                Self::Map(_) => "Map",
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
    #[inline]
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
    #[inline]
    pub fn create(proto: Value) -> Self {
        let mut obj = Self::default();
        obj.prototype = proto;
        obj
    }

    /// Return a new Boolean object whose `[[BooleanData]]` internal slot is set to argument.
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
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

    /// Checks if it is an `ArrayIterator` object.
    #[inline]
    pub fn is_array_iterator(&self) -> bool {
        matches!(self.data, ObjectData::ArrayIterator(_))
    }

    #[inline]
    pub fn as_array_iterator(&self) -> Option<&ArrayIterator> {
        match self.data {
            ObjectData::ArrayIterator(ref iter) => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn as_array_iterator_mut(&mut self) -> Option<&mut ArrayIterator> {
        match &mut self.data {
            ObjectData::ArrayIterator(iter) => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string_iterator_mut(&mut self) -> Option<&mut StringIterator> {
        match &mut self.data {
            ObjectData::StringIterator(iter) => Some(iter),
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
    pub fn prototype_instance(&self) -> &Value {
        &self.prototype
    }

    #[track_caller]
    #[inline]
    pub fn set_prototype_instance(&mut self, prototype: Value) {
        assert!(prototype.is_null() || prototype.is_object());
        self.prototype = prototype
    }

    /// Similar to `Value::new_object`, but you can pass a prototype to create from, plus a kind
    #[inline]
    pub fn with_prototype(proto: Value, data: ObjectData) -> Object {
        let mut object = Object::default();
        object.data = data;
        object.set_prototype_instance(proto);
        object
    }

    /// Returns `true` if it holds an Rust type that implements `NativeObject`.
    #[inline]
    pub fn is_native_object(&self) -> bool {
        matches!(self.data, ObjectData::NativeObject(_))
    }

    /// Reeturn `true` if it is a native object and the native type is `T`.
    #[inline]
    pub fn is<T>(&self) -> bool
    where
        T: NativeObject,
    {
        match self.data {
            ObjectData::NativeObject(ref object) => object.deref().as_any().is::<T>(),
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
            ObjectData::NativeObject(ref object) => object.deref().as_any().downcast_ref::<T>(),
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
            ObjectData::NativeObject(ref mut object) => {
                object.deref_mut().as_mut_any().downcast_mut::<T>()
            }
            _ => None,
        }
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
    name: RcString,
}

impl From<&str> for FunctionBinding {
    #[inline]
    fn from(name: &str) -> Self {
        let name: RcString = name.into();

        Self {
            binding: name.clone().into(),
            name,
        }
    }
}

impl From<String> for FunctionBinding {
    #[inline]
    fn from(name: String) -> Self {
        let name: RcString = name.into();

        Self {
            binding: name.clone().into(),
            name,
        }
    }
}

impl From<RcString> for FunctionBinding {
    #[inline]
    fn from(name: RcString) -> Self {
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
    function: BuiltInFunction,
    name: Option<String>,
    length: usize,
    callable: bool,
    constructable: bool,
}

impl<'context> FunctionBuilder<'context> {
    /// Create a new `FunctionBuilder`
    #[inline]
    pub fn new(context: &'context mut Context, function: NativeFunction) -> Self {
        Self {
            context,
            function: function.into(),
            name: None,
            length: 0,
            callable: true,
            constructable: false,
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
        self.name = Some(name.as_ref().into());
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

    /// Specify the whether the object function object can be called.
    ///
    /// The default is `true`.
    #[inline]
    pub fn callable(&mut self, yes: bool) -> &mut Self {
        self.callable = yes;
        self
    }

    /// Specify the whether the object function object can be called with `new` keyword.
    ///
    /// The default is `false`.
    #[inline]
    pub fn constructable(&mut self, yes: bool) -> &mut Self {
        self.constructable = yes;
        self
    }

    /// Build the function object.
    #[inline]
    pub fn build(&mut self) -> GcObject {
        let mut function = Object::function(
            Function::BuiltIn(
                self.function,
                FunctionFlags::from_parameters(self.callable, self.constructable),
            ),
            self.context
                .standard_objects()
                .function_object()
                .prototype()
                .into(),
        );
        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT;
        if let Some(name) = self.name.take() {
            function.insert_property("name", name, attribute);
        } else {
            function.insert_property("name", "", attribute);
        }
        function.insert_property("length", self.length, attribute);

        GcObject::new(function)
    }
}

/// Builder for creating objects with properties.
///
/// # Examples
///
/// ```
/// # use boa::{Context, Value, object::ObjectInitializer, property::Attribute};
/// let mut context = Context::new();
/// let object = ObjectInitializer::new(&mut context)
///     .property("hello", "world", Attribute::all())
///     .property(1, 1, Attribute::all())
///     .function(|_, _, _| Ok(Value::undefined()), "func", 0)
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
    object: GcObject,
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
        let function = FunctionBuilder::new(self.context, function)
            .name(binding.name)
            .length(length)
            .callable(true)
            .constructable(false)
            .build();

        self.object.borrow_mut().insert_property(
            binding.binding,
            function,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );
        self
    }

    /// Add a property to the object.
    #[inline]
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        let property = Property::data_descriptor(value.into(), attribute);
        self.object.borrow_mut().insert(key, property);
        self
    }

    /// Build the object.
    #[inline]
    pub fn build(&mut self) -> GcObject {
        self.object.clone()
    }
}

/// Builder for creating constructors objects, like `Array`.
pub struct ConstructorBuilder<'context> {
    context: &'context mut Context,
    constrcutor_function: NativeFunction,
    constructor_object: GcObject,
    prototype: GcObject,
    name: Option<String>,
    length: usize,
    callable: bool,
    constructable: bool,
    inherit: Option<Value>,
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
            constrcutor_function: constructor,
            constructor_object: GcObject::new(Object::default()),
            prototype: GcObject::new(Object::default()),
            length: 0,
            name: None,
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
            constrcutor_function: constructor,
            constructor_object: object.constructor,
            prototype: object.prototype,
            length: 0,
            name: None,
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
        let function = FunctionBuilder::new(self.context, function)
            .name(binding.name)
            .length(length)
            .callable(true)
            .constructable(false)
            .build();

        self.prototype.borrow_mut().insert_property(
            binding.binding,
            function,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
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
        let function = FunctionBuilder::new(self.context, function)
            .name(binding.name)
            .length(length)
            .callable(true)
            .constructable(false)
            .build();

        self.constructor_object.borrow_mut().insert_property(
            binding.binding,
            function,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );
        self
    }

    /// Add new property to the constructors prototype.
    #[inline]
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        let property = Property::data_descriptor(value.into(), attribute);
        self.prototype.borrow_mut().insert(key, property);
        self
    }

    /// Add new static property to the constructors object itself.
    #[inline]
    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        let property = Property::data_descriptor(value.into(), attribute);
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
        self.name = Some(name.as_ref().into());
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
    pub fn inherit(&mut self, prototype: Value) -> &mut Self {
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
    pub fn build(&mut self) -> GcObject {
        // Create the native function
        let function = Function::BuiltIn(
            self.constrcutor_function.into(),
            FunctionFlags::from_parameters(self.callable, self.constructable),
        );

        let length = Property::data_descriptor(
            self.length.into(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
        let name = Property::data_descriptor(
            self.name
                .take()
                .unwrap_or_else(|| String::from("[object]"))
                .into(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );

        {
            let mut constructor = self.constructor_object.borrow_mut();
            constructor.data = ObjectData::Function(function);
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
                self.prototype.clone(),
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            );
        }

        {
            let mut prototype = self.prototype.borrow_mut();
            prototype.insert_property(
                "constructor",
                self.constructor_object.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
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
