//! Implements a weak set type that preserves insertion order.

use crate::object::{JsObject, Object};
use boa_gc::{custom_trace, Finalize, GcRefCell, Trace, WeakGc};
use indexmap::IndexSet;
use std::{
    collections::hash_map::RandomState,
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
};

/// A type wrapping `JsObject` that can be used as a key in [`WeakOrderedSet`].
#[derive(Clone, Trace, Finalize)]
struct WeakSetObject(WeakGc<GcRefCell<Object>>);

impl Debug for WeakSetObject {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(formatter)
    }
}

impl PartialEq for WeakSetObject {
    fn eq(&self, other: &Self) -> bool {
        match (self.0.upgrade(), other.0.upgrade()) {
            (Some(a), Some(b)) => std::ptr::eq(a.as_ref(), b.as_ref()),
            _ => false,
        }
    }
}

impl Eq for WeakSetObject {}

impl Hash for WeakSetObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(obj) = self.0.upgrade() {
            std::ptr::hash(obj.as_ref(), state);
        } else {
            std::ptr::hash(self, state);
        }
    }
}

/// A ordered set of weak references to `JsObject`s.
#[derive(Clone)]
pub struct WeakOrderedSet<S = RandomState> {
    inner: IndexSet<WeakSetObject, S>,
}

impl<S: BuildHasher> Finalize for WeakOrderedSet<S> {}

unsafe impl<S: BuildHasher> Trace for WeakOrderedSet<S> {
    custom_trace!(this, {
        for v in this.inner.iter() {
            mark(v);
        }
    });
}

impl Debug for WeakOrderedSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.inner.fmt(formatter)
    }
}

impl Default for WeakOrderedSet {
    fn default() -> Self {
        Self::new()
    }
}

impl WeakOrderedSet {
    /// Creates a new empty `WeakOrderedSet`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: IndexSet::new(),
        }
    }

    /// Insert the value into the set.
    pub fn add(&mut self, value: &JsObject) -> bool {
        self.inner.insert(WeakSetObject(WeakGc::new(value.inner())))
    }

    /// Remove the value from the set, and return `true` if it was present.
    pub fn delete(&mut self, value: &JsObject) -> bool {
        self.inner
            .shift_remove(&WeakSetObject(WeakGc::new(value.inner())))
    }

    /// Return `true` if an equivalent to value exists in the set.
    pub fn contains(&self, value: &JsObject) -> bool {
        self.inner
            .contains(&WeakSetObject(WeakGc::new(value.inner())))
    }

    /// Returns `true` if all weak objects in the set have been collected.
    #[cfg(test)]
    pub(super) fn all_collected(&self) -> bool {
        self.inner.iter().all(|v| v.0.upgrade().is_none())
    }
}
