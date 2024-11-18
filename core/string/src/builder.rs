use crate::{
    alloc_overflow, tagged::Tagged, JsStr, JsStrVariant, JsString, RawJsString, RefCount,
    TaggedLen, DATA_OFFSET,
};

use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    cell::Cell,
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Add, AddAssign},
    ptr::{self, addr_of_mut, NonNull},
    str::{self},
};

/// A mutable builder to create instance of `JsString`.
///
#[derive(Debug)]
pub struct JsStringBuilder<D: Copy> {
    cap: usize,
    len: usize,
    inner: NonNull<RawJsString>,
    phantom_data: PhantomData<D>,
}

impl<D: Copy> Default for JsStringBuilder<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: Copy> JsStringBuilder<D> {
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

    /// Returns the number of elements that inner `RawJsString` holds.
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

    /// Returns the allocated byte of inner `RawJsString`'s data.
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

    /// Checks if the inner `RawJsString` is allocated.
    #[must_use]
    fn is_allocated(&self) -> bool {
        self.inner != NonNull::dangling()
    }

    /// Returns the inner `RawJsString`'s layout.
    ///
    /// # Safety
    ///
    /// Caller should ensure that the inner is allocated.
    #[must_use]
    unsafe fn current_layout(&self) -> Layout {
        // SAFETY:
        // Caller should ensure that the inner is allocated.
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
            // SAFETY:
            // Allocation check has been made above.
            let old_layout = unsafe { self.current_layout() };
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

    /// Appends an element to the inner `RawJsString` of `JsStringBuilder`.
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
    ///
    /// [`reserve`]: JsStringBuilder::reserve
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        if additional > self.capacity().wrapping_sub(self.len) {
            let Some(cap) = self.len().checked_add(additional) else {
                alloc_overflow()
            };
            self.allocate_inner(Self::new_layout(cap));
        }
    }

    /// Allocates memory to the inner `RawJsString` by the given capacity.
    /// Capacity calculation is from [`std::vec::Vec::reserve`].
    fn allocate(&mut self, cap: usize) {
        let cap = std::cmp::max(self.capacity() * 2, cap);
        let cap = std::cmp::max(Self::MIN_NON_ZERO_CAP, cap);
        self.allocate_inner(Self::new_layout(cap));
    }

    /// Appends an element to the inner `RawJsString` of `JsStringBuilder` without doing bounds check.
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

    /// Checks if all bytes in inner `RawJsString`'s data are ascii.
    #[inline]
    #[must_use]
    pub fn is_ascii(&self) -> bool {
        // SAFETY:
        // `NonNull` verified for us that the pointer returned by `alloc` is valid,
        // meaning we can read to its pointed memory.
        let data = unsafe {
            std::slice::from_raw_parts(self.data().cast::<u8>(), self.allocated_data_byte_len())
        };
        data.is_ascii()
    }

