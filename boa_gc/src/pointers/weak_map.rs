#![allow(unreachable_pub, unused)]

use hashbrown::{
    hash_map::DefaultHashBuilder,
    raw::{Bucket, RawIter, RawTable},
    TryReserveError,
};

use crate::{custom_trace, Allocator, Ephemeron, Finalize, Gc, GcRefCell, Trace};
use std::{fmt, hash::BuildHasher, marker::PhantomData, mem};

/// A map that holds weak references to its keys and is traced by the garbage collector.
#[derive(Clone, Debug, Default, Trace, Finalize)]
pub struct WeakMap<K: Trace + Sized + 'static, V: Trace + Sized + 'static> {
    pub(crate) inner: Gc<GcRefCell<RawWeakMap<K, V>>>,
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
        self.inner.borrow_mut().insert(key, value);
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    #[inline]
    pub fn remove(&mut self, key: &Gc<K>) -> Option<V> {
        self.inner.borrow_mut().remove(&key)
    }

    /// Returns `true` if the map contains a value for the specified key.
    #[must_use]
    #[inline]
    pub fn contains_key(&self, key: &Gc<K>) -> bool {
        self.inner.borrow().contains_key(&key)
    }

    /// Returns a reference to the value corresponding to the key.
    #[must_use]
    #[inline]
    pub fn get(&self, key: &Gc<K>) -> Option<V> {
        self.inner.borrow().get(&key)
    }
}

pub(crate) struct RawWeakMap<K, V, S = DefaultHashBuilder>
where
    K: Trace + 'static,
    V: Trace + 'static,
{
    hash_builder: S,
    table: RawTable<Ephemeron<K, V>>,
}

impl<K, V, S> Finalize for RawWeakMap<K, V, S>
where
    K: Trace + 'static,
    V: Trace + 'static,
{
}

unsafe impl<K, V, S> Trace for RawWeakMap<K, V, S>
where
    K: Trace + 'static,
    V: Trace + 'static,
{
    custom_trace!(this, {
        for eph in this.iter() {
            mark(eph)
        }
    });
}

impl<K, V, S> Default for RawWeakMap<K, V, S>
where
    S: Default,
    K: Trace + 'static,
    V: Trace + 'static,
{
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

impl<K, V> RawWeakMap<K, V, DefaultHashBuilder>
where
    K: Trace + 'static,
    V: Trace + 'static,
{
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHashBuilder::default())
    }
}

