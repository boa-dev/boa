//! This module implements the `JsObject` structure.
//!
//! The `JsObject` is a garbage collected Object.

use super::{
    internal_methods::{InternalObjectMethods, ARRAY_EXOTIC_INTERNAL_METHODS},
    shape::RootShape,
    JsPrototype, NativeObject, Object, PrivateName, PropertyMap,
};
use crate::{
    context::intrinsics::Intrinsics,
    error::JsNativeError,
    object::{ObjectData, ObjectKind},
    property::{PropertyDescriptor, PropertyKey},
    string::utf16,
    value::PreferredType,
    Context, JsResult, JsValue,
};
use boa_gc::{self, Finalize, Gc, GcRefCell, Trace};
use boa_interner::Sym;
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt::{self, Debug, Display},
    hash::Hash,
    result::Result as StdResult,
};
use thin_vec::ThinVec;

/// A wrapper type for an immutably borrowed type T.
pub type Ref<'a, T> = boa_gc::GcRef<'a, T>;

/// A wrapper type for a mutably borrowed type T.
pub type RefMut<'a, T, U> = boa_gc::GcRefMut<'a, T, U>;

/// Garbage collected `Object`.
#[derive(Trace, Finalize, Clone)]
pub struct JsObject {
    inner: Gc<VTableObject>,
}

/// An `Object` that has an additional `vtable` with its internal methods.
// We have to skip implementing `Debug` for this because not using the
// implementation of `Debug` for `JsObject` could easily cause stack overflows,
// so we have to force our users to debug the `JsObject` instead.
#[allow(missing_debug_implementations)]
#[derive(Trace, Finalize)]
pub struct VTableObject {
    object: GcRefCell<Object>,
    #[unsafe_ignore_trace]
    vtable: &'static InternalObjectMethods,
}

impl Default for JsObject {
    fn default() -> Self {
        let data = ObjectData::ordinary();
        Self::from_object_and_vtable(Object::default(), data.internal_methods)
    }
}

impl JsObject {
    /// Creates a new `JsObject` from its inner object and its vtable.
    pub(crate) fn from_object_and_vtable(
        object: Object,
        vtable: &'static InternalObjectMethods,
    ) -> Self {
        Self {
            inner: Gc::new(VTableObject {
                object: GcRefCell::new(object),
                vtable,
            }),
        }
    }
    /// Creates a new ordinary object with its prototype set to the `Object` prototype.
    ///
    /// This is equivalent to calling the specification's abstract operation
    /// [`OrdinaryObjectCreate(%Object.prototype%)`][call].
    ///
    /// [call]: https://tc39.es/ecma262/#sec-ordinaryobjectcreate
    #[inline]
    #[must_use]
    pub fn with_object_proto(intrinsics: &Intrinsics) -> Self {
        Self::from_proto_and_data(
            intrinsics.constructors().object().prototype(),
            ObjectData::ordinary(),
        )
    }

    /// Creates a new ordinary object, with its prototype set to null.
    ///
    /// This is equivalent to calling the specification's abstract operation
    /// [`OrdinaryObjectCreate(null)`][call].
    ///
    /// [call]: https://tc39.es/ecma262/#sec-ordinaryobjectcreate
    #[inline]
    #[must_use]
    pub fn with_null_proto() -> Self {
        Self::from_proto_and_data(None, ObjectData::ordinary())
    }

    /// Creates a new object with the provided prototype and object data.
    ///
    /// This is equivalent to calling the specification's abstract operation [`OrdinaryObjectCreate`],
    /// with the difference that the `additionalInternalSlotsList` parameter is automatically set by
    /// the [`ObjectData`] provided.
    ///
    /// [`OrdinaryObjectCreate`]: https://tc39.es/ecma262/#sec-ordinaryobjectcreate
    pub fn from_proto_and_data<O: Into<Option<Self>>>(prototype: O, data: ObjectData) -> Self {
        Self {
            inner: Gc::new(VTableObject {
                object: GcRefCell::new(Object {
                    kind: data.kind,
                    properties: PropertyMap::from_prototype_unique_shape(prototype.into()),
                    extensible: true,
                    private_elements: ThinVec::new(),
                }),
                vtable: data.internal_methods,
            }),
        }
    }

