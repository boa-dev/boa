//! Pointers represents the External types returned by the Boa Garbage Collector

pub mod gc_ptr;
pub mod weak_ptr;

pub use gc_ptr::Gc;
pub use weak_ptr::WeakGc;
