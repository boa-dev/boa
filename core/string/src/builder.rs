use crate::{
    alloc_overflow, tagged::Tagged, JsStr, JsStrVariant, JsString, RawJsString, RefCount,
    TaggedLen, DATA_OFFSET,
};

use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    cell::Cell,
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::AddAssign,
    ptr::{self, addr_of_mut, NonNull},
    str::{self},
};

#[doc(hidden)]
mod private {
    pub trait Sealed {}

    impl Sealed for u8 {}
    impl Sealed for u16 {}
}

/// Inner elements represented for `JsStringBuilder`.
pub trait JsStringData: private::Sealed {}

impl JsStringData for u8 {}
impl JsStringData for u16 {}

/// A mutable builder to create instance of `JsString`.
///
#[derive(Debug)]
pub struct JsStringBuilder<T: JsStringData> {
    cap: usize,
    len: usize,
    inner: NonNull<RawJsString>,
    phantom_data: PhantomData<T>,
}

impl<D: JsStringData> Clone for JsStringBuilder<D> {
    #[inline]
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

impl<D: JsStringData> Default for JsStringBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: JsStringData> JsStringBuilder<D> {
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

    /// Returns the capacity calculated from given layout.
    #[must_use]
    const fn capacity_from_layout(layout: Layout) -> usize {
        (layout.size() - DATA_OFFSET) / Self::DATA_SIZE
    }

    /// Create a new `JsStringBuilder` with specific capacity
    #[inline]
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

    /// Allocates when there is not sufficient capacity.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn allocate_if_needed(&mut self, reuired_cap: usize) {
        if reuired_cap > self.capacity() {
            self.allocate(reuired_cap);
        }
    }

    /// Inner logic of `allocate`.
    ///
    /// Use `realloc` here because it has a better performance than using combination of `alloc`, `copy` and `dealloc`.
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
    #[inline]
    pub fn push(&mut self, v: D) {
        let required_cap = self.len() + 1;
        self.allocate_if_needed(required_cap);
        // SAFETY:
        // Capacity has been expanded to be large enough to hold elements.
        unsafe {
            self.push_unchecked(v);
        }
    }

    /// Pushes elements from slice to `JsStringBuilder` without doing capacity check.
    ///
    /// Unlike the standard vector, our holded element types are only `u8` and `u16`, which is [`Copy`] derived,
    ///
    /// so we only need to copy them instead of cloning.
    ///
    /// # Safety
    ///
    /// Caller should ensure the capacity is large enough to hold elements.
    #[inline]
    pub unsafe fn extend_from_slice_unchecked(&mut self, v: &[D]) {
        // SAFETY: Caller should ensure the capacity is large enough to hold elements.
        unsafe {
            ptr::copy_nonoverlapping(v.as_ptr(), self.data().add(self.len()), v.len());
        }
        self.len += v.len();
    }

    /// Pushes elements from slice to `JsStringBuilder`.
    #[inline]
    pub fn extend_from_slice(&mut self, v: &[D]) {
        let required_cap = self.len() + v.len();
        self.allocate_if_needed(required_cap);
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
    #[inline]
    pub fn extend<I: IntoIterator<Item = D>>(&mut self, iter: I) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        let require_cap = self.len() + lower_bound;
        self.allocate_if_needed(require_cap);
        iterator.for_each(|c| self.push(c));
    }

