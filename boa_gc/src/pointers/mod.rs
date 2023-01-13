//! Pointers represents the External types returned by the Boa Garbage Collector

mod ephemeron;
mod gc;
mod rootable;
mod weak;

use std::ptr::{self, addr_of_mut};

pub use ephemeron::Ephemeron;
pub use gc::Gc;
pub use weak::WeakGc;

// Technically, this function is safe, since we're just modifying the address of a pointer without
// dereferencing it.
pub(crate) fn set_data_ptr<T: ?Sized, U>(mut ptr: *mut T, data: *mut U) -> *mut T {
    // SAFETY: this should be safe as ptr must be a valid nonnull
    unsafe {
        ptr::write(addr_of_mut!(ptr).cast::<*mut u8>(), data.cast::<u8>());
    }
    ptr
}
