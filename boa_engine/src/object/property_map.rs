use super::{PropertyDescriptor, PropertyKey};
use crate::{property::PropertyDescriptorBuilder, JsString, JsSymbol, JsValue};
use boa_gc::{custom_trace, Finalize, Trace};
use indexmap::IndexMap;
use rustc_hash::{FxHashMap, FxHasher};
use std::{collections::hash_map, hash::BuildHasherDefault, iter::FusedIterator};

/// Type alias to make it easier to work with the string properties on the global object.
pub(crate) type GlobalPropertyMap =
    IndexMap<JsString, PropertyDescriptor, BuildHasherDefault<FxHasher>>;

/// Wrapper around `indexmap::IndexMap` for usage in `PropertyMap`.
#[derive(Debug, Finalize)]
struct OrderedHashMap<K: Trace>(IndexMap<K, PropertyDescriptor, BuildHasherDefault<FxHasher>>);

impl<K: Trace> Default for OrderedHashMap<K> {
    fn default() -> Self {
        Self(IndexMap::with_hasher(BuildHasherDefault::default()))
    }
}

unsafe impl<K: Trace> Trace for OrderedHashMap<K> {
    custom_trace!(this, {
        for (k, v) in this.0.iter() {
            mark(k);
            mark(v);
        }
    });
}

/// This represents all the indexed properties.
///
/// The index properties can be stored in two storage methods:
/// - `Dense` Storage
/// - `Sparse` Storage
///
/// By default it is dense storage.
#[derive(Debug, Trace, Finalize)]
enum IndexedProperties {
    /// Dense storage holds a contiguous array of properties where the index in the array is the key of the property.
    /// These are known to be data descriptors with a value field, writable field set to `true`, configurable field set to `true`, enumerable field set to `true`.
    ///
    /// Since we know the properties of the property descriptors (and they are all the same) we can omit it and just store only
    /// the value field and construct the data property descriptor on demand.
    ///
    /// This storage method is used by default.
    Dense(Vec<JsValue>),

    /// Sparse storage this storage is used as a backup if the element keys are not continuous or the property descriptors
    /// are not data descriptors with with a value field, writable field set to `true`, configurable field set to `true`, enumerable field set to `true`.
    ///
    /// This method uses more space, since we also have to store the property descriptors, not just the value.
    /// It is also slower because we need to to a hash lookup.
    Sparse(FxHashMap<u32, PropertyDescriptor>),
}

impl Default for IndexedProperties {
    #[inline]
    fn default() -> Self {
        Self::Dense(Vec::new())
    }
}

impl IndexedProperties {
    /// Get a property descriptor if it exists.
    #[inline]
    fn get(&self, key: u32) -> Option<PropertyDescriptor> {
        match self {
            Self::Sparse(ref map) => map.get(&key).cloned(),
            Self::Dense(ref vec) => vec.get(key as usize).map(|value| {
                PropertyDescriptorBuilder::new()
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .value(value.clone())
                    .build()
            }),
        }
    }

