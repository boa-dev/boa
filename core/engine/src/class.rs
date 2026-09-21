//! Traits and structs for implementing native classes.
//!
//! Native classes are implemented through the [`Class`][class-trait] trait.
//!
//! [class-trait]: ./trait.Class.html

use crate::{
    Context, JsData, JsResult, JsValue,
    context::intrinsics::StandardConstructor,
    error::JsNativeError,
    native_function::NativeFunction,
    object::{ConstructorBuilder, FunctionBinding, JsFunction, JsObject, NativeObject, PROTOTYPE},
    property::{Attribute, PropertyDescriptor, PropertyKey},
};
use boa_gc::{Finalize, GcRefCell, Trace};
use std::any::{Any, TypeId};

/// Sentinel type for native classes with no parent.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct NoParent;

impl Class for NoParent {
    const NAME: &'static str = "";
    const LENGTH: usize = 0;

    type Parent = NoParent;
    type Data = ();
    const DEPTH: usize = 0;
    const INDEX: usize = 0;

    fn init(_: &mut ClassBuilder<'_>) -> JsResult<()> {
        Ok(())
    }

    fn data_constructor(_new_target: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<Self::Data> {
        Ok(())
    }
}

/// Native class.
///
/// See the [module-level documentation][self] for more details.
pub trait Class: NativeObject + Sized {
    /// The binding name of this class.
    const NAME: &'static str;
    /// The amount of arguments this class' constructor takes. Default is `0`.
    const LENGTH: usize = 0;
    /// The property attributes of this class' constructor in the global object.
    /// Default is `writable`, `enumerable`, `configurable`.
    const ATTRIBUTES: Attribute = Attribute::all();

    /// The parent class in the native inheritance chain.
    type Parent: Class;
    /// The data owned by this class' layer in the final instance layout.
    type Data: Any + Trace + JsData;

    /// Number of real layers from the chain root down to and including `Self`.
    /// `NoParent::DEPTH = 0`; a root class has `DEPTH = 1`.
    const DEPTH: usize = <Self::Parent as Class>::DEPTH + 1;
    /// The slot position of `Self`'s own data: `DEPTH - 1`.
    const INDEX: usize = Self::DEPTH - 1;

    /// Initializes the properties and methods of this class.
    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()>;

    /// Creates the data for this class' layer from the arguments passed to `new`.
    fn data_constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self::Data>;

    /// Returns the arguments that should be forwarded to the parent class'
    /// constructor. By default the same arguments are forwarded unchanged.
    #[allow(unused_variables)]
    fn parent_args<'a>(
        new_target: &JsValue,
        args: &'a [JsValue],
        context: &mut Context,
    ) -> JsResult<&'a [JsValue]> {
        Ok(&[])
    }

    /// Initializes the properties of the constructed object for an instance of this class.
    ///
    /// Useful to initialize additional properties for the constructed object that aren't
    /// stored inside the native data.
    #[allow(unused_variables)]
    fn object_constructor(
        instance: &JsObject,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<()> {
        Ok(())
    }

    /// Creates a new [`JsObject`] with its internal data set to a [`NativeClassData`]
    /// sized for the inheritance chain rooted at this class, then recursively builds
    /// every parent layer and this layer in order.
    ///
    /// # Errors
    ///
    /// - Throws an error if `new_target` is undefined.
    /// - Throws an error if this class is not registered in `new_target`'s realm.
    ///   See [`Context::register_global_class`].
    fn construct(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsObject> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "cannot call constructor of native class `{}` without new",
                    Self::NAME
                ))
                .into());
        }

        let prototype = 'proto: {
            let realm = if let Some(constructor) = new_target.as_object() {
                if let Some(proto) = constructor.get(PROTOTYPE, context)?.as_object() {
                    break 'proto proto.clone();
                }
                constructor.get_function_realm(context)?
            } else {
                context.realm().clone()
            };
            realm
                .get_class::<Self>()
                .ok_or_else(|| {
                    JsNativeError::typ().with_message(format!(
                        "could not find native class `{}` in the map of registered classes",
                        Self::NAME
                    ))
                })?
                .prototype()
        };

        let data = NativeClassData::new::<Self>();

        let object =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);
        let object = object.upcast();

        build_chain::<Self>(&object, args, context)?;
        Self::object_constructor(&object, args, context)?;

        Ok(object)
    }

    /// Constructs an instance of this class from its own layer data.
    ///
    /// # Errors
    /// - Throws an error if this class is not registered in the context's realm. See
    ///   [`Context::register_global_class`].
    /// - Panics if this class has a native parent; use the JS `new` path or construct the
    ///   full chain from Rust.
    fn from_data(data: Self::Data, context: &mut Context) -> JsResult<JsObject> {
        assert!(
            TypeId::of::<Self::Parent>() == TypeId::of::<NoParent>(),
            "Class::from_data is only valid for root classes"
        );

        let prototype = context
            .get_global_class::<Self>()
            .ok_or_else(|| {
                JsNativeError::typ().with_message(format!(
                    "could not find native class `{}` in the map of registered classes",
                    Self::NAME
                ))
            })?
            .prototype();

        let carrier = NativeClassData::new::<Self>();
        carrier.set_slot::<Self>(data);

        let object =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, carrier);
        let object = object.upcast();

        Self::object_constructor(&object, &[], context)?;

        Ok(object)
    }
}

