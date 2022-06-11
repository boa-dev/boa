// This module is a wrapper for the Map Builtin Javascript Object
//
// TODO: improve Iterator object interaction: entries, keys, values
// forEach implementation is missing
use crate::{
    builtins::map::{add_entries_from_iterable, ordered_map::OrderedMap},
    builtins::Map,
    object::{JsMapIterator, JsObject, JsObjectType, ObjectData},
    Context, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsMap {
    inner: JsObject,
}

impl JsMap {
    /// Create new Empty Map Object
    #[inline]
    pub fn new(context: &mut Context) -> Self {
        let map = Self::create_map(context);
        Self { inner: map }
    }

    /// Create a new map object for any object that has a `@@Iterator` field.
    #[inline]
    pub fn from_js_iterable(iterable: &JsValue, context: &mut Context) -> JsResult<Self> {
        // Create a new map
        let map = Self::create_map(context);

        // Let adder be Get(map, "set") per spec. This action should not fail with default map
        let adder = map
            .get("set", context)
            .expect("creating a map with the default prototype must not fail");

        let _completion_record = add_entries_from_iterable(&map, iterable, &adder, context)?;

        Ok(Self { inner: map })
    }

    /// Create a [`JsMap`] from a [`JsObject`], if the object is not a `Map`, throw a `TypeError`.
    #[inline]
    pub fn from_object(object: JsObject, context: &mut Context) -> JsResult<Self> {
        if object.borrow().is_map() {
            Ok(Self { inner: object })
        } else {
            context.throw_type_error("object is not a Map")
        }
    }

    // utility function to generate default Map object
    fn create_map(context: &mut Context) -> JsObject {
        // Get default Map prototype
        let prototype = context.intrinsics().constructors().map().prototype();

        // Create a default map object with [[MapData]] as a new empty list
        JsObject::from_proto_and_data(prototype, ObjectData::map(OrderedMap::new()))
    }

    /// Return a new Iterator object that contains the [key, value] pairs in order of assertion
    #[inline]
    pub fn entries(&self, context: &mut Context) -> JsResult<JsMapIterator> {
        let iterator_record = Map::entries(&self.inner.clone().into(), &[], context)?
            .get_iterator(context, None, None)?;
        let map_iterator_object = iterator_record
            .iterator_object()
            .as_object()
            .expect("MapIterator for Map.prototype.entries() should not fail");
        JsMapIterator::from_object(map_iterator_object.clone(), context)
    }

    /// Return the keys Iterator object
    #[inline]
    pub fn keys(&self, context: &mut Context) -> JsResult<JsMapIterator> {
        let iterator_record = Map::keys(&self.inner.clone().into(), &[], context)?.get_iterator(context, None, None)?;
        let map_iterator_object = iterator_record.iterator_object().as_object().expect("MapIterator for Map.prototype.keys() should not fail");
        JsMapIterator::from_object(map_iterator_object.clone(), context)
    }

    /// Insert a new entry into the Map object
    #[inline]
    pub fn set<T>(&self, key: T, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Map::set(
            &self.inner.clone().into(),
            &[key.into(), value.into()],
            context,
        )
    }

    /// Obtains the size of the map
    #[inline]
    pub fn get_size(&self, context: &mut Context) -> JsResult<JsValue> {
        Map::get_size(&self.inner.clone().into(), &[], context)
    }

    /// Remove entry from Map associated with key
    #[inline]
    pub fn delete<T>(&self, key: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Map::delete(&self.inner.clone().into(), &[key.into()], context)
    }

    /// Returns value associated to key or undefined
    #[inline]
    pub fn get<T>(&self, key: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Map::get(&self.inner.clone().into(), &[key.into()], context)
    }

    /// Removes all entries from a map
    #[inline]
    pub fn clear(&self, context: &mut Context) -> JsResult<JsValue> {
        Map::clear(&self.inner.clone().into(), &[], context)
    }

    /// Checks if map contains provided key
    #[inline]
    pub fn has<T>(&self, key: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        Map::has(&self.inner.clone().into(), &[key.into()], context)
    }

    /// Returns new Iterator object of value elements of the Map
    #[inline]
    pub fn values(&self, context: &mut Context) -> JsResult<JsMapIterator> {
        let iterator_record = Map::values(&self.inner.clone().into(), &[], context)?.get_iterator(context, None, None)?;
        let map_iterator_object = iterator_record.iterator_object().as_object().expect("MapIterator for Map.prototype.values() should not fail");
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
