use crate::gc::{custom_trace, Finalize, Trace};
use crate::object::JsObject;
use crate::JsValue;
use indexmap::{Equivalent, IndexSet};
use std::hash::Hasher;
use std::{
    collections::hash_map::RandomState,
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

#[derive(PartialEq, Eq, Clone, Debug)]
enum SetKey {
    Key(JsValue),
    Empty(usize), // Necessary to ensure empty keys are still unique.
}

impl SetKey {
    fn as_key(&self) -> Option<&JsValue> {
        match self {
            SetKey::Key(key) => Some(key),
            SetKey::Empty(_) => None,
        }
    }
}

// This ensures that a SetKey::Key(value) hashes to the same as value. The derived PartialEq implementation still holds.
#[allow(clippy::derive_hash_xor_eq)]
impl Hash for SetKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            SetKey::Key(v) => v.hash(state),
            SetKey::Empty(e) => e.hash(state),
        }
    }
}

impl Equivalent<SetKey> for JsValue {
    fn equivalent(&self, key: &SetKey) -> bool {
        match key {
            SetKey::Key(v) => v == self,
            _ => false,
        }
    }
}

/// A newtype wrapping indexmap::IndexSet
#[derive(Clone, Default)]
pub struct OrderedSet<S = RandomState> {
    set: IndexSet<SetKey, S>,
    lock: u32,
    empty_count: usize,
}

impl<S: BuildHasher> Finalize for OrderedSet<S> {}
unsafe impl<S: BuildHasher> Trace for OrderedSet<S> {
    custom_trace!(this, {
        for k in this.set.iter() {
            if let SetKey::Key(key) = k {
                mark(key);
            }
        }
    });
}

impl Debug for OrderedSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.set.fmt(formatter)
    }
}

impl OrderedSet {
    pub fn new() -> Self {
        OrderedSet {
            set: IndexSet::new(),
            lock: 0,
            empty_count: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        OrderedSet {
            set: IndexSet::with_capacity(capacity),
            lock: 0,
            empty_count: 0,
        }
    }

    /// Return the number of values in the set, including empty values.
    ///
    /// Computes in **O(1)** time.
    pub fn full_len(&self) -> usize {
        self.set.len()
    }

    /// Gets the number of values in the set, not including empty values.
    ///
    /// Computes in **O(1)** time.
    pub fn len(&self) -> usize {
        self.set.len() - self.empty_count
    }

    /// Returns true if the set contains no elements.
    ///
    /// Computes in **O(1)** time.
    pub fn is_empty(&self) -> bool {
        self.set.len() == 0
    }

    /// Insert a value in the set.
    ///
    /// If an equivalent value already exists in the set: ???
    ///
    /// If no equivalent value existed in the set: the new value is
    /// inserted, last in order, and false
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn add(&mut self, value: JsValue) -> bool {
        self.set.insert(SetKey::Key(value))
    }

    /// Delete the `value` from the set and return true if successful
    ///
    /// Return `false` if `value` is not in set.
    ///
    /// Computes in **O(n)** time (average).
    pub fn delete(&mut self, value: &JsValue) -> bool {
        if self.lock == 0 {
            self.set.shift_remove(value)
        } else if self.set.contains(value) {
            self.set.insert(SetKey::Empty(self.empty_count));
            self.empty_count += 1;
            self.set.swap_remove(value)
        } else {
            false
        }
    }

    /// Checks if a given value is present in the set
    ///
    /// Return `true` if `value` is present in set, false otherwise.
    ///
    /// Computes in **O(n)** time (average).
    pub fn contains(&self, value: &JsValue) -> bool {
        self.set.contains(value)
    }

    /// Get a key-value pair by index
    /// Valid indices are 0 <= index < self.full_len()
    /// Computes in O(1) time.
    pub fn get_index(&self, index: usize) -> Option<&JsValue> {
        self.set.get_index(index)?.as_key()
    }

    /// Return an iterator over the values of the set, in their order
    pub fn iter(&self) -> impl Iterator<Item = &JsValue> {
        self.set.iter().filter_map(SetKey::as_key)
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
            self.set.retain(|k| matches!(k, SetKey::Key(_)));
            self.empty_count = 0;
        }
    }
}

/// Increases the lock count of the set for the lifetime of the guard. This should not be dropped until iteration has completed.
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
