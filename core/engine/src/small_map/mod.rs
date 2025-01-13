// TODO: Maybe extract to a separate crate? It could be useful for some applications.
#![allow(unreachable_pub)]
#![allow(unused)]

use std::{
    borrow::Borrow,
    collections::{btree_map, BTreeMap},
    fmt,
    hash::{Hash, Hasher},
    iter::FusedIterator,
    ops::{Index, IndexMut},
};

use arrayvec::ArrayVec;

mod entry;

pub use entry::{Entry, OccupiedEntry, VacantEntry};

use Entry::{Occupied, Vacant};

/// A map that is initially backed by an inline vec, but changes its backing to a heap map if its
/// number of elements exceeds `ARRAY_SIZE`.
#[derive(Clone)]
pub(crate) struct SmallMap<K, V, const ARRAY_SIZE: usize> {
    inner: Inner<K, V, ARRAY_SIZE>,
}

#[derive(Debug, Clone)]
enum Inner<K, V, const ARRAY_SIZE: usize> {
    Inline(ArrayVec<(K, V), ARRAY_SIZE>),
    Heap(BTreeMap<K, V>),
}

/// An iterator over the entries of a `SmallMap`.
///
/// This `struct` is created by the [`iter`] method on [`SmallMap`]. See its
/// documentation for more.
///
/// [`iter`]: SmallMap::iter
#[derive(Clone)]
pub struct Iter<'a, K, V> {
    inner: InnerIter<'a, K, V>,
}

#[derive(Clone)]
enum InnerIter<'a, K, V> {
    Inline(std::slice::Iter<'a, (K, V)>),
    Heap(btree_map::Iter<'a, K, V>),
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            InnerIter::Inline(i) => f.debug_tuple("Inline").field(i).finish(),
            InnerIter::Heap(h) => f.debug_tuple("Heap").field(h).finish(),
        }
    }
}

impl<K, V> Default for Iter<'_, K, V> {
    /// Creates an empty `small_map::Iter`.
    fn default() -> Self {
        Self {
            inner: InnerIter::Inline(std::slice::Iter::default()),
        }
    }
}

/// A mutable iterator over the entries of a `SmallMap`.
///
/// This `struct` is created by the [`iter_mut`] method on [`SmallMap`]. See its
/// documentation for more.
///
/// [`iter_mut`]: SmallMap::iter_mut
pub struct IterMut<'a, K, V> {
    inner: InnerIterMut<'a, K, V>,
}

enum InnerIterMut<'a, K, V> {
    Inline(std::slice::IterMut<'a, (K, V)>),
    Heap(btree_map::IterMut<'a, K, V>),
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IterMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            InnerIterMut::Inline(i) => f.debug_tuple("Inline").field(i).finish(),
            InnerIterMut::Heap(h) => f.debug_tuple("Heap").field(h).finish(),
        }
    }
}

impl<K, V> Default for IterMut<'_, K, V> {
    /// Creates an empty `small_map::IterMut`.
    fn default() -> Self {
        Self {
            inner: InnerIterMut::Inline(std::slice::IterMut::default()),
        }
    }
}

/// An owning iterator over the entries of a `SmallMap`.
///
/// This `struct` is created by the [`into_iter`] method on [`SmallMap`]
/// (provided by the [`IntoIterator`] trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
pub struct IntoIter<K, V, const ARRAY_SIZE: usize> {
    inner: InnerIntoIter<K, V, ARRAY_SIZE>,
}

enum InnerIntoIter<K, V, const ARRAY_SIZE: usize> {
    Inline(arrayvec::IntoIter<(K, V), ARRAY_SIZE>),
    Heap(btree_map::IntoIter<K, V>),
}

impl<K: fmt::Debug, V: fmt::Debug, const ARRAY_SIZE: usize> fmt::Debug
    for IntoIter<K, V, ARRAY_SIZE>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            InnerIntoIter::Inline(i) => f.debug_tuple("Inline").field(i).finish(),
            InnerIntoIter::Heap(h) => f.debug_tuple("Heap").field(h).finish(),
        }
    }
}

impl<K, V, const ARRAY_SIZE: usize> Default for IntoIter<K, V, ARRAY_SIZE> {
    /// Creates an empty `small_map::IntoIter`.
    fn default() -> Self {
        Self {
            inner: InnerIntoIter::Inline(ArrayVec::new().into_iter()),
        }
    }
}

