// Implementation taken partly from https://docs.rs/hashbrown/0.14.0/src/hashbrown/lib.rs.html,
// but with some adjustments to use `Ephemeron<K,V>` instead of `(K,V)`

use hashbrown::{
    hash_table::{Entry as RawEntry, Iter as RawIter},
    DefaultHashBuilder, HashTable, TryReserveError,
};

use crate::{custom_trace, Allocator, Ephemeron, Finalize, Gc, GcRefCell, Trace};
use std::{fmt, hash::BuildHasher, marker::PhantomData};

/// A map that holds weak references to its keys and is traced by the garbage collector.
#[derive(Clone, Debug, Default, Finalize)]
pub struct WeakMap<K: Trace + ?Sized + 'static, V: Trace + 'static> {
    pub(crate) inner: Gc<GcRefCell<RawWeakMap<K, V>>>,
}

unsafe impl<K: Trace + ?Sized + 'static, V: Trace + 'static> Trace for WeakMap<K, V> {
    custom_trace!(this, mark, {
        mark(&this.inner);
    });
}

impl<K: Trace + ?Sized, V: Trace + Clone> WeakMap<K, V> {
    /// Creates a new `WeakMap`.
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
        self.inner.borrow_mut().remove(key)
    }

    /// Returns `true` if the map contains a value for the specified key.
    #[must_use]
    #[inline]
    pub fn contains_key(&self, key: &Gc<K>) -> bool {
        self.inner.borrow().contains_key(key)
    }

    /// Returns a reference to the value corresponding to the key.
    #[must_use]
    #[inline]
    pub fn get(&self, key: &Gc<K>) -> Option<V> {
        self.inner.borrow().get(key)
    }
}

/// A hash map where the bucket type is an <code>[Ephemeron]\<K, V\></code>.
///
/// This data structure allows associating a <code>[Gc]\<K\></code> with a value `V` that will be
/// invalidated when the `Gc<K>` gets collected. In other words, all key entries on the map are weakly
/// held.
pub(crate) struct RawWeakMap<K, V, S = DefaultHashBuilder>
where
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
    hash_builder: S,
    table: HashTable<Ephemeron<K, V>>,
}

impl<K, V, S> Finalize for RawWeakMap<K, V, S>
where
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
}

// SAFETY: The implementation correctly marks all ephemerons inside the map.
unsafe impl<K, V, S> Trace for RawWeakMap<K, V, S>
where
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
    custom_trace!(this, mark, {
        for eph in this.iter() {
            mark(eph);
        }
    });
}

impl<K, V, S> Default for RawWeakMap<K, V, S>
where
    S: Default,
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
    fn default() -> Self {
        Self::with_hasher(Default::default())
    }
}

impl<K, V> RawWeakMap<K, V, DefaultHashBuilder>
where
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
    /// Creates an empty `RawWeakMap`.
    ///
    /// The map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Creates an empty `RawWeakMap` with the specified capacity.
    ///
    /// The map will be able to hold at least `capacity` elements without reallocating.
    /// If `capacity` is 0, the map will not allocate.
    #[allow(unused)]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHashBuilder::default())
    }
}

impl<K, V, S> RawWeakMap<K, V, S>
where
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
    /// Creates an empty `RawWeakMap` which will use the given hash builder to hash
    /// keys.
    ///
    /// The map is initially created with a capacity of 0, so it will not allocate until it is first
    /// inserted into.
    pub(crate) const fn with_hasher(hash_builder: S) -> Self {
        Self {
            hash_builder,
            table: HashTable::new(),
        }
    }

    /// Creates an empty `RawWeakMap` with the specified capacity, using `hash_builder`
    /// to hash the keys.
    ///
    /// The map will be able to hold at least `capacity` elements without reallocating.
    /// If `capacity` is 0, the map will not allocate.
    pub(crate) fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            hash_builder,
            table: HashTable::with_capacity(capacity),
        }
    }

    /// Returns a reference to the map's [`BuildHasher`].
    #[allow(unused)]
    pub(crate) const fn hasher(&self) -> &S {
        &self.hash_builder
    }

    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// This number is a lower bound; the map might be able to hold more, but is guaranteed to be
    /// able to hold at least this many.
    #[allow(unused)]
    pub(crate) fn capacity(&self) -> usize {
        self.table.capacity()
    }

    /// An iterator visiting all entries in arbitrary order.
    /// The iterator element type is <code>[Ephemeron]<K, V></code>.
    pub(crate) fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.table.iter(),
            marker: PhantomData,
        }
    }

    /// Returns the number of elements in the map.
    ///
    /// This is an upper bound; the map might contain some expired keys which haven't been
    /// removed.
    #[allow(unused)]
    pub(crate) fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// This might return `false` if the map has expired keys that are still pending to be
    /// cleaned up.
    #[allow(unused)]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Retains only the elements specified by the predicate. Keeps the
    /// allocated memory for reuse.
    ///
    /// In other words, remove all ephemerons <code>[Ephemeron]<K, V></code> such that
    /// `f(&eph)` returns `false`.
    /// The elements are visited in unsorted (and unspecified) order.
    pub(crate) fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Ephemeron<K, V>) -> bool,
    {
        // SAFETY:
        // - `item` is only used internally, which means it outlives self.
        // - `item` pointer is not used after the call to `erase`.
        self.table.retain(|item| f(item));
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    #[allow(unused)]
    pub(crate) fn clear(&mut self) {
        self.table.clear();
    }
}

