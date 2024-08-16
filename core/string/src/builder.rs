use crate::{
    alloc_overflow, tagged::Tagged, JsString, RawJsString, RefCount, TaggedLen, DATA_OFFSET,
};
use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    cell::Cell,
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::AddAssign,
    ptr::{self, addr_of, addr_of_mut, NonNull},
};

#[doc(hidden)]
mod private {
    /// Inner elements represented for `JsStringBuilder`.
    pub trait JsStringData {}

    impl JsStringData for u8 {}
    impl JsStringData for u16 {}
}

/// A mutable builder to create instance of `JsString`.
///
/// # Examples
///
/// ```rust
/// use boa_string::JsStringBuilder;
/// let mut s = JsStringBuilder::new();
/// s.push(b'x');
/// s.extend_from_slice(&[b'1', b'2', b'3']);
/// s.extend([b'1', b'2', b'3']);
/// let js_string = s.build();
/// ```
#[derive(Debug)]
pub struct JsStringBuilder<T: private::JsStringData> {
    cap: usize,
    len: usize,
    inner: NonNull<RawJsString>,
    phantom_data: PhantomData<T>,
}

impl<D: private::JsStringData> Clone for JsStringBuilder<D> {
    #[must_use]
    fn clone(&self) -> Self {
        let mut builder = Self::with_capacity(self.capacity());
        // SAFETY:
        // - `inner` must be a valid pointer, since it comes from a `NonNull`
        // allocated above with the capacity of `s`, and initialize to `s.len()` in
        // ptr::copy_to_non_overlapping below.
        unsafe {
            builder
                .inner
                .as_ptr()
                .cast::<u8>()
                .copy_from_nonoverlapping(self.inner.as_ptr().cast(), self.allocated_byte_len());

            builder.set_len(self.len());
        }
        builder
    }
}

impl<D: private::JsStringData> Default for JsStringBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: private::JsStringData> JsStringBuilder<D> {
    const DATA_SIZE: usize = size_of::<D>();
    const MIN_NON_ZERO_CAP: usize = 8 / Self::DATA_SIZE;

