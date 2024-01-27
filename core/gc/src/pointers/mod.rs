//! Pointers represents the External types returned by the Boa Garbage Collector

mod ephemeron;
mod gc;
mod weak;
mod weak_map;

pub use ephemeron::Ephemeron;
pub use gc::Gc;
pub use weak::WeakGc;
pub use weak_map::WeakMap;

pub(crate) use gc::NonTraceable;
pub(crate) use weak_map::RawWeakMap;

// Replace with std::ptr::addr_eq when 1.76 releases
#[allow(clippy::ptr_as_ptr, clippy::ptr_eq)]
fn addr_eq<T: ?Sized, U: ?Sized>(p: *const T, q: *const U) -> bool {
    (p as *const ()) == (q as *const ())
}