    /// Creates a new object with the provided prototype and object data.
    ///
    /// This is equivalent to calling the specification's abstract operation [`OrdinaryObjectCreate`],
    /// with the difference that the `additionalInternalSlotsList` parameter is automatically set by
    /// the [`ObjectData`] provided.
    ///
    /// [`OrdinaryObjectCreate`]: https://tc39.es/ecma262/#sec-ordinaryobjectcreate
    pub(crate) fn from_proto_and_data_with_shared_shape<O: Into<Option<Self>>>(
        root_shape: &RootShape,
        prototype: O,
        data: ObjectData,
    ) -> Self {
        Self {
            inner: Gc::new(VTableObject {
                object: GcRefCell::new(Object {
                    kind: data.kind,
                    properties: PropertyMap::from_prototype_with_shared_shape(
                        root_shape,
                        prototype.into(),
                    ),
                    extensible: true,
                    private_elements: ThinVec::new(),
                }),
                vtable: data.internal_methods,
            }),
        }
    }

    /// Immutably borrows the `Object`.
    ///
    /// The borrow lasts until the returned `Ref` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn borrow(&self) -> Ref<'_, Object> {
        self.try_borrow().expect("Object already mutably borrowed")
    }

    /// Mutably borrows the Object.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope.
    /// The object cannot be borrowed while this borrow is active.
    ///
    /// # Panics
    /// Panics if the object is currently borrowed.
    #[inline]
    #[track_caller]
    pub fn borrow_mut(&self) -> RefMut<'_, Object, Object> {
        self.try_borrow_mut().expect("Object already borrowed")
    }

    /// Immutably borrows the `Object`, returning an error if the value is currently mutably borrowed.
    ///
    /// The borrow lasts until the returned `GcCellRef` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    #[inline]
    pub fn try_borrow(&self) -> StdResult<Ref<'_, Object>, BorrowError> {
        self.inner.object.try_borrow().map_err(|_| BorrowError)
    }

    /// Mutably borrows the object, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts until the returned `GcCellRefMut` exits scope.
    /// The object be borrowed while this borrow is active.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    #[inline]
    pub fn try_borrow_mut(&self) -> StdResult<RefMut<'_, Object, Object>, BorrowMutError> {
        self.inner
            .object
            .try_borrow_mut()
            .map_err(|_| BorrowMutError)
    }

    /// Checks if the garbage collected memory is the same.
    #[inline]
    pub fn equals(lhs: &Self, rhs: &Self) -> bool {
        std::ptr::eq(lhs.as_ref(), rhs.as_ref())
    }

    /// Converts an object to a primitive.
    ///
    /// Diverges from the spec to prevent a stack overflow when the object is recursive.
    /// For example,
    /// ```javascript
    /// let a = [1];
    /// a[1] = a;
    /// console.log(a.toString()); // We print "1,"
    /// ```
    /// The spec doesn't mention what to do in this situation, but a naive implementation
    /// would overflow the stack recursively calling `toString()`. We follow v8 and SpiderMonkey
    /// instead by returning a default value for the given `hint` -- either `0.` or `""`.
    /// Example in v8: <https://repl.it/repls/IvoryCircularCertification#index.js>
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinarytoprimitive
    pub(crate) fn ordinary_to_primitive(
        &self,
        context: &mut Context<'_>,
        hint: PreferredType,
    ) -> JsResult<JsValue> {
        // 1. Assert: Type(O) is Object.
        //      Already is JsObject by type.
        // 2. Assert: Type(hint) is String and its value is either "string" or "number".
        debug_assert!(hint == PreferredType::String || hint == PreferredType::Number);

        // Diverge from the spec here to make sure we aren't going to overflow the stack by converting
        // a recursive structure
        // We can follow v8 & SpiderMonkey's lead and return a default value for the hint in this situation
        // (see https://repl.it/repls/IvoryCircularCertification#index.js)
        let recursion_limiter = RecursionLimiter::new(self.as_ref());
        if recursion_limiter.live {
            // we're in a recursive object, bail
            return Ok(match hint {
                PreferredType::Number => JsValue::new(0),
                PreferredType::String => JsValue::new(""),
                PreferredType::Default => unreachable!("checked type hint in step 2"),
            });
        }

        // 3. If hint is "string", then
        //    a. Let methodNames be « "toString", "valueOf" ».
        // 4. Else,
        //    a. Let methodNames be « "valueOf", "toString" ».
        let method_names = if hint == PreferredType::String {
            [utf16!("toString"), utf16!("valueOf")]
        } else {
            [utf16!("valueOf"), utf16!("toString")]
        };

        // 5. For each name in methodNames in List order, do
        for name in method_names {
            // a. Let method be ? Get(O, name).
            let method = self.get(name, context)?;

            // b. If IsCallable(method) is true, then
            if let Some(method) = method.as_callable() {
                // i. Let result be ? Call(method, O).
                let result = method.call(&self.clone().into(), &[], context)?;

                // ii. If Type(result) is not Object, return result.
                if !result.is_object() {
                    return Ok(result);
                }
            }
        }

        // 6. Throw a TypeError exception.
        Err(JsNativeError::typ()
            .with_message("cannot convert object to primitive value")
            .into())
    }

    /// Return `true` if it is a native object and the native type is `T`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[track_caller]
    pub fn is<T>(&self) -> bool
    where
        T: NativeObject,
    {
        self.borrow().is::<T>()
    }

    /// Downcast a reference to the object,
    /// if the object is type native object type `T`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[track_caller]
    pub fn downcast_ref<T>(&self) -> Option<Ref<'_, T>>
    where
        T: NativeObject,
    {
        let object = self.borrow();
        if object.is::<T>() {
            Some(Ref::map(object, |x| {
                x.downcast_ref::<T>().expect("downcasting reference failed")
            }))
        } else {
            None
        }
    }

    /// Downcast a mutable reference to the object,
    /// if the object is type native object type `T`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently borrowed.
    #[track_caller]
    pub fn downcast_mut<T>(&self) -> Option<RefMut<'_, Object, T>>
    where
        T: NativeObject,
    {
        let object = self.borrow_mut();
        if object.is::<T>() {
            Some(RefMut::map(object, |x| {
                x.downcast_mut::<T>()
                    .expect("downcasting mutable reference failed")
            }))
        } else {
            None
        }
    }

    /// Get the prototype of the object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn prototype(&self) -> JsPrototype {
        self.borrow().prototype()
    }

    /// Get the extensibility of the object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    pub(crate) fn extensible(&self) -> bool {
        self.borrow().extensible
    }

    /// Set the prototype of the object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed
    #[inline]
    #[track_caller]
    pub fn set_prototype(&self, prototype: JsPrototype) -> bool {
        self.borrow_mut().set_prototype(prototype)
    }

    /// Checks if it's an `Array` object.
    #[inline]
    #[track_caller]
    pub fn is_array(&self) -> bool {
        std::ptr::eq(self.vtable(), &ARRAY_EXOTIC_INTERNAL_METHODS)
    }

    /// Checks if it's a `DataView` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_data_view(&self) -> bool {
        self.borrow().is_data_view()
    }

    /// Checks if it is an `ArrayIterator` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_array_iterator(&self) -> bool {
        self.borrow().is_array_iterator()
    }

    /// Checks if it's an `ArrayBuffer` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_array_buffer(&self) -> bool {
        self.borrow().is_array_buffer()
    }

    /// Checks if it is a `Map` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_map(&self) -> bool {
        self.borrow().is_map()
    }

    /// Checks if it's a `MapIterator` object
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_map_iterator(&self) -> bool {
        self.borrow().is_map_iterator()
    }

    /// Checks if it is a `Set` object
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_set(&self) -> bool {
        self.borrow().is_set()
    }

    /// Checks if it is a `SetIterator` object
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_set_iterator(&self) -> bool {
        self.borrow().is_set_iterator()
    }

    /// Checks if it's a `String` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_string(&self) -> bool {
        self.borrow().is_string()
    }

    /// Checks if it's a `Function` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_function(&self) -> bool {
        self.borrow().is_function()
    }

    /// Checks if it's a `Generator` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_generator(&self) -> bool {
        self.borrow().is_generator()
    }

    /// Checks if it's a `Symbol` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_symbol(&self) -> bool {
        self.borrow().is_symbol()
    }

    /// Checks if it's an `Error` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_error(&self) -> bool {
        self.borrow().is_error()
    }

    /// Checks if it's a `Boolean` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_boolean(&self) -> bool {
        self.borrow().is_boolean()
    }

    /// Checks if it's a `Number` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_number(&self) -> bool {
        self.borrow().is_number()
    }

    /// Checks if it's a `BigInt` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_bigint(&self) -> bool {
        self.borrow().is_bigint()
    }

    /// Checks if it's a `Date` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_date(&self) -> bool {
        self.borrow().is_date()
    }

    /// Checks if it's a `RegExp` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_regexp(&self) -> bool {
        self.borrow().is_regexp()
    }

    /// Checks if it's a `TypedArray` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_typed_array(&self) -> bool {
        self.borrow().is_typed_array()
    }

    /// Checks if it's a `Uint8Array` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_typed_uint8_array(&self) -> bool {
        self.borrow().is_typed_uint8_array()
    }

    /// Checks if it's a `Int8Array` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_typed_int8_array(&self) -> bool {
        self.borrow().is_typed_int8_array()
    }

    /// Checks if it's a `Uint16Array` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_typed_uint16_array(&self) -> bool {
        self.borrow().is_typed_uint16_array()
    }

    /// Checks if it's a `Int16Array` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_typed_int16_array(&self) -> bool {
        self.borrow().is_typed_int16_array()
    }

    /// Checks if it's a `Uint32Array` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_typed_uint32_array(&self) -> bool {
        self.borrow().is_typed_uint32_array()
    }

    /// Checks if it's a `Int32Array` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_typed_int32_array(&self) -> bool {
        self.borrow().is_typed_int32_array()
    }

    /// Checks if it's a `Float32Array` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_typed_float32_array(&self) -> bool {
        self.borrow().is_typed_float32_array()
    }

    /// Checks if it's a `Float64Array` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_typed_float64_array(&self) -> bool {
        self.borrow().is_typed_float64_array()
    }

    /// Checks if it's a `Promise` object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_promise(&self) -> bool {
        self.borrow().is_promise()
    }

    /// Checks if it's an ordinary object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_ordinary(&self) -> bool {
        self.borrow().is_ordinary()
    }

    /// Checks if it's a proxy object.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_proxy(&self) -> bool {
        self.borrow().is_proxy()
    }

    /// Returns `true` if it holds an Rust type that implements `NativeObject`.
    ///
    /// # Panics
    ///
    /// Panics if the object is currently mutably borrowed.
    #[inline]
    #[track_caller]
    pub fn is_native_object(&self) -> bool {
        self.borrow().is_native_object()
    }

    /// The abstract operation `ToPropertyDescriptor`.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-topropertydescriptor
    pub fn to_property_descriptor(
        &self,
        context: &mut Context<'_>,
    ) -> JsResult<PropertyDescriptor> {
        // 1 is implemented on the method `to_property_descriptor` of value

        // 2. Let desc be a new Property Descriptor that initially has no fields.
        let mut desc = PropertyDescriptor::builder();

        // 3. Let hasEnumerable be ? HasProperty(Obj, "enumerable").
        // 4. If hasEnumerable is true, then ...
        if self.has_property(utf16!("enumerable"), context)? {
            // a. Let enumerable be ! ToBoolean(? Get(Obj, "enumerable")).
            // b. Set desc.[[Enumerable]] to enumerable.
            desc = desc.enumerable(self.get(utf16!("enumerable"), context)?.to_boolean());
        }

        // 5. Let hasConfigurable be ? HasProperty(Obj, "configurable").
        // 6. If hasConfigurable is true, then ...
        if self.has_property(utf16!("configurable"), context)? {
            // a. Let configurable be ! ToBoolean(? Get(Obj, "configurable")).
            // b. Set desc.[[Configurable]] to configurable.
            desc = desc.configurable(self.get(utf16!("configurable"), context)?.to_boolean());
        }

        // 7. Let hasValue be ? HasProperty(Obj, "value").
        // 8. If hasValue is true, then ...
        if self.has_property(utf16!("value"), context)? {
            // a. Let value be ? Get(Obj, "value").
            // b. Set desc.[[Value]] to value.
            desc = desc.value(self.get(utf16!("value"), context)?);
        }

        // 9. Let hasWritable be ? HasProperty(Obj, ).
        // 10. If hasWritable is true, then ...
        if self.has_property(utf16!("writable"), context)? {
            // a. Let writable be ! ToBoolean(? Get(Obj, "writable")).
            // b. Set desc.[[Writable]] to writable.
            desc = desc.writable(self.get(utf16!("writable"), context)?.to_boolean());
        }

        // 11. Let hasGet be ? HasProperty(Obj, "get").
        // 12. If hasGet is true, then
        let get = if self.has_property(utf16!("get"), context)? {
            // a. Let getter be ? Get(Obj, "get").
            let getter = self.get(utf16!("get"), context)?;
            // b. If IsCallable(getter) is false and getter is not undefined, throw a TypeError exception.
            // todo: extract IsCallable to be callable from Value
            if !getter.is_undefined() && getter.as_object().map_or(true, |o| !o.is_callable()) {
                return Err(JsNativeError::typ()
                    .with_message("Property descriptor getter must be callable")
                    .into());
            }
            // c. Set desc.[[Get]] to getter.
            Some(getter)
        } else {
            None
        };

        // 13. Let hasSet be ? HasProperty(Obj, "set").
        // 14. If hasSet is true, then
        let set = if self.has_property(utf16!("set"), context)? {
            // 14.a. Let setter be ? Get(Obj, "set").
            let setter = self.get(utf16!("set"), context)?;
            // 14.b. If IsCallable(setter) is false and setter is not undefined, throw a TypeError exception.
            // todo: extract IsCallable to be callable from Value
            if !setter.is_undefined() && setter.as_object().map_or(true, |o| !o.is_callable()) {
                return Err(JsNativeError::typ()
                    .with_message("Property descriptor setter must be callable")
                    .into());
            }
            // 14.c. Set desc.[[Set]] to setter.
            Some(setter)
        } else {
            None
        };

        // 15. If desc.[[Get]] is present or desc.[[Set]] is present, then ...
        // a. If desc.[[Value]] is present or desc.[[Writable]] is present, throw a TypeError exception.
        if get.as_ref().or(set.as_ref()).is_some() && desc.inner().is_data_descriptor() {
            return Err(JsNativeError::typ()
                .with_message(
                    "Invalid property descriptor.\
Cannot both specify accessors and a value or writable attribute",
                )
                .into());
        }

        desc = desc.maybe_get(get).maybe_set(set);

        // 16. Return desc.
        Ok(desc.build())
    }

    /// `7.3.25 CopyDataProperties ( target, source, excludedItems )`
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-copydataproperties
    pub fn copy_data_properties<K>(
        &self,
        source: &JsValue,
        excluded_keys: Vec<K>,
        context: &mut Context<'_>,
    ) -> JsResult<()>
    where
        K: Into<PropertyKey>,
    {
        // 1. Assert: Type(target) is Object.
        // 2. Assert: excludedItems is a List of property keys.
        // 3. If source is undefined or null, return target.
        if source.is_null_or_undefined() {
            return Ok(());
        }

        // 4. Let from be ! ToObject(source).
        let from = source
            .to_object(context)
            .expect("function ToObject should never complete abruptly here");

        // 5. Let keys be ? from.[[OwnPropertyKeys]]().
        // 6. For each element nextKey of keys, do
        let excluded_keys: Vec<PropertyKey> = excluded_keys.into_iter().map(Into::into).collect();
        for key in from.__own_property_keys__(context)? {
            // a. Let excluded be false.
            let mut excluded = false;

            // b. For each element e of excludedItems, do
            for e in &excluded_keys {
                // i. If SameValue(e, nextKey) is true, then
                if *e == key {
                    // 1. Set excluded to true.
                    excluded = true;
                    break;
                }
            }
            // c. If excluded is false, then
            if !excluded {
                // i. Let desc be ? from.[[GetOwnProperty]](nextKey).
                let desc = from.__get_own_property__(&key, context)?;

                // ii. If desc is not undefined and desc.[[Enumerable]] is true, then
                if let Some(desc) = desc {
                    if let Some(enumerable) = desc.enumerable() {
                        if enumerable {
                            // 1. Let propValue be ? Get(from, nextKey).
                            let prop_value = from.__get__(&key, from.clone().into(), context)?;

                            // 2. Perform ! CreateDataPropertyOrThrow(target, nextKey, propValue).
                            self.create_data_property_or_throw(key, prop_value, context)
                                .expect(
                                    "CreateDataPropertyOrThrow should never complete abruptly here",
                                );
                        }
                    }
                }
            }
        }

        // 7. Return target.
        Ok(())
    }

    pub(crate) fn get_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        let mut obj = Some(self.clone());

        while let Some(o) = obj {
            if let Some(v) = o.borrow().properties.get(key) {
                return Some(v);
            }
            obj = o.borrow().prototype().clone();
        }
        None
    }

    /// Helper function for property insertion.
    #[track_caller]
    pub(crate) fn insert<K, P>(&self, key: K, property: P) -> bool
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.borrow_mut().insert(key, property)
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name, than `true` is returned
    /// with that field, otherwise `false` is returned.
    pub fn insert_property<K, P>(&self, key: K, property: P) -> bool
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.insert(key.into(), property)
    }

    /// It determines if Object is a callable function with a `[[Call]]` internal method.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iscallable
    #[inline]
    #[track_caller]
    pub fn is_callable(&self) -> bool {
        self.inner.vtable.__call__.is_some()
    }

    /// It determines if Object is a function object with a `[[Construct]]` internal method.
    ///
    /// More information:
    /// - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isconstructor
    #[inline]
    #[track_caller]
    pub fn is_constructor(&self) -> bool {
        self.inner.vtable.__construct__.is_some()
    }

    /// Returns true if the `JsObject` is the global for a Realm
    pub fn is_global(&self) -> bool {
        matches!(self.inner.object.borrow().kind, ObjectKind::Global)
    }

    pub(crate) fn vtable(&self) -> &'static InternalObjectMethods {
        self.inner.vtable
    }

    pub(crate) const fn inner(&self) -> &Gc<VTableObject> {
        &self.inner
    }

    /// Create a new private name with this object as the unique identifier.
    pub(crate) fn private_name(&self, description: Sym) -> PrivateName {
        let ptr: *const _ = self.as_ref();
        PrivateName::new(description, ptr as usize)
    }
}

