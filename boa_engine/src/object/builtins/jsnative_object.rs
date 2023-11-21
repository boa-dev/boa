//! A Rust API wrapper for [`NativeObject`]s stored in Boa's builtin [`JsObject`].

use std::{marker::PhantomData, ops::Deref};

use boa_gc::{GcRef, GcRefMut};
use boa_macros::{Finalize, Trace};

use crate::{
    class::Class,
    object::{JsObjectType, NativeObject, Object, ObjectData},
    value::TryFromJs,
    Context, JsNativeError, JsObject, JsResult, JsValue,
};

/// [`JsNativeObject<T>`] provides a wrapper for a Rust type `T` stored as a
/// [`NativeObject`] in Boa's [`JsObject`].
#[derive(Debug, Trace, Finalize)]
pub struct JsNativeObject<T: NativeObject> {
    // INVARIANT: `inner.is::<T>() == true`
    inner: JsObject,
    marker: PhantomData<T>,
}

impl<T: NativeObject> Clone for JsNativeObject<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: self.marker,
        }
    }
}

impl<T: NativeObject> JsNativeObject<T> {
    /// Creates a [`JsNativeObject<T>`] from a valid [`JsObject`], or returns a `TypeError`
    /// if the provided object is not a [`NativeObject`] with type `T`.
    ///
    /// # Examples
    ///
    /// ### Valid Example - Matching types
    /// ```
    /// # use boa_engine::{
    /// #    object::{ObjectInitializer, builtins::JsNativeObject},
    /// #    Context, JsResult
    /// # };
    /// #
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::default();
    /// // Create a native object represented as a `JsObject`
    /// let buffer: Vec<u8> = vec![42; 6];
    /// let js_buffer = ObjectInitializer::with_native(buffer, context).build();
    ///
    /// // Create `JsNativeObject` from `JsObject`
    /// let js_buffer = JsNativeObject::<Vec<u8>>::from_object(js_buffer)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Invalid Example - Mismatching types
    /// ```
    /// # use boa_engine::{
    /// #    object::{ObjectInitializer, builtins::JsNativeObject},
    /// #    Context, JsObject, JsResult
    /// # };
    /// #
    /// # let context = &mut Context::default();
    /// let js_int = ObjectInitializer::with_native(42, context).build();
    ///
    /// // js_int is an int, not unit
    /// assert!(JsNativeObject::<()>::from_object(js_int).is_err());
    /// ```
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is::<T>() {
            Ok(Self {
                inner: object,
                marker: PhantomData,
            })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a native object of the expected type")
                .into())
        }
    }

    /// Creates a [`JsNativeObject<T>`] with the provided prototype and native object
    /// data of type `T`
    pub fn new_with_proto<P>(prototype: P, native_object: T, context: &mut Context) -> Self
    where
        P: Into<Option<JsObject>>,
    {
        let instance = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ObjectData::native_object(native_object),
        );

        Self {
            inner: instance,
            marker: PhantomData,
        }
    }

    /// Creates a [`JsNativeObject<T>`] with the native object data of type `T`
    /// and prototype of the native class implemented by `T`, or returns a
    /// `TypeError` if the native class is not registered in the context.
    ///
    /// # Example
    ///
    /// Create a [`JsNativeObject<Animal>`] using the example native class from
    /// [`Class`][./trait.Class.html].
    /// ```
    /// # use boa_engine::{
    /// #    js_string,
    /// #    class::{Class, ClassBuilder},
    /// #    object::{ObjectInitializer, builtins::JsNativeObject},
    /// #    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
    /// # };
    /// # use boa_gc::{Finalize, Trace};
    ///
    /// #[derive(Debug, Trace, Finalize)]
    /// enum Animal {
    ///     Cat,
    ///     Dog,
    ///     Other,
    /// }
    ///
    /// // Implement native class for `Animal` through the `Class` trait.
    /// impl Class for Animal {
    ///       // See `Class` documentation for implementation.
    ///       // ...
    /// #     // We set the binging name of this function to be `"Animal"`.
    /// #     const NAME: &'static str = "Animal";
    /// #
    /// #     // We set the length to `1` since we accept 1 arguments in the constructor.
    /// #     const LENGTH: usize = 1;
    /// #
    /// #     // This is what is called when we do `new Animal()` to construct the inner data of the class.
    /// #     fn make_data(_new_target: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
    /// #         // This is equivalent to `String(arg)`.
    /// #         let kind = args.get_or_undefined(0).to_string(context)?;
    /// #
    /// #         let animal = match kind.to_std_string_escaped().as_str() {
    /// #             "cat" => Self::Cat,
    /// #             "dog" => Self::Dog,
    /// #             _ => Self::Other,
    /// #         };
    /// #
    /// #         Ok(animal)
    /// #     }
    /// #
    /// #     /// This is where the object is initialized.
    /// #     fn init(class: &mut ClassBuilder) -> JsResult<()> {
    /// #         class.method(
    /// #             js_string!("speak"),
    /// #             0,
    /// #             NativeFunction::from_fn_ptr(|this, _args, _ctx| {
    /// #                 if let Some(object) = this.as_object() {
    /// #                     if let Some(animal) = object.downcast_ref::<Animal>() {
    /// #                         match &*animal {
    /// #                             Self::Cat => println!("meow"),
    /// #                             Self::Dog => println!("woof"),
    /// #                             Self::Other => println!(r"¯\_(ツ)_/¯"),
    /// #                         }
    /// #                     }
    /// #                 }
    /// #                 Ok(JsValue::undefined())
    /// #             }),
    /// #         );
    /// #         Ok(())
    /// #     }
    /// }
    ///
    /// # let context = &mut Context::default();
    ///
    /// // Create a `JsNativeObject<Animal>`
    /// let js_dog = JsNativeObject::new(Animal::Dog, context);
    /// ```
    pub fn new(native_object: T, context: &mut Context) -> JsResult<Self>
    where
        T: Class,
    {
        let class = context.get_global_class::<T>().ok_or_else(|| {
            JsNativeError::typ().with_message(format!(
                "could not find native class `{}` in the map of registered classes",
                T::NAME
            ))
        })?;

        Ok(Self::new_with_proto(
            class.prototype(),
            native_object,
            context,
        ))
    }

    /// Returns a reference to the native object data of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[must_use]
    pub fn as_ref(&self) -> GcRef<'_, T> {
        // SAFETY: The invariant of `JsNativeObject<T>` ensures that
        // `inner.is::<T>() == true`.
        self.inner
            .downcast_ref::<T>()
            .expect("Type mismatch in `JsNativeObject`")
    }

    /// Returns a mutable reference to the native object of type `T`.
    ///
    /// # Panic
    ///
    /// Panics if the object is currently borrowed.
    #[must_use]
    pub fn as_mut(&self) -> GcRefMut<'_, Object, T> {
        self.inner
            .downcast_mut::<T>()
            .expect("Type mismatch in `JsNativeObject`")
    }
}

impl<T> From<JsNativeObject<T>> for JsObject
where
    T: NativeObject,
{
    #[inline]
    fn from(o: JsNativeObject<T>) -> Self {
        o.inner.clone()
    }
}

impl<T> From<JsNativeObject<T>> for JsValue
where
    T: NativeObject,
{
    #[inline]
    fn from(o: JsNativeObject<T>) -> Self {
        o.inner.clone().into()
    }
}

impl<T> Deref for JsNativeObject<T>
where
    T: NativeObject,
{
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> JsObjectType for JsNativeObject<T> where T: NativeObject {}

impl<T> TryFromJs for JsNativeObject<T>
where
    T: NativeObject,
{
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Self::from_object(o.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("value is not a native object")
                .into()),
        }
    }
}
