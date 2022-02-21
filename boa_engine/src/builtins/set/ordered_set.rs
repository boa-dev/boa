use boa_gc::{custom_trace, Finalize, Trace};
use indexmap::{
    set::{IntoIter, Iter},
    IndexSet,
};
use std::{
    collections::hash_map::RandomState,
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

/// A type wrapping `indexmap::IndexSet`
#[derive(Clone)]
pub struct OrderedSet<V, S = RandomState>
where
    V: Hash + Eq,
{
    inner: IndexSet<V, S>,
}

impl<V: Eq + Hash + Trace, S: BuildHasher> Finalize for OrderedSet<V, S> {}
unsafe impl<V: Eq + Hash + Trace, S: BuildHasher> Trace for OrderedSet<V, S> {
    custom_trace!(this, {
        for v in this.inner.iter() {
            mark(v);
        }
    });
}

impl<V: Hash + Eq + Debug> Debug for OrderedSet<V> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.inner.fmt(formatter)
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
        Self {
            inner: IndexSet::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: IndexSet::with_capacity(capacity),
        }
    }

    /// Return the number of key-value pairs in the map.
    ///
    /// Computes in **O(1)** time.
    pub fn size(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the map contains no elements.
    ///
    /// Computes in **O(1)** time.
    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
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
        self.inner.insert(value)
    }

    /// Delete the `value` from the set and return true if successful
    ///
    /// Return `false` if `value` is not in map.
    ///
    /// Computes in **O(n)** time (average).
    pub fn delete(&mut self, value: &V) -> bool {
        self.inner.shift_remove(value)
    }

    /// Checks if a given value is present in the set
    ///
    /// Return `true` if `value` is present in set, false otherwise.
    ///
    /// Computes in **O(n)** time (average).
    pub fn contains(&self, value: &V) -> bool {
        self.inner.contains(value)
    }

    /// Get a key-value pair by index
    /// Valid indices are 0 <= index < self.len()
    /// Computes in O(1) time.
    pub fn get_index(&self, index: usize) -> Option<&V> {
        self.inner.get_index(index)
    }

    /// Return an iterator over the values of the set, in their order
    pub fn iter(&self) -> Iter<'_, V> {
        self.inner.iter()
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
        self.inner.iter()
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
        self.inner.into_iter()
    }
}