    /// Similar to [`Vec::reserve`]
    ///
    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `JsStringBuilder<D>`. The collection may reserve more space to
    /// speculatively avoid frequent reallocations. After calling `reserve`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if additional > self.capacity().wrapping_sub(self.len) {
            let Some(cap) = self.len().checked_add(additional) else {
                alloc_overflow()
            };
            self.allocate(cap);
        }
    }

    /// Similar to [`Vec::reserve_exact`]
    ///
    /// Reserves the minimum capacity for at least `additional` more elements to
    /// be inserted in the given `JsStringBuilder<D>`. Unlike [`reserve`], this will not
    /// deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to
    /// `self.len() + additional`. Does nothing if the capacity is already
    /// sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore, capacity can not be relied upon to be precisely
    /// minimal. Prefer [`reserve`] if future insertions are expected.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        if additional > self.capacity().wrapping_sub(self.len) {
            vec![].reserve_exact(additional);
            let Some(cap) = self.len().checked_add(additional) else {
                alloc_overflow()
            };
            self.allocate_inner(cap);
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
    #[inline]
    pub unsafe fn push_unchecked(&mut self, v: D) {
        // SAFETY: Caller should ensure the capacity is large enough to hold elements.
        unsafe {
            self.data().add(self.len()).write(v);
            self.len += 1;
        }
    }

    /// Returns true if this `JsStringBuilder` has a length of zero, and false otherwise.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if all bytes in this inner is ascii.
    fn is_ascii(&self) -> bool {
        // SAFETY:
        // `NonNull` verified for us that the pointer returned by `alloc` is valid,
        // meaning we can read to its pointed memory.
        let data = unsafe {
            std::slice::from_raw_parts(self.data().cast::<u8>(), self.allocated_data_byte_len())
        };
        data.is_ascii()
    }

    /// Extracts a slice containing the elements in the inner.
    #[inline]
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

    /// Builds `JsString` from `JsStringBuilder`
    #[inline]
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

impl<D: JsStringData> Drop for JsStringBuilder<D> {
    /// Set cold since [`JsStringBuilder`] should be created to build `JsString`
    #[cold]
    #[inline]
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

impl<D: JsStringData> AddAssign<&JsStringBuilder<D>> for JsStringBuilder<D> {
    fn add_assign(&mut self, rhs: &JsStringBuilder<D>) {
        self.extend_from_slice(rhs.as_slice());
    }
}

impl<D: JsStringData> AddAssign<&[D]> for JsStringBuilder<D> {
    fn add_assign(&mut self, rhs: &[D]) {
        self.extend_from_slice(rhs);
    }
}

impl<D: JsStringData> FromIterator<D> for JsStringBuilder<D> {
    fn from_iter<T: IntoIterator<Item = D>>(iter: T) -> Self {
        let mut builder = Self::new();
        builder.extend(iter);
        builder
    }
}

/// **1 byte** encoded `JsStringBuilder`
/// # Warning
/// If you are not sure the characters that will be added and don't want to preprocess them,
/// use [`CommonJsStringBuilder`] instead.
/// ## Examples
///
/// ```rust
/// use boa_string::Latin1StringBuilder;
/// let mut s = Latin1StringBuilder::new();
/// s.push(b'x');
/// s.extend_from_slice(&[b'1', b'2', b'3']);
/// s.extend([b'1', b'2', b'3']);
/// let js_string = s.build();
/// ```
pub type Latin1StringBuilder = JsStringBuilder<u8>;

/// **2 bytes** encoded `JsStringBuilder`
/// # Warning
/// If you are not sure the characters that will be added and don't want to preprocess them,
/// use [`CommonJsStringBuilder`] instead.
/// ## Examples
///
/// ```rust
/// use boa_string::Utf16StringBuilder;
/// let mut s = Utf16StringBuilder::new();
/// s.push(b'x' as u16);
/// s.extend_from_slice(&[b'1', b'2', b'3'].map(u16::from));
/// s.extend([0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5,]); // 🎹🎶🎵
/// let js_string = s.build();
/// ```
pub type Utf16StringBuilder = JsStringBuilder<u16>;

/// String segment to build [`JsString`]
#[derive(Clone, Debug)]
pub enum Segment<'a> {
    String(JsString),
    Str(JsStr<'a>),
    Latin1(u8),
    CodePoint(char),
}

impl Segment<'_> {
    #[inline]
    #[must_use]
    fn is_latin1(&self) -> bool {
        match self {
            Segment::String(s) => s.as_str().is_latin1(),
            Segment::Str(s) => s.is_latin1(),
            Segment::Latin1(_) => true,
            Segment::CodePoint(ch) => *ch as u32 <= 0xFF,
        }
    }
}

impl From<JsString> for Segment<'_> {
    fn from(value: JsString) -> Self {
        Self::String(value)
    }
}

impl From<String> for Segment<'_> {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl From<&[u16]> for Segment<'_> {
    fn from(value: &[u16]) -> Self {
        Self::String(value.into())
    }
}

impl From<&str> for Segment<'_> {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl<'seg, 'ref_str: 'seg> From<JsStr<'ref_str>> for Segment<'seg> {
    fn from(value: JsStr<'ref_str>) -> Self {
        Self::Str(value)
    }
}