impl<K, V, S> RawWeakMap<K, V, S>
where
    K: Trace + 'static,
    V: Trace + 'static,
{
    #[inline]
    pub const fn with_hasher(hash_builder: S) -> Self {
        Self {
            hash_builder,
            table: RawTable::new(),
        }
    }

    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            hash_builder,
            table: RawTable::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn hasher(&self) -> &S {
        &self.hash_builder
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.table.capacity()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        // Here we tie the lifetime of self to the iter.
        unsafe {
            Iter {
                inner: self.table.iter(),
                marker: PhantomData,
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Ephemeron<K, V>) -> bool,
    {
        // Here we only use `iter` as a temporary, preventing use-after-free
        unsafe {
            for item in self.table.iter() {
                let eph = item.as_ref();
                if !f(eph) {
                    self.table.erase(item);
                }
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.table.clear();
    }
}

impl<K, V, S> RawWeakMap<K, V, S>
where
    K: Trace + 'static,
    V: Trace + Clone + 'static,
    S: BuildHasher,
{
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.table
            .reserve(additional, make_hasher(&self.hash_builder));
    }

    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.table
            .try_reserve(additional, make_hasher(&self.hash_builder))
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.table
            .shrink_to(0, make_hasher::<_, V, S>(&self.hash_builder));
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.table
            .shrink_to(min_capacity, make_hasher::<_, V, S>(&self.hash_builder));
    }

    #[inline]
    pub fn get(&self, k: &Gc<K>) -> Option<V> {
        if self.table.is_empty() {
            None
        } else {
            let hash = make_hash_from_gc(&self.hash_builder, k);
            self.table.get(hash, equivalent_key(k))?.value()
        }
    }

    #[inline]
    pub fn contains_key(&self, k: &Gc<K>) -> bool {
        self.get(k).is_some()
    }

    #[inline]
    pub fn insert(&mut self, k: &Gc<K>, v: V) -> Option<Ephemeron<K, V>> {
        let hash = make_hash_from_gc(&self.hash_builder, k);
        let hasher = make_hasher(&self.hash_builder);
        let eph = Ephemeron::new(k, v);
        match self
            .table
            .find_or_find_insert_slot(hash, equivalent_key(k), hasher)
        {
            Ok(bucket) => Some(mem::replace(unsafe { bucket.as_mut() }, eph)),
            Err(slot) => {
                unsafe {
                    self.table.insert_in_slot(hash, slot, eph);
                }
                None
            }
        }
    }

    #[inline]
    pub fn remove(&mut self, k: &Gc<K>) -> Option<V> {
        let hash = make_hash_from_gc(&self.hash_builder, k);
        self.table.remove_entry(hash, equivalent_key(k))?.value()
    }

    #[inline]
    pub fn clear_expired(&mut self) {
        self.retain(|eph| eph.value().is_some());
    }
}

pub struct Iter<'a, K, V>
where
    K: Trace + 'static,
    V: Trace + 'static,
{
    inner: RawIter<Ephemeron<K, V>>,
    marker: PhantomData<&'a Ephemeron<K, V>>,
}

impl<K, V> Clone for Iter<'_, K, V>
where
    K: Trace + 'static,
    V: Trace + 'static,
{
    #[inline]
    fn clone(&self) -> Self {
        Iter {
            inner: self.inner.clone(),
            marker: PhantomData,
        }
    }
}

impl<K, V> fmt::Debug for Iter<'_, K, V>
where
    K: Trace + 'static + fmt::Debug,
    V: Trace + 'static + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Trace + 'static,
    V: Trace + 'static,
{
    type Item = &'a Ephemeron<K, V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.inner.next().map(|b| b.as_ref()) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V, S> fmt::Debug for RawWeakMap<K, V, S>
where
    K: fmt::Debug + Trace + Finalize,
    V: fmt::Debug + Trace + Finalize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter().fmt(f)
    }
}

fn make_hasher<K, V, S>(hash_builder: &S) -> impl Fn(&Ephemeron<K, V>) -> u64 + '_
where
    S: BuildHasher,
    K: Trace + 'static,
    V: Trace + 'static,
{
    move |val| make_hash_from_eph::<K, V, S>(hash_builder, &val)
}

fn make_hash_from_eph<K, V, S>(hash_builder: &S, eph: &Ephemeron<K, V>) -> u64
where
    S: BuildHasher,
    K: Trace + 'static,
    V: Trace + 'static,
{
    use std::hash::Hasher;
    let mut state = hash_builder.build_hasher();
    unsafe {
        if let Some(val) = eph.inner().key() {
            std::ptr::hash(val, &mut state);
        } else {
            std::ptr::hash(eph.inner_ptr().as_ptr(), &mut state);
        }
    }
    state.finish()
}

fn make_hash_from_gc<K, S>(hash_builder: &S, gc: &Gc<K>) -> u64
where
    S: BuildHasher,
    K: Trace + 'static,
{
    use std::hash::Hasher;
    let mut state = hash_builder.build_hasher();
    std::ptr::hash(gc.inner_ptr().as_ptr(), &mut state);
    state.finish()
}

fn equivalent_key<K, V>(k: &Gc<K>) -> impl Fn(&Ephemeron<K, V>) -> bool + '_
where
    K: Trace + 'static,
    V: Trace + 'static,
{
    move |eph| unsafe {
        if let Some(val) = eph.inner().key() {
            let val: *const _ = val;
            std::ptr::eq(val, k.inner_ptr().as_ptr())
        } else {
            false
        }
    }
}
