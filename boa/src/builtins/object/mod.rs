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
        property::{Attribute, Property, PropertyKey},
        value::{RcBigInt, RcString, RcSymbol, Value},
        BigInt, Date, RegExp,
    },
    exec::Interpreter,
    BoaProfiler, Result,
};
use gc::{Finalize, Trace};
use rustc_hash::FxHashMap;
use std::any::Any;
use std::fmt::{Debug, Display, Error, Formatter};
use std::result::Result as StdResult;

use super::function::{
    make_builtin_fn, make_constructor_fn, BuiltInFunction, FunctionFlags, NativeFunction,
};
use crate::builtins::value::same_value;

mod gcobject;
mod internal_methods;
mod iter;

pub use gcobject::{GcObject, Ref, RefMut};
pub use iter::*;

#[cfg(test)]
mod tests;

/// Static `prototype`, usually set on constructors as a key to point to their respective prototype object.
pub static PROTOTYPE: &str = "prototype";

/// This trait allows Rust types to be passed around as objects.
///
/// This is automatically implemented, when a type implements `Debug`, `Any` and `Trace`.
pub trait NativeObject: Debug + Any + Trace {
    fn as_any(&self) -> &dyn Any;
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

/// Native class.
pub trait Class: NativeObject + Sized {
    /// The binding name of the object.
    const NAME: &'static str;
    /// The amount of arguments the class `constructor` takes, default is `0`.
    const LENGTH: usize = 0;
    /// The attibutes the class will be binded with, default is `writable`, `enumerable`, `configurable`.
    const ATTRIBUTE: Attribute = Attribute::all();

