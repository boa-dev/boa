use super::{
    shape::{
        property_table::PropertyTableInner,
        shared_shape::TransitionKey,
        slot::{Slot, SlotAttributes},
        ChangeTransitionAction, RootShape, Shape, UniqueShape,
    },
    JsPrototype, ObjectStorage, PropertyDescriptor, PropertyKey,
};
use crate::{property::PropertyDescriptorBuilder, JsString, JsSymbol, JsValue};
use boa_gc::{custom_trace, Finalize, Trace};
use indexmap::IndexMap;
use rustc_hash::{FxHashMap, FxHasher};
use std::{collections::hash_map, hash::BuildHasherDefault, iter::FusedIterator};
use thin_vec::ThinVec;

/// Wrapper around `indexmap::IndexMap` for usage in `PropertyMap`.
#[derive(Debug, Finalize)]
struct OrderedHashMap<K: Trace>(IndexMap<K, PropertyDescriptor, BuildHasherDefault<FxHasher>>);

impl<K: Trace> Default for OrderedHashMap<K> {
    fn default() -> Self {
        Self(IndexMap::with_hasher(BuildHasherDefault::default()))
    }
}

unsafe impl<K: Trace> Trace for OrderedHashMap<K> {
    custom_trace!(this, mark, {
        for (k, v) in &this.0 {
            mark(k);
            mark(v);
        }
    });
}

/// This represents all the indexed properties.
///
/// The index properties can be stored in two storage methods:
///
/// ## Dense Storage
///
/// Dense storage holds a contiguous array of properties where the index in the array is the key of the property.
/// These are known to be data descriptors with a value field, writable field set to `true`, configurable field set to `true`, enumerable field set to `true`.
///
/// Since we know the properties of the property descriptors (and they are all the same) we can omit it and just store only
/// the value field and construct the data property descriptor on demand.
///
/// ## Sparse Storage
///
/// This storage is used as a backup if the element keys are not continuous or the property descriptors
/// are not data descriptors with with a value field, writable field set to `true`, configurable field set to `true`, enumerable field set to `true`.
///
/// This method uses more space, since we also have to store the property descriptors, not just the value.
/// It is also slower because we need to do a hash lookup.
#[derive(Debug, Trace, Finalize)]
pub enum IndexedProperties {
    /// Dense [`i32`] storage.
    DenseI32(ThinVec<i32>),

    /// Dense [`JsValue`] storage.
    DenseElement(ThinVec<JsValue>),

    /// Sparse [`JsValue`] storage.
    Sparse(Box<FxHashMap<u32, PropertyDescriptor>>),
}

impl Default for IndexedProperties {
    #[inline]
    fn default() -> Self {
        Self::DenseI32(ThinVec::new())
    }
}

impl IndexedProperties {
    pub(crate) fn from_dense_js_value(elements: ThinVec<JsValue>) -> Self {
        if elements.is_empty() {
            return Self::default();
        }
        Self::DenseElement(elements)
    }

    /// Get a property descriptor if it exists.
    fn get(&self, key: u32) -> Option<PropertyDescriptor> {
        let value = match self {
            Self::DenseI32(ref vec) => vec.get(key as usize).copied()?.into(),
            Self::DenseElement(ref vec) => vec.get(key as usize)?.clone(),
            Self::Sparse(ref map) => return map.get(&key).cloned(),
        };

        Some(
            PropertyDescriptorBuilder::new()
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .value(value)
                .build(),
        )
    }

    /// Helper function for converting from a dense storage type to sparse storage type.
    fn convert_dense_to_sparse(vec: &mut ThinVec<JsValue>) -> FxHashMap<u32, PropertyDescriptor> {
        let data = std::mem::take(vec);

        data.into_iter()
            .enumerate()
            .map(|(index, value)| {
                (
                    index as u32,
                    PropertyDescriptorBuilder::new()
                        .writable(true)
                        .enumerable(true)
                        .configurable(true)
                        .value(value)
                        .build(),
                )
            })
            .collect()
    }

