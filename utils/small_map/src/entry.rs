use std::{
    collections::{btree_map, BTreeMap},
    fmt::Debug,
};

use arrayvec::ArrayVec;

use super::SmallMap;

use Entry::{Occupied, Vacant};

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`SmallMap`].
///
/// [`entry`]: SmallMap::entry
pub enum Entry<'a, K, V, const ARRAY_SIZE: usize> {
    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V, ARRAY_SIZE>),
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V, ARRAY_SIZE>),
}

impl<K: Debug + Ord, V: Debug, const ARRAY_SIZE: usize> Debug for Entry<'_, K, V, ARRAY_SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vacant(ref v) => f.debug_tuple("Entry").field(v).finish(),
            Self::Occupied(ref o) => f.debug_tuple("Entry").field(o).finish(),
        }
    }
}

/// A view into a vacant entry in a `SmallMap`.
/// It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, K, V, const ARRAY_SIZE: usize> {
    pub(super) inner: InnerVacant<'a, K, V, ARRAY_SIZE>,
}

pub(super) enum InnerVacant<'a, K, V, const ARRAY_SIZE: usize> {
    Inline(InlineVacantEntry<'a, K, V, ARRAY_SIZE>),
    Heap(btree_map::VacantEntry<'a, K, V>),
}

impl<K: Debug + Ord, V, const ARRAY_SIZE: usize> Debug for VacantEntry<'_, K, V, ARRAY_SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("VacantEntry").field(self.key()).finish()
    }
}

/// A view into an occupied entry in a `SmallMap`.
/// It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, K, V, const ARRAY_SIZE: usize> {
    pub(super) inner: InnerOccupied<'a, K, V, ARRAY_SIZE>,
}

pub(super) enum InnerOccupied<'a, K, V, const ARRAY_SIZE: usize> {
    Inline(InlineOccupiedEntry<'a, K, V, ARRAY_SIZE>),
    Heap(btree_map::OccupiedEntry<'a, K, V>),
}

impl<K: Ord + Debug, V: Debug, const ARRAY_SIZE: usize> Debug
    for OccupiedEntry<'_, K, V, ARRAY_SIZE>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("key", self.key())
            .field("value", self.get())
            .finish()
    }
}

impl<'a, K: Ord, V, const ARRAY_SIZE: usize> Entry<'a, K, V, ARRAY_SIZE> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function a reference to the key that was moved during the `.entry(key)` method call.
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is
    /// unnecessary, unlike with `.or_insert_with(|| ... )`.
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    /// Returns a reference to this entry's key.
    pub fn key(&self) -> &K {
        match self {
            Occupied(entry) => entry.key(),
            Vacant(entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Occupied(mut entry) => {
                f(entry.get_mut());
                Occupied(entry)
            }
            Vacant(entry) => Vacant(entry),
        }
    }
}

impl<'a, K: Ord, V: Default, const ARRAY_SIZE: usize> Entry<'a, K, V, ARRAY_SIZE> {
    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_default(self) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(Default::default()),
        }
    }
}

impl<'a, K: Ord, V, const ARRAY_SIZE: usize> VacantEntry<'a, K, V, ARRAY_SIZE> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    pub fn key(&self) -> &K {
        match &self.inner {
            InnerVacant::Inline(i) => i.key(),
            InnerVacant::Heap(v) => v.key(),
        }
    }

    /// Takes ownership of the key.
    pub fn into_key(self) -> K {
        match self.inner {
            InnerVacant::Inline(i) => i.into_key(),
            InnerVacant::Heap(v) => v.into_key(),
        }
    }

    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V {
        match self.inner {
            InnerVacant::Inline(i) => i.insert(value),
            InnerVacant::Heap(v) => v.insert(value),
        }
    }
}

impl<'a, K: Ord, V, const ARRAY_SIZE: usize> OccupiedEntry<'a, K, V, ARRAY_SIZE> {
    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &K {
        match &self.inner {
            InnerOccupied::Inline(o) => o.key(),
            InnerOccupied::Heap(o) => o.key(),
        }
    }

