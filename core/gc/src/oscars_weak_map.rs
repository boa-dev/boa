//! Dummy `WeakMap` implementation for the `oscars_backend` feature.
//!
//! We define this here instead of in `oscars` because `boa_engine` needs to be able to modify the `WeakMap` even when it is shared, which it handles by using `GcRefCell`.
//! Additionally, the `mark_sweep_branded` backend never frees memory, making a true weak map impossible.
//! Defining a dummy wrapper in `boa_gc` fulfills engine requirements without polluting it with conditional compilation gates.
//! All operations are leaky strong map operations to maintain API compatibility.

use crate::{Finalize, Gc, MutationContext, Trace, Tracer};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Result};

#[derive(Clone)]
pub struct WeakMap<K: Trace + ?Sized, V: Trace> {
    map: HashMap<usize, V>,
    _marker: std::marker::PhantomData<(*const K, *const V)>,
}

impl<K: Trace + ?Sized, V: Trace> Default for WeakMap<K, V> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<K: Trace + ?Sized, V: Trace> WeakMap<K, V> {
    /// Creates a new, empty `WeakMap`.
    ///
    /// The `_mc` argument mirrors the non-oscars API; it is unused here.
    #[must_use]
    #[inline]
    pub fn new(_mc: &MutationContext<'_, '_>) -> Self {
        Self {
            map: HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Inserts a key value pair into the map
    #[inline]
    pub fn insert(&mut self, key: &Gc<'_, K>, value: V) {
        self.map
            .insert(std::ptr::from_ref(&**key).cast::<()>() as usize, value);
    }

    /// Removes a key from the map, returning `true` if the key was present.
    /// Acts as a leaky strong map, so memory is never actually freed.
    #[inline]
    pub fn remove(&mut self, key: &Gc<'_, K>) -> bool {
        self.map
            .remove(&(std::ptr::from_ref(&**key).cast::<()>() as usize))
            .is_some()
    }

    /// Returns `true` if the map contains the key.
    #[must_use]
    #[inline]
    pub fn contains_key(&self, key: &Gc<'_, K>) -> bool {
        self.map
            .contains_key(&(std::ptr::from_ref(&**key).cast::<()>() as usize))
    }

    /// Returns the value associated with `key`, or `None`
    #[must_use]
    #[inline]
    pub fn get(&self, key: &Gc<'_, K>) -> Option<V>
    where
        V: Clone,
    {
        self.map
            .get(&(std::ptr::from_ref(&**key).cast::<()>() as usize))
            .cloned()
    }

    /// Alias for `get` to match the `boa_gc` backend's `WeakMap` API.
    #[must_use]
    #[inline]
    pub fn get_value(&self, key: &Gc<'_, K>) -> Option<V>
    where
        V: Clone,
    {
        self.get(key)
    }
}

impl<K: Trace + ?Sized, V: Trace> Debug for WeakMap<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("WeakMap").finish()
    }
}

impl<K: Trace + ?Sized, V: Trace> Finalize for WeakMap<K, V> {}

unsafe impl<K: Trace + ?Sized, V: Trace> Trace for WeakMap<K, V> {
    unsafe fn trace(&self, tracer: &mut Tracer<'_>) {
        for value in self.map.values() {
            unsafe { value.trace(tracer) };
        }
    }
    unsafe fn trace_non_roots(&self) {
        for value in self.map.values() {
            unsafe { value.trace_non_roots() };
        }
    }
    fn run_finalizer(&self) {
        Finalize::finalize(self);
    }
}