    /// Helper function for converting from a dense storage type to sparse storage type.
    fn convert_dense_i32_to_sparse(vec: &mut ThinVec<i32>) -> FxHashMap<u32, PropertyDescriptor> {
        let data = std::mem::take(vec);

        data.into_iter()
            .enumerate()
            .map(|(index, value)| {
                (
                    index as u32,
                    PropertyDescriptorBuilder::new()
                        .writable(true)
                        .enumerable(true)
                        .configurable(true)
                        .value(value)
                        .build(),
                )
            })
            .collect()
    }

    /// Inserts a property descriptor with the specified key.
    fn insert(&mut self, key: u32, property: PropertyDescriptor) -> bool {
        let mut map =
            match self {
                Self::DenseI32(vec) => {
                    let len = vec.len() as u32;
                    if key <= len
                        && property.value().is_some()
                        && property.writable().unwrap_or(false)
                        && property.enumerable().unwrap_or(false)
                        && property.configurable().unwrap_or(false)
                    {
                        // Fast Path: continues array access.
                        let value = property.value().cloned().expect(
                            "already checked that the property descriptor has a value field",
                        );

                        if let Some(value) = value.as_integer() {
                            // If the key is pointing one past the last element, we push it!
                            //
                            // Since the previous key is the current key - 1. Meaning that the elements are continuos.
                            if key == len {
                                vec.push(value);
                                return false;
                            }

                            // If it the key points in at a already taken index, swap and return it.
                            vec[key as usize] = value;
                            return true;
                        }

                        let mut vec = vec
                            .iter()
                            .copied()
                            .map(JsValue::from)
                            .collect::<ThinVec<JsValue>>();

                        // If the key is pointing one past the last element, we push it!
                        //
                        // Since the previous key is the current key - 1. Meaning that the elements are continuos.
                        if key == len {
                            vec.push(value);
                            *self = Self::DenseElement(vec);
                            return false;
                        }

                        // If it the key points in at a already taken index, set it.
                        vec[key as usize] = value;
                        *self = Self::DenseElement(vec);
                        return true;
                    }

                    // Slow path: converting to sparse storage.
                    Self::convert_dense_i32_to_sparse(vec)
                }
                Self::DenseElement(vec) => {
                    let len = vec.len() as u32;
                    if key <= len
                        && property.value().is_some()
                        && property.writable().unwrap_or(false)
                        && property.enumerable().unwrap_or(false)
                        && property.configurable().unwrap_or(false)
                    {
                        // Fast Path: continues array access.
                        let value = property.value().cloned().expect(
                            "already checked that the property descriptor has a value field",
                        );

                        // If the key is pointing one past the last element, we push it!
                        //
                        // Since the previous key is the current key - 1. Meaning that the elements are continuos.
                        if key == len {
                            vec.push(value);
                            return false;
                        }

                        // If it the key points in at a already taken index, set it.
                        vec[key as usize] = value;
                        return true;
                    }

                    // Slow path: converting to sparse storage.
                    Self::convert_dense_to_sparse(vec)
                }
                Self::Sparse(map) => return map.insert(key, property).is_some(),
            };

        // Slow path: converting to sparse storage.
        let replaced = map.insert(key, property).is_some();
        *self = Self::Sparse(Box::new(map));

        replaced
    }

    /// Removes a property descriptor with the specified key.
    fn remove(&mut self, key: u32) -> bool {
        let mut map = match self {
            Self::DenseI32(vec) => {
                // Fast Path: contiguous storage.

                // If out of range, nothing to delete!
                if key as usize >= vec.len() {
                    return false;
                }

                // If the key is pointing at the last element, then we pop it.
                //
                // It does not make the storage sparse.
                if key as usize == vec.len().wrapping_sub(1) {
                    vec.pop().expect("Already checked if it is out of bounds");
                    return true;
                }

                // Slow Path: conversion to sparse storage.
                Self::convert_dense_i32_to_sparse(vec)
            }
            Self::DenseElement(vec) => {
                // Fast Path: contiguous storage.

                // If out of range, nothing to delete!
                if key as usize >= vec.len() {
                    return false;
                }

                // If the key is pointing at the last element, then we pop it.
                //
                // It does not make the storage sparse.
                if key as usize == vec.len().wrapping_sub(1) {
                    vec.pop().expect("Already checked if it is out of bounds");
                    return true;
                }

                // Slow Path: conversion to sparse storage.
                Self::convert_dense_to_sparse(vec)
            }
            Self::Sparse(map) => {
                return map.remove(&key).is_some();
            }
        };

        let removed = map.remove(&key).is_some();
        *self = Self::Sparse(Box::new(map));

        removed
    }