    /// Takes ownership of the key and value from the map.
    pub fn remove_entry(self) -> (K, V) {
        match self.inner {
            InnerOccupied::Inline(o) => o.remove_entry(),
            InnerOccupied::Heap(o) => o.remove_entry(),
        }
    }

    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        match &self.inner {
            InnerOccupied::Inline(o) => o.get(),
            InnerOccupied::Heap(o) => o.get(),
        }
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` that may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: OccupiedEntry::into_mut
    pub fn get_mut(&mut self) -> &mut V {
        match &mut self.inner {
            InnerOccupied::Inline(o) => o.get_mut(),
            InnerOccupied::Heap(o) => o.get_mut(),
        }
    }

    /// Converts the entry into a mutable reference to its value.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: OccupiedEntry::get_mut
    pub fn into_mut(self) -> &'a mut V {
        match self.inner {
            InnerOccupied::Inline(o) => o.into_mut(),
            InnerOccupied::Heap(o) => o.into_mut(),
        }
    }

    /// Sets the value of the entry with the `OccupiedEntry`'s key,
    /// and returns the entry's old value.
    pub fn insert(&mut self, value: V) -> V {
        match &mut self.inner {
            InnerOccupied::Inline(o) => o.insert(value),
            InnerOccupied::Heap(o) => o.insert(value),
        }
    }

    /// Takes the value of the entry out of the map, and returns it.
    pub fn remove(self) -> V {
        match self.inner {
            InnerOccupied::Inline(o) => o.remove(),
            InnerOccupied::Heap(o) => o.remove(),
        }
    }
}

pub(super) struct InlineVacantEntry<'a, K, V, const ARRAY_SIZE: usize> {
    pub(super) key: K,
    pub(super) map: &'a mut SmallMap<K, V, ARRAY_SIZE>,
}

impl<'a, K: Ord + Eq, V, const ARRAY_SIZE: usize> InlineVacantEntry<'a, K, V, ARRAY_SIZE> {
    pub(super) fn key(&self) -> &K {
        &self.key
    }

    pub(super) fn into_key(self) -> K {
        self.key
    }

    pub(super) fn insert(self, value: V) -> &'a mut V {
        let InlineVacantEntry { key, map } = self;

        let vec = match &mut map.inner {
            super::Inner::Inline(vec) => {
                if !vec.is_full() {
                    let len = vec.len();
                    vec.push((key, value));

                    // Workaround for Problem case 3 of the current borrow checker.
                    // https://rust-lang.github.io/rfcs/2094-nll.html#problem-case-3-conditional-control-flow-across-functions

                    match &mut map.inner {
                        super::Inner::Inline(vec) => return &mut vec[len].1,
                        super::Inner::Heap(_) => unreachable!(),
                    }
                }

                std::mem::take(vec)
            }
            super::Inner::Heap(_) => unreachable!(),
        };

        // Need to convert to a heap allocated map.

        let btree = BTreeMap::from_iter(vec);

        *map = SmallMap {
            inner: super::Inner::Heap(btree),
        };

        match &mut map.inner {
            super::Inner::Inline(_) => unreachable!(),
            super::Inner::Heap(h) => h.entry(key).or_insert(value),
        }
    }
}

pub(super) struct InlineOccupiedEntry<'a, K, V, const ARRAY_SIZE: usize> {
    pub(super) index: usize,
    pub(super) array: &'a mut ArrayVec<(K, V), ARRAY_SIZE>,
}

impl<'a, K, V, const ARRAY_SIZE: usize> InlineOccupiedEntry<'a, K, V, ARRAY_SIZE> {
    pub(super) fn key(&self) -> &K {
        &self.array[self.index].0
    }

    pub(super) fn remove_entry(self) -> (K, V) {
        self.array.remove(self.index)
    }

    pub(super) fn get(&self) -> &V {
        &self.array[self.index].1
    }

    pub(super) fn get_mut(&mut self) -> &mut V {
        &mut self.array[self.index].1
    }

    pub(super) fn into_mut(self) -> &'a mut V {
        &mut self.array[self.index].1
    }

    pub(super) fn insert(&mut self, value: V) -> V {
        std::mem::replace(&mut self.array[self.index].1, value)
    }

    pub(super) fn remove(self) -> V {
        self.remove_entry().1
    }
}
