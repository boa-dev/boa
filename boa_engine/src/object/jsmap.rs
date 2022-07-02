// This module is a wrapper for the Map Builtin Javascript Object
use crate::{
    builtins::map::{add_entries_from_iterable, ordered_map::OrderedMap},
    builtins::Map,
    object::{JsFunction, JsMapIterator, JsObject, JsObjectType, ObjectData},
    Context, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

<<<<<<< HEAD
/// `JsMap` provides an API wrapper for Boa's implementation of the Javascript `Map` object.
///
/// # Examples
///
/// Create a `JsMap` and set a new entry
/// ```
/// fn main() -> JsResult<()> {
///     // Create default `Context`
///     let context = &mut Context::default();
///     // Create a new empty `JsMap`.
///     let map = JsMap::new(context);
///
///     // Set key-value pairs for the `JsMap`.
///     map.set("Key-1", "Value-1", context)?;
///     map.set("Key-2", 10, context)?;
///
///     assert_eq!(map.get_size(context)?, 2.into());
///
///     Ok(())
/// }
/// ```
///
/// Create a `JsMap` from a `JsArray`
/// ```
/// fn main() -> JsResult<()> {
///     // Create a default `Context`
///     let context = &mut Context::default();
///
///     // Create an array of two `[key, value]` pairs
///     let js_array = JsArray::new(context);
///     
///     // Create a `[key, value]` pair of JsValues
///     let vec_one: Vec<JsValue> = vec![JsValue::new("first-key"), JsValue::new("first-value")];
///
///     // We create an push our `[key, value]` pair onto our array as a `JsArray`
///     js_array.push(JsArray::from_iter(vec_one, context), context)?;
///
///     // Create a `JsMap` from the `JsArray` using it's iterable property.
///     let js_iterable_map = JsMap::from_js_iterable(&js_array.into(), context)?;
///
///     assert_eq!(iter_map.get("first-key", context)?, "first-value".into());
///
///     Ok(())
/// }
/// ```
///
=======
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsMap {
    inner: JsObject,
}

impl JsMap {
<<<<<<< HEAD
    /// Creates a new empty [`JsMap`] object.
    ///
    /// # Example
    ///
    /// ```
    /// fn main -> JsResult<()> {
    ///    // Create a new context.
    ///    let context = &mut Context::default();
    ///
    ///    // Create a new empty `JsMap`.
    ///    let map = JsMap::new(context);
    /// }
    /// ```
=======
    /// Create new Empty Map Object.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn new(context: &mut Context) -> Self {
        let map = Self::create_map(context);
        Self { inner: map }
    }

<<<<<<< HEAD
    /// Create a new [`JsMap`] object from a [`JsObject`] that has an `@@Iterator` field.
    ///
    /// # Examples
    /// ```
    /// fn main() -> JsResult<()> {
    ///     // Create a default `Context`
    ///     let context = &mut Context::default();
    ///
    ///     // Create an array of two `[key, value]` pairs
    ///     let js_array = JsArray::new(context);
    ///     
    ///     // Create a `[key, value]` pair of JsValues and add it to the `JsArray` as a `JsArray`
    ///     let vec_one: Vec<JsValue> = vec![JsValue::new("first-key"), JsValue::new("first-value")];
    ///     js_array.push(JsArray::from_iter(vec_one, context), context)?;
    ///
    ///     // Create a `JsMap` from the `JsArray` using it's iterable property.
    ///     let js_iterable_map = JsMap::from_js_iterable(&js_array.into(), context)?;
    /// }
    /// ```
    ///
=======
    /// Create a new map object for any object that has a `@@Iterator` field.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn from_js_iterable(iterable: &JsValue, context: &mut Context) -> JsResult<Self> {
        // Create a new map object.
        let map = Self::create_map(context);

        // Let adder be Get(map, "set") per spec. This action should not fail with default map.
        let adder = map
            .get("set", context)
            .expect("creating a map with the default prototype must not fail");

        let _completion_record = add_entries_from_iterable(&map, iterable, &adder, context)?;

        Ok(Self { inner: map })
    }