// ── Native class data carrier ─────────────────────────────────────────

/// Trait object stored in each slot of a [`NativeClassData`].
pub trait ClassSlot: Any + Trace {
    /// Returns a reference to the underlying data as `dyn Any`.
    fn as_any_ref(&self) -> &dyn Any;
    /// Returns a mutable reference to the underlying data as `dyn Any`.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Trace> ClassSlot for T {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// The per-instance native data store for a class hierarchy.
///
/// One slot per real layer in the chain (`NoParent` takes none); `T::INDEX`
/// addresses a class' slot directly. The slot list is sized to the leaf class'
/// `DEPTH` once at construction time and never resized.
#[derive(Trace, Finalize, JsData)]
pub struct NativeClassData {
    slots: Box<[GcRefCell<Option<Box<dyn ClassSlot>>>]>,
}

impl std::fmt::Debug for NativeClassData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeClassData")
            .field("slots", &self.slots.len())
            .finish()
    }
}

impl NativeClassData {
    /// Build a registry sized for a chain ending at `T` (the leaf class).
    fn new<T: Class>() -> Self {
        Self {
            slots: (0..T::DEPTH).map(|_| GcRefCell::new(None)).collect(),
        }
    }

    /// Write `T`'s data into its slot. `T::INDEX` is a compile-time constant.
    fn set_slot<T: Class>(&self, data: T::Data) {
        *self.slots[T::INDEX].borrow_mut() = Some(Box::new(data));
    }

    /// Read a class' data through the callback.
    fn with<T: Class, R>(&self, f: impl FnOnce(&T::Data) -> R) -> JsResult<R> {
        let slot = self.slots.get(T::INDEX).ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "native class data index {} out of range",
                T::INDEX
            ))
        })?;

        let borrowed = slot.borrow();
        let Some(slot_ref) = borrowed.as_deref() else {
            return Err(JsNativeError::typ()
                .with_message("native class data slot not initialized")
                .into());
        };

        let data = slot_ref.as_any_ref().downcast_ref::<T::Data>().ok_or_else(|| {
            JsNativeError::typ().with_message("native class data type mismatch")
        })?;

        Ok(f(data))
    }

    /// Mutate a class' data through the callback.
    fn with_mut<T: Class, R>(&self, f: impl FnOnce(&mut T::Data) -> R) -> JsResult<R> {
        let slot = self.slots.get(T::INDEX).ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "native class data index {} out of range",
                T::INDEX
            ))
        })?;

        let mut borrowed = slot.borrow_mut();
        let Some(slot_ref) = borrowed.as_deref_mut() else {
            return Err(JsNativeError::typ()
                .with_message("native class data slot not initialized")
                .into());
        };

        let data = slot_ref.as_any_mut().downcast_mut::<T::Data>().ok_or_else(|| {
            JsNativeError::typ().with_message("native class data type mismatch")
        })?;

        Ok(f(data))
    }
}