impl AsRef<GcRefCell<Object>> for JsObject {
    #[inline]
    fn as_ref(&self) -> &GcRefCell<Object> {
        &self.inner.object
    }
}

impl From<Gc<VTableObject>> for JsObject {
    #[inline]
    fn from(inner: Gc<VTableObject>) -> Self {
        Self { inner }
    }
}

impl PartialEq for JsObject {
    fn eq(&self, other: &Self) -> bool {
        Self::equals(self, other)
    }
}

impl Eq for JsObject {}

impl Hash for JsObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.as_ref(), state);
    }
}

/// An error returned by [`JsObject::try_borrow`](struct.JsObject.html#method.try_borrow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowError;

impl Display for BorrowError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already mutably borrowed", f)
    }
}

impl Error for BorrowError {}

/// An error returned by [`JsObject::try_borrow_mut`](struct.JsObject.html#method.try_borrow_mut).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowMutError;

impl Display for BorrowMutError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already borrowed", f)
    }
}

impl Error for BorrowMutError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RecursionValueState {
    /// This value is "live": there's an active RecursionLimiter that hasn't been dropped.
    Live,
    /// This value has been seen before, but the recursion limiter has been dropped.
    /// For example:
    /// ```javascript
    /// let b = [];
    /// JSON.stringify([ // Create a recursion limiter for the root here
    ///    b,            // state for b's &JsObject here is None
    ///    b,            // state for b's &JsObject here is Visited
    /// ]);
    /// ```
    Visited,
}