    /// Check if we contain the key to a property descriptor.
    fn contains_key(&self, key: u32) -> bool {
        match self {
            Self::DenseI32(vec) => (0..vec.len() as u32).contains(&key),
            Self::DenseElement(vec) => (0..vec.len() as u32).contains(&key),
            Self::Sparse(map) => map.contains_key(&key),
        }
    }

    fn iter(&self) -> IndexProperties<'_> {
        match self {
            Self::DenseI32(vec) => IndexProperties::DenseI32(vec.iter().enumerate()),
            Self::DenseElement(vec) => IndexProperties::DenseElement(vec.iter().enumerate()),
            Self::Sparse(map) => IndexProperties::Sparse(map.iter()),
        }
    }

    fn keys(&self) -> IndexPropertyKeys<'_> {
        match self {
            Self::DenseI32(vec) => IndexPropertyKeys::Dense(0..vec.len() as u32),
            Self::DenseElement(vec) => IndexPropertyKeys::Dense(0..vec.len() as u32),
            Self::Sparse(map) => IndexPropertyKeys::Sparse(map.keys()),
        }
    }

    fn values(&self) -> IndexPropertyValues<'_> {
        match self {
            Self::DenseI32(vec) => IndexPropertyValues::DenseI32(vec.iter()),
            Self::DenseElement(vec) => IndexPropertyValues::DenseElement(vec.iter()),
            Self::Sparse(map) => IndexPropertyValues::Sparse(map.values()),
        }
    }
}

impl<'a> IntoIterator for &'a IndexedProperties {
    type IntoIter = IndexProperties<'a>;
    type Item = (u32, PropertyDescriptor);
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A [`PropertyMap`] contains all the properties of an object.
///
/// The property values are stored in different data structures based on keys.
#[derive(Default, Debug, Trace, Finalize)]
pub struct PropertyMap {
    /// Properties stored with integers as keys.
    pub(crate) indexed_properties: IndexedProperties,

    pub(crate) shape: Shape,
    pub(crate) storage: ObjectStorage,
}

impl PropertyMap {
    /// Create a new [`PropertyMap`].
    #[must_use]
    #[inline]
    pub fn new(shape: Shape, indexed_properties: IndexedProperties) -> Self {
        Self {
            indexed_properties,
            shape,
            storage: Vec::default(),
        }
    }

    /// Construct a [`PropertyMap`] from with the given prototype with an unique [`Shape`].
    #[must_use]
    #[inline]
    pub fn from_prototype_unique_shape(prototype: JsPrototype) -> Self {
        Self {
            indexed_properties: IndexedProperties::default(),
            shape: UniqueShape::new(prototype, PropertyTableInner::default()).into(),
            storage: Vec::default(),
        }
    }

    /// Construct a [`PropertyMap`] from with the given prototype with a shared shape [`Shape`].
    #[must_use]
    #[inline]
    pub fn from_prototype_with_shared_shape(
        root_shape: &RootShape,
        prototype: JsPrototype,
    ) -> Self {
        let shape = root_shape.shape().change_prototype_transition(prototype);
        Self {
            indexed_properties: IndexedProperties::default(),
            shape: shape.into(),
            storage: Vec::default(),
        }
    }

    /// Get the property with the given key from the [`PropertyMap`].
    #[must_use]
    pub fn get(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        if let PropertyKey::Index(index) = key {
            return self.indexed_properties.get(index.get());
        }
        if let Some(slot) = self.shape.lookup(key) {
            return Some(self.get_storage(slot));
        }

        None
    }

    /// Get the property with the given key from the [`PropertyMap`].
    #[must_use]
    pub(crate) fn get_with_slot(
        &self,
        key: &PropertyKey,
        out_slot: &mut Slot,
    ) -> Option<PropertyDescriptor> {
        if let PropertyKey::Index(index) = key {
            return self.indexed_properties.get(index.get());
        }
        if let Some(slot) = self.shape.lookup(key) {
            out_slot.index = slot.index;

            // Remove all descriptor attributes, but keep inline caching bits.
            out_slot.attributes = (out_slot.attributes & SlotAttributes::INLINE_CACHE_BITS)
                | slot.attributes
                | SlotAttributes::FOUND;
            return Some(self.get_storage(slot));
        }

        None
    }