/// Recursively build every layer of the hierarchy into the same final instance,
/// parent first.
fn build_chain<T: Class>(instance: &JsObject, args: &[JsValue], context: &mut Context) -> JsResult<()> {
    if TypeId::of::<T::Parent>() != TypeId::of::<NoParent>() {
        let new_target = JsValue::undefined();
        let parent_args = T::parent_args(&new_target, args, context)?;
        build_chain::<T::Parent>(instance, parent_args, context)?;
    }

    let new_target = JsValue::undefined();
    let data = T::data_constructor(&new_target, args, context)?;
    set_class_data::<T>(instance, data)?;

    Ok(())
}

/// Write a class' layer data into the instance's native data carrier.
fn set_class_data<T: Class>(instance: &JsObject, data: T::Data) -> JsResult<()> {
    let registry = instance
        .downcast_ref::<NativeClassData>()
        .ok_or_else(|| JsNativeError::typ().with_message("instance has no native class data"))?;
    registry.set_slot::<T>(data);
    Ok(())
}

/// Read a class' layer data from the instance.
pub fn with_class_data<T: Class, R>(
    instance: &JsObject,
    f: impl FnOnce(&T::Data) -> R,
) -> JsResult<R> {
    let registry = instance
        .downcast_ref::<NativeClassData>()
        .ok_or_else(|| JsNativeError::typ().with_message("instance has no native class data"))?;
    registry.with::<T, R>(f)
}

/// Mutate a class' layer data on the instance.
pub fn with_class_data_mut<T: Class, R>(
    instance: &JsObject,
    f: impl FnOnce(&mut T::Data) -> R,
) -> JsResult<R> {
    let registry = instance
        .downcast_ref::<NativeClassData>()
        .ok_or_else(|| JsNativeError::typ().with_message("instance has no native class data"))?;
    registry.with_mut::<T, R>(f)
}

// ── Class builder ─────────────────────────────────────────────────────

/// Class builder which allows adding methods and static methods to the class.
#[derive(Debug)]
pub struct ClassBuilder<'ctx> {
    builder: ConstructorBuilder<'ctx>,
}

impl<'ctx> ClassBuilder<'ctx> {
    /// Create a new `ClassBuilder` from a [`Class`] type.
    pub fn new<T>(context: &'ctx mut Context) -> Self
    where
        T: Class,
    {
        let (parent_proto, parent_ctor): (Option<JsObject>, Option<JsObject>) =
            if TypeId::of::<T::Parent>() != TypeId::of::<NoParent>() {
                let parent_sc = context.get_global_class::<T::Parent>().expect(
                    "parent class must be registered before its children",
                );
                (Some(parent_sc.prototype()), Some(parent_sc.constructor()))
            } else {
                (None, None)
            };

        let mut builder = ConstructorBuilder::new(
            context,
            NativeFunction::from_fn_ptr(|t, a, c| T::construct(t, a, c).map(JsValue::from)),
        );
        builder.name(T::NAME);
        builder.length(T::LENGTH);

        if let Some(proto) = parent_proto {
            builder.inherit(proto);
        }
        if let Some(ctor) = parent_ctor {
            builder.custom_prototype(ctor);
        }

        Self { builder }
    }

    /// Create the [`StandardConstructor`] from this class builder.
    #[must_use]
    pub fn build(self) -> StandardConstructor {
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
    pub fn context(&mut self) -> &mut Context {
        self.builder.context()
    }
}