    /// Create a new `JsStringBuilder` with capacity of zero.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cap: 0,
            len: 0,
            inner: NonNull::dangling(),
            phantom_data: PhantomData,
        }
    }

    /// Returns the number of elements that inner holds.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Forces the length of the [`JsStringBuilder`] to `new_len`.
    ///
    /// # Safety
    ///
    /// - `new_len` must be less than or equal to `capacity()`.
    /// - The elements at `old_len..new_len` must be initialized.
    ///
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());

        self.len = new_len;
    }

    /// Returns the total number of elements can hold without reallocating
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.cap
    }

    /// Returns the allocated byte of inner.
    #[must_use]
    const fn allocated_byte_len(&self) -> usize {
        DATA_OFFSET + self.allocated_data_byte_len()
    }

    /// Returns the allocated byte of inner's data.
    #[must_use]
    const fn allocated_data_byte_len(&self) -> usize {
        self.len() * Self::DATA_SIZE
    }

    /// Returns the capacity calculted from given layout.
    #[must_use]
    const fn capacity_from_layout(layout: Layout) -> usize {
        (layout.size() - DATA_OFFSET) / Self::DATA_SIZE
    }

    /// create a new `JsStringBuilder` with specific capacity
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        if cap == 0 {
            return Self::new();
        }
        let layout = Self::new_layout(cap);
        #[allow(clippy::cast_ptr_alignment)]
        // SAFETY:
        // The layout size of `RawJsString` is never zero, since it has to store
        // the length of the string and the reference count.
        let ptr = unsafe { alloc(layout) };

        let Some(ptr) = NonNull::new(ptr.cast()) else {
            std::alloc::handle_alloc_error(layout)
        };
        Self {
            cap: Self::capacity_from_layout(layout),
            len: 0,
            inner: ptr,
            phantom_data: PhantomData,
        }
    }

    /// Checks if the inner is allocated.
    #[must_use]
    fn is_allocated(&self) -> bool {
        self.inner != NonNull::dangling()
    }

    /// Returns the inner's layout.
    #[must_use]
    fn current_layout(&self) -> Layout {
        // SAFETY:
        // All the checks for the validity of the layout have already been made on `new_layout`,
        // so we can skip the unwrap.
        unsafe {
            Layout::for_value(self.inner.as_ref())
                .extend(Layout::array::<D>(self.capacity()).unwrap_unchecked())
                .unwrap_unchecked()
                .0
                .pad_to_align()
        }
    }

    /// Returns the pointer of `data` of inner.
    ///
    /// # Safety
    ///
    /// Caller should ensure that the inner is allocated.
    #[must_use]
    unsafe fn data(&self) -> *mut D {
        // SAFETY:
        // Caller should ensure that the inner is allocated.
        unsafe { addr_of_mut!((*self.inner.as_ptr()).data).cast() }
    }

    /// Inner logic of `allocate`.
    ///
    /// Use `realloc` here because it has a better performance than using `alloc`, `copy` and `dealloc`.
    #[allow(clippy::cast_ptr_alignment)]
    fn allocate_inner(&mut self, new_layout: Layout) {
        let new_ptr = if self.is_allocated() {
            let old_ptr = self.inner.as_ptr();
            let old_layout = self.current_layout();
            // SAFETY:
            // Valid pointer is required by `realloc` and pointer is checked above to be valid.
            // The layout size of `RawJsString` is never zero, since it has to store
            // the length of the string and the reference count.
            unsafe { realloc(old_ptr.cast(), old_layout, new_layout.size()) }
        } else {
            // SAFETY:
            // The layout size of `RawJsString` is never zero, since it has to store
            // the length of the string and the reference count.
            unsafe { alloc(new_layout) }
        };
        let Some(new_ptr) = NonNull::new(new_ptr.cast::<RawJsString>()) else {
            std::alloc::handle_alloc_error(new_layout)
        };
        self.inner = new_ptr;
        self.cap = Self::capacity_from_layout(new_layout);
    }

    /// Appends an element to the inner of `JsStringBuilder`.
    pub fn push(&mut self, v: D) {
        let len = self.len();
        if len == self.capacity() {
            self.allocate(len + 1);
        }
        // SAFETY:
        // Capacity has been expanded to be large enough to hold elements.
        unsafe {
            self.push_unchecked(v);
        }
    }

    /// Push elements from slice to `JsStringBuilder` without doing capacity check.
    ///
    /// Unlike the standard vector, our holded element types are only `u8` and `u16`, which is [`Copy`] derived,
    ///
    /// so we only need to copy them instead of cloning.
    ///
    /// # Safety
    ///
    /// Caller should ensure the capacity is large enough to hold elements.
    pub unsafe fn extend_from_slice_unchecked(&mut self, v: &[D]) {
        // SAFETY: Caller should ensure the capacity is large enough to hold elements.
        unsafe {
            ptr::copy_nonoverlapping(v.as_ptr(), self.data().add(self.len()), v.len());
        }
        self.len += v.len();
    }

    /// push elements from slice to `JsStringBuilder`.
    pub fn extend_from_slice(&mut self, v: &[D]) {
        let required_cap = self.len() + v.len();
        if required_cap > self.capacity() {
            self.allocate(required_cap);
        }
        // SAFETY:
        // Capacity has been expanded to be large enough to hold elements.
        unsafe {
            self.extend_from_slice_unchecked(v);
        }
    }

    fn new_layout(cap: usize) -> Layout {
        let new_layout = Layout::array::<D>(cap)
            .and_then(|arr| Layout::new::<RawJsString>().extend(arr))
            .map(|(layout, offset)| (layout.pad_to_align(), offset))
            .map_err(|_| None);
        match new_layout {
            Ok((new_layout, offset)) => {
                debug_assert_eq!(offset, DATA_OFFSET);
                new_layout
            }
            Err(None) => alloc_overflow(),
            Err(Some(layout)) => std::alloc::handle_alloc_error(layout),
        }
    }

    /// Extends `JsStringBuilder` with the contents of an iterator.
    pub fn extend<I: IntoIterator<Item = D>>(&mut self, iter: I) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        let require_cap = self.len() + lower_bound;
        if require_cap > self.capacity() {
            self.allocate(require_cap);
        }
        iterator.for_each(|c| self.push(c));
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `JsStringBuilder<D>`. The collection may reserve more space to
    /// speculatively avoid frequent reallocations. After calling `reserve`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    pub fn reserve(&mut self, additional: usize) {
        if additional > self.capacity().wrapping_sub(self.len) {
            let Some(cap) = self.len().checked_add(additional) else {
                alloc_overflow()
            };
            self.allocate(cap);
        }
    }

    /// Allocates memory to the inner by the given capacity.
    /// Capacity calculation is from [`std::vec::Vec::reserve`].
    fn allocate(&mut self, cap: usize) {
        let cap = std::cmp::max(self.capacity() * 2, cap);
        let cap = std::cmp::max(Self::MIN_NON_ZERO_CAP, cap);
        self.allocate_inner(Self::new_layout(cap));
    }

    /// Appends an element to the inner of `JsStringBuilder` without doing bounds check.
    /// # Safety
    ///
    /// Caller should ensure the capacity is large enough to hold elements.
    pub unsafe fn push_unchecked(&mut self, v: D) {
        // SAFETY: Caller should ensure the capacity is large enough to hold elements.
        unsafe {
            self.data().add(self.len()).write(v);
            self.len += 1;
        }
    }

    /// Returns true if this `JsStringBuilder` has a length of zero, and false otherwise.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if all bytes in this inner is ascii.
    fn is_ascii(&self) -> bool {
        let ptr = self.inner.as_ptr();
        // SAFETY:
        // `NonNull` verified for us that the pointer returned by `alloc` is valid,
        // meaning we can read to its pointed memory.
        let data = unsafe {
            std::slice::from_raw_parts(
                addr_of!((*ptr).data).cast::<u8>(),
                self.allocated_data_byte_len(),
            )
        };
        data.is_ascii()
    }

    /// Extracts a slice containing the elements in the inner.
    #[must_use]
    pub fn as_slice(&self) -> &[D] {
        if self.is_allocated() {
            // SAFETY:
            // The inner is allocated which means it is not null.
            unsafe { std::slice::from_raw_parts(self.data(), self.len()) }
        } else {
            &[]
        }
    }

    /// build `JsString` from `JsStringBuilder`
    #[must_use]
    pub fn build(mut self) -> JsString {
        if self.is_empty() {
            return JsString::default();
        }
        let len = self.len();

        // Shrink to fit the length.
        if len != self.capacity() {
            let layout = Self::new_layout(self.len());
            self.allocate_inner(layout);
        }

        let inner = self.inner;

        // SAFETY:
        // `NonNull` verified for us that the pointer returned by `alloc` is valid,
        // meaning we can write to its pointed memory.
        unsafe {
            inner.as_ptr().write(RawJsString {
                tagged_len: TaggedLen::new(len, self.is_ascii()),
                refcount: RefCount {
                    read_write: ManuallyDrop::new(Cell::new(1)),
                },
                data: [0; 0],
            });
        }

        // Tell the compiler not to call the destructor of `JsStringBuilder`,
        // becuase we move inner `RawJsString` to `JsString`.
        std::mem::forget(self);
        JsString {
            ptr: Tagged::from_non_null(inner),
        }
    }
}

