//! Traits and structs for implementing native classes.
//!
//! Native classes are implemented through the [`Class`][class-trait] trait.
//! ```
//!# use boa_engine::{
//!#    property::Attribute,
//!#    class::{Class, ClassBuilder},
//!#    Context, JsResult, JsValue,
//!#    builtins::JsArgs,
//!# };
//!# use boa_gc::{Finalize, Trace};
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
//!     fn constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
//!         // This is equivalent to `String(arg)`.
//!         let kind = args.get_or_undefined(0).to_string(context)?;
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
//!     fn init(class: &mut ClassBuilder) -> JsResult<()> {
//!         class.method("speak", 0, |this, _args, _ctx| {
//!             if let Some(object) = this.as_object() {
//!                 if let Some(animal) = object.downcast_ref::<Animal>() {
//!                     match &*animal {
//!                         Self::Cat => println!("meow"),
//!                         Self::Dog => println!("woof"),
//!                         Self::Other => println!(r"¯\_(ツ)_/¯"),
//!                     }
//!                 }
//!             }
//!             Ok(JsValue::undefined())
//!         });
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! [class-trait]: ./trait.Class.html

use crate::{
    builtins::function::NativeFunctionSignature,
    object::{ConstructorBuilder, JsFunction, JsObject, NativeObject, ObjectData, PROTOTYPE},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    Context, JsResult, JsValue,
};

/// Native class.
pub trait Class: NativeObject + Sized {
    /// The binding name of the object.
    const NAME: &'static str;
    /// The amount of arguments the class `constructor` takes, default is `0`.
    const LENGTH: usize = 0;
    /// The attibutes the class will be binded with, default is `writable`, `enumerable`, `configurable`.
    const ATTRIBUTES: Attribute = Attribute::all();

    /// The constructor of the class.
    fn constructor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self>;

    /// Initializes the internals and the methods of the class.
    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()>;
}

/// This is a wrapper around `Class::constructor` that sets the internal data of a class.
///
/// This is automatically implemented, when a type implements `Class`.
pub trait ClassConstructor: Class {
    /// The raw constructor that matches the `NativeFunction` signature.
    fn raw_constructor(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue>
    where
        Self: Sized;
}

impl<T: Class> ClassConstructor for T {
    fn raw_constructor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue>
    where
        Self: Sized,
    {
        if this.is_undefined() {
            return context.throw_type_error(format!(
                "cannot call constructor of native class `{}` without new",
                T::NAME
            ));
        }

        let class_constructor = context.global_object().clone().get(T::NAME, context)?;
        let class_constructor = if let JsValue::Object(ref obj) = class_constructor {
            obj
        } else {
            return context.throw_type_error(format!(
                "invalid constructor for native class `{}` ",
                T::NAME
            ));
        };
        let class_prototype =
            if let JsValue::Object(ref obj) = class_constructor.get(PROTOTYPE, context)? {
                obj.clone()
            } else {
                return context.throw_type_error(format!(
                    "invalid default prototype for native class `{}`",
                    T::NAME
                ));
            };

        let prototype = this
            .as_object()
            .cloned()
            .map(|obj| {
                obj.get(PROTOTYPE, context)
                    .map(|val| val.as_object().cloned())
            })
            .transpose()?
            .flatten()
            .unwrap_or(class_prototype);

        let native_instance = Self::constructor(this, args, context)?;
        let object_instance = JsObject::from_proto_and_data(
            prototype,
            ObjectData::native_object(Box::new(native_instance)),
        );
        Ok(object_instance.into())
    }
}

/// Class builder which allows adding methods and static methods to the class.
#[derive(Debug)]
pub struct ClassBuilder<'context> {
    builder: ConstructorBuilder<'context>,
}

impl<'context> ClassBuilder<'context> {
    #[inline]
    pub(crate) fn new<T>(context: &'context mut Context) -> Self
    where
        T: ClassConstructor,
    {
        let mut builder = ConstructorBuilder::new(context, T::raw_constructor);
        builder.name(T::NAME);
        builder.length(T::LENGTH);
        Self { builder }
    }

    #[inline]
    pub(crate) fn build(mut self) -> JsFunction {
        JsFunction::from_object_unchecked(self.builder.build().into())
    }

    /// Add a method to the class.
    ///
    /// It is added to `prototype`.
    #[inline]
    pub fn method<N>(
        &mut self,
        name: N,
        length: usize,
        function: NativeFunctionSignature,
    ) -> &mut Self
    where
        N: AsRef<str>,
    {
        self.builder.method(function, name.as_ref(), length);
        self
    }

    /// Add a static method to the class.
    ///
    /// It is added to class object itself.
    #[inline]
    pub fn static_method<N>(
        &mut self,
        name: N,
        length: usize,
        function: NativeFunctionSignature,
    ) -> &mut Self
    where
        N: AsRef<str>,
    {
        self.builder.static_method(function, name.as_ref(), length);
        self
    }

    /// Add a data property to the class, with the specified attribute.
    ///
    /// It is added to `prototype`.
    #[inline]
    pub fn property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        self.builder.property(key, value, attribute);
        self
    }

    /// Add a static data property to the class, with the specified attribute.
    ///
    /// It is added to class object itself.
    #[inline]
    pub fn static_property<K, V>(&mut self, key: K, value: V, attribute: Attribute) -> &mut Self
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        self.builder.static_property(key, value, attribute);
        self
    }

    /// Add an accessor property to the class, with the specified attribute.
    ///
    /// It is added to `prototype`.
    #[inline]
    pub fn accessor<K>(
        &mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        self.builder.accessor(key, get, set, attribute);
        self
    }

    /// Add a static accessor property to the class, with the specified attribute.
    ///
    /// It is added to class object itself.
    #[inline]
    pub fn static_accessor<K>(
        &mut self,
        key: K,
        get: Option<JsFunction>,
        set: Option<JsFunction>,
        attribute: Attribute,
    ) -> &mut Self
    where
        K: Into<PropertyKey>,
    {
        self.builder.static_accessor(key, get, set, attribute);
        self
    }

    /// Add a property descriptor to the class, with the specified attribute.
    ///
    /// It is added to `prototype`.
    #[inline]
    pub fn property_descriptor<K, P>(&mut self, key: K, property: P) -> &mut Self
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.builder.property_descriptor(key, property);
        self
    }

    /// Add a static property descriptor to the class, with the specified attribute.
    ///
    /// It is added to class object itself.
    #[inline]
    pub fn static_property_descriptor<K, P>(&mut self, key: K, property: P) -> &mut Self
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.builder.static_property_descriptor(key, property);
        self
    }

    /// Return the current context.
    #[inline]
    pub fn context(&mut self) -> &'_ mut Context {
        self.builder.context()
    }
}
