use crate::{trace::Trace, Gc, GcBox};
use std::{
    cell::Cell,
    ptr::{self, NonNull},
};

/// The `EphemeronBoxHeader` contains the `EphemeronBoxHeader`'s current state for the `Collector`'s
/// Mark/Sweep as well as a pointer to the next ephemeron in the heap.
///
/// These flags include:
///  - Root Count
///  - Mark Flag Bit
///
/// The next node is set by the `Allocator` during initialization and by the
/// `Collector` during the sweep phase.
pub(crate) struct EphemeronBoxHeader {
    marked: Cell<bool>,
    ref_count: Cell<u32>,
    non_root_count: Cell<u32>,
    pub(crate) next: Cell<Option<NonNull<dyn ErasedEphemeronBox>>>,
}

impl EphemeronBoxHeader {
    /// Creates a new `EphemeronBoxHeader` with a root of 1 and next set to None.
    pub(crate) fn new() -> Self {
        Self {
            marked: Cell::new(false),
            ref_count: Cell::new(1),
            non_root_count: Cell::new(0),
            next: Cell::new(None),
        }
    }

    /// Returns a bool for whether `EphemeronBoxHeader`'s mark bit is 1.
    pub(crate) fn is_marked(&self) -> bool {
        self.marked.get()
    }

    /// Sets `EphemeronBoxHeader`'s mark bit to 1.
    pub(crate) fn mark(&self) {
        self.marked.set(true);
    }

    /// Sets `EphemeronBoxHeader`'s mark bit to 0.
    pub(crate) fn unmark(&self) {
        self.marked.set(false);
    }
}

impl core::fmt::Debug for EphemeronBoxHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EphemeronBoxHeader")
            .field("marked", &self.is_marked())
            .field("ref_count", &self.ref_count.get())
            .field("non_root_count", &self.non_root_count.get())
            .finish()
    }
}

/// The inner allocation of an [`Ephemeron`][crate::Ephemeron] pointer.
pub(crate) struct EphemeronBox<K: Trace + ?Sized + 'static, V: Trace + 'static> {
    pub(crate) header: EphemeronBoxHeader,
    data: Cell<Option<NonNull<Data<K, V>>>>,
}

impl<K: Trace + ?Sized + 'static, V: Trace + 'static> Drop for EphemeronBox<K, V> {
    fn drop(&mut self) {
        if let Some(data) = self.data.take() {
            // SAFETY: `data` comes from an `into_raw` call, so this pointer is safe to pass to
            // `from_raw`.
            drop(unsafe { Box::from_raw(data.as_ptr()) });
        }
    }
}

struct Data<K: Trace + ?Sized + 'static, V: Trace + 'static> {
    key: NonNull<GcBox<K>>,
    value: V,
}

impl<K: Trace + ?Sized, V: Trace> EphemeronBox<K, V> {
    pub(crate) fn new(key: &Gc<K>, value: V) -> Self {
        let data = Box::into_raw(Box::new(Data {
            key: key.inner_ptr(),
            value,
        }));
        // SAFETY: `Box::into_raw` must always return a non-null pointer.
        let data = unsafe { NonNull::new_unchecked(data) };
        Self {
            header: EphemeronBoxHeader::new(),
            data: Cell::new(Some(data)),
        }
    }

    /// Returns `true` if the two references refer to the same `GcBox`.
    pub(crate) fn ptr_eq(this: &Self, other: &Self) -> bool {
        // Use .header to ignore fat pointer vtables, to work around
        // https://github.com/rust-lang/rust/issues/46139
        ptr::eq(&this.header, &other.header)
    }

    /// Returns a reference to the ephemeron's value or None.
    pub(crate) fn value(&self) -> Option<&V> {
        // SAFETY: the garbage collector ensures `ptr` is valid as long as `data` is `Some`.
        unsafe { self.data.get().map(|ptr| &ptr.as_ref().value) }
    }

    /// Marks this `EphemeronBox` as live.
    ///
    /// This doesn't mark the inner value of the ephemeron. [`ErasedEphemeronBox::trace`]
    /// does this, and it's called by the garbage collector on demand.
    pub(crate) unsafe fn mark(&self) {
        self.header.mark();
    }

    #[inline]
    pub(crate) fn inc_ref_count(&self) {
        self.header.ref_count.set(self.header.ref_count.get() + 1);
    }

    #[inline]
    pub(crate) fn dec_ref_count(&self) {
        self.header.ref_count.set(self.header.ref_count.get() - 1);
    }

    #[inline]
    pub(crate) fn inc_non_root_count(&self) {
        self.header
            .non_root_count
            .set(self.header.non_root_count.get() + 1);
    }
}

pub(crate) trait ErasedEphemeronBox {
    /// Gets the header of the `EphemeronBox`.
    fn header(&self) -> &EphemeronBoxHeader;

    /// Traces through the `EphemeronBox`'s held value, but only if it's marked and its key is also
    /// marked. Returns `true` if the ephemeron successfuly traced through its value. This also
    /// considers ephemerons that are marked but don't have their value anymore as
    /// "successfully traced".
    unsafe fn trace(&self) -> bool;

    fn trace_non_roots(&self);

    fn get_ref_count(&self) -> u32;

    fn get_non_root_count(&self) -> u32;

    fn reset_non_root_count(&self);

    /// Runs the finalization logic of the `EphemeronBox`'s held value, if the key is still live,
    /// and clears its contents.
    fn finalize_and_clear(&self);
}

impl<K: Trace + ?Sized, V: Trace> ErasedEphemeronBox for EphemeronBox<K, V> {
    fn header(&self) -> &EphemeronBoxHeader {
        &self.header
    }

    unsafe fn trace(&self) -> bool {
        if !self.header.is_marked() {
            return false;
        }

        let Some(data) = self.data.get() else {
            return true;
        };

        // SAFETY: `data` comes from a `Box`, so it is safe to dereference.
        let data = unsafe { data.as_ref() };
        // SAFETY: `key` comes from a `Gc`, and the garbage collector only invalidates
        // `key` when it is unreachable, making `key` always valid.
        let key = unsafe { data.key.as_ref() };

        let is_key_marked = key.is_marked();

        if is_key_marked {
            // SAFETY: this is safe to call, since we want to trace all reachable objects
            // from a marked ephemeron that holds a live `key`.
            unsafe { data.value.trace() }
        }

        is_key_marked
    }

    fn trace_non_roots(&self) {
        let Some(data) = self.data.get() else {
            return;
        };
        // SAFETY: `data` comes from a `Box`, so it is safe to dereference.
        unsafe {
            data.as_ref().value.trace_non_roots();
        };
    }

    #[inline]
    fn get_ref_count(&self) -> u32 {
        self.header.ref_count.get()
    }

    #[inline]
    fn get_non_root_count(&self) -> u32 {
        self.header.non_root_count.get()
    }

    #[inline]
    fn reset_non_root_count(&self) {
        self.header.non_root_count.set(0);
    }

    fn finalize_and_clear(&self) {
        if let Some(data) = self.data.take() {
            // SAFETY: `data` comes from an `into_raw` call, so this pointer is safe to pass to
            // `from_raw`.
            let _contents = unsafe { Box::from_raw(data.as_ptr()) };
        }
    }
}
