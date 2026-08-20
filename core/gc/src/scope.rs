use std::cell::RefCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::{Gc, Trace, Tracer};

/// A type-erased local root for tracing.
#[derive(Clone, Copy)]
pub(crate) struct ErasedRoot {
    /// The erased `PoolPointer` (`NonNull<()>`) from a `Gc<'_, T>`.
    ptr: NonNull<()>,
    /// A function that casts the pointer back to `Gc<'_, T>` and marks it.
    trace_fn: unsafe fn(NonNull<()>, &mut Tracer<'_>),
}

impl ErasedRoot {
    fn new<T: Trace>(gc: Gc<'_, T>) -> Self {
        unsafe fn trace_gc<T: Trace>(ptr: NonNull<()>, tracer: &mut Tracer<'_>) {
            unsafe {
                // Reconstruct the Gc pointer
                let gc: Gc<'_, T> = std::mem::transmute(ptr);
                tracer.mark(&gc);
            }
        }

        Self {
            // Safe because Gc has exactly the same memory layout as NonNull.
            ptr: unsafe { std::mem::transmute_copy(&gc) },
            trace_fn: trace_gc::<T>,
        }
    }

    pub(crate) unsafe fn trace(&self, tracer: &mut Tracer<'_>) {
        unsafe { (self.trace_fn)(self.ptr, tracer) };
    }
}

thread_local! {
    /// A stack of handle scopes for the current thread.
    pub(crate) static SCOPE_STACK: RefCell<Vec<Vec<ErasedRoot>>> = RefCell::new(Vec::new());
}

/// A scope for tracking local handles.
pub struct HandleScope {
    _marker: PhantomData<*mut ()>, // Not Send or Sync
}

impl HandleScope {
    /// Enter a new handle scope.
    #[must_use]
    pub fn enter() -> Self {
        SCOPE_STACK.with(|stack| stack.borrow_mut().push(Vec::new()));
        Self {
            _marker: PhantomData,
        }
    }
}

impl Drop for HandleScope {
    fn drop(&mut self) {
        SCOPE_STACK.with(|stack| {
            stack
                .borrow_mut()
                .pop()
                .expect("HandleScope popped without being pushed");
        });
    }
}

/// A local handle to a GC-managed value, scoped to the current `HandleScope`.
#[derive(Debug)]
pub struct Local<'gc, T: Trace + 'gc> {
    inner: Gc<'gc, T>,
}

impl<'gc, T: Trace + 'gc> Local<'gc, T> {
    /// Create a new local handle from a GC pointer.
    pub fn new(gc: Gc<'gc, T>) -> Self {
        SCOPE_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            if let Some(top) = stack.last_mut() {
                top.push(ErasedRoot::new(gc));
            } else {
                panic!("Cannot create Local without an active HandleScope");
            }
        });

        Self { inner: gc }
    }

    pub fn into_inner(self) -> Gc<'gc, T> {
        self.inner
    }
}

impl<'gc, T: Trace + 'gc> Clone for Local<'gc, T> {
    fn clone(&self) -> Self {
        Self::new(self.inner)
    }
}

impl<'gc, T: Trace + 'gc> Copy for Local<'gc, T> {}

impl<'gc, T: Trace + 'gc> std::ops::Deref for Local<'gc, T> {
    type Target = Gc<'gc, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
