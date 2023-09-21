use crate::{trace::Trace, Gc, GcBox};
use std::{
    cell::{Cell, UnsafeCell},
    ptr::{self, NonNull},
};

const MARK_MASK: u32 = 1 << (u32::BITS - 1);
const NON_ROOTS_MASK: u32 = !MARK_MASK;
const NON_ROOTS_MAX: u32 = NON_ROOTS_MASK;

/// The `EphemeronBoxHeader` contains the `EphemeronBoxHeader`'s current state for the `Collector`'s
/// Mark/Sweep as well as a pointer to the next ephemeron in the heap.
///
/// `ref_count` is the number of Gc instances, and `non_root_count` is the number of
/// Gc instances in the heap. `non_root_count` also includes Mark Flag bit.
///
/// The next node is set by the `Allocator` during initialization and by the
/// `Collector` during the sweep phase.
pub(crate) struct EphemeronBoxHeader {
    ref_count: Cell<u32>,
    non_root_count: Cell<u32>,
    pub(crate) next: Cell<Option<NonNull<dyn ErasedEphemeronBox>>>,
}

impl EphemeronBoxHeader {
    /// Creates a new `EphemeronBoxHeader` with a root of 1 and next set to None.
    pub(crate) fn new() -> Self {
        Self {
            ref_count: Cell::new(1),
            non_root_count: Cell::new(0),
            next: Cell::new(None),
        }
    }

    /// Returns the `EphemeronBoxHeader`'s current ref count
    pub(crate) fn get_ref_count(&self) -> u32 {
        self.ref_count.get()
    }

    /// Returns a count for non-roots.
    pub(crate) fn get_non_root_count(&self) -> u32 {
        self.non_root_count.get() & NON_ROOTS_MASK
    }

    /// Increments `EphemeronBoxHeader`'s non-roots count.
    pub(crate) fn inc_non_root_count(&self) {
        let non_root_count = self.non_root_count.get();

        if (non_root_count & NON_ROOTS_MASK) < NON_ROOTS_MAX {
            self.non_root_count.set(non_root_count.wrapping_add(1));
        } else {
            // TODO: implement a better way to handle root overload.
            panic!("non roots counter overflow");
        }
    }

    /// Reset non-roots count to zero.
    pub(crate) fn reset_non_root_count(&self) {
        self.non_root_count
            .set(self.non_root_count.get() & !NON_ROOTS_MASK);
    }

    /// Returns a bool for whether `GcBoxHeader`'s mark bit is 1.
    pub(crate) fn is_marked(&self) -> bool {
        self.non_root_count.get() & MARK_MASK != 0
    }

    /// Sets `GcBoxHeader`'s mark bit to 1.
    pub(crate) fn mark(&self) {
        self.non_root_count
            .set(self.non_root_count.get() | MARK_MASK);
    }

    /// Sets `GcBoxHeader`'s mark bit to 0.
    pub(crate) fn unmark(&self) {
        self.non_root_count
            .set(self.non_root_count.get() & !MARK_MASK);
    }
}

impl core::fmt::Debug for EphemeronBoxHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EphemeronBoxHeader")
            .field("marked", &self.is_marked())
            .field("ref_count", &self.get_ref_count())
            .field("non_root_count", &self.get_non_root_count())
            .finish()
    }
}

/// The inner allocation of an [`Ephemeron`][crate::Ephemeron] pointer.
pub(crate) struct EphemeronBox<K: Trace + 'static, V: Trace + 'static> {
    pub(crate) header: EphemeronBoxHeader,
    data: UnsafeCell<Option<Data<K, V>>>,
}

struct Data<K: Trace + 'static, V: Trace + 'static> {
    key: NonNull<GcBox<K>>,
    value: V,
}

impl<K: Trace, V: Trace> EphemeronBox<K, V> {
    /// Creates a new `EphemeronBox` that tracks `key` and has `value` as its inner data.
    pub(crate) fn new(key: &Gc<K>, value: V) -> Self {
        Self {
            header: EphemeronBoxHeader::new(),
            data: UnsafeCell::new(Some(Data {
                key: key.inner_ptr(),
                value,
            })),
        }
    }

    /// Creates a new `EphemeronBox` with its inner data in the invalidated state.
    pub(crate) fn new_empty() -> Self {
        Self {
            header: EphemeronBoxHeader::new(),
            data: UnsafeCell::new(None),
        }
    }

    /// Returns `true` if the two references refer to the same `EphemeronBox`.
    pub(crate) fn ptr_eq(this: &Self, other: &Self) -> bool {
        // Use .header to ignore fat pointer vtables, to work around
        // https://github.com/rust-lang/rust/issues/46139
        ptr::eq(&this.header, &other.header)
    }

    /// Returns a reference to the ephemeron's value or None.
    ///
    /// # Safety
    ///
    /// The caller must ensure there are no live mutable references to the ephemeron box's data
    /// before calling this method.
    pub(crate) unsafe fn value(&self) -> Option<&V> {
        // SAFETY: the garbage collector ensures the ephemeron doesn't mutate until
        // finalization.
        let data = unsafe { &*self.data.get() };
        data.as_ref().map(|data| &data.value)
    }

    /// Returns a reference to the ephemeron's key or None.
    ///
    /// # Safety
    ///
    /// The caller must ensure there are no live mutable references to the ephemeron box's data
    /// before calling this method.
    pub(crate) unsafe fn key(&self) -> Option<&GcBox<K>> {
        // SAFETY: the garbage collector ensures the ephemeron doesn't mutate until
        // finalization.
        unsafe {
            let data = &*self.data.get();
            data.as_ref().map(|data| data.key.as_ref())
        }
    }

    /// Marks this `EphemeronBox` as live.
    ///
    /// This doesn't mark the inner value of the ephemeron. [`ErasedEphemeronBox::trace`]
    /// does this, and it's called by the garbage collector on demand.
    pub(crate) unsafe fn mark(&self) {
        self.header.mark();
    }

    /// Sets the inner data of the `EphemeronBox` to the specified key and value.
    ///
    /// # Safety
    ///
    /// The caller must ensure there are no live mutable references to the ephemeron box's data
    /// before calling this method.
    pub(crate) unsafe fn set(&self, key: &Gc<K>, value: V) {
        // SAFETY: The caller must ensure setting the key and value of the ephemeron box is safe.
        unsafe {
            *self.data.get() = Some(Data {
                key: key.inner_ptr(),
                value,
            });
        }
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
        self.header.inc_non_root_count();
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

    /// Runs the finalization logic of the `EphemeronBox`'s held value, if the key is still live,
    /// and clears its contents.
    fn finalize_and_clear(&self);
}

impl<K: Trace, V: Trace> ErasedEphemeronBox for EphemeronBox<K, V> {
    fn header(&self) -> &EphemeronBoxHeader {
        &self.header
    }

    unsafe fn trace(&self) -> bool {
        if !self.header.is_marked() {
            return false;
        }

        // SAFETY: the garbage collector ensures the ephemeron doesn't mutate until
        // finalization.
        let data = unsafe { &*self.data.get() };
        let Some(data) = data.as_ref() else {
            return true;
        };

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
        // SAFETY: Tracing always executes before collecting, meaning this cannot cause
        // use after free.
        unsafe {
            if let Some(value) = self.value() {
                value.trace_non_roots();
            }
        }
    }

    fn finalize_and_clear(&self) {
        // SAFETY: the invariants of the garbage collector ensures this is only executed when
        // there are no remaining references to the inner data.
        unsafe { (*self.data.get()).take() };
    }
}
