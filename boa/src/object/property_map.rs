use super::{PropertyDescriptor, PropertyKey};
use crate::{
    gc::{custom_trace, Finalize, Trace},
    JsString, JsSymbol,
};
use indexmap::IndexMap;
use rustc_hash::{FxHashMap, FxHasher};
use std::{collections::hash_map, hash::BuildHasherDefault, iter::FusedIterator};

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

#[derive(Default, Debug, Trace, Finalize)]
pub struct PropertyMap {
    indexed_properties: FxHashMap<u32, PropertyDescriptor>,
    /// Properties
    string_properties: OrderedHashMap<JsString>,
    /// Symbol Properties
    symbol_properties: OrderedHashMap<JsSymbol>,
}

impl PropertyMap {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self, key: &PropertyKey) -> Option<&PropertyDescriptor> {
        match key {
            PropertyKey::Index(index) => self.indexed_properties.get(index),
            PropertyKey::String(string) => self.string_properties.0.get(string),
            PropertyKey::Symbol(symbol) => self.symbol_properties.0.get(symbol),
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
            PropertyKey::Index(index) => self.indexed_properties.remove(index),
            PropertyKey::String(string) => self.string_properties.0.shift_remove(string),
            PropertyKey::Symbol(symbol) => self.symbol_properties.0.shift_remove(symbol),
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
        IndexProperties(self.indexed_properties.iter())
    }

    /// An iterator visiting all index keys in arbitrary order. The iterator element type is `&'a u32`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn index_property_keys(&self) -> IndexPropertyKeys<'_> {
        IndexPropertyKeys(self.indexed_properties.keys())
    }

    /// An iterator visiting all index values in arbitrary order. The iterator element type is `&'a Property`.
    ///
    /// This iterator does not recurse down the prototype chain.
    #[inline]
    pub fn index_property_values(&self) -> IndexPropertyValues<'_> {
        IndexPropertyValues(self.indexed_properties.values())
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
            PropertyKey::Index(index) => self.indexed_properties.contains_key(index),
            PropertyKey::String(string) => self.string_properties.0.contains_key(string),
            PropertyKey::Symbol(symbol) => self.symbol_properties.0.contains_key(symbol),
        }
    }

    // Internal method to efficiently work on string properties.
    #[inline]
    pub(crate) fn string_property_map_mut(
        &mut self,
    ) -> &mut IndexMap<JsString, PropertyDescriptor, BuildHasherDefault<FxHasher>> {
        &mut self.string_properties.0
    }
}

/// An iterator over the property entries of an `Object`
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    indexed_properties: hash_map::Iter<'a, u32, PropertyDescriptor>,
    string_properties: indexmap::map::Iter<'a, JsString, PropertyDescriptor>,
    symbol_properties: indexmap::map::Iter<'a, JsSymbol, PropertyDescriptor>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (PropertyKey, &'a PropertyDescriptor);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((key, value)) = self.indexed_properties.next() {
            Some(((*key).into(), value))
        } else if let Some((key, value)) = self.string_properties.next() {
            Some((key.clone().into(), value))
        } else {
            let (key, value) = self.symbol_properties.next()?;
            Some((key.clone().into(), value))
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
    type Item = &'a PropertyDescriptor;
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
pub struct IndexProperties<'a>(hash_map::Iter<'a, u32, PropertyDescriptor>);

impl<'a> Iterator for IndexProperties<'a> {
    type Item = (&'a u32, &'a PropertyDescriptor);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for IndexProperties<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for IndexProperties<'_> {}

/// An iterator over the index keys (`u32`) of an `Object`.
#[derive(Debug, Clone)]
pub struct IndexPropertyKeys<'a>(hash_map::Keys<'a, u32, PropertyDescriptor>);

impl<'a> Iterator for IndexPropertyKeys<'a> {
    type Item = &'a u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for IndexPropertyKeys<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for IndexPropertyKeys<'_> {}

/// An iterator over the index values (`Property`) of an `Object`.
#[derive(Debug, Clone)]
pub struct IndexPropertyValues<'a>(hash_map::Values<'a, u32, PropertyDescriptor>);

impl<'a> Iterator for IndexPropertyValues<'a> {
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

impl ExactSizeIterator for IndexPropertyValues<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
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