impl<D: private::JsStringData> Drop for JsStringBuilder<D> {
    /// Set cold since [`JsStringBuilder`] should be created to build `JsString`
    #[cold]
    fn drop(&mut self) {
        if self.is_allocated() {
            let layout = self.current_layout();
            // SAFETY:
            // layout: All the checks for the validity of the layout have already been made on `allocate_inner`.
            // `NonNull` verified for us that the pointer returned by `alloc` is valid,
            // meaning we can free its pointed memory.
            unsafe {
                dealloc(self.inner.as_ptr().cast(), layout);
            }
        }
    }
}

impl<D: private::JsStringData> AddAssign<&JsStringBuilder<D>> for JsStringBuilder<D> {
    fn add_assign(&mut self, rhs: &JsStringBuilder<D>) {
        self.extend_from_slice(rhs.as_slice());
    }
}

impl<D: private::JsStringData> AddAssign<&[D]> for JsStringBuilder<D> {
    fn add_assign(&mut self, rhs: &[D]) {
        self.extend_from_slice(rhs);
    }
}

impl<D: private::JsStringData> FromIterator<D> for JsStringBuilder<D> {
    fn from_iter<T: IntoIterator<Item = D>>(iter: T) -> Self {
        let mut builder = Self::new();
        builder.extend(iter);
        builder
    }
}