impl From<u8> for Segment<'_> {
    fn from(value: u8) -> Self {
        Self::Latin1(value)
    }
}

impl From<char> for Segment<'_> {
    fn from(value: char) -> Self {
        Self::CodePoint(value)
    }
}

/// Originally based on [kiesel-js](https://codeberg.org/kiesel-js/kiesel/src/branch/main/src/types/language/String/Builder.zig)
///
/// Common `JsString` builder that accepts multiple variant of string or character.
#[derive(Clone, Debug, Default)]
pub struct CommonJsStringBuilder<'a> {
    segments: Vec<Segment<'a>>,
}

impl<'seg, 'ref_str: 'seg> CommonJsStringBuilder<'seg> {
    /// Creates a new `CommonJsStringBuilder` with capacity of zero.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Creates a new `CommonJsStringBuilder` with specific capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            segments: Vec::with_capacity(capacity),
        }
    }

    /// Calls the same method of inner vec.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.segments.reserve(additional);
    }

    /// Calls the same method of inner vec.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.segments.reserve_exact(additional);
    }

    /// Appends string segments to the back of the inner vector.
    #[inline]
    pub fn push<T: Into<Segment<'ref_str>>>(&mut self, seg: T) {
        self.segments.push(seg.into());
    }

    /// Checks if all string segments are latin1.
    #[inline]
    #[must_use]
    fn is_latin1(&self) -> bool {
        self.segments.iter().all(Segment::is_latin1)
    }

    /// Returns the number of string segment in inner vector.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Returns true if this `CommonJsStringBuilder` has a length of zero, and false otherwise.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Builds `JsString` from latin1 segments.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_lossless)]
    fn build_from_latin1(self) -> JsString {
        let mut builder = Latin1StringBuilder::new();
        for seg in self.segments {
            match seg {
                Segment::String(s) => {
                    let js_str = s.as_str();
                    let Some(s) = js_str.as_latin1() else {
                        unreachable!("all string segments are checked to be latin1")
                    };
                    builder.extend_from_slice(s);
                }
                Segment::Str(s) => {
                    let Some(s) = s.as_latin1() else {
                        unreachable!("all string segments are checked to be latin1")
                    };
                    builder.extend_from_slice(s);
                }
                Segment::Latin1(latin1) => builder.push(latin1),
                Segment::CodePoint(code_point) => builder.push(code_point as u8),
            }
        }
        builder.build()
    }

    /// Builds `JsString` from utf16 segments.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    fn build_from_utf16(self) -> JsString {
        let mut builder = Utf16StringBuilder::new();
        for seg in self.segments {
            match seg {
                Segment::String(s) => {
                    let js_str = s.as_str();
                    match js_str.variant() {
                        JsStrVariant::Latin1(s) => builder.extend(s.iter().copied().map(u16::from)),
                        JsStrVariant::Utf16(s) => builder.extend_from_slice(s),
                    }
                }
                Segment::Str(s) => match s.variant() {
                    JsStrVariant::Latin1(s) => builder.extend(s.iter().copied().map(u16::from)),
                    JsStrVariant::Utf16(s) => builder.extend_from_slice(s),
                },
                Segment::Latin1(latin1) => builder.push(u16::from(latin1)),
                Segment::CodePoint(code_point) => {
                    // Inline char::encode_utf16 here for better performance
                    let mut code_point = code_point as u32;
                    if code_point < 0x10000 {
                        builder.push(code_point as u16);
                    } else {
                        code_point -= 0x10000;
                        builder.extend_from_slice(&[
                            0xD800 | ((code_point >> 10) as u16),
                            0xDC00 | ((code_point as u16) & 0x3FF),
                        ]);
                    }
                }
            }
        }
        builder.build()
    }

    /// Builds `JsString` from `CommonJsStringBuilder`
    #[inline]
    #[must_use]
    pub fn build(self) -> JsString {
        if self.is_empty() {
            JsString::default()
        } else if self.is_latin1() {
            self.build_from_latin1()
        } else {
            self.build_from_utf16()
        }
    }
}

impl<'ref_str, T: Into<Segment<'ref_str>>> AddAssign<T> for CommonJsStringBuilder<'ref_str> {
    fn add_assign(&mut self, rhs: T) {
        self.push(rhs);
    }
}