    /// Helper function for converting from a dense storage type to sparse storage type.
    #[inline]
    fn convert_dense_to_sparse(vec: &mut Vec<JsValue>) -> FxHashMap<u32, PropertyDescriptor> {
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
    #[inline]
    fn insert(&mut self, key: u32, property: PropertyDescriptor) -> Option<PropertyDescriptor> {
        let vec = match self {
            Self::Sparse(map) => return map.insert(key, property),
            Self::Dense(vec) => {
                let len = vec.len() as u32;
                if key <= len
                    && property.value().is_some()
                    && property.writable().unwrap_or(false)
                    && property.enumerable().unwrap_or(false)
                    && property.configurable().unwrap_or(false)
                {
                    // Fast Path: continues array access.

                    let mut value = property
                        .value()
                        .cloned()
                        .expect("already checked that the property descriptor has a value field");

                    // If the key is pointing one past the last element, we push it!
                    //
                    // Since the previous key is the current key - 1. Meaning that the elements are continuos.
                    if key == len {
                        vec.push(value);
                        return None;
                    }

                    // If it the key points in at a already taken index, swap and return it.
                    std::mem::swap(&mut vec[key as usize], &mut value);
                    return Some(
                        PropertyDescriptorBuilder::new()
                            .writable(true)
                            .enumerable(true)
                            .configurable(true)
                            .value(value)
                            .build(),
                    );
                }

                vec
            }
        };

        // Slow path: converting to sparse storage.
        let mut map = Self::convert_dense_to_sparse(vec);
        let old_property = map.insert(key, property);
        *self = Self::Sparse(map);

        old_property
    }

    /// Inserts a property descriptor with the specified key.
    #[inline]
    fn remove(&mut self, key: u32) -> Option<PropertyDescriptor> {
        let vec = match self {
            Self::Sparse(map) => return map.remove(&key),
            Self::Dense(vec) => {
                // Fast Path: contiguous storage.

                // Has no elements or out of range, nothing to delete!
                if vec.is_empty() || key as usize >= vec.len() {
                    return None;
                }

                // If the key is pointing at the last element, then we pop it and return it.
                //
                // It does not make the storage sparse
                if key as usize == vec.len().wrapping_sub(1) {
                    let value = vec.pop().expect("Already checked if it is out of bounds");
                    return Some(
                        PropertyDescriptorBuilder::new()
                            .writable(true)
                            .enumerable(true)
                            .configurable(true)
                            .value(value)
                            .build(),
                    );
                }

                vec
            }
        };

        // Slow Path: conversion to sparse storage.
        let mut map = Self::convert_dense_to_sparse(vec);
        let old_property = map.remove(&key);
        *self = Self::Sparse(map);

        old_property
    }

    /// Check if we contain the key to a property descriptor.
    fn contains_key(&self, key: u32) -> bool {
        match self {
            Self::Sparse(map) => map.contains_key(&key),
            Self::Dense(vec) => (0..vec.len() as u32).contains(&key),
        }
    }

    fn iter(&self) -> IndexProperties<'_> {
        match self {
            Self::Dense(vec) => IndexProperties::Dense(vec.iter().enumerate()),
            Self::Sparse(map) => IndexProperties::Sparse(map.iter()),
        }
    }

    fn keys(&self) -> IndexPropertyKeys<'_> {
        match self {
            Self::Dense(vec) => IndexPropertyKeys::Dense(0..vec.len() as u32),
            Self::Sparse(map) => IndexPropertyKeys::Sparse(map.keys()),
        }
    }

    fn values(&self) -> IndexPropertyValues<'_> {
        match self {
            Self::Dense(vec) => IndexPropertyValues::Dense(vec.iter()),
            Self::Sparse(map) => IndexPropertyValues::Sparse(map.values()),
        }
    }
}

#[derive(Default, Debug, Trace, Finalize)]
pub struct PropertyMap {
    indexed_properties: IndexedProperties,
    /// Properties
    string_properties: OrderedHashMap<JsString>,
    /// Symbol Properties
    symbol_properties: OrderedHashMap<JsSymbol>,
}