/// Prevents infinite recursion during `Debug::fmt`, `JSON.stringify`, and other conversions.
/// This uses a thread local, so is not safe to use where the object graph will be traversed by
/// multiple threads!
#[derive(Debug)]
pub struct RecursionLimiter {
    /// If this was the first `JsObject` in the tree.
    top_level: bool,
    /// The ptr being kept in the HashSet, so we can delete it when we drop.
    ptr: usize,
    /// If this JsObject has been visited before in the graph, but not in the current branch.
    pub visited: bool,
    /// If this JsObject has been visited in the current branch of the graph.
    pub live: bool,
}

impl Drop for RecursionLimiter {
    fn drop(&mut self) {
        if self.top_level {
            // When the top level of the graph is dropped, we can free the entire map for the next traversal.
            SEEN.with(|hm| hm.borrow_mut().clear());
        } else if !self.live {
            // This was the first RL for this object to become live, so it's no longer live now that it's dropped.
            SEEN.with(|hm| {
                hm.borrow_mut()
                    .insert(self.ptr, RecursionValueState::Visited)
            });
        }
    }
}

thread_local! {
    /// The map of pointers to `JsObject` that have been visited during the current `Debug::fmt` graph,
    /// and the current state of their RecursionLimiter (dropped or live -- see `RecursionValueState`)
    static SEEN: RefCell<HashMap<usize, RecursionValueState>> = RefCell::new(HashMap::new());
}