    /// The constructor of the class.
    fn constructor(this: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Self>;

    /// Initializes the internals and the methods of the class.
    fn methods(class: &mut ClassBuilder<'_>) -> Result<()>;
}

/// This is a wrapper around `Class::constructor` that sets the internal data of a class.
///
/// This is automatically implemented, when a type implements `Class`.
pub trait ClassConstructor: Class {
    fn raw_constructor(this: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Value>
    where
        Self: Sized;
}

impl<T: Class> ClassConstructor for T {
    fn raw_constructor(this: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Value>
    where
        Self: Sized,
    {
        let object_instance = Self::constructor(this, args, ctx)?;
        this.set_data(ObjectData::NativeObject(Box::new(object_instance)));
        Ok(this.clone())
    }
}

/// Class builder which allows adding methods and static methods to the class.
#[derive(Debug)]
pub struct ClassBuilder<'context> {
    context: &'context mut Interpreter,
    object: GcObject,
    prototype: GcObject,
}

impl<'context> ClassBuilder<'context> {
    pub(crate) fn new<T>(context: &'context mut Interpreter) -> Self
    where
        T: ClassConstructor,
    {
        let global = context.global();

        let prototype = {
            let object_prototype = global.get_field("Object").get_field(PROTOTYPE);

            let object = Object::create(object_prototype);
            GcObject::new(object)
        };
        // Create the native function
        let function = Function::BuiltIn(
            BuiltInFunction(T::raw_constructor),
            FunctionFlags::CONSTRUCTABLE,
        );

        // Get reference to Function.prototype
        // Create the function object and point its instance prototype to Function.prototype
        let mut constructor =
            Object::function(function, global.get_field("Function").get_field(PROTOTYPE));

        let length = Property::data_descriptor(
            T::LENGTH.into(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
        constructor.insert_property("length", length);

        let name = Property::data_descriptor(
            T::NAME.into(),
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
        constructor.insert_property("name", name);

        let constructor = GcObject::new(constructor);

        prototype
            .borrow_mut()
            .insert_field("constructor", constructor.clone().into());

        constructor
            .borrow_mut()
            .insert_field(PROTOTYPE, prototype.clone().into());

        Self {
            context,
            object: constructor,
            prototype,
        }
    }

    pub(crate) fn build(self) -> GcObject {
        self.object
    }

    /// Add a method to the class.
    ///
    /// It is added to `prototype`.
    pub fn method<N>(&mut self, name: N, length: usize, function: NativeFunction)
    where
        N: Into<String>,
    {
        let name = name.into();
        let mut function = Object::function(
            Function::BuiltIn(function.into(), FunctionFlags::CALLABLE),
            self.context
                .global()
                .get_field("Function")
                .get_field("prototype"),
        );

        function.insert_field("length", Value::from(length));
        function.insert_field("name", Value::from(name.as_str()));

        self.prototype
            .borrow_mut()
            .insert_field(name, Value::from(function));
    }

    /// Add a static method to the class.
    ///
    /// It is added to class object itself.
    pub fn static_method<N>(&mut self, name: N, length: usize, function: NativeFunction)
    where
        N: Into<String>,
    {
        let name = name.into();
        let mut function = Object::function(
            Function::BuiltIn(function.into(), FunctionFlags::CALLABLE),
            self.context
                .global()
                .get_field("Function")
                .get_field("prototype"),
        );

        function.insert_field("length", Value::from(length));
        function.insert_field("name", Value::from(name.as_str()));

        self.object
            .borrow_mut()
            .insert_field(name, Value::from(function));
    }

    /// Add a property to the class, with the specified attribute.
    ///
    /// It is added to `prototype`.
    #[inline]
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute)
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        // We bitwise or (`|`) with `Attribute::default()` (`READONLY | NON_ENUMERABLE | PERMANENT`)
        // so we dont get an empty attribute.
        let property = Property::data_descriptor(value.into(), attribute | Attribute::default());
        self.prototype
            .borrow_mut()
            .insert_property(key.into(), property);
    }

    /// Add a static property to the class, with the specified attribute.
    ///
    /// It is added to class object itself.
    #[inline]
    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute)
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        // We bitwise or (`|`) with `Attribute::default()` (`READONLY | NON_ENUMERABLE | PERMANENT`)
        // so we dont get an empty attribute.
        let property = Property::data_descriptor(value.into(), attribute | Attribute::default());
        self.object
            .borrow_mut()
            .insert_property(key.into(), property);
    }

    pub fn context(&mut self) -> &'_ mut Interpreter {
        self.context
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

    pub fn prototype(&self) -> &Value {
        &self.prototype
    }

    pub fn set_prototype(&mut self, prototype: Value) {
        assert!(prototype.is_null() || prototype.is_object());
        self.prototype = prototype
    }

    pub fn is_native_object(&self) -> bool {
        matches!(self.data, ObjectData::NativeObject(_))
    }

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

/// Create a new object.
pub fn make_object(_: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Value> {
    if let Some(arg) = args.get(0) {
        if !arg.is_null_or_undefined() {
            return arg.to_object(ctx);
        }
    }

    Ok(Value::new_object(Some(ctx.global())))
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
pub fn create(_: &Value, args: &[Value], interpreter: &mut Interpreter) -> Result<Value> {
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
pub fn is(_: &Value, args: &[Value], _: &mut Interpreter) -> Result<Value> {
    let x = args.get(0).cloned().unwrap_or_else(Value::undefined);
    let y = args.get(1).cloned().unwrap_or_else(Value::undefined);

    Ok(same_value(&x, &y).into())
}

/// Get the `prototype` of an object.
pub fn get_prototype_of(_: &Value, args: &[Value], _: &mut Interpreter) -> Result<Value> {
    let obj = args.get(0).expect("Cannot get object");
    Ok(obj
        .as_object()
        .map_or_else(Value::undefined, |object| object.prototype.clone()))
}

/// Set the `prototype` of an object.
pub fn set_prototype_of(_: &Value, args: &[Value], _: &mut Interpreter) -> Result<Value> {
    let obj = args.get(0).expect("Cannot get object").clone();
    let proto = args.get(1).expect("Cannot get object").clone();
    obj.as_object_mut().unwrap().prototype = proto;
    Ok(obj)
}

/// Define a property in an object
pub fn define_property(_: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Value> {
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
pub fn to_string(this: &Value, _: &[Value], _: &mut Interpreter) -> Result<Value> {
    // FIXME: it should not display the object.
    Ok(this.display().to_string().into())
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
pub fn has_own_property(this: &Value, args: &[Value], ctx: &mut Interpreter) -> Result<Value> {
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

pub fn property_is_enumerable(
    this: &Value,
    args: &[Value],
    ctx: &mut Interpreter,
) -> Result<Value> {
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
