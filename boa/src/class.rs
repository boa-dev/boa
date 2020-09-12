//! Traits and structs for implementing native classes.
//!
//! Native classes are implemented through the [`Class`][class-trait] trait.
//! ```
//!# use boa::{
//!#    property::Attribute,
//!#    class::{Class, ClassBuilder},
//!#    gc::{Finalize, Trace},
//!#    Context, Result, Value,
//!# };
//!#
//! // This does not have to be an enum it can also be a struct.
//! #[derive(Debug, Trace, Finalize)]
//! enum Animal {
//!     Cat,
//!     Dog,
//!     Other,
//! }
//!
//! impl Class for Animal {
//!     // we set the binging name of this function to be `"Animal"`.
//!     const NAME: &'static str = "Animal";
//!
//!     // We set the length to `1` since we accept 1 arguments in the constructor.
//!     const LENGTH: usize = 1;
//!
//!     // This is what is called when we do `new Animal()`
//!     fn constructor(_this: &Value, args: &[Value], ctx: &mut Context) -> Result<Self> {
//!         // This is equivalent to `String(arg)`.
//!         let kind = args.get(0).cloned().unwrap_or_default().to_string(ctx)?;
//!
//!         let animal = match kind.as_str() {
//!             "cat" => Self::Cat,
//!             "dog" => Self::Dog,
//!             _ => Self::Other,
//!         };
//!
//!         Ok(animal)
//!     }
//!
//!     /// This is where the object is intitialized.
//!     fn init(class: &mut ClassBuilder) -> Result<()> {
//!         class.method("speak", 0, |this, _args, _ctx| {
//!             if let Some(object) = this.as_object() {
//!                 if let Some(animal) = object.downcast_ref::<Animal>() {
//!                     match animal {
//!                         Self::Cat => println!("meow"),
//!                         Self::Dog => println!("woof"),
//!                         Self::Other => println!(r"¯\_(ツ)_/¯"),
//!                     }
//!                 }
//!             }
//!             Ok(Value::undefined())
//!         });
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! [class-trait]: ./trait.Class.html

use crate::{
    builtins::function::{BuiltInFunction, Function, FunctionFlags, NativeFunction},
    object::{GcObject, NativeObject, Object, ObjectData, PROTOTYPE},
    property::{Attribute, Property, PropertyKey},
    Context, Result, Value,
};
use std::fmt::Debug;

/// Native class.
pub trait Class: NativeObject + Sized {
    /// The binding name of the object.
    const NAME: &'static str;
    /// The amount of arguments the class `constructor` takes, default is `0`.
    const LENGTH: usize = 0;
    /// The attibutes the class will be binded with, default is `writable`, `enumerable`, `configurable`.
    const ATTRIBUTE: Attribute = Attribute::all();

    /// The constructor of the class.
    fn constructor(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Self>;

    /// Initializes the internals and the methods of the class.
    fn init(class: &mut ClassBuilder<'_>) -> Result<()>;
}

/// This is a wrapper around `Class::constructor` that sets the internal data of a class.
///
/// This is automatically implemented, when a type implements `Class`.
pub trait ClassConstructor: Class {
    /// The raw constructor that mathces the `NativeFunction` signature.
    fn raw_constructor(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value>
    where
        Self: Sized;
}

impl<T: Class> ClassConstructor for T {
    fn raw_constructor(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value>
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
    /// The current context.
    context: &'context mut Context,

    /// The constructor object.
    object: GcObject,

    /// The prototype of the object.
    prototype: GcObject,
}

impl<'context> ClassBuilder<'context> {
    pub(crate) fn new<T>(context: &'context mut Context) -> Self
    where
        T: ClassConstructor,
    {
        let global = context.global_object();

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
                .global_object()
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
                .global_object()
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

    /// Return the current context.
    pub fn context(&mut self) -> &'_ mut Context {
        self.context
    }
}