impl PropertyMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        match key {
            PropertyKey::Index(index) => self.indexed_properties.get(*index),
            PropertyKey::String(string) => self.string_properties.0.get(string).cloned(),
            PropertyKey::Symbol(symbol) => self.symbol_properties.0.get(symbol).cloned(),
        }
    }

    pub fn insert(
        &mut self,
        key: &PropertyKey,
        property: PropertyDescriptor,
    ) -> Option<PropertyDescriptor> {
        match &key {
            PropertyKey::Index(index) => self.indexed_properties.insert(*index, property),
            PropertyKey::String(string) => {
                self.string_properties.0.insert(string.clone(), property)
            }
            PropertyKey::Symbol(symbol) => {
                self.symbol_properties.0.insert(symbol.clone(), property)
            }
        }
    }

    pub fn remove(&mut self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        match key {
            PropertyKey::Index(index) => self.indexed_properties.remove(*index),
            PropertyKey::String(string) => self.string_properties.0.shift_remove(string),
            PropertyKey::Symbol(symbol) => self.symbol_properties.0.shift_remove(symbol),
        }
    }

    /// Overrides all the indexed properties, setting it to dense storage.
    pub(crate) fn override_indexed_properties(&mut self, properties: Vec<JsValue>) {
        self.indexed_properties = IndexedProperties::Dense(properties);
    }

    /// Returns the vec of dense indexed properties if they exist.
    pub(crate) fn dense_indexed_properties(&self) -> Option<&Vec<JsValue>> {
        if let IndexedProperties::Dense(properties) = &self.indexed_properties {
            Some(properties)
        } else {
            None
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order. The iterator element type is `(PropertyKey, &'a Property)`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            indexed_properties: self.indexed_properties.iter(),
            string_properties: self.string_properties.0.iter(),
            symbol_properties: self.symbol_properties.0.iter(),
        }
    }

    /// An iterator visiting all keys in arbitrary order. The iterator element type is `PropertyKey`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn keys(&self) -> Keys<'_> {
        Keys(self.iter())
    }

    /// An iterator visiting all values in arbitrary order. The iterator element type is `&'a Property`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn values(&self) -> Values<'_> {
        Values(self.iter())
    }

    /// An iterator visiting all symbol key-value pairs in arbitrary order. The iterator element type is `(&'a RcSymbol, &'a Property)`.
    ///
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn symbol_properties(&self) -> SymbolProperties<'_> {
        SymbolProperties(self.symbol_properties.0.iter())
    }

    /// An iterator visiting all symbol keys in arbitrary order. The iterator element type is `&'a RcSymbol`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn symbol_property_keys(&self) -> SymbolPropertyKeys<'_> {
        SymbolPropertyKeys(self.symbol_properties.0.keys())
    }

    /// An iterator visiting all symbol values in arbitrary order. The iterator element type is `&'a Property`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn symbol_property_values(&self) -> SymbolPropertyValues<'_> {
        SymbolPropertyValues(self.symbol_properties.0.values())
    }

    /// An iterator visiting all indexed key-value pairs in arbitrary order. The iterator element type is `(&'a u32, &'a Property)`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn index_properties(&self) -> IndexProperties<'_> {
        self.indexed_properties.iter()
    }

    /// An iterator visiting all index keys in arbitrary order. The iterator element type is `&'a u32`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn index_property_keys(&self) -> IndexPropertyKeys<'_> {
        self.indexed_properties.keys()
    }

    /// An iterator visiting all index values in arbitrary order. The iterator element type is `&'a Property`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn index_property_values(&self) -> IndexPropertyValues<'_> {
        self.indexed_properties.values()
    }

    /// An iterator visiting all string key-value pairs in arbitrary order. The iterator element type is `(&'a RcString, &'a Property)`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn string_properties(&self) -> StringProperties<'_> {
        StringProperties(self.string_properties.0.iter())
    }

    /// An iterator visiting all string keys in arbitrary order. The iterator element type is `&'a RcString`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn string_property_keys(&self) -> StringPropertyKeys<'_> {
        StringPropertyKeys(self.string_properties.0.keys())
    }

    /// An iterator visiting all string values in arbitrary order. The iterator element type is `&'a Property`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn string_property_values(&self) -> StringPropertyValues<'_> {
        StringPropertyValues(self.string_properties.0.values())
    }

    #[inline]
    pub fn contains_key(&self, key: &PropertyKey) -> bool {
        match key {
            PropertyKey::Index(index) => self.indexed_properties.contains_key(*index),
            PropertyKey::String(string) => self.string_properties.0.contains_key(string),
            PropertyKey::Symbol(symbol) => self.symbol_properties.0.contains_key(symbol),
        }
    }

    #[inline]
    pub(crate) fn string_property_map(&self) -> &GlobalPropertyMap {
        &self.string_properties.0
    }

    #[inline]
    pub(crate) fn string_property_map_mut(&mut self) -> &mut GlobalPropertyMap {
        &mut self.string_properties.0
    }
}

/// An iterator over the property entries of an `Object`
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    indexed_properties: IndexProperties<'a>,
    string_properties: indexmap::map::Iter<'a, JsString, PropertyDescriptor>,
    symbol_properties: indexmap::map::Iter<'a, JsSymbol, PropertyDescriptor>,
}

impl<'a> Iterator for Iter<'a> {
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

impl FusedIterator for Iter<'_> {}

/// An iterator over the keys (`PropertyKey`) of an `Object`.
#[derive(Debug, Clone)]
pub struct Keys<'a>(Iter<'a>);

impl<'a> Iterator for Keys<'a> {
    type Item = PropertyKey;
    fn next(&mut self) -> Option<Self::Item> {
        let (key, _) = self.0.next()?;
        Some(key)
    }
}

impl ExactSizeIterator for Keys<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for Keys<'_> {}

/// An iterator over the values (`Property`) of an `Object`.
#[derive(Debug, Clone)]
pub struct Values<'a>(Iter<'a>);

impl<'a> Iterator for Values<'a> {
    type Item = PropertyDescriptor;
    fn next(&mut self) -> Option<Self::Item> {
        let (_, value) = self.0.next()?;
        Some(value)
    }
}

impl ExactSizeIterator for Values<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for Values<'_> {}

