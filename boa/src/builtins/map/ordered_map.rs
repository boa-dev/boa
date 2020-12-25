use crate::gc::{custom_trace, Finalize, Trace};
use indexmap::{map::IntoIter, map::Iter, map::IterMut, IndexMap};
use std::{
    collections::hash_map::RandomState,
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

/// A newtype wrapping indexmap::IndexMap
#[derive(Clone)]
pub struct OrderedMap<K, V, S = RandomState>(IndexMap<K, V, S>)
where
    K: Hash + Eq;

impl<K: Eq + Hash + Trace, V: Trace, S: BuildHasher> Finalize for OrderedMap<K, V, S> {}
unsafe impl<K: Eq + Hash + Trace, V: Trace, S: BuildHasher> Trace for OrderedMap<K, V, S> {
    custom_trace!(this, {
        for (k, v) in this.0.iter() {
            mark(k);
            mark(v);
        }
    });
}

impl<K: Hash + Eq + Debug, V: Debug> Debug for OrderedMap<K, V> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(formatter)
    }
}

impl<K: Hash + Eq, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> OrderedMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        OrderedMap(IndexMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        OrderedMap(IndexMap::with_capacity(capacity))
    }

    /// Return the number of key-value pairs in the map.
    ///
    /// Computes in **O(1)** time.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the map contains no elements.
    ///
    /// Computes in **O(1)** time.
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    /// Insert a key-value pair in the map.
    ///
    /// If an equivalent key already exists in the map: the key remains and
    /// retains in its place in the order, its corresponding value is updated
    /// with `value` and the older value is returned inside `Some(_)`.
    ///
    /// If no equivalent key existed in the map: the new key-value pair is
    /// inserted, last in order, and `None` is returned.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.0.insert(key, value)
    }

    /// Remove the key-value pair equivalent to `key` and return
    /// its value.
    ///
    /// Like `Vec::remove`, the pair is removed by shifting all of the
    /// elements that follow it, preserving their relative order.
    /// **This perturbs the index of all of those elements!**
    ///
    /// Return `None` if `key` is not in map.
    ///
    /// Computes in **O(n)** time (average).
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.0.shift_remove(key)
    }

    /// Return a reference to the value stored for `key`, if it is present,
    /// else `None`.
    ///
    /// Computes in **O(1)** time (average).
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    /// Get a key-value pair by index
    /// Valid indices are 0 <= index < self.len()
    /// Computes in O(1) time.
    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        self.0.get_index(index)
    }

    /// Return an iterator over the key-value pairs of the map, in their order
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.0.iter()
    }

    /// Return `true` if an equivalent to `key` exists in the map.
    ///
    /// Computes in **O(1)** time (average).
    pub fn contains_key(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }
}

impl<'a, K, V, S> IntoIterator for &'a OrderedMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut OrderedMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<K, V, S> IntoIterator for OrderedMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> IntoIter<K, V> {
        self.0.into_iter()
    }
}
