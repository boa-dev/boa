//! Pointers represents the External types returned by the Boa Garbage Collector

mod ephemeron;
mod gc;
mod rootable;
mod weak;

pub use ephemeron::Ephemeron;
pub use gc::Gc;
pub use weak::WeakGc;
