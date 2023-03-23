//! Implements a set type that preserves insertion order.

use crate::{builtins::map::ordered_map::MapKey, object::JsObject, JsValue};
use boa_gc::{custom_trace, Finalize, Trace};
use indexmap::IndexSet;
use std::{collections::hash_map::RandomState, fmt::Debug, hash::BuildHasher};

/// A type wrapping `indexmap::IndexSet`
#[derive(Clone, Finalize)]
pub struct OrderedSet<S = RandomState> {
    inner: IndexSet<MapKey, S>,
    lock: u32,
    empty_count: usize,
}

unsafe impl<S: BuildHasher> Trace for OrderedSet<S> {
    custom_trace!(this, {
        for v in this.inner.iter() {
            if let MapKey::Key(v) = v {
                mark(v);
            }
        }
    });
}

impl Debug for OrderedSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.inner.fmt(formatter)
    }
}

impl Default for OrderedSet {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderedSet {
    /// Creates a new empty `OrderedSet`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: IndexSet::new(),
            lock: 0,
            empty_count: 0,
        }
    }

    /// Creates a new empty `OrderedSet` with the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: IndexSet::with_capacity(capacity),
            lock: 0,
            empty_count: 0,
        }
    }

    /// Return the number of elements in the set, including empty elements.
    ///
    /// Computes in **O(1)** time.
    #[must_use]
    pub fn full_len(&self) -> usize {
        self.inner.len()
    }

    /// Return the number of elements in the set.
    ///
    /// Computes in **O(1)** time.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len() - self.empty_count
    }

    /// Returns true if the set contains no elements.
    ///
    /// Computes in **O(1)** time.
    #[must_use]
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
    pub fn add(&mut self, value: JsValue) -> bool {
        self.inner.insert(MapKey::Key(value))
    }

    /// Delete the `value` from the set and return true if successful
    ///
    /// Return `false` if `value` is not in set.
    ///
    /// Computes in **O(n)** time (average).
    pub fn delete(&mut self, value: &JsValue) -> bool {
        if self.lock == 0 {
            self.inner.shift_remove(value)
        } else if self.inner.contains(value) {
            self.inner.insert(MapKey::Empty(self.empty_count));
            self.empty_count += 1;
            self.inner.swap_remove(value)
        } else {
            false
        }
    }

    /// Removes all elements in the set, while preserving its capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
        self.inner.shrink_to_fit();
        self.empty_count = 0;
    }

    /// Checks if a given value is present in the set
    ///
    /// Return `true` if `value` is present in set, false otherwise.
    ///
    /// Computes in **O(n)** time (average).
    pub fn contains(&self, value: &JsValue) -> bool {
        self.inner.contains(value)
    }

    /// Get a key-value pair by index
    /// Valid indices are 0 <= index < self.len()
    /// Computes in O(1) time.
    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<&JsValue> {
        if let MapKey::Key(value) = self.inner.get_index(index)? {
            Some(value)
        } else {
            None
        }
    }

    /// Return an iterator over the values of the set, in their order
    pub fn iter(&self) -> impl Iterator<Item = &JsValue> {
        self.inner.iter().filter_map(|v| {
            if let MapKey::Key(v) = v {
                Some(v)
            } else {
                None
            }
        })
    }

    /// Increases the lock counter and returns a lock object that will decrement the counter when dropped.
    ///
    /// This allows objects to be removed from the set during iteration without affecting the indexes until the iteration has completed.
    pub(crate) fn lock(&mut self, set: JsObject) -> SetLock {
        self.lock += 1;
        SetLock(set)
    }

    /// Decreases the lock counter and, if 0, removes all empty entries.
    fn unlock(&mut self) {
        self.lock -= 1;
        if self.lock == 0 {
            self.inner.retain(|k| matches!(k, MapKey::Key(_)));
            self.empty_count = 0;
        }
    }
}

/// Increases the lock count of the set for the lifetime of the guard.
/// This should not be dropped until iteration has completed.
#[derive(Debug, Trace)]
pub(crate) struct SetLock(JsObject);

impl Clone for SetLock {
    fn clone(&self) -> Self {
        let mut set = self.0.borrow_mut();
        let set = set.as_set_mut().expect("SetLock does not point to a set");
        set.lock(self.0.clone())
    }
}

impl Finalize for SetLock {
    fn finalize(&self) {
        let mut set = self.0.borrow_mut();
        let set = set.as_set_mut().expect("SetLock does not point to a set");
        set.unlock();
    }
}