impl RecursionLimiter {
    /// Determines if the specified `T` has been visited, and returns a struct that will free it when dropped.
    ///
    /// This is done by maintaining a thread-local hashset containing the pointers of `T` values that have been
    /// visited. The first `T` visited will clear the hashset, while any others will check if they are contained
    /// by the hashset.
    pub fn new<T>(o: &T) -> Self {
        // We shouldn't have to worry too much about this being moved during Debug::fmt.
        #[allow(trivial_casts)]
        let ptr = (o as *const _) as usize;
        let (top_level, visited, live) = SEEN.with(|hm| {
            let mut hm = hm.borrow_mut();
            let top_level = hm.is_empty();
            let old_state = hm.insert(ptr, RecursionValueState::Live);

            (
                top_level,
                old_state == Some(RecursionValueState::Visited),
                old_state == Some(RecursionValueState::Live),
            )
        });

        Self {
            top_level,
            ptr,
            visited,
            live,
        }
    }
}

impl Debug for JsObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr: *const _ = self.vtable();
        let limiter = RecursionLimiter::new(self.as_ref());

        // Typically, using `!limiter.live` would be good enough here.
        // However, the JS object hierarchy involves quite a bit of repitition, and the sheer amount of data makes
        // understanding the Debug output impossible; limiting the usefulness of it.
        //
        // Instead, we check if the object has appeared before in the entire graph. This means that objects will appear
        // at most once, hopefully making things a bit clearer.
        if !limiter.visited && !limiter.live {
            f.debug_struct("JsObject")
                .field("vtable", &ptr)
                .field("object", &self.inner.object)
                .finish()
        } else {
            f.write_str("{ ... }")
        }
    }
}
