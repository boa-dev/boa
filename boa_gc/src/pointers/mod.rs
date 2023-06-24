//! Pointers represents the External types returned by the Boa Garbage Collector

mod ephemeron;
mod gc;
mod weak;
mod weak_map;

pub use ephemeron::Ephemeron;
pub use gc::Gc;
pub use weak::WeakGc;
pub use weak_map::WeakMap;
