use crate::{trace::Trace, Gc, GcBox, Tracer};
use std::{
    cell::UnsafeCell,
    ptr::{self, NonNull},
};

use super::GcHeader;

/// The inner allocation of an [`Ephemeron`][crate::Ephemeron] pointer.
pub(crate) struct EphemeronBox<K: Trace + ?Sized + 'static, V: Trace + 'static> {
    pub(crate) header: GcHeader,
    data: UnsafeCell<Option<Data<K, V>>>,
}

struct Data<K: Trace + ?Sized + 'static, V: Trace + 'static> {
    key: NonNull<GcBox<K>>,
    value: V,
}

impl<K: Trace + ?Sized, V: Trace> EphemeronBox<K, V> {
    /// Creates a new `EphemeronBox` that tracks `key` and has `value` as its inner data.
    pub(crate) fn new(key: &Gc<K>, value: V) -> Self {
        Self {
            header: GcHeader::new(),
            data: UnsafeCell::new(Some(Data {
                key: key.inner_ptr(),
                value,
            })),
        }
    }

    /// Creates a new `EphemeronBox` with its inner data in the invalidated state.
    pub(crate) fn new_empty() -> Self {
        Self {
            header: GcHeader::new(),
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

    /// Returns the pointer to the ephemeron's key or None.
    ///
    /// # Safety
    ///
    /// The caller must ensure there are no live mutable references to the ephemeron box's data
    /// before calling this method.
    pub(crate) unsafe fn key_ptr(&self) -> Option<NonNull<GcBox<K>>> {
        // SAFETY: the garbage collector ensures the ephemeron doesn't mutate until
        // finalization.
        unsafe {
            let data = &*self.data.get();
            data.as_ref().map(|data| data.key)
        }
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
        unsafe { self.key_ptr().map(|data| data.as_ref()) }
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
        self.header.inc_ref_count();
    }

    #[inline]
    pub(crate) fn dec_ref_count(&self) {
        self.header.dec_ref_count();
    }

    #[inline]
    pub(crate) fn inc_non_root_count(&self) {
        self.header.inc_non_root_count();
    }
}

pub(crate) trait ErasedEphemeronBox {
    /// Gets the header of the `EphemeronBox`.
    fn header(&self) -> &GcHeader;

    /// Traces through the `EphemeronBox`'s held value, but only if it's marked and its key is also
    /// marked. Returns `true` if the ephemeron successfuly traced through its value. This also
    /// considers ephemerons that are marked but don't have their value anymore as
    /// "successfully traced".
    unsafe fn trace(&self, tracer: &mut Tracer) -> bool;

    fn trace_non_roots(&self);

    /// Runs the finalization logic of the `EphemeronBox`'s held value, if the key is still live,
    /// and clears its contents.
    fn finalize_and_clear(&self);
}

impl<K: Trace + ?Sized, V: Trace> ErasedEphemeronBox for EphemeronBox<K, V> {
    fn header(&self) -> &GcHeader {
        &self.header
    }

    unsafe fn trace(&self, tracer: &mut Tracer) -> bool {
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
            unsafe { data.value.trace(tracer) }
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
