//! Implements a map type that preserves insertion order.

use crate::{JsData, JsValue};
use boa_gc::{Finalize, Trace, custom_trace};
use indexmap::{Equivalent, IndexMap};
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) enum MapKey {
    Key(JsValue),
    Empty(usize), // Necessary to ensure empty keys are still unique.
}

// This ensures that a MapKey::Key(value) hashes to the same as value. The derived PartialEq implementation still holds.
impl Hash for MapKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Key(v) => v.hash(state),
            Self::Empty(e) => e.hash(state),
        }
    }
}

impl Equivalent<MapKey> for JsValue {
    fn equivalent(&self, key: &MapKey) -> bool {
        match key {
            MapKey::Key(v) => v == self,
            MapKey::Empty(_) => false,
        }
    }
}

/// A lock on an [`OrderedMap`] or an [`OrderedSet`], released when dropped.
///
/// The lock count is the strong count of the collection's `Rc`, so releasing a lock touches
/// neither the collection nor its cell: it can't fail, even while the collection is borrowed.
///
/// [`OrderedSet`]: crate::builtins::set::ordered_set::OrderedSet
#[derive(Debug)]
pub(crate) struct CollectionLock {
    _token: Rc<()>,
}

impl CollectionLock {
    /// Takes a lock counted by `token`, the collection's own `Rc`, creating it on first use.
    pub(crate) fn new(token: &mut Option<Rc<()>>) -> Self {
        Self {
            _token: Rc::clone(token.get_or_insert_with(Rc::default)),
        }
    }
}

/// A structure wrapping `indexmap::IndexMap`.
#[derive(Finalize, JsData)]
pub struct OrderedMap<V> {
    map: IndexMap<MapKey, Option<V>>,
    lock: Option<Rc<()>>,
    empty_count: usize,
}

impl<V: Clone> Clone for OrderedMap<V> {
    /// Clones the entries only: the clone has no empty entries and no locks.
    fn clone(&self) -> Self {
        let mut map = self.map.clone();
        if self.empty_count > 0 {
            map.retain(|k, _| matches!(k, MapKey::Key(_)));
        }
        Self {
            map,
            lock: None,
            empty_count: 0,
        }
    }
}

unsafe impl<V: Trace> Trace for OrderedMap<V> {
    custom_trace!(this, mark, {
        for (k, v) in &this.map {
            if let MapKey::Key(key) = k {
                mark(key);
            }
            mark(v);
        }
    });
}

impl<V: Debug> Debug for OrderedMap<V> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.map.fmt(formatter)
    }
}

impl<V> Default for OrderedMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> OrderedMap<V> {
    /// Creates a new empty `OrderedMap`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
            lock: None,
            empty_count: 0,
        }
    }

    /// Creates a new empty `OrderedMap` with the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            lock: None,
            empty_count: 0,
        }
    }

    /// Return the number of key-value pairs in the map, including empty values.
    ///
    /// Computes in **O(1)** time.
    #[must_use]
    pub fn full_len(&self) -> usize {
        self.map.len()
    }

    /// Gets the number of key-value pairs in the map, not including empty values.
    ///
    /// Computes in **O(1)** time.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len() - self.empty_count
    }

    /// Returns true if the map contains no elements.
    ///
    /// Computes in **O(1)** time.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    pub fn insert(&mut self, key: JsValue, value: V) -> Option<V> {
        self.compact();
        self.map.insert(MapKey::Key(key), Some(value)).flatten()
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
    pub fn remove(&mut self, key: &JsValue) -> Option<V> {
        self.compact();
        if !self.is_locked() {
            self.map.shift_remove(key).flatten()
        } else if self.map.contains_key(key) {
            self.map.insert(MapKey::Empty(self.empty_count), None);
            self.empty_count += 1;
            self.map.swap_remove(key).flatten()
        } else {
            None
        }
    }

    /// Removes all elements from the map and resets the counter of
    /// empty entries.
    pub fn clear(&mut self) {
        if self.is_locked() {
            let len = self.map.len();
            self.map.clear();
            for i in 0..len {
                self.map.insert(MapKey::Empty(i), None);
            }
            self.empty_count = len;
        } else {
            self.map.clear();
            self.map.shrink_to_fit();
            self.empty_count = 0;
        }
    }

    /// Return a reference to the value stored for `key`, if it is present,
    /// else `None`.
    ///
    /// Computes in **O(1)** time (average).
    pub fn get(&self, key: &JsValue) -> Option<&V> {
        self.map.get(key).and_then(Option::as_ref)
    }

    /// Get a key-value pair by index.
    ///
    /// Valid indices are `0 <= index < self.full_len()`.
    ///
    /// Computes in O(1) time.
    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<(&JsValue, &V)> {
        if let (MapKey::Key(key), Some(value)) = self.map.get_index(index)? {
            Some((key, value))
        } else {
            None
        }
    }

    /// Return an iterator over the key-value pairs of the map, in their order
    pub fn iter(&self) -> impl Iterator<Item = (&JsValue, &V)> {
        self.map.iter().filter_map(|o| {
            if let (MapKey::Key(key), Some(value)) = o {
                Some((key, value))
            } else {
                None
            }
        })
    }

    /// Return `true` if an equivalent to `key` exists in the map.
    ///
    /// Computes in **O(1)** time (average).
    #[must_use]
    pub fn contains_key(&self, key: &JsValue) -> bool {
        self.map.contains_key(key)
    }

    /// Locks the map until the returned lock is dropped.
    ///
    /// This allows objects to be removed from the map during iteration without affecting the indexes until the iteration has completed.
    /// The empty entries left by removals are dropped by the next mutation after the last lock is released.
    pub(crate) fn lock(&mut self) -> CollectionLock {
        self.compact();
        CollectionLock::new(&mut self.lock)
    }

    /// Returns `true` if a lock on the map is alive.
    fn is_locked(&self) -> bool {
        self.lock
            .as_ref()
            .is_some_and(|lock| Rc::strong_count(lock) > 1)
    }

    /// Removes all empty entries, unless the map is locked.
    fn compact(&mut self) {
        if self.empty_count > 0 && !self.is_locked() {
            self.map.retain(|k, _| matches!(k, MapKey::Key(_)));
            self.empty_count = 0;
        }
    }
}