    /// Get the property with the given key from the [`PropertyMap`].
    #[must_use]
    pub(crate) fn get_storage(&self, Slot { index, attributes }: Slot) -> PropertyDescriptor {
        let index = index as usize;
        let mut builder = PropertyDescriptor::builder()
            .configurable(attributes.contains(SlotAttributes::CONFIGURABLE))
            .enumerable(attributes.contains(SlotAttributes::ENUMERABLE));
        if attributes.is_accessor_descriptor() {
            if attributes.has_get() {
                builder = builder.get(self.storage[index].clone());
            }
            if attributes.has_set() {
                builder = builder.set(self.storage[index + 1].clone());
            }
        } else {
            builder = builder.writable(attributes.contains(SlotAttributes::WRITABLE));
            builder = builder.value(self.storage[index].clone());
        }
        builder.build()
    }

    /// Insert the given property descriptor with the given key [`PropertyMap`].
    pub fn insert(&mut self, key: &PropertyKey, property: PropertyDescriptor) -> bool {
        let mut dummy_slot = Slot::new();
        self.insert_with_slot(key, property, &mut dummy_slot)
    }

    /// Insert the given property descriptor with the given key [`PropertyMap`].
    pub(crate) fn insert_with_slot(
        &mut self,
        key: &PropertyKey,
        property: PropertyDescriptor,
        out_slot: &mut Slot,
    ) -> bool {
        if let PropertyKey::Index(index) = key {
            return self.indexed_properties.insert(index.get(), property);
        }

        let attributes = property.to_slot_attributes();

        if let Some(slot) = self.shape.lookup(key) {
            let index = slot.index as usize;

            if slot.attributes != attributes {
                let key = TransitionKey {
                    property_key: key.clone(),
                    attributes,
                };
                let transition = self.shape.change_attributes_transition(key);
                self.shape = transition.shape;
                match transition.action {
                    ChangeTransitionAction::Nothing => {}
                    ChangeTransitionAction::Remove => {
                        self.storage.remove(slot.index as usize + 1);
                    }
                    ChangeTransitionAction::Insert => {
                        // insert after index which is (index + 1).
                        self.storage.insert(index, JsValue::undefined());
                    }
                }
            }

            if attributes.is_accessor_descriptor() {
                if attributes.has_get() {
                    self.storage[index] = property
                        .get()
                        .cloned()
                        .map(JsValue::new)
                        .unwrap_or_default();
                }
                if attributes.has_set() {
                    self.storage[index + 1] = property
                        .set()
                        .cloned()
                        .map(JsValue::new)
                        .unwrap_or_default();
                }
            } else {
                self.storage[index] = property.expect_value().clone();
            }
            out_slot.index = slot.index;
            out_slot.attributes =
                (out_slot.attributes & SlotAttributes::INLINE_CACHE_BITS) | attributes;
            return true;
        }

        let transition_key = TransitionKey {
            property_key: key.clone(),
            attributes,
        };
        self.shape = self.shape.insert_property_transition(transition_key);

        // Make Sure that if we are inserting, it has the correct slot index.
        debug_assert_eq!(
            self.shape.lookup(key),
            Some(Slot {
                index: self.storage.len() as u32,
                attributes
            })
        );

        out_slot.index = self.storage.len() as u32;
        out_slot.attributes =
            (out_slot.attributes & SlotAttributes::INLINE_CACHE_BITS) | attributes;

        if attributes.is_accessor_descriptor() {
            self.storage.push(
                property
                    .get()
                    .cloned()
                    .map(JsValue::new)
                    .unwrap_or_default(),
            );
            self.storage.push(
                property
                    .set()
                    .cloned()
                    .map(JsValue::new)
                    .unwrap_or_default(),
            );
        } else {
            self.storage
                .push(property.value().cloned().unwrap_or_default());
        }

        false
    }

    /// Remove the property with the given key from the [`PropertyMap`].
    pub fn remove(&mut self, key: &PropertyKey) -> bool {
        if let PropertyKey::Index(index) = key {
            return self.indexed_properties.remove(index.get());
        }
        if let Some(slot) = self.shape.lookup(key) {
            // shift all elements when removing.
            if slot.attributes.is_accessor_descriptor() {
                self.storage.remove(slot.index as usize + 1);
            }
            self.storage.remove(slot.index as usize);

            self.shape = self.shape.remove_property_transition(key);
            return true;
        }

        false
    }