    /// Extracts a slice containing the elements in the inner `RawJsString`.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[D] {
        if self.is_allocated() {
            // SAFETY:
            // The inner `RawJsString` is allocated which means it is not null.
            unsafe { std::slice::from_raw_parts(self.data(), self.len()) }
        } else {
            &[]
        }
    }

    /// Extracts a mutable slice containing the elements in the inner `RawJsString`.
    ///
    /// # Safety
    /// The caller must ensure that the content of the slice is valid encoding before the borrow ends.
    /// Use of a builder whose contents are not valid encoding is undefined behavior.
    #[inline]
    #[must_use]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [D] {
        if self.is_allocated() {
            // SAFETY:
            // The inner `RawJsString` is allocated which means it is not null.
            unsafe { std::slice::from_raw_parts_mut(self.data(), self.len()) }
        } else {
            &mut []
        }
    }

    /// Builds `JsString` from `JsStringBuilder`
    #[inline]
    #[must_use]
    fn build_inner(mut self, latin1: bool) -> JsString {
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
                tagged_len: TaggedLen::new(len, latin1),
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

impl<D: Copy> Drop for JsStringBuilder<D> {
    /// Set cold since [`JsStringBuilder`] should be created to build `JsString`
    #[cold]
    #[inline]
    fn drop(&mut self) {
        if self.is_allocated() {
            // SAFETY:
            // Allocation check has been made above.
            let layout = unsafe { self.current_layout() };
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

impl<D: Copy> AddAssign<&JsStringBuilder<D>> for JsStringBuilder<D> {
    #[inline]
    fn add_assign(&mut self, rhs: &JsStringBuilder<D>) {
        self.extend_from_slice(rhs.as_slice());
    }
}

impl<D: Copy> AddAssign<&[D]> for JsStringBuilder<D> {
    #[inline]
    fn add_assign(&mut self, rhs: &[D]) {
        self.extend_from_slice(rhs);
    }
}

impl<D: Copy> Add<&JsStringBuilder<D>> for JsStringBuilder<D> {
    type Output = Self;

    #[inline]
    #[must_use]
    fn add(mut self, rhs: &JsStringBuilder<D>) -> Self::Output {
        self.extend_from_slice(rhs.as_slice());
        self
    }
}

impl<D: Copy> Add<&[D]> for JsStringBuilder<D> {
    type Output = Self;

    #[inline]
    #[must_use]
    fn add(mut self, rhs: &[D]) -> Self::Output {
        self.extend_from_slice(rhs);
        self
    }
}

impl<D: Copy> Extend<D> for JsStringBuilder<D> {
    #[inline]
    fn extend<I: IntoIterator<Item = D>>(&mut self, iter: I) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        let require_cap = self.len() + lower_bound;
        self.allocate_if_needed(require_cap);
        iterator.for_each(|c| self.push(c));
    }
}

impl<D: Copy> FromIterator<D> for JsStringBuilder<D> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = D>>(iter: T) -> Self {
        let mut builder = Self::new();
        builder.extend(iter);
        builder
    }
}

impl<D: Copy> From<&[D]> for JsStringBuilder<D> {
    #[inline]
    #[must_use]
    fn from(value: &[D]) -> Self {
        let mut builder = Self::with_capacity(value.len());
        // SAFETY: The capacity is large enough to hold elements.
        unsafe { builder.extend_from_slice_unchecked(value) };
        builder
    }
}

impl<D: Copy + Eq + PartialEq> PartialEq for JsStringBuilder<D> {
    #[inline]
    #[must_use]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<D: Copy> Clone for JsStringBuilder<D> {
    #[inline]
    #[must_use]
    fn clone(&self) -> Self {
        if self.is_allocated() {
            let mut builder = Self::with_capacity(self.capacity());
            // SAFETY: The capacity is large enough to hold elements.
            unsafe { builder.extend_from_slice_unchecked(self.as_slice()) };
            builder
        } else {
            Self::new()
        }
    }

    /// Performs copy-assignment from `source`.
    ///
    /// Rewritten to avoid unnecessary allocation.
    #[inline]
    fn clone_from(&mut self, source: &Self) {
        let source_len = source.len();

        if source_len > self.capacity() {
            self.allocate(source_len);
        } else {
            // At this point, inner `RawJsString` of self or source can be not allocated,
            // returns earlier to avoid copying from/to `null`.
            if source_len == 0 {
                // SAFETY: 0 is always less or equal to self's capacity.
                unsafe { self.set_len(0) };
                return;
            }
        }

        // SAFETY: self shoud be allocated after allocation.
        let self_data = unsafe { self.data() };

        // SAFETY: source_len is greter than 0 so source shoud be allocated.
        let source_data = unsafe { source.data() };

        // SAFETY: Borrow checker should not allow this to be overlapped and pointers are valid.
        unsafe { ptr::copy_nonoverlapping(source_data, self_data, source_len) };

        // SAFETY: source_len has checked to be less or equal to self's capacity.
        unsafe { self.set_len(source_len) };
    }
}

/// **`Latin1`** encoded `JsStringBuilder`
/// # Warning
/// If you are not sure the characters that will be added and don't want to preprocess them,
/// use [`CommonJsStringBuilder`] instead.
/// ## Examples
///
/// ```rust
/// use boa_string::Latin1JsStringBuilder;
/// let mut s = Latin1JsStringBuilder::new();
/// s.push(b'x');
/// s.extend_from_slice(&[b'1', b'2', b'3']);
/// s.extend([b'1', b'2', b'3']);
/// let js_string = s.build();
/// ```
pub type Latin1JsStringBuilder = JsStringBuilder<u8>;

impl Latin1JsStringBuilder {
    /// Builds a `JsString` if the current instance is strictly `ASCII`.
    ///
    /// When the string contains characters outside the `ASCII` range, it cannot be determined
    /// whether the encoding is `Latin1` or others. Therefore, this method only returns a
    /// valid `JsString` when the instance is entirely `ASCII`. If any non-`ASCII` characters
    /// are present, it returns `None` to avoid ambiguity in encoding.
    ///
    /// If the caller is certain that the string is encoded in `Latin1`,
    /// [`build_as_latin1`](Self::build_as_latin1) can be used to avoid the `ASCII` check.
    #[inline]
    #[must_use]
    pub fn build(self) -> Option<JsString> {
        if self.is_ascii() {
            Some(self.build_inner(true))
        } else {
            None
        }
    }

    /// Builds `JsString` from `Latin1JsStringBuilder`, assume that the inner data is `Latin1` encoded
    ///
    /// # Safety
    /// Caller must ensure that the string is encoded in `Latin1`.
    ///
    /// If the string contains characters outside the `Latin1` range, it may lead to encoding errors,
    /// resulting in an incorrect or malformed `JsString`. This could cause undefined behavior
    /// when the resulting string is used in further operations or when interfacing with other
    /// parts of the system that expect valid `Latin1` encoded string.
    #[inline]
    #[must_use]
    pub unsafe fn build_as_latin1(self) -> JsString {
        self.build_inner(true)
    }
}

/// **`UTF-16`** encoded `JsStringBuilder`
/// ## Examples
///
/// ```rust
/// use boa_string::Utf16JsStringBuilder;
/// let mut s = Utf16JsStringBuilder::new();
/// s.push(b'x' as u16);
/// s.extend_from_slice(&[b'1', b'2', b'3'].map(u16::from));
/// s.extend([0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5,]); // 🎹🎶🎵
/// let js_string = s.build();
/// ```
pub type Utf16JsStringBuilder = JsStringBuilder<u16>;

impl Utf16JsStringBuilder {
    /// Builds `JsString` from `Utf16JsStringBuilder`
    #[inline]
    #[must_use]
    pub fn build(self) -> JsString {
        self.build_inner(false)
    }
}

/// Represents a segment of a string used to construct a [`JsString`].
#[derive(Clone, Debug)]
pub enum Segment<'a> {
    /// A string segment represented as a `JsString`.
    String(JsString),

    /// A string segment represented as a `JsStr`.
    Str(JsStr<'a>),

    /// A string segment represented as a byte.
    Latin1(u8),

    /// A Unicode code point segment represented as a character.
    CodePoint(char),
}

impl Segment<'_> {
    /// Checks if the segment consists solely of `ASCII` characters.
    #[inline]
    #[must_use]
    fn is_ascii(&self) -> bool {
        match self {
            Segment::String(s) => s.as_str().is_latin1(),
            Segment::Str(s) => s.is_latin1(),
            Segment::Latin1(b) => *b <= 0x7f,
            Segment::CodePoint(ch) => *ch as u32 <= 0x7F,
        }
    }
}

impl From<JsString> for Segment<'_> {
    #[inline]
    fn from(value: JsString) -> Self {
        Self::String(value)
    }
}