impl<K, V, S> RawWeakMap<K, V, S>
where
    K: Trace + ?Sized + 'static,
    V: Trace + Clone + 'static,
    S: BuildHasher,
{
    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `RawWeakMap`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds [`isize::MAX`] bytes and [`abort`](std::process::abort)
    /// the program in case of allocation error. Use [`try_reserve`](RawWeakMap::try_reserve) instead
    /// if you want to handle memory allocation failure.
    #[allow(unused)]
    pub(crate) fn reserve(&mut self, additional: usize) {
        self.table
            .reserve(additional, make_hasher(&self.hash_builder));
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted
    /// in the given `RawWeakMap<K,V>`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Errors
    ///
    /// If the capacity overflows, or the allocator reports a failure, then an error
    /// is returned.
    #[allow(unused)]
    pub(crate) fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.table
            .try_reserve(additional, make_hasher(&self.hash_builder))
    }

    /// Shrinks the capacity of the map as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    #[allow(unused)]
    pub(crate) fn shrink_to_fit(&mut self) {
        self.table
            .shrink_to(0, make_hasher::<_, V, S>(&self.hash_builder));
    }

    /// Shrinks the capacity of the map with a lower limit. It will drop
    /// down no lower than the supplied limit while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    /// This function does nothing if the current capacity is smaller than the
    /// supplied minimum capacity.
    #[allow(unused)]
    pub(crate) fn shrink_to(&mut self, min_capacity: usize) {
        self.table
            .shrink_to(min_capacity, make_hasher::<_, V, S>(&self.hash_builder));
    }

    /// Returns the value corresponding to the supplied key.
    // TODO: make this return a reference instead of cloning.
    pub(crate) fn get(&self, k: &Gc<K>) -> Option<V> {
        if self.table.is_empty() {
            None
        } else {
            let hash = make_hash_from_gc(&self.hash_builder, k);
            self.table.find(hash, equivalent_key(k))?.value()
        }
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub(crate) fn contains_key(&self, k: &Gc<K>) -> bool {
        self.get(k).is_some()
    }

    // Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated.
    pub(crate) fn insert(&mut self, k: &Gc<K>, v: V) -> Option<Ephemeron<K, V>> {
        let hash = make_hash_from_gc(&self.hash_builder, k);
        let hasher = make_hasher(&self.hash_builder);
        let entry = self.table.entry(hash, equivalent_key(k), hasher);
        let (old, slot) = match entry {
            RawEntry::Occupied(occupied_entry) => {
                let (v, slot) = occupied_entry.remove();
                (Some(v), slot)
            }
            RawEntry::Vacant(vacant_entry) => (None, vacant_entry),
        };

        slot.insert(Ephemeron::new(k, v));
        old
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map. Keeps the allocated memory for reuse.
    pub(crate) fn remove(&mut self, k: &Gc<K>) -> Option<V> {
        let hash = make_hash_from_gc(&self.hash_builder, k);
        self.table
            .find_entry(hash, equivalent_key(k))
            .ok()?
            .remove()
            .0
            .value()
    }

    /// Clears all the expired keys in the map.
    pub(crate) fn clear_expired(&mut self) {
        self.retain(|eph| eph.value().is_some());
    }
}

pub(crate) struct Iter<'a, K, V>
where
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
    inner: RawIter<'a, Ephemeron<K, V>>,
    marker: PhantomData<&'a Ephemeron<K, V>>,
}

impl<K, V> Clone for Iter<'_, K, V>
where
    K: Trace + ?Sized + 'static,
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
    K: Trace + ?Sized + 'static + fmt::Debug,
    V: Trace + 'static + fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
    type Item = &'a Ephemeron<K, V>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K, V, S> fmt::Debug for RawWeakMap<K, V, S>
where
    K: fmt::Debug + ?Sized + Trace + Finalize,
    V: fmt::Debug + Trace + Finalize,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter().fmt(f)
    }
}

fn make_hasher<K, V, S>(hash_builder: &S) -> impl Fn(&Ephemeron<K, V>) -> u64 + '_
where
    S: BuildHasher,
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
    move |val| make_hash_from_eph::<K, V, S>(hash_builder, val)
}

fn make_hash_from_eph<K, V, S>(hash_builder: &S, eph: &Ephemeron<K, V>) -> u64
where
    S: BuildHasher,
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
    use std::hash::Hasher;
    let mut state = hash_builder.build_hasher();
    // TODO: Is this true for custom hashers? if not, rewrite `key` to be safe.
    // SAFETY: The return value of `key` is only used to hash it, which
    // cannot trigger a garbage collection,
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
    K: Trace + ?Sized + 'static,
{
    use std::hash::Hasher;
    let mut state = hash_builder.build_hasher();
    std::ptr::hash(gc.inner_ptr().as_ptr(), &mut state);
    state.finish()
}

fn equivalent_key<K, V>(k: &Gc<K>) -> impl Fn(&Ephemeron<K, V>) -> bool + '_
where
    K: Trace + ?Sized + 'static,
    V: Trace + 'static,
{
    // SAFETY: The return value of `key` is only used inside eq, which
    // cannot trigger a garbage collection.
    move |eph| unsafe {
        eph.inner().key().is_some_and(|val| {
            let val: *const _ = val;
            std::ptr::eq(val, k.inner_ptr().as_ptr())
        })
    }
}