    /// Overrides all the indexed properties, setting it to dense storage.
    pub(crate) fn override_indexed_properties(&mut self, properties: ThinVec<JsValue>) {
        self.indexed_properties = IndexedProperties::DenseElement(properties);
    }

    pub(crate) fn get_dense_property(&self, index: u32) -> Option<JsValue> {
        let index = index as usize;
        match &self.indexed_properties {
            IndexedProperties::DenseI32(properties) => {
                properties.get(index).copied().map(JsValue::from)
            }
            IndexedProperties::DenseElement(properties) => properties.get(index).cloned(),
            IndexedProperties::Sparse(_) => None,
        }
    }

    pub(crate) fn set_dense_property(&mut self, index: u32, value: &JsValue) -> bool {
        let index = index as usize;

        match &mut self.indexed_properties {
            IndexedProperties::DenseI32(properties) => {
                let Some(element) = properties.get_mut(index) else {
                    return false;
                };

                if let Some(value) = value.as_integer() {
                    *element = value;
                } else {
                    let mut properties = properties
                        .iter()
                        .copied()
                        .map(JsValue::from)
                        .collect::<ThinVec<_>>();
                    properties[index] = value.clone();
                    self.indexed_properties = IndexedProperties::DenseElement(properties);
                }
                true
            }
            IndexedProperties::DenseElement(properties) => {
                let Some(element) = properties.get_mut(index) else {
                    return false;
                };
                *element = value.clone();
                true
            }
            IndexedProperties::Sparse(_) => false,
        }
    }

    /// Returns the vec of dense indexed properties if they exist.
    pub(crate) fn to_dense_indexed_properties(&self) -> Option<ThinVec<JsValue>> {
        match &self.indexed_properties {
            IndexedProperties::DenseI32(properties) => {
                Some(properties.iter().copied().map(JsValue::from).collect())
            }
            IndexedProperties::DenseElement(properties) => Some(properties.clone()),
            IndexedProperties::Sparse(_) => None,
        }
    }

    /// Returns the vec of dense indexed properties if they exist.
    pub(crate) fn dense_indexed_properties_mut(&mut self) -> Option<&mut ThinVec<JsValue>> {
        if let IndexedProperties::DenseElement(properties) = &mut self.indexed_properties {
            Some(properties)
        } else {
            None
        }
    }

    /// An iterator visiting all indexed key-value pairs in arbitrary order. The iterator element type is `(&'a u32, &'a Property)`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    #[must_use]
    pub fn index_properties(&self) -> IndexProperties<'_> {
        self.indexed_properties.iter()
    }

    /// An iterator visiting all index keys in arbitrary order. The iterator element type is `&'a u32`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    #[must_use]
    pub fn index_property_keys(&self) -> IndexPropertyKeys<'_> {
        self.indexed_properties.keys()
    }

    /// An iterator visiting all index values in arbitrary order. The iterator element type is `&'a Property`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    #[must_use]
    pub fn index_property_values(&self) -> IndexPropertyValues<'_> {
        self.indexed_properties.values()
    }

    /// Returns `true` if the given key is contained in the [`PropertyMap`].
    #[inline]
    #[must_use]
    pub fn contains_key(&self, key: &PropertyKey) -> bool {
        if let PropertyKey::Index(index) = key {
            return self.indexed_properties.contains_key(index.get());
        }
        if self.shape.lookup(key).is_some() {
            return true;
        }

        false
    }
}

/// An iterator over the property entries of an `Object`
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    indexed_properties: IndexProperties<'a>,
    string_properties: indexmap::map::Iter<'a, JsString, PropertyDescriptor>,
    symbol_properties: indexmap::map::Iter<'a, JsSymbol, PropertyDescriptor>,
}

impl Iterator for Iter<'_> {
    type Item = (PropertyKey, PropertyDescriptor);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((key, value)) = self.indexed_properties.next() {
            Some((key.into(), value))
        } else if let Some((key, value)) = self.string_properties.next() {
            Some((key.clone().into(), value.clone()))
        } else {
            let (key, value) = self.symbol_properties.next()?;
            Some((key.clone().into(), value.clone()))
        }
    }
}

