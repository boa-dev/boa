use crate::gc::{custom_trace, Finalize, Trace};
use indexmap::{
    set::{IntoIter, Iter},
    IndexSet,
};
use std::{
    collections::hash_map::RandomState,
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

/// A newtype wrapping indexmap::IndexSet
#[derive(Clone)]
pub struct OrderedSet<V, S = RandomState>(IndexSet<V, S>)
where
    V: Hash + Eq;

impl<V: Eq + Hash + Trace, S: BuildHasher> Finalize for OrderedSet<V, S> {}
unsafe impl<V: Eq + Hash + Trace, S: BuildHasher> Trace for OrderedSet<V, S> {
    custom_trace!(this, {
        for v in this.0.iter() {
            mark(v);
        }
    });
}

impl<V: Hash + Eq + Debug> Debug for OrderedSet<V> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(formatter)
    }
}

impl<V: Hash + Eq> Default for OrderedSet<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> OrderedSet<V>
where
    V: Hash + Eq,
{
    pub fn new() -> Self {
        OrderedSet(IndexSet::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        OrderedSet(IndexSet::with_capacity(capacity))
    }

    /// Return the number of key-value pairs in the map.
    ///
    /// Computes in **O(1)** time.
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the map contains no elements.
    ///
    /// Computes in **O(1)** time.
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    /// Insert a value pair in the set.
    ///
    /// If an equivalent value already exists in the set: ???
    ///
    /// If no equivalent value existed in the set: the new value is
    /// inserted, last in order, and false
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn add(&mut self, value: V) -> bool {
        self.0.insert(value)
    }

    /// Delete the `value` from the set and return true if successful
    ///
    /// Return `false` if `value` is not in map.
    ///
    /// Computes in **O(n)** time (average).
    pub fn delete(&mut self, value: &V) -> bool {
        self.0.shift_remove(value)
    }

    /// Checks if a given value is present in the set
    ///
    /// Return `true` if `value` is present in set, false otherwise.
    ///
    /// Computes in **O(n)** time (average).
    pub fn contains(&self, value: &V) -> bool {
        self.0.contains(value)
    }

    /// Get a key-value pair by index
    /// Valid indices are 0 <= index < self.len()
    /// Computes in O(1) time.
    pub fn get_index(&self, index: usize) -> Option<&V> {
        self.0.get_index(index)
    }

    /// Return an iterator over the values of the set, in their order
    pub fn iter(&self) -> Iter<'_, V> {
        self.0.iter()
    }
}

impl<'a, V, S> IntoIterator for &'a OrderedSet<V, S>
where
    V: Hash + Eq,
    S: BuildHasher,
{
    type Item = &'a V;
    type IntoIter = Iter<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<V, S> IntoIterator for OrderedSet<V, S>
where
    V: Hash + Eq,
    S: BuildHasher,
{
    type Item = V;
    type IntoIter = IntoIter<V>;
    fn into_iter(self) -> IntoIter<V> {
        self.0.into_iter()
    }
}