impl From<String> for Segment<'_> {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl From<&[u16]> for Segment<'_> {
    #[inline]
    fn from(value: &[u16]) -> Self {
        Self::String(value.into())
    }
}

impl From<&str> for Segment<'_> {
    #[inline]
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl<'seg, 'ref_str: 'seg> From<JsStr<'ref_str>> for Segment<'seg> {
    #[inline]
    fn from(value: JsStr<'ref_str>) -> Self {
        Self::Str(value)
    }
}

impl From<u8> for Segment<'_> {
    #[inline]
    fn from(value: u8) -> Self {
        Self::Latin1(value)
    }
}

impl From<char> for Segment<'_> {
    #[inline]
    fn from(value: char) -> Self {
        Self::CodePoint(value)
    }
}

/// Common `JsString` builder that accepts multiple variant of string or character.
///
/// Originally based on [kiesel-js](https://codeberg.org/kiesel-js/kiesel/src/branch/main/src/types/language/String/Builder.zig)
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

    /// Similar to `Vec::with_capacity`.
    ///
    /// Creates a new `CommonJsStringBuilder` with given capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            segments: Vec::with_capacity(capacity),
        }
    }

    /// Similar to `Vec::reserve`.
    ///
    /// Reserves additional capacity for the inner vector.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.segments.reserve(additional);
    }

    /// Similar to `Vec::reserve_exact`.
    ///
    /// Reserves the minimum capacity for the inner vector.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.segments.reserve_exact(additional);
    }

    /// Appends string segments to the back of the inner vector.
    #[inline]
    pub fn push<T: Into<Segment<'ref_str>>>(&mut self, seg: T) {
        self.segments.push(seg.into());
    }

    /// Checks if all string segments contains only `ASCII` bytes.
    #[inline]
    #[must_use]
    pub fn is_ascii(&self) -> bool {
        self.segments.iter().all(Segment::is_ascii)
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

    /// Builds `Latin1` encoded `JsString` from string segments.
    ///
    /// This doesn't consume the builder itself because it may fails to build
    /// and the caller may wants to keep the builder for further operations.
    ///
    /// This processes the following types of segments:
    ///
    /// - `Segment::String(s)`: Encodes the string if it can be represented in `Latin1`.
    /// - `Segment::Str(s)`: Encodes the string slice if it can be represented in `Latin1`.
    /// - `Segment::Latin1(b)`: Encodes the byte if it's within the `ASCII` range.
    /// - `Segment::CodePoint(ch)`: Encodes the code point by converting it to a byte if it's within the `ASCII` range.
    ///
    /// Return `None` if any segment fails to encode.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_lossless)]
    pub fn build_from_latin1(&self) -> Option<JsString> {
        let mut builder = Latin1JsStringBuilder::new();
        for seg in &self.segments {
            match seg {
                Segment::String(s) => {
                    if let Some(data) = s.as_str().as_latin1() {
                        builder.extend_from_slice(data);
                    } else {
                        return None;
                    }
                }
                Segment::Str(s) => {
                    if let Some(data) = s.as_latin1() {
                        builder.extend_from_slice(data);
                    } else {
                        return None;
                    }
                }
                Segment::Latin1(b) => {
                    if *b <= 0x7f {
                        builder.push(*b);
                    } else {
                        return None;
                    }
                }
                Segment::CodePoint(ch) => {
                    if let Ok(b) = u8::try_from(*ch as u32) {
                        builder.push(b);
                    } else {
                        return None;
                    }
                }
            }
        }
        builder.build()
    }

    /// Builds `Utf-16` encoded `JsString` from string segments.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn build_from_utf16(self) -> JsString {
        let mut builder = Utf16JsStringBuilder::new();
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
                    builder.extend_from_slice(code_point.encode_utf16(&mut [0_u16; 2]));
                }
            }
        }
        builder.build()
    }

    /// Builds `JsString` from `CommonJsStringBuilder`,
    ///
    /// This function first checks if the instance is empty:
    /// - If it is empty, it returns the default `JsString`.
    /// - If it contains only ASCII characters, it safely encodes it as `Latin1`.
    /// - If it contains non-ASCII characters, it falls back to encoding using `UTF-16`.
    #[inline]
    #[must_use]
    pub fn build(self) -> JsString {
        if self.is_empty() {
            JsString::default()
        } else if self.is_ascii() {
            // SAFETY:
            // All string segment contains only ascii byte, so this can be encoded as `Latin1`.
            unsafe { self.build_as_latin1() }
        } else {
            self.build_from_utf16()
        }
    }

    /// Builds `Latin1` encoded `JsString` from `CommonJsStringBuilder`, return `None` if segments can't be encoded as `Latin1`
    ///
    /// # Safety
    /// Caller must ensure that the string segments can be `Latin1` encoded.
    ///
    /// If string segments can't be `Latin1` encoded, it may lead to encoding errors,
    /// resulting in an incorrect or malformed `JsString`. This could cause undefined behavior
    /// when the resulting string is used in further operations or when interfacing with other
    /// parts of the system that expect valid `Latin1` encoded string.
    #[inline]
    #[must_use]
    pub unsafe fn build_as_latin1(self) -> JsString {
        let mut builder = Latin1JsStringBuilder::new();
        for seg in self.segments {
            match seg {
                Segment::String(s) => {
                    let js_str = s.as_str();
                    let Some(s) = js_str.as_latin1() else {
                        unreachable!("string segment shoud be latin1")
                    };
                    builder.extend_from_slice(s);
                }
                Segment::Str(s) => {
                    let Some(s) = s.as_latin1() else {
                        unreachable!("string segment shoud be latin1")
                    };
                    builder.extend_from_slice(s);
                }
                Segment::Latin1(latin1) => builder.push(latin1),
                Segment::CodePoint(code_point) => builder.push(code_point as u8),
            }
        }
        // SAFETY: All string segments can be encoded as `Latin1` string.
        unsafe { builder.build_as_latin1() }
    }
}

impl<'ref_str, T: Into<Segment<'ref_str>>> AddAssign<T> for CommonJsStringBuilder<'ref_str> {
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        self.push(rhs);
    }
}

impl<'ref_str, T: Into<Segment<'ref_str>>> Add<T> for CommonJsStringBuilder<'ref_str> {
    type Output = Self;

    #[inline]
    #[must_use]
    fn add(mut self, rhs: T) -> Self::Output {
        self.push(rhs);
        self
    }
}
