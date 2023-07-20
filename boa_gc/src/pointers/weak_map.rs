use crate::{Allocator, Finalize, Gc, GcRefCell, Trace, WeakGc};
use std::collections::HashMap;

/// A map that holds weak references to its keys and is traced by the garbage collector.
#[derive(Clone, Debug, Default, Trace, Finalize)]
pub struct WeakMap<K: Trace + Sized + 'static, V: Trace + Sized + 'static> {
    pub(crate) inner: Gc<GcRefCell<HashMap<WeakGc<K>, V>>>,
}

impl<K: Trace, V: Trace + Clone> WeakMap<K, V> {
    /// Creates a new [`WeakMap`].
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Allocator::alloc_weak_map()
    }

    /// Inserts a key-value pair into the map.
    #[inline]
    pub fn insert(&mut self, key: &Gc<K>, value: V) {
        self.inner.borrow_mut().insert(WeakGc::new(key), value);
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    #[inline]
    pub fn remove(&mut self, key: &Gc<K>) -> Option<V> {
        self.inner.borrow_mut().remove(&WeakGc::new(key))
    }

    /// Returns `true` if the map contains a value for the specified key.
    #[must_use]
    #[inline]
    pub fn contains_key(&self, key: &Gc<K>) -> bool {
        self.inner.borrow().contains_key(&WeakGc::new(key))
    }

    /// Returns a reference to the value corresponding to the key.
    #[must_use]
    #[inline]
    pub fn get(&self, key: &Gc<K>) -> Option<V> {
        self.inner.borrow().get(&WeakGc::new(key)).cloned()
    }
}
