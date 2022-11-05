//! This module will implement the internal types GcBox and Ephemeron
use crate::trace::Trace;
use crate::Finalize;
use crate::{finalizer_safe, GcBox};
use std::cell::Cell;
use std::ptr::NonNull;

/// Implementation of an Ephemeron structure
pub struct Ephemeron<K: Trace + ?Sized + 'static, V: Trace + ?Sized + 'static> {
    key: Cell<Option<NonNull<GcBox<K>>>>,
    value: V,
}

impl<K: Trace + ?Sized> Ephemeron<K, ()> {
    pub unsafe fn new(value: NonNull<GcBox<K>>) -> Self {
        let ptr = NonNull::new_unchecked(value.as_ptr());
        Ephemeron {
            key: Cell::new(Some(ptr)),
            value: (),
        }
    }
}

impl<K: Trace, V: Trace> Ephemeron<K, V> {
    pub unsafe fn new_pair(key: NonNull<GcBox<K>>, value: V) -> Self {
        let ptr = NonNull::new_unchecked(key.as_ptr());

        Ephemeron {
            key: Cell::new(Some(ptr)),
            value,
        }
    }
}

impl<K: Trace + ?Sized, V: Trace + ?Sized> Ephemeron<K, V> {
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
        self.key.get().map(|key_node| key_node.as_ptr())
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
    pub fn key(&self) -> Option<&K> {
        if let Some(key_box) = self.inner_key() {
            Some(key_box.value())
        } else {
            None
        }
    }

    #[inline]
    pub fn value(&self) -> &V {
        &self.value
    }

    #[inline]
    unsafe fn weak_trace_key(&self) {
        if let Some(key) = self.inner_key() {
            key.weak_trace_inner()
        }
    }

    #[inline]
    unsafe fn weak_trace_value(&self) {
        self.value().weak_trace()
    }
}

impl<K: Trace + ?Sized, V: Trace + ?Sized> Finalize for Ephemeron<K, V> {
    #[inline]
    fn finalize(&self) {
        self.key.set(None)
    }
}

unsafe impl<K: Trace + ?Sized, V: Trace + ?Sized> Trace for Ephemeron<K, V> {
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
        // An ephemeron is never rotted in the GcBoxHeader
    }

    #[inline]
    fn run_finalizer(&self) {
        Finalize::finalize(self)
    }
}
