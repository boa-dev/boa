use crate::Trace;

use super::{vtable_of, DropFn, GcHeader, RunFinalizerFn, TraceFn, TraceNonRootsFn, VTable};

/// A garbage collected allocation.
#[derive(Debug)]
#[repr(C)]
pub struct GcBox<T: Trace + ?Sized + 'static> {
    pub(crate) header: GcHeader,
    pub(crate) vtable: &'static VTable,
    value: T,
}

impl<T: Trace> GcBox<T> {
    /// Returns a new `GcBox` with a rooted `GcBoxHeader`.
    pub(crate) fn new(value: T) -> Self {
        let vtable = vtable_of::<T>();
        Self {
            header: GcHeader::new(),
            vtable,
            value,
        }
    }
}

impl<T: Trace + ?Sized> GcBox<T> {
    /// Returns a reference to the `GcBox`'s value.
    pub(crate) const fn value(&self) -> &T {
        &self.value
    }

    /// Returns `true` if the header is marked.
    pub(crate) fn is_marked(&self) -> bool {
        self.header.is_marked()
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

    pub(crate) fn reset_non_root_count(&self) {
        self.header.reset_non_root_count();
    }

    /// Check if the gc object is rooted.
    ///
    /// # Note
    ///
    /// This only gives valid result if the we have run through the
    /// tracing non roots phase.
    pub(crate) fn is_rooted(&self) -> bool {
        self.header.is_rooted()
    }

    pub(crate) fn trace_fn(&self) -> TraceFn {
        self.vtable.trace_fn()
    }

    pub(crate) fn trace_non_roots_fn(&self) -> TraceNonRootsFn {
        self.vtable.trace_non_roots_fn()
    }

    pub(crate) fn run_finalizer_fn(&self) -> RunFinalizerFn {
        self.vtable.run_finalizer_fn()
    }

    pub(crate) fn drop_fn(&self) -> DropFn {
        self.vtable.drop_fn()
    }

    pub(crate) fn size(&self) -> usize {
        self.vtable.size()
    }
}