impl<K, V, const ARRAY_SIZE: usize> SmallMap<K, V, ARRAY_SIZE> {
    /// Makes a new, empty `SmallMap`.
    pub const fn new() -> Self {
        Self {
            inner: Inner::Inline(ArrayVec::new_const()),
        }
    }

    /// Clears the map, removing all elements.
    ///
    /// The current implementation will preserve the heap map allocation
    /// if the map has already transitioned to the fallback heap map.
    pub fn clear(&mut self) {
        match &mut self.inner {
            Inner::Inline(v) => v.clear(),
            Inner::Heap(h) => h.clear(),
        }
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord + Eq,
        Q: ?Sized + Ord + Eq,
    {
        match &self.inner {
            Inner::Inline(v) => v.iter().find(|(k, _)| k.borrow() == key).map(|(_, v)| v),
            Inner::Heap(h) => h.get(key),
        }
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The supplied key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[allow(clippy::map_identity)]
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q> + Ord + Eq,
        Q: ?Sized + Ord + Eq,
    {
        match &self.inner {
            Inner::Inline(v) => v
                .iter()
                .find(|(k, _)| k.borrow() == key)
                .map(|(k, v)| (k, v)),
            Inner::Heap(h) => h.get_key_value(key),
        }
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord + Eq,
        Q: ?Sized + Ord + Eq,
    {
        self.get(key).is_some()
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord + Eq,
        Q: ?Sized + Ord + Eq,
    {
        match &mut self.inner {
            Inner::Inline(v) => v
                .iter_mut()
                .find(|(k, _)| k.borrow() == key)
                .map(|(_, v)| v),
            Inner::Heap(h) => h.get_mut(key),
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical. See the [**Insert and complex keys**][keys]
    /// section from the [`std::collections`] module documentation for more information.
    ///
    /// [keys]: https://doc.rust-lang.org/std/collections/index.html#insert-and-complex-keys
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Eq + Ord,
    {
        match self.entry(key) {
            Occupied(mut entry) => Some(entry.insert(value)),
            Vacant(entry) => {
                entry.insert(value);
                None
            }
        }
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord + Eq,
        Q: ?Sized + Ord + Eq,
    {
        self.remove_entry(key).map(|(_, v)| v)
    }

    /// Removes a key from the map, returning the stored key and value if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: ?Sized + Ord,
    {
        match &mut self.inner {
            Inner::Inline(v) => v
                .iter()
                .position(|(k, _)| k.borrow() == key)
                .map(|idx| v.remove(idx)),
            Inner::Heap(h) => h.remove_entry(key),
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)` returns `false`.
    pub fn retain<F>(&mut self, mut f: F)
    where
        K: Ord,
        F: FnMut(&K, &mut V) -> bool,
    {
        match &mut self.inner {
            Inner::Inline(v) => v.retain(|(k, v)| f(k, v)),
            Inner::Heap(h) => h.retain(f),
        }
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    ///
    /// If a key from `other` is already present in `self`, the respective
    /// value from `self` will be overwritten with the respective value from `other`.
    pub fn append<const OTHER_SIZE: usize>(&mut self, other: &mut SmallMap<K, V, OTHER_SIZE>)
    where
        K: Ord + Eq,
    {
        if other.is_empty() {
            return;
        }

        let inline = matches!(other.inner, Inner::Inline(_));

        let other = std::mem::replace(
            other,
            SmallMap {
                inner: if inline {
                    Inner::Inline(ArrayVec::new())
                } else {
                    Inner::Heap(BTreeMap::new())
                },
            },
        );

        self.extend(other);
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V, ARRAY_SIZE>
    where
        K: Eq + Ord,
    {
        match &mut self.inner {
            Inner::Inline(array) => {
                let Some(index) = array.iter().position(|(k, _)| *k == key) else {
                    return Vacant(VacantEntry {
                        inner: entry::InnerVacant::Inline(entry::InlineVacantEntry {
                            key,
                            map: self,
                        }),
                    });
                };

                // Workaround for Problem case 3 of the current borrow checker.
                // https://rust-lang.github.io/rfcs/2094-nll.html#problem-case-3-conditional-control-flow-across-functions
                // Hopefully we can remove this with some improvements to the borrow checker.
                match &mut self.inner {
                    Inner::Inline(array) => Occupied(OccupiedEntry {
                        inner: entry::InnerOccupied::Inline(entry::InlineOccupiedEntry {
                            index,
                            array,
                        }),
                    }),
                    Inner::Heap(_) => unreachable!(),
                }
            }
            // Same workaround as above.
            Inner::Heap(_) => match &mut self.inner {
                Inner::Heap(h) => match h.entry(key) {
                    btree_map::Entry::Vacant(entry) => Vacant(VacantEntry {
                        inner: entry::InnerVacant::Heap(entry),
                    }),
                    btree_map::Entry::Occupied(entry) => Occupied(OccupiedEntry {
                        inner: entry::InnerOccupied::Heap(entry),
                    }),
                },
                Inner::Inline(_) => unreachable!(),
            },
        }
    }
}

impl<'a, K, V, const ARRAY_SIZE: usize> IntoIterator for &'a SmallMap<K, V, ARRAY_SIZE> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[allow(clippy::map_identity)]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            InnerIter::Inline(i) => i.next().map(|(k, v)| (k, v)),
            InnerIter::Heap(h) => h.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            InnerIter::Inline(i) => i.size_hint(),
            InnerIter::Heap(h) => h.size_hint(),
        }
    }

    #[allow(clippy::map_identity)]
    fn last(self) -> Option<(&'a K, &'a V)> {
        match self.inner {
            InnerIter::Inline(i) => i.last().map(|(k, v)| (k, v)),
            InnerIter::Heap(h) => h.last(),
        }
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Iter<'a, K, V> {
    #[allow(clippy::map_identity)]
    fn next_back(&mut self) -> Option<(&'a K, &'a V)> {
        match &mut self.inner {
            InnerIter::Inline(i) => i.next_back().map(|(k, v)| (k, v)),
            InnerIter::Heap(h) => h.next_back(),
        }
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        match &self.inner {
            InnerIter::Inline(i) => i.len(),
            InnerIter::Heap(h) => h.len(),
        }
    }
}

impl<'a, K, V, const ARRAY_SIZE: usize> IntoIterator for &'a mut SmallMap<K, V, ARRAY_SIZE> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            InnerIterMut::Inline(i) => i.next().map(|(k, v)| (&*k, v)),
            InnerIterMut::Heap(h) => h.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            InnerIterMut::Inline(i) => i.size_hint(),
            InnerIterMut::Heap(h) => h.size_hint(),
        }
    }

    fn last(self) -> Option<(&'a K, &'a mut V)> {
        match self.inner {
            InnerIterMut::Inline(i) => i.last().map(|(k, v)| (&*k, v)),
            InnerIterMut::Heap(h) => h.last(),
        }
    }
}

impl<K, V> FusedIterator for IterMut<'_, K, V> {}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a K, &'a mut V)> {
        match &mut self.inner {
            InnerIterMut::Inline(i) => i.next_back().map(|(k, v)| (&*k, v)),
            InnerIterMut::Heap(h) => h.next_back(),
        }
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    fn len(&self) -> usize {
        match &self.inner {
            InnerIterMut::Inline(i) => i.len(),
            InnerIterMut::Heap(h) => h.len(),
        }
    }
}

impl<K, V, const ARRAY_SIZE: usize> IntoIterator for SmallMap<K, V, ARRAY_SIZE> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, ARRAY_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        match self.inner {
            Inner::Inline(i) => IntoIter {
                inner: InnerIntoIter::Inline(i.into_iter()),
            },
            Inner::Heap(h) => IntoIter {
                inner: InnerIntoIter::Heap(h.into_iter()),
            },
        }
    }
}

impl<K, V, const ARRAY_SIZE: usize> Iterator for IntoIter<K, V, ARRAY_SIZE> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        match &mut self.inner {
            InnerIntoIter::Inline(i) => i.next(),
            InnerIntoIter::Heap(h) => h.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            InnerIntoIter::Inline(i) => i.size_hint(),
            InnerIntoIter::Heap(h) => h.size_hint(),
        }
    }
}

impl<K, V, const ARRAY_SIZE: usize> DoubleEndedIterator for IntoIter<K, V, ARRAY_SIZE> {
    fn next_back(&mut self) -> Option<(K, V)> {
        match &mut self.inner {
            InnerIntoIter::Inline(i) => i.next_back(),
            InnerIntoIter::Heap(h) => h.next_back(),
        }
    }
}

impl<K, V, const ARRAY_SIZE: usize> ExactSizeIterator for IntoIter<K, V, ARRAY_SIZE> {
    fn len(&self) -> usize {
        match &self.inner {
            InnerIntoIter::Inline(i) => i.len(),
            InnerIntoIter::Heap(h) => h.len(),
        }
    }
}

impl<K, V, const ARRAY_SIZE: usize> FusedIterator for IntoIter<K, V, ARRAY_SIZE> {}

impl<K: Eq + Ord, V, const ARRAY_SIZE: usize> Extend<(K, V)> for SmallMap<K, V, ARRAY_SIZE> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |(k, v)| {
            self.insert(k, v);
        });
    }
}

impl<'a, K: Eq + Ord + Copy, V: Copy, const ARRAY_SIZE: usize> Extend<(&'a K, &'a V)>
    for SmallMap<K, V, ARRAY_SIZE>
{
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().map(|(&key, &value)| (key, value)));
    }
}

impl<K: Hash, V: Hash, const ARRAY_SIZE: usize> Hash for SmallMap<K, V, ARRAY_SIZE> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // TODO: track https://github.com/rust-lang/rust/issues/96762
        // state.write_length_prefix(self.len());
        state.write_usize(self.len());
        for elt in self {
            elt.hash(state);
        }
    }
}

impl<K, V, const ARRAY_SIZE: usize> Default for SmallMap<K, V, ARRAY_SIZE> {
    /// Creates an empty `SmallMap`.
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialEq + Ord, V: PartialEq, const LHS_SIZE: usize, const RHS_SIZE: usize>
    PartialEq<SmallMap<K, V, RHS_SIZE>> for SmallMap<K, V, LHS_SIZE>
{
    fn eq(&self, other: &SmallMap<K, V, RHS_SIZE>) -> bool {
        if let (Inner::Heap(lhs), Inner::Heap(rhs)) = (&self.inner, &other.inner) {
            return lhs == rhs;
        }

        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).is_some_and(|v| *value == *v))
    }
}

impl<K: Eq + Ord, V: Eq, const ARRAY_SIZE: usize> Eq for SmallMap<K, V, ARRAY_SIZE> {}

impl<K: fmt::Debug, V: fmt::Debug, const ARRAY_SIZE: usize> fmt::Debug
    for SmallMap<K, V, ARRAY_SIZE>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, Q: ?Sized, V, const ARRAY_SIZE: usize> Index<&Q> for SmallMap<K, V, ARRAY_SIZE>
where
    K: Eq + Ord + Borrow<Q>,
    Q: Eq + Ord,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).expect("no entry found for key")
    }
}

impl<K, Q: ?Sized, V, const ARRAY_SIZE: usize> IndexMut<&Q> for SmallMap<K, V, ARRAY_SIZE>
where
    K: Eq + Ord + Borrow<Q>,
    Q: Eq + Ord,
{
    fn index_mut(&mut self, index: &Q) -> &mut Self::Output {
        self.get_mut(index).expect("no entry found for key")
    }
}

impl<K, V, const ARRAY_SIZE: usize> SmallMap<K, V, ARRAY_SIZE> {
    /// Gets an iterator over the entries of the map.
    pub fn iter(&self) -> Iter<'_, K, V> {
        match &self.inner {
            Inner::Inline(i) => Iter {
                inner: InnerIter::Inline(i.iter()),
            },
            Inner::Heap(h) => Iter {
                inner: InnerIter::Heap(h.iter()),
            },
        }
    }

    /// Gets a mutable iterator over the entries of the map.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        match &mut self.inner {
            Inner::Inline(i) => IterMut {
                inner: InnerIterMut::Inline(i.iter_mut()),
            },
            Inner::Heap(h) => IterMut {
                inner: InnerIterMut::Heap(h.iter_mut()),
            },
        }
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        match &self.inner {
            Inner::Inline(i) => i.len(),
            Inner::Heap(h) => h.len(),
        }
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
