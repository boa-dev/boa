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
    property::{AccessorDescriptor, Attribute, DataDescriptor, PropertyDescriptor, PropertyKey},
    BoaProfiler, Context, JsBigInt, JsString, JsSymbol, Value,
};
use rustc_hash::FxHashMap;
use std::{
    any::Any,
    fmt::{self, Debug, Display},
    ops::{Deref, DerefMut},
};

#[cfg(test)]
mod tests;

mod gcobject;
mod internal_methods;
mod iter;

use crate::builtins::object::for_in_iterator::ForInIterator;
pub use gcobject::{GcObject, RecursionLimiter, Ref, RefMut};
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
    indexed_properties: FxHashMap<u32, PropertyDescriptor>,
    /// Properties
    string_properties: FxHashMap<JsString, PropertyDescriptor>,
    /// Symbol Properties
    symbol_properties: FxHashMap<JsSymbol, PropertyDescriptor>,
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
    Map(OrderedMap<Value>),
    MapIterator(MapIterator),
    RegExp(Box<RegExp>),
    RegExpStringIterator(RegExpStringIterator),
    BigInt(JsBigInt),
    Boolean(bool),
    ForInIterator(ForInIterator),
    Function(Function),
    Set(OrderedSet<Value>),
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

impl Display for ObjectData {
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
        let mut obj = Self::new();
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
        S: Into<JsString>,
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
    pub fn bigint(value: JsBigInt) -> Self {
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

    /// It determines if Object is a callable function with a `[[Call]]` internal method.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iscallable
    #[inline]
    pub fn is_callable(&self) -> bool {
        matches!(self.data, ObjectData::Function(_))
    }

    /// It determines if Object is a function object with a `[[Construct]]` internal method.
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

    #[inline]
    pub fn as_regexp_string_iterator_mut(&mut self) -> Option<&mut RegExpStringIterator> {
        match &mut self.data {
            ObjectData::RegExpStringIterator(iter) => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn as_for_in_iterator(&self) -> Option<&ForInIterator> {
        match &self.data {
            ObjectData::ForInIterator(iter) => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn as_for_in_iterator_mut(&mut self) -> Option<&mut ForInIterator> {
        match &mut self.data {
            ObjectData::ForInIterator(iter) => Some(iter),
            _ => None,
        }
    }

    /// Checks if it is a `Map` object.pub
    #[inline]
    pub fn is_map(&self) -> bool {
        matches!(self.data, ObjectData::Map(_))
    }

    #[inline]
    pub fn as_map_ref(&self) -> Option<&OrderedMap<Value>> {
        match self.data {
            ObjectData::Map(ref map) => Some(map),
            _ => None,
        }
    }

    #[inline]
    pub fn as_map_mut(&mut self) -> Option<&mut OrderedMap<Value>> {
        match &mut self.data {
            ObjectData::Map(map) => Some(map),
            _ => None,
        }
    }

    #[inline]
    pub fn as_map_iterator_mut(&mut self) -> Option<&mut MapIterator> {
        match &mut self.data {
            ObjectData::MapIterator(iter) => Some(iter),
            _ => None,
        }
    }

    #[inline]
    pub fn is_set(&self) -> bool {
        matches!(self.data, ObjectData::Set(_))
    }

    #[inline]
    pub fn as_set_ref(&self) -> Option<&OrderedSet<Value>> {
        match self.data {
            ObjectData::Set(ref set) => Some(set),
            _ => None,
        }
    }

    #[inline]
    pub fn as_set_mut(&mut self) -> Option<&mut OrderedSet<Value>> {
        match &mut self.data {
            ObjectData::Set(set) => Some(set),
            _ => None,
        }
    }

    #[inline]
    pub fn as_set_iterator_mut(&mut self) -> Option<&mut SetIterator> {
        match &mut self.data {
            ObjectData::SetIterator(iter) => Some(iter),
            _ => None,
        }
    }

    /// Checks if it a `String` object.
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self.data, ObjectData::String(_))
    }

    #[inline]
    pub fn as_string(&self) -> Option<JsString> {
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
    pub fn as_symbol(&self) -> Option<JsSymbol> {
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
    pub fn as_bigint(&self) -> Option<&JsBigInt> {
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

    /// Sets the prototype instance of the object.
    ///
    /// [More information][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-invariants-of-the-essential-internal-methods
    #[inline]
    #[track_caller]
    pub fn set_prototype_instance(&mut self, prototype: Value) -> bool {
        assert!(prototype.is_null() || prototype.is_object());
        if self.extensible {
            self.prototype = prototype;
            true
        } else {
            // If target is non-extensible, [[SetPrototypeOf]] must return false
            // unless V is the SameValue as the target's observed [[GetPrototypeOf]] value.
            Value::same_value(&prototype, &self.prototype)
        }
    }

    /// Similar to `Value::new_object`, but you can pass a prototype to create from, plus a kind
    #[inline]
    pub fn with_prototype(proto: Value, data: ObjectData) -> Object {
        let mut object = Object::new();
        object.data = data;
        object.set_prototype_instance(proto);
        object
    }

    /// Returns `true` if it holds an Rust type that implements `NativeObject`.
    #[inline]
    pub fn is_native_object(&self) -> bool {
        matches!(self.data, ObjectData::NativeObject(_))
    }

    #[inline]
    pub fn as_native_object(&self) -> Option<&dyn NativeObject> {
        match self.data {
            ObjectData::NativeObject(ref object) => Some(object.as_ref()),
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
    name: Option<String>,
    length: usize,
}

impl<'context> FunctionBuilder<'context> {
    /// Create a new `FunctionBuilder`
    #[inline]
    pub fn native(context: &'context mut Context, function: NativeFunction) -> Self {
        Self {
            context,
            function: Some(Function::Native {
                function: function.into(),
                constructable: false,
            }),
            name: None,
            length: 0,
        }
    }

    #[inline]
    pub fn closure<F>(context: &'context mut Context, function: F) -> Self
    where
        F: Fn(&Value, &[Value], &mut Context) -> Result<Value, Value> + 'static,
    {
        Self {
            context,
            function: Some(Function::Closure {
                function: Box::new(function),
                constructable: false,
            }),
            name: None,
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
    pub fn build(&mut self) -> GcObject {
        let mut function = Object::function(
            self.function.take().unwrap(),
            self.context
                .standard_objects()
                .function_object()
                .prototype()
                .into(),
        );
        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        if let Some(name) = self.name.take() {
            function.insert_property("name", name, attribute);
        } else {
            function.insert_property("name", "", attribute);
        }
        function.insert_property("length", self.length, attribute);

        GcObject::new(function)
    }

    /// Initializes the `Function.prototype` function object.
    pub(crate) fn build_function_prototype(&mut self, object: &GcObject) {
        let mut object = object.borrow_mut();
        object.data = ObjectData::Function(self.function.take().unwrap());
        object.set_prototype_instance(
            self.context
                .standard_objects()
                .object_object()
                .prototype()
                .into(),
        );
        let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
        if let Some(name) = self.name.take() {
            object.insert_property("name", name, attribute);
        } else {
            object.insert_property("name", "", attribute);
        }
        object.insert_property("length", self.length, attribute);
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
        let function = FunctionBuilder::native(self.context, function)
            .name(binding.name)
            .length(length)
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
        let property = DataDescriptor::new(value, attribute);
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
    constructor_function: NativeFunction,
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
            constructor_function: constructor,
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
            constructor_function: constructor,
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
        let function = FunctionBuilder::native(self.context, function)
            .name(binding.name)
            .length(length)
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
        let function = FunctionBuilder::native(self.context, function)
            .name(binding.name)
            .length(length)
            .constructable(false)
            .build();

        self.constructor_object.borrow_mut().insert_property(
            binding.binding,
            function,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );
        self
    }

    /// Add new data property to the constructor's prototype.
    #[inline]
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        let property = DataDescriptor::new(value, attribute);
        self.prototype.borrow_mut().insert(key, property);
        self
    }

    /// Add new static data property to the constructor object itself.
    #[inline]
    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        let property = DataDescriptor::new(value, attribute);
        self.constructor_object.borrow_mut().insert(key, property);
        self
    }

    /// Add new accessor property to the constructor's prototype.
    #[inline]
    pub fn accessor<K>(
        &mut self,
        key: K,
        get: Option<GcObject>,
        set: Option<GcObject>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        let property = AccessorDescriptor::new(get, set, attribute);
        self.prototype.borrow_mut().insert(key, property);
        self
    }

    /// Add new static accessor property to the constructor object itself.
    #[inline]
    pub fn static_accessor<K>(
        &mut self,
        key: K,
        get: Option<GcObject>,
        set: Option<GcObject>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        let property = AccessorDescriptor::new(get, set, attribute);
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
        let function = Function::Native {
            function: self.constructor_function.into(),
            constructable: self.constructable,
        };

        let length = DataDescriptor::new(
            self.length,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );
        let name = DataDescriptor::new(
            self.name.take().unwrap_or_else(|| String::from("[object]")),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
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
