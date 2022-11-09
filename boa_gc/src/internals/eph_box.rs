use crate::trace::Trace;
use crate::{finalizer_safe, GcBox};
use crate::{Finalize, Gc};
use std::cell::Cell;
use std::ptr::NonNull;

/// The inner allocation of an [`Ephemeron`][crate::Ephemeron] pointer.
pub(crate) struct EphemeronBox<K: Trace + ?Sized + 'static, V: Trace + ?Sized + 'static> {
    key: Cell<Option<NonNull<GcBox<K>>>>,
    value: V,
}

impl<K: Trace + ?Sized, V: Trace> EphemeronBox<K, V> {
    pub(crate) fn new(key: &Gc<K>, value: V) -> Self {
        EphemeronBox {
            key: Cell::new(Some(key.inner_ptr())),
            value,
        }
    }
}

impl<K: Trace + ?Sized, V: Trace + ?Sized> EphemeronBox<K, V> {
    #[inline]
    pub(crate) fn is_marked(&self) -> bool {
        if let Some(key) = self.inner_key() {
            key.is_marked()
        } else {
            false
        }
    }

    #[inline]
    fn inner_key_ptr(&self) -> Option<*mut GcBox<K>> {
        assert!(finalizer_safe());
        self.key.get().map(NonNull::as_ptr)
    }

    #[inline]
    fn inner_key(&self) -> Option<&GcBox<K>> {
        unsafe {
            if let Some(inner_key) = self.inner_key_ptr() {
                Some(&*inner_key)
            } else {
                None
            }
        }
    }

    #[inline]
    pub(crate) fn key(&self) -> Option<&K> {
        if let Some(key_box) = self.inner_key() {
            Some(key_box.value())
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn value(&self) -> &V {
        &self.value
    }

    #[inline]
    unsafe fn weak_trace_key(&self) {
        if let Some(key) = self.inner_key() {
            key.weak_trace_inner();
        }
    }

    #[inline]
    unsafe fn weak_trace_value(&self) {
        self.value().weak_trace();
    }
}

impl<K: Trace + ?Sized, V: Trace + ?Sized> Finalize for EphemeronBox<K, V> {
    #[inline]
    fn finalize(&self) {
        self.key.set(None);
    }
}

unsafe impl<K: Trace + ?Sized, V: Trace + ?Sized> Trace for EphemeronBox<K, V> {
    #[inline]
    unsafe fn trace(&self) {
        /* An ephemeron is never traced with Phase One Trace */
        /* May be traced in phase 3, so this still may need to be implemented */
    }

    #[inline]
    unsafe fn is_marked_ephemeron(&self) -> bool {
        self.is_marked()
    }

    #[inline]
    unsafe fn weak_trace(&self) {
        if self.is_marked() {
            self.weak_trace_key();
            self.weak_trace_value();
        }
    }

    #[inline]
    unsafe fn root(&self) {
        // An ephemeron here should probably not be rooted.
    }

    #[inline]
    unsafe fn unroot(&self) {
        // An ephemeron is never rooted in the GcBoxHeader
    }

    #[inline]
    fn run_finalizer(&self) {
        Finalize::finalize(self);
    }
}
