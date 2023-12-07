mod ephemeron_box;
mod gc_box;
mod gc_header;
mod vtable;
mod weak_map_box;

pub(crate) use self::ephemeron_box::{EphemeronBox, ErasedEphemeronBox};
pub(crate) use self::gc_header::GcHeader;
pub(crate) use self::weak_map_box::{ErasedWeakMapBox, WeakMapBox};
pub(crate) use vtable::{vtable_of, DropFn, RunFinalizerFn, TraceFn, TraceNonRootsFn, VTable};

pub use self::gc_box::GcBox;
