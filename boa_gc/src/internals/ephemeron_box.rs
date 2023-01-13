use crate::{trace::Trace, Gc, GcBox};
use std::{
    cell::Cell,
    ptr::{self, NonNull},
};

// Age and Weak Flags
const MARK_MASK: usize = 1 << (usize::BITS - 1);
const ROOTS_MASK: usize = !MARK_MASK;
const ROOTS_MAX: usize = ROOTS_MASK;

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
    roots: Cell<usize>,
    pub(crate) next: Cell<Option<NonNull<dyn ErasedEphemeronBox>>>,
}

impl EphemeronBoxHeader {
    /// Creates a new `EphemeronBoxHeader` with a root of 1 and next set to None.
    pub(crate) fn new() -> Self {
        Self {
            roots: Cell::new(1),
            next: Cell::new(None),
        }
    }

    /// Returns the `EphemeronBoxHeader`'s current root count
    pub(crate) fn roots(&self) -> usize {
        self.roots.get() & ROOTS_MASK
    }

    /// Increments `EphemeronBoxHeader`'s root count.
    pub(crate) fn inc_roots(&self) {
        let roots = self.roots.get();

        if (roots & ROOTS_MASK) < ROOTS_MAX {
            self.roots.set(roots + 1);
        } else {
            // TODO: implement a better way to handle root overload.
            panic!("roots counter overflow");
        }
    }

    /// Decreases `EphemeronBoxHeader`'s current root count.
    pub(crate) fn dec_roots(&self) {
        // Underflow check as a stop gap for current issue when dropping.
        if self.roots.get() > 0 {
            self.roots.set(self.roots.get() - 1);
        }
    }

    /// Returns a bool for whether `EphemeronBoxHeader`'s mark bit is 1.
    pub(crate) fn is_marked(&self) -> bool {
        self.roots.get() & MARK_MASK != 0
    }

    /// Sets `EphemeronBoxHeader`'s mark bit to 1.
    pub(crate) fn mark(&self) {
        self.roots.set(self.roots.get() | MARK_MASK);
    }

    /// Sets `EphemeronBoxHeader`'s mark bit to 0.
    pub(crate) fn unmark(&self) {
        self.roots.set(self.roots.get() & !MARK_MASK);
    }
}

impl core::fmt::Debug for EphemeronBoxHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EphemeronBoxHeader")
            .field("roots", &self.roots())
            .field("marked", &self.is_marked())
            .finish()
    }
}

/// The inner allocation of an [`Ephemeron`][crate::Ephemeron] pointer.
pub(crate) struct EphemeronBox<K: Trace + ?Sized + 'static, V: Trace + 'static> {
    pub(crate) header: EphemeronBoxHeader,
    data: Cell<Option<NonNull<Data<K, V>>>>,
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

    /// Increases the root count on this `EphemeronBox`.
    ///
    /// Roots prevent the `EphemeronBox` from being destroyed by the garbage collector.
    pub(crate) fn root(&self) {
        self.header.inc_roots();
    }

    /// Decreases the root count on this `EphemeronBox`.
    ///
    /// Roots prevent the `EphemeronBox` from being destroyed by the garbage collector.
    pub(crate) fn unroot(&self) {
        self.header.dec_roots();
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

    fn finalize_and_clear(&self) {
        if let Some(data) = self.data.take() {
            // SAFETY: `data` comes from an `into_raw` call, so this pointer is safe to pass to
            // `from_raw`.
            let contents = unsafe { Box::from_raw(data.as_ptr()) };
            contents.value.finalize();
        }
    }
}
