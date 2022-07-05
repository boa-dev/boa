use crate::interned_str::InternedStr;

#[derive(Debug)]
pub(super) struct FixedString<Char> {
    inner: Vec<Char>,
}

impl<Char> Default for FixedString<Char> {
    fn default() -> Self {
        Self {
            inner: Vec::default(),
        }
    }
}

impl<Char> FixedString<Char> {
    /// Creates a new, pinned [`FixedString`].
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Gets the maximum capacity of the [`FixedString`].
    pub(super) fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Returns `true` if the [`FixedString`] has length zero,
    /// and `false` otherwise.
    pub(super) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<Char> FixedString<Char>
where
    Char: Clone,
{
    /// Tries to push `string` to the [`FixedString`], and returns
    /// an [`InternedStr`] pointer to the stored `string`, or
    /// `None` if the capacity is not enough to store `string`.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring `self` outlives the returned
    /// [`InternedStr`].
    pub(super) unsafe fn push(&mut self, string: &[Char]) -> Option<InternedStr<Char>> {
        let capacity = self.inner.capacity();
        (capacity >= self.inner.len() + string.len()).then(|| {
            // SAFETY:
            // The caller is responsible for extending the lifetime
            // of `self` to outlive the return value.
            unsafe { self.push_unchecked(string) }
        })
    }

    /// Pushes `string` to the [`FixedString`], and returns
    /// an [`InternedStr`] pointer to the stored `string`, without
    /// checking if the total `capacity` is enough to store `string`,
    /// and without checking if the string is correctly aligned.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that `self` outlives the returned
    /// [`InternedStr`] and that it has enough capacity to store `string` without
    /// reallocating.
    pub(super) unsafe fn push_unchecked(&mut self, string: &[Char]) -> InternedStr<Char> {
        let old_len = self.inner.len();
        self.inner.extend_from_slice(string);

        // SAFETY: The caller is responsible for extending the lifetime
        // of `self` to outlive the return value, and for ensuring
        // the alignment of `string` is correct.
        let ptr = &self.inner[old_len..self.inner.len()];
        unsafe { InternedStr::new(ptr.into()) }
    }
}
