use crate::interned_str::InternedStr;

#[derive(Debug, Default)]
pub(super) struct FixedString {
    inner: String,
}

impl FixedString {
    /// Creates a new, pinned [`FixedString`].
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            inner: String::with_capacity(capacity),
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

    /// Tries to push `string` to the [`FixedString`], and returns
    /// an [`InternedStr`] pointer to the stored `string`, or
    /// `None` if the capacity is not enough to store `string`.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring `self` outlives the returned
    /// `InternedStr`.
    pub(super) unsafe fn push(&mut self, string: &str) -> Option<InternedStr> {
        let capacity = self.inner.capacity();
        (capacity >= self.inner.len() + string.len()).then(|| {
            let old_len = self.inner.len();
            self.inner.push_str(string);
            // SAFETY: The caller is responsible for extending the lifetime
            // of `self` to outlive the return value.
            unsafe { InternedStr::new(self.inner[old_len..self.inner.len()].into()) }
        })
    }

    /// Pushes `string` to the [`FixedString`], and returns
    /// an [`InternedStr`] pointer to the stored `string`, without
    /// checking if the total `capacity` is enough to store `string`.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that `self` outlives the returned
    /// `InternedStr` and that it has enough capacity to store `string` without
    /// reallocating.
    pub(super) unsafe fn push_unchecked(&mut self, string: &str) -> InternedStr {
        let old_len = self.inner.len();
        self.inner.push_str(string);
        // SAFETY: The caller is responsible for extending the lifetime
        // of `self` to outlive the return value.
        unsafe { InternedStr::new(self.inner[old_len..self.inner.len()].into()) }
    }
}
