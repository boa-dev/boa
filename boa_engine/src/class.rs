//! Traits and structs for implementing native classes.
//!
//! Native classes are implemented through the [`Class`][class-trait] trait.
//! ```
//! # use boa_engine::{
//! #    NativeFunction,
//! #    property::Attribute,
//! #    class::{Class, ClassBuilder},
//! #    Context, JsResult, JsValue,
//! #    JsArgs,
//! #    js_string,
//! # };
//! # use boa_gc::{Finalize, Trace};
//! #
//! // Can also be a struct containing `Trace` types.
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
//!     // This is what is called when we do `new Animal()` to construct the inner data of the class.
//!     fn make_data(_this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<Self> {
//!         // This is equivalent to `String(arg)`.
//!         let kind = args.get_or_undefined(0).to_string(context)?;
//!
//!         let animal = match kind.to_std_string_escaped().as_str() {
//!             "cat" => Self::Cat,
//!             "dog" => Self::Dog,
//!             _ => Self::Other,
//!         };
//!
//!         Ok(animal)
//!     }
//!
//!     /// This is where the object is initialized.
//!     fn init(class: &mut ClassBuilder) -> JsResult<()> {
//!         class.method(
//!             js_string!("speak"),
//!             0,
//!             NativeFunction::from_fn_ptr(|this, _args, _ctx| {
//!                 if let Some(object) = this.as_object() {
//!                     if let Some(animal) = object.downcast_ref::<Animal>() {
//!                         match &*animal {
//!                             Self::Cat => println!("meow"),
//!                             Self::Dog => println!("woof"),
//!                             Self::Other => println!(r"¯\_(ツ)_/¯"),
//!                         }
//!                     }
//!                 }
//!                 Ok(JsValue::undefined())
//!             }),
//!         );
//!         Ok(())
//!     }
//! }
//! ```
//!
//! [class-trait]: ./trait.Class.html

use crate::{
    context::intrinsics::StandardConstructor,
    error::JsNativeError,
    native_function::NativeFunction,
    object::{
        ConstructorBuilder, FunctionBinding, JsFunction, JsObject, NativeObject, ObjectData,
        PROTOTYPE,
    },
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    Context, JsResult, JsValue,
};

/// Native class.
pub trait Class: NativeObject + Sized {
    /// The binding name of this class.
    const NAME: &'static str;
    /// The amount of arguments this class' constructor takes. Default is `0`.
    const LENGTH: usize = 0;
    /// The property attributes of this class' constructor in the global object.
    /// Default is `writable`, `enumerable`, `configurable`.
    const ATTRIBUTES: Attribute = Attribute::all();

    /// Creates the internal data for an instance of this class.
    ///
    /// This method can also be called the "native constructor" of this class.
    fn make_data(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<Self>;

    /// Initializes the properties and methods of this class.
    fn init(class: &mut ClassBuilder<'_, '_>) -> JsResult<()>;

    /// Creates a new [`JsObject`] with its internal data set to the result of calling `Self::make_data`.
    ///
    /// # Note
    ///
    /// This will throw an error if this class is not registered in the context's active realm.
    /// See [`Context::register_global_class`].
    ///
    /// # Warning
    ///
    /// Overriding this method could be useful for certain usages, but incorrectly implementing this
    /// could lead to weird errors like missing inherited methods or incorrect internal data.
    fn construct(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "cannot call constructor of native class `{}` without new",
                    Self::NAME
                ))
                .into());
        }

        let class = context.get_global_class::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "could not find native class `{}` in the map of registered classes",
                Self::NAME
            ))
        })?;

        let prototype = new_target
            .as_object()
            .map(|obj| {
                obj.get(PROTOTYPE, context)
                    .map(|val| val.as_object().cloned())
            })
            .transpose()?
            .flatten()
            .unwrap_or_else(|| class.prototype());

        let data = Self::make_data(new_target, args, context)?;
        let instance = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ObjectData::native_object(data),
        );
        Ok(instance)
    }
}

/// Class builder which allows adding methods and static methods to the class.
#[derive(Debug)]
pub struct ClassBuilder {
    builder: ConstructorBuilder,
}

impl ClassBuilder {
    pub(crate) fn new<T>(realm: Realm) -> Self
    where
        T: Class,
    {
        let mut builder = ConstructorBuilder::new(
            context,
            NativeFunction::from_fn_ptr(|t, a, c| T::construct(t, a, c).map(JsValue::from)),
        );
        builder.name(T::NAME);
        builder.length(T::LENGTH);
        Self { builder }
    }

    pub(crate) fn build(self) -> StandardConstructor {
        self.builder.build()
    }

    /// Add a method to the class.
    ///
    /// It is added to `prototype`.
    pub fn method<N>(&mut self, name: N, length: usize, function: NativeFunction) -> &mut Self
    where
        N: Into<FunctionBinding>,
    {
        self.builder.method(function, name, length);
        self
    }

    /// Add a static method to the class.
    ///
    /// It is added to class object itself.
    pub fn static_method<N>(
        &mut self,
        name: N,
        length: usize,
        function: NativeFunction,
    ) -> &mut Self
    where
        N: Into<FunctionBinding>,
    {
        self.builder.static_method(function, name, length);
        self
    }

    /// Add a data property to the class, with the specified attribute.
    ///
    /// It is added to `prototype`.
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
    pub const fn realm(&self) -> &Realm {
        self.builder.realm()
    }
}
