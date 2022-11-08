//! Pointers represents the External types returned by the Boa Garbage Collector

mod ephemeron;
mod gc_ptr;
mod weak_ptr;

pub use ephemeron::Ephemeron;
pub use gc_ptr::Gc;
pub use weak_ptr::WeakGc;