/// An iterator over the `Symbol` property entries of an `Object`
#[derive(Debug, Clone)]
pub struct SymbolProperties<'a>(indexmap::map::Iter<'a, JsSymbol, PropertyDescriptor>);

impl<'a> Iterator for SymbolProperties<'a> {
    type Item = (&'a JsSymbol, &'a PropertyDescriptor);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for SymbolProperties<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for SymbolProperties<'_> {}

/// An iterator over the keys (`RcSymbol`) of an `Object`.
#[derive(Debug, Clone)]
pub struct SymbolPropertyKeys<'a>(indexmap::map::Keys<'a, JsSymbol, PropertyDescriptor>);

impl<'a> Iterator for SymbolPropertyKeys<'a> {
    type Item = &'a JsSymbol;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for SymbolPropertyKeys<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for SymbolPropertyKeys<'_> {}

/// An iterator over the `Symbol` values (`Property`) of an `Object`.
#[derive(Debug, Clone)]
pub struct SymbolPropertyValues<'a>(indexmap::map::Values<'a, JsSymbol, PropertyDescriptor>);

impl<'a> Iterator for SymbolPropertyValues<'a> {
    type Item = &'a PropertyDescriptor;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for SymbolPropertyValues<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for SymbolPropertyValues<'_> {}

/// An iterator over the indexed property entries of an `Object`
#[derive(Debug, Clone)]
pub enum IndexProperties<'a> {
    Dense(std::iter::Enumerate<std::slice::Iter<'a, JsValue>>),
    Sparse(hash_map::Iter<'a, u32, PropertyDescriptor>),
}

impl<'a> Iterator for IndexProperties<'a> {
    type Item = (u32, PropertyDescriptor);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Dense(vec) => vec.next().map(|(index, value)| {
                (
                    index as u32,
                    PropertyDescriptorBuilder::new()
                        .writable(true)
                        .configurable(true)
                        .enumerable(true)
                        .value(value.clone())
                        .build(),
                )
            }),
            Self::Sparse(map) => map.next().map(|(index, value)| (*index, value.clone())),
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

impl ExactSizeIterator for IndexProperties<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Dense(vec) => vec.len(),
            Self::Sparse(map) => map.len(),
        }
    }
}

impl FusedIterator for IndexProperties<'_> {}

/// An iterator over the index keys (`u32`) of an `Object`.
#[derive(Debug, Clone)]
pub enum IndexPropertyKeys<'a> {
    Dense(std::ops::Range<u32>),
    Sparse(hash_map::Keys<'a, u32, PropertyDescriptor>),
}

impl<'a> Iterator for IndexPropertyKeys<'a> {
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
    Dense(std::slice::Iter<'a, JsValue>),
    Sparse(hash_map::Values<'a, u32, PropertyDescriptor>),
}

impl<'a> Iterator for IndexPropertyValues<'a> {
    type Item = PropertyDescriptor;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Dense(vec) => vec.next().map(|value| {
                PropertyDescriptorBuilder::new()
                    .writable(true)
                    .configurable(true)
                    .enumerable(true)
                    .value(value.clone())
                    .build()
            }),
            Self::Sparse(map) => map.next().cloned(),
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

impl ExactSizeIterator for IndexPropertyValues<'_> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Dense(vec) => vec.len(),
            Self::Sparse(map) => map.len(),
        }
    }
}

impl FusedIterator for IndexPropertyValues<'_> {}

/// An iterator over the `String` property entries of an `Object`
#[derive(Debug, Clone)]
pub struct StringProperties<'a>(indexmap::map::Iter<'a, JsString, PropertyDescriptor>);

impl<'a> Iterator for StringProperties<'a> {
    type Item = (&'a JsString, &'a PropertyDescriptor);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for StringProperties<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for StringProperties<'_> {}

/// An iterator over the string keys (`RcString`) of an `Object`.
#[derive(Debug, Clone)]
pub struct StringPropertyKeys<'a>(indexmap::map::Keys<'a, JsString, PropertyDescriptor>);

impl<'a> Iterator for StringPropertyKeys<'a> {
    type Item = &'a JsString;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for StringPropertyKeys<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for StringPropertyKeys<'_> {}

/// An iterator over the string values (`Property`) of an `Object`.
#[derive(Debug, Clone)]
pub struct StringPropertyValues<'a>(indexmap::map::Values<'a, JsString, PropertyDescriptor>);

impl<'a> Iterator for StringPropertyValues<'a> {
    type Item = &'a PropertyDescriptor;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for StringPropertyValues<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for StringPropertyValues<'_> {}
