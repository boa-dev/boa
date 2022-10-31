//! Pointers represents the External types returned by the Boa Garbage Collector

mod gc_ptr;
mod weak_pair;
mod weak_ptr;

pub use gc_ptr::Gc;
pub use weak_pair::WeakPair;
pub use weak_ptr::WeakGc;
