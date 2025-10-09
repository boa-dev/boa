use std::any::TypeId;

use crate::{GcBox, GcErasedPointer, MEM_POOL, Trace, Tracer};

// Workaround: https://users.rust-lang.org/t/custom-vtables-with-integers/78508
pub(crate) const fn vtable_of<T: Trace + 'static>() -> &'static VTable {
    trait HasVTable: Trace + Sized + 'static {
        const VTABLE: &'static VTable;

        unsafe fn trace_fn(this: GcErasedPointer, tracer: &mut Tracer) {
            // SAFETY: The caller must ensure that the passed erased pointer is `GcBox<Self>`.
            let value = unsafe { this.cast::<GcBox<Self>>().as_ref().value() };

            // SAFETY: The implementor must ensure that `trace` is correctly implemented.
            unsafe {
                Trace::trace(value, tracer);
            }
        }

        unsafe fn trace_non_roots_fn(this: GcErasedPointer) {
            // SAFETY: The caller must ensure that the passed erased pointer is `GcBox<Self>`.
            let value = unsafe { this.cast::<GcBox<Self>>().as_ref().value() };

            // SAFETY: The implementor must ensure that `trace_non_roots` is correctly implemented.
            unsafe {
                Self::trace_non_roots(value);
            }
        }

        unsafe fn run_finalizer_fn(this: GcErasedPointer) {
            // SAFETY: The caller must ensure that the passed erased pointer is `GcBox<Self>`.
            let value = unsafe { this.cast::<GcBox<Self>>().as_ref().value() };

            Self::run_finalizer(value);
        }

        // SAFETY: The caller must ensure that the passed erased pointer is `GcBox<Self>`.
        unsafe fn drop_fn(this: GcErasedPointer) {
            // SAFETY: The caller must ensure that the passed erased pointer is `GcBox<Self>`.
            let this = this.cast::<GcBox<Self>>();

            // SAFETY: The caller must ensure the erased pointer is not droped or deallocated.
            // let _value = unsafe { Box::from_raw(this.as_ptr()) };
            let _ = MEM_POOL.try_with(|pool| {
                if !pool.borrow_mut().dealloc(this.cast()) {
                    drop(unsafe { Box::from_raw(this.as_ptr()) });
                }
            });
        }

        fn type_id_fn() -> TypeId {
            // NOTE: Currently `TypeId::of::<T>()` is not const, so we have to wrap it in function call.
            //       See issue: <https://github.com/rust-lang/rust/issues/77125>
            TypeId::of::<Self>()
        }
    }

    impl<T: Trace + 'static> HasVTable for T {
        const VTABLE: &'static VTable = &VTable {
            trace_fn: T::trace_fn,
            trace_non_roots_fn: T::trace_non_roots_fn,
            run_finalizer_fn: T::run_finalizer_fn,
            drop_fn: T::drop_fn,
            type_id_fn: T::type_id_fn,
            size: size_of::<GcBox<T>>(),
        };
    }

    T::VTABLE
}

pub(crate) type TraceFn = unsafe fn(this: GcErasedPointer, tracer: &mut Tracer);
pub(crate) type TraceNonRootsFn = unsafe fn(this: GcErasedPointer);
pub(crate) type RunFinalizerFn = unsafe fn(this: GcErasedPointer);
pub(crate) type DropFn = unsafe fn(this: GcErasedPointer);
pub(crate) type TypeIdFn = fn() -> TypeId;

#[derive(Debug)]
pub(crate) struct VTable {
    trace_fn: TraceFn,
    trace_non_roots_fn: TraceNonRootsFn,
    run_finalizer_fn: RunFinalizerFn,
    drop_fn: DropFn,
    type_id_fn: TypeIdFn,
    size: usize,
}

impl VTable {
    pub(crate) fn trace_fn(&self) -> TraceFn {
        self.trace_fn
    }

    pub(crate) fn trace_non_roots_fn(&self) -> TraceNonRootsFn {
        self.trace_non_roots_fn
    }

    pub(crate) fn run_finalizer_fn(&self) -> RunFinalizerFn {
        self.run_finalizer_fn
    }

    pub(crate) fn drop_fn(&self) -> DropFn {
        self.drop_fn
    }

    pub(crate) fn type_id(&self) -> TypeId {
        (self.type_id_fn)()
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }
}
