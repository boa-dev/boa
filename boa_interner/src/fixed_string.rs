use crate::{interned_str::InternedStr, JStrRef};

#[derive(Debug, Default)]
pub(super) struct FixedString {
    inner: Vec<u8>,
}

impl FixedString {
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

    /// Tries to push `string` to the [`FixedString`], and returns
    /// an [`InternedStr`] pointer to the stored `string`, or
    /// `None` if the capacity is not enough to store `string`.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring `self` outlives the returned
    /// `InternedStr`.
    pub(super) unsafe fn push(&mut self, string: JStrRef<'_>) -> Option<InternedStr> {
        let capacity = self.inner.capacity();
        let padding_len = match string.encoding() {
            crate::Encoding::Utf8 => 0,
            crate::Encoding::Utf16 => {
                if self.inner.len() % 2 == 0 {
                    0
                } else {
                    1
                }
            }
        };
        (capacity >= self.inner.len() + string.byte_len() + padding_len).then(|| {
            for _ in 0..padding_len {
                self.inner.push(0);
            }
            // SAFETY:
            // - The caller is responsible for extending the lifetime
            // of `self` to outlive the return value.
            // - With the checks above, we've already ensured
            // that, if `string` is a UTF-16 string, then the next push to
            // `inner` will be 2-byte aligned.
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
    /// `InternedStr`, that it has enough capacity to store `string` without
    /// reallocating and that, if `string` is UTF-16 encoded, then the pushed bytes
    /// will be 2-byte aligned in `inner` .
    pub(super) unsafe fn push_unchecked(&mut self, string: JStrRef<'_>) -> InternedStr {
        let str_bytes = string.as_byte_slice();
        let old_len = self.inner.len();
        self.inner.extend_from_slice(str_bytes);

        // SAFETY: The caller is responsible for extending the lifetime
        // of `self` to outlive the return value, and for ensuring
        // the alignment of `string` is correct.
        let ptr = std::ptr::addr_of!(self.inner[old_len]);
        unsafe { InternedStr::new(ptr, string.slice_len(), string.encoding()) }
    }
}