<<<<<<< HEAD
    /// Creates a [`JsMap`] from a valid [`JsObject`], or returns a `TypeError` if the provided object is not a [`JsMap`]
    ///
    /// # Examples
    ///
    /// Valid Example - returns JsMap object
    /// ```
    ///     // `some_object` can be any JavaScript `Map` object.
    ///     let some_object = JsObject::from_proto_and_data(
    ///         context.intrinsics().constructors().map().prototype(),
    ///         ObjectData::map(Ordered::new())
    ///     );
    ///     
    ///     // Create `JsMap` object with incoming object.
    ///     let js_map = JsMap::from_object(some_object, context)?;
    /// ```
    /// 
    /// Invalid Example - returns `TypeError` with the message "object is not a Map"
    /// ```
    ///     let some_object = JsArray::new(context);
    /// 
    ///     let js_map = JsMap::from_object(some_object, context)?;
    /// ```
=======
    /// Create a [`JsMap`] from a [`JsObject`], if the object is not a `Map`, throw a `TypeError`.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn from_object(object: JsObject, context: &mut Context) -> JsResult<Self> {
        if object.borrow().is_map() {
            Ok(Self { inner: object })
        } else {
            context.throw_type_error("object is not a Map")
        }
    }

<<<<<<< HEAD
    // Utility function to generate the default `Map` object.
=======
    // Utility function to generate default Map object.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    fn create_map(context: &mut Context) -> JsObject {
        // Get default Map prototype
        let prototype = context.intrinsics().constructors().map().prototype();

        // Create a default map object with [[MapData]] as a new empty list
        JsObject::from_proto_and_data(prototype, ObjectData::map(OrderedMap::new()))
    }

<<<<<<< HEAD
    /// Returns a new [`JsMapIterator`] object that yields the `[key, value]` pairs within the [`JsMap`] in insertion order.
=======
    /// Return a new Iterator object that contains the [key, value] pairs in order of assertion.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn entries(&self, context: &mut Context) -> JsResult<JsMapIterator> {
        let iterator_record = Map::entries(&self.inner.clone().into(), &[], context)?
            .get_iterator(context, None, None)?;
        let map_iterator_object = iterator_record.iterator();
        JsMapIterator::from_object(map_iterator_object.clone(), context)
    }

<<<<<<< HEAD
    /// Returns a new [`JsMapIterator`] object that yields the `key` for each element within the [`JsMap`] in insertion order.
=======
    /// Return the keys Iterator object
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn keys(&self, context: &mut Context) -> JsResult<JsMapIterator> {
        let iterator_record = Map::keys(&self.inner.clone().into(), &[], context)?
            .get_iterator(context, None, None)?;
        let map_iterator_object = iterator_record.iterator();
        JsMapIterator::from_object(map_iterator_object.clone(), context)
    }

<<<<<<< HEAD
    /// Inserts a new entry into the [`JsMap`] object
    ///
    /// # Example
    ///
    /// ```
    ///     let map = JsMap::new(context);
    ///
    ///     map.set("foo", "bar", context)?;
    ///     map.set(2, 4, context)?;
    ///
    ///     assert_eq!(map.get("foo", context)?, "bar".into());
    ///     assert_eq!(map.get(2, context)?, 4.into())
    ///
    /// ```
    #[inline]
    pub fn set<K, V>(&self, key: K, value: V, context: &mut Context) -> JsResult<JsValue>
    where
        K: Into<JsValue>,
        V: Into<JsValue>,