impl ExactSizeIterator for Iter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.indexed_properties.len() + self.string_properties.len() + self.symbol_properties.len()
    }
}

/// An iterator over the indexed property entries of an `Object`.
#[derive(Debug, Clone)]
pub enum IndexProperties<'a> {
    /// An iterator over dense i32, Vec backed indexed property entries of an `Object`.
    DenseI32(std::iter::Enumerate<std::slice::Iter<'a, i32>>),

    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseElement(std::iter::Enumerate<std::slice::Iter<'a, JsValue>>),

    /// An iterator over sparse, HashMap backed indexed property entries of an `Object`.
    Sparse(hash_map::Iter<'a, u32, PropertyDescriptor>),
}

impl Iterator for IndexProperties<'_> {
    type Item = (u32, PropertyDescriptor);

    fn next(&mut self) -> Option<Self::Item> {
        let (index, value) = match self {
            Self::DenseI32(vec) => vec
                .next()
                .map(|(index, value)| (index, JsValue::from(*value)))?,
            Self::DenseElement(vec) => vec.next().map(|(index, value)| (index, value.clone()))?,
            Self::Sparse(map) => return map.next().map(|(index, value)| (*index, value.clone())),
        };

        Some((
            index as u32,
            PropertyDescriptorBuilder::new()
                .writable(true)
                .configurable(true)
                .enumerable(true)
                .value(value.clone())
                .build(),
        ))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::DenseI32(vec) => vec.size_hint(),
            Self::DenseElement(vec) => vec.size_hint(),
            Self::Sparse(map) => map.size_hint(),
        }
    }
}

impl ExactSizeIterator for IndexProperties<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::DenseI32(vec) => vec.len(),
            Self::DenseElement(vec) => vec.len(),
            Self::Sparse(map) => map.len(),
        }
    }
}

impl FusedIterator for IndexProperties<'_> {}

/// An iterator over the index keys (`u32`) of an `Object`.
#[derive(Debug, Clone)]
pub enum IndexPropertyKeys<'a> {
    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    Dense(std::ops::Range<u32>),

    /// An iterator over sparse, HashMap backed indexed property entries of an `Object`.
    Sparse(hash_map::Keys<'a, u32, PropertyDescriptor>),
}

impl Iterator for IndexPropertyKeys<'_> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Dense(vec) => vec.next(),
            Self::Sparse(map) => map.next().copied(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Dense(vec) => vec.size_hint(),
            Self::Sparse(map) => map.size_hint(),
        }
    }
}

impl ExactSizeIterator for IndexPropertyKeys<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Dense(vec) => vec.len(),
            Self::Sparse(map) => map.len(),
        }
    }
}

impl FusedIterator for IndexPropertyKeys<'_> {}

/// An iterator over the index values (`Property`) of an `Object`.
#[derive(Debug, Clone)]
pub enum IndexPropertyValues<'a> {
    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseI32(std::slice::Iter<'a, i32>),

    /// An iterator over dense, Vec backed indexed property entries of an `Object`.
    DenseElement(std::slice::Iter<'a, JsValue>),

    /// An iterator over sparse, HashMap backed indexed property entries of an `Object`.
    Sparse(hash_map::Values<'a, u32, PropertyDescriptor>),
}

impl Iterator for IndexPropertyValues<'_> {
    type Item = PropertyDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        let value = match self {
            Self::DenseI32(vec) => vec.next().copied()?.into(),
            Self::DenseElement(vec) => vec.next().cloned()?,
            Self::Sparse(map) => return map.next().cloned(),
        };

        Some(
            PropertyDescriptorBuilder::new()
                .writable(true)
                .configurable(true)
                .enumerable(true)
                .value(value)
                .build(),
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::DenseI32(vec) => vec.size_hint(),
            Self::DenseElement(vec) => vec.size_hint(),
            Self::Sparse(map) => map.size_hint(),
        }
    }
}

impl ExactSizeIterator for IndexPropertyValues<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::DenseI32(vec) => vec.len(),
            Self::DenseElement(vec) => vec.len(),
            Self::Sparse(map) => map.len(),
        }
    }
}