=======
    /// Insert a new entry into the Map object
    #[inline]
    pub fn set<T>(&self, key: T, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    {
        Map::set(
            &self.inner.clone().into(),
            &[key.into(), value.into()],
            context,
        )
    }

<<<<<<< HEAD
    /// Gets the size of the [`JsMap`] object.
    ///
    /// # Example
    ///
    /// ```
    ///     let map = JsMap::new(context);
    ///
    ///     map.set("foo", "bar", context)?;
    ///
    ///     let map_size = map.get_size(context)?;
    ///
    ///     assert_eq!(map_size, 1.into());
    /// ```
=======
    /// Obtains the size of the map.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn get_size(&self, context: &mut Context) -> JsResult<JsValue> {
        Map::get_size(&self.inner.clone().into(), &[], context)
    }

<<<<<<< HEAD
    /// Removes element from [`JsMap`] with a matching `key` value.
    ///
    /// # Example
    ///
    /// ```
    ///     js_map.set("foo", "bar", context)?;
    ///     js_map.set("hello", world, context)?;
    ///
    ///     js_map.delete("foo", context)?;
    ///
    ///     assert_eq!(js_map.get_size(context)?, 1.into());
    ///     assert_eq!(js_map.get("foo", context)?, JsValue::undefined());
    /// ```
=======
    /// Remove entry from Map associated with key.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn delete<T>(&self, key: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Map::delete(&self.inner.clone().into(), &[key.into()], context)
    }

<<<<<<< HEAD
    /// Gets the value associated with the specified key within the [`JsMap`], or `undefined` if the key does not exist.
    ///
    /// # Example
    ///
    /// ```
    ///     js_map.set("foo", "bar", context)?;
    ///
    ///     let retrieved_value = js_map.get("foo", context)?;
    ///
    ///     assert_eq!(retrieved_value, "bar".into());
    /// ```
=======
    /// Returns value associated to key or undefined.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn get<T>(&self, key: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Map::get(&self.inner.clone().into(), &[key.into()], context)
    }

<<<<<<< HEAD
    /// Removes all entries from the [`JsMap`].
    ///
    /// # Example
    ///
    /// ```
    ///     js_map.set("foo", "bar", context)?;
    ///     js_map.set("hello", world, context)?;
    ///
    ///     js_map.clear(context)?;
    ///
    ///     assert_eq!(js_map.get_size(context)?, 0.into());
=======
    /// Removes all entries from a map.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn clear(&self, context: &mut Context) -> JsResult<JsValue> {
        Map::clear(&self.inner.clone().into(), &[], context)
    }

<<<<<<< HEAD
    /// Checks if [`JsMap`] has an entry with the provided `key` value.
    ///
    /// # Example
    ///
    /// ```
    ///     js_map.set("foo", "bar", context)?;
    ///
    ///     let has_key = js_map.has("foo", context)?;
    ///
    ///     assert_eq!(has_key, true.into());
    /// ```
=======
    /// Checks if map contains provided key.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn has<T>(&self, key: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Map::has(&self.inner.clone().into(), &[key.into()], context)
    }

<<<<<<< HEAD
    /// Executes the provided callback function for each key-value pair within the [`JsMap`].
=======
    /// Executes provided callback function for each key-value pair.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn for_each(
        &self,
        callback: JsFunction,
        this_arg: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Map::for_each(
            &self.inner.clone().into(),
            &[callback.into(), this_arg],
            context,
        )
    }

<<<<<<< HEAD
    /// Returns a new [`JsMapIterator`] object that yields the `value` for each element within the [`JsMap`] in insertion order.
=======
    /// Returns new Iterator object of value elements of the Map.
>>>>>>> 787313dc88736c4b6592156b2d8a7edc2a837f45
    #[inline]
    pub fn values(&self, context: &mut Context) -> JsResult<JsMapIterator> {
        let iterator_record = Map::values(&self.inner.clone().into(), &[], context)?
            .get_iterator(context, None, None)?;
        let map_iterator_object = iterator_record.iterator();
        JsMapIterator::from_object(map_iterator_object.clone(), context)
    }
}

impl From<JsMap> for JsObject {
    #[inline]
    fn from(o: JsMap) -> Self {
        o.inner.clone()
    }
}

impl From<JsMap> for JsValue {
    #[inline]
    fn from(o: JsMap) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsMap {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsMap {}
